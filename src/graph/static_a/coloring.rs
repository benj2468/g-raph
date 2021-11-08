//! Relating to all things coloring

use rand::Rng;

use super::super::*;
use std::{cmp::max, collections::HashSet};

pub type Coloring<T> = HashMap<T, usize>;

/// Coloring a Graph
pub trait Colorer<T, W> {
    /// Colors a graph using a specific technique outlined in [Lemma 2.6](https://arxiv.org/pdf/1905.00566.pdf#page=7)
    fn color_degeneracy(&self) -> Coloring<T>;

    fn randomized(&self) -> Coloring<T>;

    fn is_proper(&self, coloring: &Coloring<T>) -> bool;

    fn is_partial(&self, coloring: &Coloring<T>) -> bool;

    fn greedy(&self, color_options: Option<HashMap<T, HashSet<u32>>>) -> Coloring<T>;
}

impl<G, T, W> Colorer<T, W> for G
where
    G: Graphed<T, W>,
    T: Hash + Eq + Copy + std::fmt::Debug + Default + PartialOrd,
    W: Hash + Eq + Clone + Default + std::fmt::Debug,
{
    fn color_degeneracy(&self) -> Coloring<T> {
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

    fn randomized(&self) -> Coloring<T> {
        let mut coloring = HashMap::new();

        let delta_1 = self
            .adj_list()
            .iter()
            .map(|a| a.1.len())
            .max()
            .unwrap_or_default()
            + 1;

        let mut conflicting_edges: HashSet<Edge<T, W>> = HashSet::new();

        for v in self.vertices() {
            let color = rand::thread_rng().gen_range(0..delta_1);

            coloring.insert(*v, color);

            if let Some(neighbors) = self.get_neighbors(v) {
                for neighbor in neighbors {
                    if coloring.get(v) == coloring.get(&neighbor.destination) {
                        conflicting_edges.insert(Edge::init(*v, neighbor.destination));
                    }
                }
            }
        }

        while !conflicting_edges.is_empty() {
            let edge = conflicting_edges.iter().next().unwrap().clone();
            let (u, _) = edge.vertices();
            let new_color = rand::thread_rng().gen_range(0..delta_1);
            coloring.insert(*u, new_color);

            if let Some(neighbors) = self.get_neighbors(u) {
                for neighbor in neighbors {
                    let edge = Edge::init(*u, neighbor.destination);
                    if coloring.get(u) == coloring.get(&neighbor.destination) {
                        conflicting_edges.insert(edge);
                    } else {
                        conflicting_edges.remove(&edge);
                    }
                }
            }
        }

        coloring
    }

    fn is_proper(&self, coloring: &Coloring<T>) -> bool {
        for (v, color) in coloring {
            if let Some(neighbors) = self.get_neighbors(&v) {
                for neighbor in neighbors {
                    if coloring
                        .get(&neighbor.destination)
                        .unwrap_or_else(|| panic!("The provided coloring is not one for the provided graph, Could not find a color for: {:?}", neighbor.destination))
                        == color
                    {
                        println!("Coloring was not proper under the following vertices: {:?}, {:?}", neighbor.destination, v);
                        return false;
                    }
                }
            }
        }
        true
    }

    fn is_partial(&self, coloring: &Coloring<T>) -> bool {
        for (v, color) in coloring {
            if let Some(neighbors) = self.get_neighbors(&v) {
                for neighbor in neighbors {
                    if coloring
                        .get(&neighbor.destination)
                        .map(|c| c == color)
                        .unwrap_or_default()
                    {
                        println!(
                            "Coloring was not proper(partial) under the following vertices: {:?}, {:?}",
                            neighbor.destination, v
                        );
                        return false;
                    }
                }
            }
        }
        true
    }

    fn greedy(&self, options: Option<HashMap<T, HashSet<u32>>>) -> Coloring<T> {
        let mut coloring: HashMap<T, usize> = HashMap::new();

        for v in self.vertices() {
            let neighbor_colors = self
                .get_neighbors(&v)
                .map(|set| {
                    set.iter()
                        .map(|e| e.destination)
                        .filter_map(|e| coloring.get(&e))
                        .cloned()
                        .map(|a| a as u32)
                        .collect::<HashSet<u32>>()
                })
                .unwrap_or_default();

            if let Some(next_color) = options
                .as_ref()
                .map(|preset| {
                    preset
                        .get(v)
                        .and_then(|colors| colors.difference(&neighbor_colors).next())
                        .map(|a| *a as usize)
                })
                .unwrap_or_else(|| {
                    (0..self.vertices().len())
                        .into_iter()
                        .find(|c| !neighbor_colors.contains(&(*c as u32)))
                })
            {
                coloring.insert(*v, next_color);
            }
        }

        coloring
    }
}

#[cfg(test)]
mod test {

    use rand::prelude::Distribution;

    use super::*;

    use crate::random_graph::{partite::BernoulliPartiteGraph, uniform::UniformGraphDistribution};

    #[test]
    fn color_graph() {
        let graph: GraphWithRecaller<_, _> =
            UniformGraphDistribution::init(100, 300).sample(&mut rand::thread_rng());

        let coloring = graph.color_degeneracy();

        assert!(graph.is_proper(&coloring))
    }

    #[test]
    fn color_random() {
        let graph: Graph<_, _> = BernoulliPartiteGraph::init(100, 0.9, 20)
            .unwrap()
            .sample(&mut rand::thread_rng());

        let coloring = graph.randomized();

        assert!(graph.is_proper(&coloring))
    }
}
