use num_integer::binomial;
use primes::PrimeSet;
use rand::{
    distributions::{Bernoulli, BernoulliError},
    prelude::{Distribution, ThreadRng},
    Rng,
};

use crate::graph::Edge;

pub struct BernoulliGraphDistribution<T> {
    /// Nodes in the Graph
    nodes: u32,
    /// Probability that an edge is added into the graph
    bern: Bernoulli,
    /// The noise (useful for edge streams in the turnstile setting)
    ///
    /// Default = `0`
    noise: u32,
    /// The allowed copies of a single edge to be injected into the stream
    ///
    /// Default = `1`
    copies: u32,
    /// The length of the stream up to the current point
    last: Option<T>,
    rng: ThreadRng,
}

impl<T> BernoulliGraphDistribution<T> {
    /// Generate a new UniformGraphDistribution over a size of nodes and edges
    ///
    ///
    /// When sampling with this, we will get a stream of size ((copies * edges) + 2*noise).
    /// What remains in the stream will be a set of edges sampled with uniform probability.
    /// The noise is simply to make the graph interesting with turnstile streams.
    pub fn init(nodes: u32, p: f64) -> Result<Self, BernoulliError> {
        if !(0.0..=1.0).contains(&p) {
            return Err(BernoulliError::InvalidProbability);
        }

        Ok(Self {
            nodes,
            bern: Bernoulli::new(p).unwrap(),
            noise: 0,
            copies: 1,
            last: None,
            rng: rand::thread_rng(),
        })
    }
    /// Add noise to our distribution
    pub fn with_noise(self, noise: u32) -> Self {
        Self {
            nodes: self.nodes,
            bern: self.bern,
            copies: self.copies,
            noise,
            last: None,
            rng: rand::thread_rng(),
        }
    }

    /// Add copies to our distribution
    pub fn with_copies(self, copies: u32) -> Self {
        Self {
            nodes: self.nodes,
            bern: self.bern,
            noise: self.noise,
            copies,
            last: None,
            rng: rand::thread_rng(),
        }
    }
}

/// Generates a Graph Stream
impl<T> rand::distributions::Distribution<Vec<(Edge<u32, ()>, bool)>>
    for BernoulliGraphDistribution<T>
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec<(Edge<u32, ()>, bool)> {
        let Self {
            nodes,
            bern,
            copies,
            ..
        } = self;

        (0..*nodes)
            .into_iter()
            .flat_map(|v1| {
                ((v1 + 1)..*nodes)
                    .into_iter()
                    .map(move |v2| (Edge::init(v1, v2), true))
            })
            .filter_map(|e| {
                if bern.sample(rng) {
                    Some(
                        (0..rng.gen_range(1..*copies + 1))
                            .into_iter()
                            .map(move |_| e),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}

impl rand::distributions::Distribution<(Edge<u32, ()>, bool)>
    for BernoulliGraphDistribution<(Edge<u32, ()>, bool)>
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> (Edge<u32, ()>, bool) {
        todo!()
    }
}

impl Iterator for BernoulliGraphDistribution<(Edge<u32, ()>, bool)> {
    type Item = (Edge<u32, ()>, bool);
    fn next(&mut self) -> Option<Self::Item> {
        let next_edge = if let Some(last) = self.last {
            let next = last.0.to_d1() + 1;
            if next >= binomial(self.nodes as u64, 2) {
                return None;
            }
            (Edge::from_d1(last.0.to_d1() + 1), true)
        } else {
            (Edge::from_d1(0), true)
        };

        if self.bern.sample(&mut self.rng) {
            self.last = Some(next_edge);
            Some(next_edge)
        } else {
            self.last = Some(next_edge);
            self.next()
        }
    }
}
