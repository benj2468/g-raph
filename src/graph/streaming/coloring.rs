use std::collections::{HashMap, HashSet};

use rand::Rng;

use crate::graph::{streaming::sparse_recovery::s_sparse::SparseRecovery, Edge, Graph};

/// Structure to support coloring a Graph in the streaming setting
///
/// Algorithm for the coloring can be found [here](https://arxiv.org/pdf/1905.00566.pdf)
pub struct StreamColoring {
    /// Number of Vertices in the Graph
    n: u32,
    /// Since of initial pallet for coloring the graph.
    l: u32,
    /// Will start by holding the random color assignment, and ultimately will become the K + 1 coloring
    colors: Vec<(u32, u32)>,
    /// The sparse recovery and detection data structure
    sparse_recovery: SparseRecovery,
}

const C: f32 = 0.4;

impl StreamColoring {
    /// Initialize a new StreamColoring Instance
    ///
    /// - *n* : Size of the Universe (number of vertices)
    /// - *k* : Guess for degeneracy of graph
    /// - *s* : Sparsity parameter
    pub fn init(n: u32, k: u64, s: u64) -> Self {
        let l = (((2 * n as u64 * k) as f32) / (s as f32)).ceil() as u32;

        let mut colors = vec![];
        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let r = rng.gen_range(0..l) as u32;
            colors.push((0, r))
        }

        let sparse_recovery = SparseRecovery::init(n as u64 * n as u64, s, 0.5);

        Self {
            colors,
            l,
            n,
            sparse_recovery,
        }
    }

    /// Feed a token (and edge insertion of deletion) into the structure
    ///
    /// - *edge* : An edge between two vertices indicated by integers within *n*
    /// - *c* : True if edge is insertion, false if deletion
    pub fn feed<W>(&mut self, edge: Edge<u32, W>, c: bool) {
        let Self {
            n,
            colors,
            sparse_recovery,
            ..
        } = self;

        let (color1, color2) = Self::get_edge_colors(&colors, &edge).unwrap();

        let edge_number = edge.to_d1(*n);

        if color1 == color2 {
            sparse_recovery.feed((edge_number, c));
        }
    }

    /// Query the structure to color the graph
    ///
    /// Returns a list of tuples where the index is the vertex, and the value is the color. Colors are tuples, each unique tuple indicates a unique color.
    pub fn query(self) -> Option<Vec<(u32, u32)>> {
        let Self {
            n,
            l,
            mut colors,
            sparse_recovery,
            ..
        } = self;

        if let Some(sparse_recovery_output) = sparse_recovery.query() {
            (0..l).into_iter().for_each(|color| {
                let mut graph =
                    Graph::from_adj_list(HashMap::<u32, HashSet<(u32, ())>>::new(), None);

                sparse_recovery_output.iter().for_each(|(edge, _)| {
                    let edge = Edge::from_d1(*edge, n);
                    let (color1, color2) = Self::get_edge_colors(&colors, &edge).unwrap();

                    if color1 == color2 && *color2 == (0, color as u32) {
                        graph.add_edge(edge);
                    }
                });

                // color Gi using palette {(i, j) : 1 <= j <= κ(Gi) + 1};
                let coloring = graph.color_degeneracy();

                coloring
                    .as_ref()
                    .into_iter()
                    .for_each(|(vertex, new_color)| {
                        if *new_color == 0 {
                            return;
                        };
                        colors
                            .get_mut(*vertex as usize)
                            .map(|current| *current = (color as u32 + 1, *new_color as u32));
                    });
            });

            Some(colors)
        } else {
            None
        }
    }

    /// Fetch the colors of an edge
    fn get_edge_colors<'a, W>(
        colors: &'a Vec<(u32, u32)>,
        edge: &'a Edge<u32, W>,
    ) -> Option<(&'a (u32, u32), &'a (u32, u32))> {
        let (u, v) = edge.vertices();
        let color1 = colors.get(u as usize)?;
        let color2 = colors.get(v as usize)?;

        Some((color1, color2))
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::min, f32::INFINITY};

    use itertools::Itertools;

    use super::*;

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
        .map(|((u, v), c)| (Edge::from_d2(u as u32, v as u32), c))
        .collect()
    }

    // (1,3),
    // (2,4),
    // (2,5),
    // (4,5)

    // minimum colors: 3

    // degeneracy: 2

    #[test]
    fn test_geometric_partition() {
        let stream = test_stream();

        let n: f32 = 10.0;
        let s = ((C * n) as f64 * (n as f64).log2()).round() as u64;

        let mut min_color = INFINITY as usize;
        let mut colorers: Vec<_> = (0..(n.log2().floor() as u32))
            .into_iter()
            .map(|i| {
                let k = (2 as u32).pow(i) as u64;
                StreamColoring::init(n as u32, k, s)
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

                println!("Guess produces #{:?} colors", count);

                min_color = min(min_color, count);
            }
        }

        println!("Coloring: {:?}", min_color)
    }
}
