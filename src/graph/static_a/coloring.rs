//! Relating to all things coloring

use super::super::*;
use std::{cmp::max, collections::HashSet};

type Coloring<T> = HashMap<T, usize>;

/// Coloring a Graph
pub trait Color<T, W> {
    /// Colors a graph using a specific technique outlined in [Lemma 2.6](https://arxiv.org/pdf/1905.00566.pdf#page=7)
    fn color_degeneracy(self) -> Coloring<T>;
}

impl<G, T, W> Color<T, W> for G
where
    G: Graphed<T, W>,
    T: Hash + Eq + Copy + std::fmt::Debug + Default + PartialOrd,
    W: Hash + Eq + Clone + Default + std::fmt::Debug,
{
    fn color_degeneracy(self) -> Coloring<T> {
        let mut ordering = vec![];

        let mut graph = self.clone();

        let mut degeneracy = 0_usize;
        while let Some((min, deg)) = graph.min_degree() {
            graph.remove_min();
            ordering.push(min);
            degeneracy = max(degeneracy, deg);
        }

        ordering.reverse();

        let mut coloring = HashMap::new();

        ordering.into_iter().for_each(|v| {
            let mut color: usize = 0;

            let neighbor_colors: HashSet<&usize> = self
                .get_neighbors(&v)
                .unwrap()
                .iter()
                .map(|e| e.destination)
                .filter_map(|v| coloring.get(&v))
                .collect();

            while neighbor_colors.contains(&color) {
                color += 1
            }

            coloring.insert(v, color);
        });

        coloring
    }
}

#[cfg(test)]
mod test {
    use itertools::Itertools;
    use rand::Rng;

    use super::*;

    use crate::graph::{streaming::coloring::combination, Graphed};

    fn random_graph(n: u32, m: u32) -> impl Graphed<u32, ()> {
        let mut graph = GraphWithRecaller::new(Default::default());
        let mut rng = rand::thread_rng();
        let max_edge = combination(n as u64, 2);
        for _ in 0..m {
            let rand_edge = rng.gen_range(0..max_edge + 1);
            graph.add_edge(Edge::from_d1(rand_edge));
        }
        graph
    }

    #[test]
    fn color_graph() {
        let graph = random_graph(100, 300);

        let colors = graph.color_degeneracy().values().unique().count();

        assert!([4, 5].contains(&colors));
    }
}
