use std::collections::{HashMap, HashSet};

use super::super::super::printdur;
use itertools::Itertools;
use rand::Rng;
use std::time::{Duration, Instant};

use crate::graph::{streaming::sparse_recovery::s_sparse::SparseRecovery, Graph};

/// L-0 Sampling
pub trait Coloring {
    /// Sample a coordinate at random from a high-demensional vector (vector of size n)
    ///
    /// In l-0-sampling we sample each coordinate with probability 1/||f||_0,
    /// meaning uniformly from the set of all distinct coordinates
    /// * `n` - Number of vertices in the graph.
    /// * `k` - Estimate for graph's degeneracy.
    /// * `s` - Sparsity parameter
    fn color(self, n: u32, k: u32, s: u32) -> Vec<(u32, u32)>;
}

const C: f32 = 0.4;

fn get_edge_colors(
    colors: &Vec<(u32, u32)>,
    (u, v): (u32, u32),
) -> Option<(&(u32, u32), &(u32, u32))> {
    let color1 = colors.get(u as usize)?;
    let color2 = colors.get(v as usize)?;

    Some((color1, color2))
}

impl<T> Coloring for T
where
    T: core::iter::Iterator<Item = ((u32, u32), bool)> + Sized,
{
    fn color(self, n: u32, k: u32, s: u32) -> Vec<(u32, u32)> {
        let before_setup = Instant::now();
        let l = (((2 * n * k) as f32) / (s as f32)).ceil() as u32;
        let lambda = 3 as f64 * (k as f64 * l as f64 * (n as f64).log2()).sqrt();

        let mut colors = vec![];
        let mut rng = rand::thread_rng();

        for _ in 0..n {
            let r = rng.gen_range(0..l);
            colors.push((0, r))
        }

        let mut sparse_recovery = SparseRecovery::init((n * (n - 1)) / 2, s, 0.5);

        printdur!("Setup", before_setup);

        let before_stream = Instant::now();

        self.for_each(|((u, v), c)| {
            let (color1, color2) = get_edge_colors(&colors, (u, v)).unwrap();

            let edge_number = (u * n) + v;

            if color1 == color2 {
                sparse_recovery.feed((edge_number, c));
            }
        });

        printdur!("Stream", before_stream);

        if let Some(w) = sparse_recovery.query() {
            (0..l).into_iter().for_each(|color| {
                let mut graph = HashMap::<u32, HashSet<(u32, ())>>::new();
                w.iter()
                    .enumerate()
                    .filter(|(_, count)| **count != 0)
                    .for_each(|(i, edge_count)| {
                        let (u, v) = (i as u32 / n, i as u32 % n);
                        let (color1, color2) = get_edge_colors(&colors, (u, v)).unwrap();

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
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let stream: Vec<((u32, u32), _)> = vec![
            ((1, 3), true),
            ((3, 2), true),
            ((2, 4), true),
            ((2, 5), true),
            ((1, 3), false),
            ((1, 3), true),
            ((3, 2), false),
        ];

        // (1,3),
        // (2,4),
        // (2,5),

        // minimum colors: 2

        let n: f32 = 100.0;
        let s = ((C * n) as f64 * (n as f64).log2()).round() as u32;
        println!("{:?}", s);
        let coloring = stream.into_iter().color(10, 1, s);

        let count = coloring.iter().unique().count();

        println!("{:?}", count);
    }
}
