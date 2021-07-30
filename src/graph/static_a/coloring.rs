//! Relating to all things coloring

use super::super::*;
use std::collections::HashSet;

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

        while let Some(min) = graph.remove_min() {
            ordering.push(min);
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
