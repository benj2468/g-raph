//! Creates a Random Graph using Uniform Distribution for edge selections

use itertools::Itertools;
use rand::prelude::IteratorRandom;

use crate::graph::{Edge, Graphed};

pub struct UniformGraphDistribution {
    /// The number of nodes in our graph
    nodes: u32,
    /// The number of edges we want to be in our graph
    edges: u32,
    /// The noise (useful for edge streams in the turnstile setting)
    ///
    /// Default = `0`
    noise: u32,
    /// The allowed copies of a single edge to be injected into the stream
    ///
    /// Default = `1`
    copies: u32,
}

impl UniformGraphDistribution {
    /// Generate a new UniformGraphDistribution over a size of nodes and edges
    ///
    ///
    /// When sampling with this, we will get a stream of size ((copies * edges) + 2*noise).
    /// What remains in the stream will be a set of edges sampled with uniform probability.
    /// The noise is simply to make the graph interesting with turnstile streams.
    pub fn init(nodes: u32, edges: u32) -> Self {
        Self {
            nodes,
            edges,
            noise: Default::default(),
            copies: 1,
        }
    }
    /// Add noise to our distribution
    pub fn with_noise(self, noise: u32) -> Self {
        Self {
            nodes: self.nodes,
            edges: self.edges,
            copies: self.copies,
            noise,
        }
    }

    /// Add copies to our distribution
    pub fn with_copies(self, copies: u32) -> Self {
        Self {
            nodes: self.nodes,
            edges: self.edges,
            noise: self.noise,
            copies,
        }
    }
}

/// Generates a Graph Stream
impl rand::distributions::Distribution<Vec<(Edge<u32, ()>, bool)>> for UniformGraphDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec<(Edge<u32, ()>, bool)> {
        let noise: Vec<_> = (0..self.noise)
            .into_iter()
            .flat_map(|_| {
                let v1 = rng.gen_range(0..self.nodes - 1);
                let edge = Edge::init(v1, rng.gen_range(v1..self.nodes));
                vec![(edge, true), (edge, false)]
            })
            .collect();

        (0..self.nodes)
            .into_iter()
            .cartesian_product(0..self.nodes)
            .filter(|(a, b)| a != b)
            .map(|(src, dst)| (Edge::init(src, dst), true))
            .choose_multiple(rng, self.edges as usize)
            .into_iter()
            .flat_map(|e| {
                (0..rng.gen_range(1..self.copies + 1))
                    .into_iter()
                    .map(move |_| e)
            })
            .chain(noise)
            .collect()
    }
}

/// Generates a Graph in Memory
impl<T> rand::distributions::Distribution<T> for UniformGraphDistribution
where
    T: Graphed<u32, ()> + Sized,
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> T {
        let mut graph = T::default();

        let edges: Vec<(Edge<u32, ()>, bool)> = self.sample(rng);

        edges.into_iter().for_each(|(edge, c)| {
            if c {
                graph.add_edge(edge)
            } else {
                graph.remove_edge(edge)
            }
        });

        graph
    }
}

#[cfg(test)]
mod test {
    use rand::prelude::Distribution;

    use super::*;

    #[test]
    fn sample_simple() {
        let mut rng = rand::thread_rng();
        let dist = UniformGraphDistribution::init(10, 30);

        let stream: Vec<_> = dist.sample(&mut rng);

        assert_eq!(stream.len(), 30)
    }

    #[test]
    fn sample_with_noise() {
        let mut rng = rand::thread_rng();
        let dist = UniformGraphDistribution::init(10, 30).with_noise(20);

        let stream: Vec<_> = dist.sample(&mut rng);

        assert_eq!(stream.len(), 70)
    }

    #[test]
    fn sample_with_copies() {
        let mut rng = rand::thread_rng();
        let dist = UniformGraphDistribution::init(10, 30).with_copies(10);

        let stream: Vec<_> = dist.sample(&mut rng);

        assert!(stream.len() >= 30);
        assert!(stream.len() <= 300);
    }

    #[test]
    fn sample_with_noise_andcopies() {
        let mut rng = rand::thread_rng();
        let dist = UniformGraphDistribution::init(10, 30)
            .with_copies(10)
            .with_noise(20);

        let stream: Vec<_> = dist.sample(&mut rng);

        assert!(stream.len() >= 70);
        assert!(stream.len() <= 300);
    }
}
