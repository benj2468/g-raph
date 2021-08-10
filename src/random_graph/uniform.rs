use itertools::Itertools;
use rand::prelude::IteratorRandom;

use crate::graph::{Edge, Graphed};

pub struct UniformGraphDistribution {
    nodes: u32,
    edges: u64,
}

impl UniformGraphDistribution {
    pub fn init(nodes: u32, edges: u64) -> Self {
        Self { nodes, edges }
    }
}

impl rand::distributions::Distribution<Vec<Edge<u32, ()>>> for UniformGraphDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec<Edge<u32, ()>> {
        (0..self.nodes)
            .into_iter()
            .cartesian_product(0..self.nodes)
            .filter(|(a, b)| a != b)
            .map(|(src, dst)| Edge::init(src, dst))
            .choose_multiple(rng, self.edges as usize)
    }
}

impl<T> rand::distributions::Distribution<T> for UniformGraphDistribution
where
    T: Graphed<u32, ()> + Sized,
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> T {
        let mut graph = T::default();

        let edges: Vec<Edge<u32, ()>> = self.sample(rng);

        edges.into_iter().for_each(|edge| {
            graph.add_edge(edge);
        });

        graph
    }
}
