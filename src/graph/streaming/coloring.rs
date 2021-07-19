use std::collections::{HashMap, HashSet};

use super::super::super::printdur;
use itertools::Itertools;
use rand::Rng;
use std::time::{Duration, Instant};

use crate::graph::{streaming::sparse_recovery::s_sparse::SparseRecovery, Graph};

/// Structure to support coloring a Graph in the streaming setting
///
/// Algorithm for the coloring can be found [here](https://arxiv.org/pdf/1905.00566.pdf)
pub struct StreamColoring {
    n: u32,
    l: u32,
    lambda: f64,
    colors: Vec<(u32, u32)>,
    sparse_recovery: SparseRecovery,
}

const C: f32 = 0.4;

impl StreamColoring {
    /// Initialize a new StreamColoring Instance
    ///
    /// - *n* : Size of the Universe (number of vertices)
    /// - *k* : Guess for degeneracy of graph
    /// - *s* : Sparsity parameter
    pub fn init(n: u32, k: u32, s: u32) -> Self {
        let before_setup = Instant::now();
        let l = (((2 * n * k) as f32) / (s as f32)).ceil() as u32;
        let lambda = 3 as f64 * (k as f64 * l as f64 * (n as f64).log2()).sqrt();

        let mut colors = vec![];
        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let r = rng.gen_range(0..l);
            colors.push((0, r))
        }

        let sparse_recovery = SparseRecovery::init(n * n, s, 0.5);

        Self {
            colors,
            l,
            lambda,
            n,
            sparse_recovery,
        }
    }

    /// Feed a token (and edge insertion of deletion) into the structure
    ///
    /// - *edge* : An edge between two vertices indicated by integers within *n*
    /// - *c* : True if edge is insertion, false if deletion
    pub fn feed(&mut self, edge: (u32, u32), c: bool) {
        let Self {
            n,
            colors,
            sparse_recovery,
            ..
        } = self;

        let (u, v) = edge;

        let (color1, color2) = Self::get_edge_colors(&colors, (u, v)).unwrap();

        let edge_number = (u * *n) + v;

        if color1 == color2 {
            sparse_recovery.feed((edge_number, c));
        }
    }

    /// Query the structure to color the graph
    ///
    /// Returns a list of tuples where the index is the vertex, and the value is the color. Colors are tuples, each unique tuple indicates a unique color.
    pub fn query(self) -> Vec<(u32, u32)> {
        let Self {
            n,
            l,
            mut colors,
            sparse_recovery,
            ..
        } = self;

        if let Some(w) = sparse_recovery.query() {
            (0..l).into_iter().for_each(|color| {
                let mut graph = HashMap::<u32, HashSet<(u32, ())>>::new();
                w.iter()
                    .enumerate()
                    .filter(|(_, count)| **count != 0)
                    .for_each(|(i, edge_count)| {
                        let (u, v) = (i as u32 / n, i as u32 % n);
                        let (color1, color2) = Self::get_edge_colors(&colors, (u, v)).unwrap();

                        if color1 == color2 && *color2 == (0, color) {
                            if !graph.contains_key(&u) {
                                graph.insert(u, HashSet::new());
                            }
                            graph.get_mut(&u).unwrap().insert((v, ()));
                            if !graph.contains_key(&v) {
                                graph.insert(v, HashSet::new());
                            }
                            graph.get_mut(&v).unwrap().insert((u, ()));
                        }
                    });

                // color Gi using palette {(i, j) : 1 <= j <= κ(Gi) + 1};
                let coloring = Graph::from_adj_list(graph, None).color_degeneracy();

                coloring
                    .as_ref()
                    .into_iter()
                    .for_each(|(vertex, new_color)| {
                        if *new_color == 0 {
                            return;
                        };
                        colors
                            .get_mut(*vertex as usize)
                            .map(|curr| *curr = (color + 1, *new_color as u32));
                    });
            })
        }

        colors
    }

    /// Fetch the colors of an edge
    fn get_edge_colors(
        colors: &Vec<(u32, u32)>,
        (u, v): (u32, u32),
    ) -> Option<(&(u32, u32), &(u32, u32))> {
        let color1 = colors.get(u as usize)?;
        let color2 = colors.get(v as usize)?;

        Some((color1, color2))
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::min, f32::INFINITY};

    use super::*;

    fn test_stream() -> Vec<((u32, u32), bool)> {
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
        let s = ((C * n) as f64 * (n as f64).log2()).round() as u32;

        let mut min_color = INFINITY as usize;
        let mut colorers: Vec<_> = (0..(n.log2().floor() as u32))
            .into_iter()
            .map(|i| {
                let k = (2 as u32).pow(i);
                StreamColoring::init(n as u32, k, s)
            })
            .collect();

        for (edge, c) in stream {
            for colorer in &mut colorers {
                colorer.feed(edge, c)
            }
        }

        for colorer in colorers {
            let coloring = colorer.query();

            let count = coloring.iter().unique().count();

            println!("Guess produces #{:?} colors", count);

            min_color = min(min_color, count);
        }

        println!("Coloring: {:?}", min_color)
    }
}
