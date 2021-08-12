use rand::{distributions::Bernoulli, Rng};

use crate::graph::Edge;

pub struct BernoulliGraphDistribution {
    /// Nodes in the Graph
    nodes: u32,
    /// Probability that an edge is added into the graph
    p: f64,
    /// The noise (useful for edge streams in the turnstile setting)
    ///
    /// Default = `0`
    noise: u32,
    /// The allowed copies of a single edge to be injected into the stream
    ///
    /// Default = `1`
    copies: u32,
}

impl BernoulliGraphDistribution {
    /// Generate a new UniformGraphDistribution over a size of nodes and edges
    ///
    ///
    /// When sampling with this, we will get a stream of size ((copies * edges) + 2*noise).
    /// What remains in the stream will be a set of edges sampled with uniform probability.
    /// The noise is simply to make the graph interesting with turnstile streams.
    pub fn init(nodes: u32, p: f64) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&p) {
            return Err("Invalid Probability".into());
        }

        Ok(Self {
            nodes,
            p,
            noise: 0,
            copies: 1,
        })
    }
    /// Add noise to our distribution
    pub fn with_noise(self, noise: u32) -> Self {
        Self {
            nodes: self.nodes,
            p: self.p,
            copies: self.copies,
            noise,
        }
    }

    /// Add copies to our distribution
    pub fn with_copies(self, copies: u32) -> Self {
        Self {
            nodes: self.nodes,
            p: self.p,
            noise: self.noise,
            copies,
        }
    }
}

/// Generates a Graph Stream
impl rand::distributions::Distribution<Vec<(Edge<u32, ()>, bool)>> for BernoulliGraphDistribution {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec<(Edge<u32, ()>, bool)> {
        let Self {
            nodes, p, copies, ..
        } = self;

        let bern = Bernoulli::new(*p).unwrap();
        (0..*nodes)
            .into_iter()
            .flat_map(|v1| {
                ((v1 + 1)..*nodes)
                    .into_iter()
                    .map(move |v2| (Edge::init(v1, v2), true))
            })
            .filter_map(|e| {
                if bern.sample(rng) {
                    Some((1..rng.gen_range(1..*copies)).into_iter().map(move |_| e))
                } else {
                    None
                }
            })
            .flatten()
            .collect()
    }
}
