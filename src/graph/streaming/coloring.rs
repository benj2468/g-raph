//! Coloring

use num_integer::binomial;
use rand::Rng;
use std::{collections::HashMap, fmt::Debug};

use crate::graph::{
    static_a::coloring::Color, streaming::sparse_recovery::s_sparse::SparseRecovery, Edge,
    GraphWithRecaller, Graphed,
};

use crate::utils::hash_function::PowerFiniteFieldHasher;

/// Representation of a Color, we use tuple to differentiate when we re-color the monochromatic components
type ColorTuple = (u32, u32);

/// Structure to support coloring a Graph in the streaming setting
///
/// Algorithm for the coloring can be found [here](https://arxiv.org/pdf/1905.00566.pdf)
///
/// Total space required = O(|V| +  slog(s/del))
#[derive(Clone)]
pub struct StreamColoring {
    /// Since of initial pallet for coloring the graph.
    ///
    /// Constant space
    palette_size: u32,
    /// Will start by holding the random color assignment, and ultimately will become the K + 1 coloring
    ///
    /// Stores n values, for n vertices, each ColorTuple is a tuple of 32bit integers, thus O(|V|) space
    colors: HashMap<u32, ColorTuple>,
    /// The sparse recovery and detection data structure
    ///
    /// Space = Space required by SparseRecovery where n(edges) = n(vertices) choose 2
    sparse_recovery: SparseRecovery<PowerFiniteFieldHasher>,
    #[cfg(test)]
    captured: Vec<u64>,
}

pub fn compute_s(n: u32) -> f64 {
    const C: f32 = 15.0;
    (C * n as f32) as f64 * (n as f64).log2()
}

impl StreamColoring {
    /// Initialize a new StreamColoring Instance
    ///
    /// - *n* : Size of the Universe (number of vertices)
    /// - *k* : Guess for degeneracy of graph
    /// - *s* : Sparsity parameter
    /// - *del* : Error Parameter for SparseRecovery
    //
    // k can be u32 as well
    pub fn init(n: u32, k: u64, del: f32) -> Self {
        // How many edges we ever want to collect
        let s = compute_s(n);
        let palette_size = (((2 * n as u64 * k) as f64) / s).ceil() as u32;

        println!("s = {}; palette_size = {}", s, palette_size);

        let mut colors = HashMap::<u32, ColorTuple>::new();
        let mut rng = rand::thread_rng();

        for i in 0..n {
            let color = rng.gen_range(0..palette_size) as u32;
            colors.insert(i, (0, color));
        }
        let sparse_recovery = SparseRecovery::init(binomial(n as u64, 2), s.ceil() as u64, del);

        Self {
            palette_size,
            colors,
            sparse_recovery,
            #[cfg(test)]
            captured: vec![],
        }
    }

    pub fn new_k(&self, n: u32, k: u64) -> Option<Self> {
        let s = compute_s(n);

        let palette_size = (((2 * n as u64 * k) as f64) / s).ceil() as u32;
        if palette_size == self.palette_size {
            return None;
        }

        println!("s = {}; palette_size = {}", s, palette_size);

        let mut colors = HashMap::<u32, ColorTuple>::new();
        let mut rng = rand::thread_rng();

        for i in 0..n {
            let color = rng.gen_range(0..palette_size) as u32;
            colors.insert(i, (0, color));
        }

        Some(Self {
            palette_size,
            colors,
            sparse_recovery: self.sparse_recovery.clone(),
            #[cfg(test)]
            captured: vec![],
        })
    }

    /// Feed a token (and edge insertion of deletion) into the structure
    ///
    /// - *edge* : An edge between two vertices indicated by integers within *n*
    /// - *c* : True if edge is insertion, false if deletion
    pub fn feed<W: Debug + Default>(&mut self, edge: Edge<u32, W>, c: bool) {
        let Self {
            colors,
            sparse_recovery,
            ..
        } = self;

        let (color1, color2) = {
            let (u, v) = edge.vertices();
            let color1 = colors.get(u).unwrap();
            let color2 = colors.get(v).unwrap();

            (color1, color2)
        };

        if color1 == color2 {
            let edge_number = edge.to_d1();
            sparse_recovery.feed((edge_number, c));
            #[cfg(test)]
            self.captured.push(edge_number);
        }
    }

    /// Query the structure to color the graph
    ///
    /// Returns a list of tuples where the index is the vertex, and the value is the color. Colors are tuples, each unique tuple indicates a unique color.
    pub fn query(self) -> Option<HashMap<u32, ColorTuple>> {
        let Self {
            palette_size,
            mut colors,
            sparse_recovery,
            ..
        } = self;

        let mut monochromatic_graphs: HashMap<(u32, u32), GraphWithRecaller<u32, ()>> = (0
            ..palette_size)
            .into_iter()
            .map(|color| ((0, color), Graphed::new(Default::default())))
            .collect();

        if let Some(sparse_recovery_output) = sparse_recovery.query() {
            sparse_recovery_output.iter().for_each(|(edge, _)| {
                let edge = Edge::from_d1(*edge);

                let (color1, color2) = {
                    let (u, v) = edge.vertices();
                    let color1 = colors.get(u).unwrap();
                    let color2 = colors.get(v).unwrap();

                    (color1, color2)
                };

                if color1 == color2 {
                    monochromatic_graphs.get_mut(color1).unwrap().add_edge(edge);
                }
            });

            // color Gi using palette {(i, j) : 1 <= j <= κ(Gi) + 1};
            monochromatic_graphs
                .into_iter()
                .filter(|(_, graph)| !graph.is_empty())
                .for_each(|((_, color), graph)| {
                    let coloring = graph.color_degeneracy();

                    coloring.into_iter().for_each(|(vertex, new_color)| {
                        if new_color == 0 {
                            return;
                        };
                        colors.insert(vertex, (color + 1, new_color as u32));
                    });
                });

            Some(colors)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::min, f32::INFINITY};

    use itertools::Itertools;

    use super::*;

    #[test]
    fn comb() {
        assert_eq!(binomial(100, 2), 4950);
    }

    fn test_stream() -> Vec<(Edge<u32, ()>, bool)> {
        vec![
            ((1, 3), true),
            ((3, 2), true),
            ((2, 4), true),
            ((2, 5), true),
            ((1, 3), false),
            ((1, 3), true),
            ((3, 2), false),
            ((4, 5), true),
        ]
        .into_iter()
        .map(|((u, v), c)| (Edge::init(u as u32, v as u32), c))
        .collect()
        // remaining edges
        //
        // (1,3)
        // (2,4)
        // (2,5)
        // (4,5)
    }

    #[test]
    fn test_geometric_partition() {
        let stream = test_stream();

        let n: f32 = 10.0;

        let mut min_color = INFINITY as usize;
        let mut colorers: Vec<_> = (0..(n.log2().floor() as u32))
            .into_iter()
            .map(|i| {
                let k = (2 as u32).pow(i) as u64;
                StreamColoring::init(n as u32, k, 0.01)
            })
            .collect();

        for (edge, c) in stream {
            for colorer in &mut colorers {
                colorer.feed(edge, c)
            }
        }

        for colorer in colorers {
            if let Some(coloring) = colorer.query() {
                let count = coloring.iter().unique().count();
                min_color = min(min_color, count);
            }
        }

        println!("{:?}", min_color);
    }
}
