use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use rand::distributions::{Bernoulli, BernoulliError};

use crate::graph::Edge;

pub struct BernoulliPartiteGraph {
    /// Nodes
    n: u32,
    /// Probability of an edge
    p: f64,
    /// Partitions
    k: u32,
    /// Copies
    copies: u32,
}

impl BernoulliPartiteGraph {
    fn init(n: u32, p: f64, k: u32) -> Result<Self, BernoulliError> {
        if !(0.0..=1.0).contains(&p) {
            return Err(BernoulliError::InvalidProbability);
        }
        Ok(Self { n, p, k, copies: 1 })
    }
}

impl rand::distributions::Distribution<Vec<(Edge<u32, ()>, bool)>> for BernoulliPartiteGraph {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Vec<(Edge<u32, ()>, bool)> {
        let Self { n, p, k, copies } = self;
        let partition: HashMap<u32, Vec<u32>> =
            (0..*n).into_iter().fold(HashMap::new(), |curr, v| {
                let mut curr = curr;
                curr.entry(rng.gen_range(0..*k)).or_default().push(v);

                curr
            });

        let bern = Bernoulli::new(*p).unwrap();

        let partition = &partition;

        (0..*k)
            .into_iter()
            .flat_map(|a| {
                (a + 1..*k).into_iter().flat_map(move |b| {
                    // Partition a and partition b
                    let a_verts = partition.get(&a).cloned().unwrap_or_default();
                    let b_verts = partition.get(&b).cloned().unwrap_or_default();

                    a_verts
                        .into_iter()
                        .cartesian_product(b_verts.into_iter())
                        .map(|(src, dst)| (Edge::init(src, dst), true))
                        .collect::<Vec<(Edge<u32, ()>, bool)>>()
                })
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

#[cfg(test)]
mod test {

    use rand::prelude::Distribution;

    use crate::graph::{static_a::coloring::Color, GraphWithRecaller, Graphed};

    use super::*;

    #[test]
    fn k_partite() {
        let mut rng = rand::thread_rng();
        let sampler = BernoulliPartiteGraph::init(50, 1.0, 10).unwrap();

        let stream = sampler.sample(&mut rng);

        let mut graph = GraphWithRecaller::new(Default::default());

        for (edge, _) in stream {
            graph.add_edge(edge)
        }

        let colors = graph.color_degeneracy().values().unique().count();

        assert_eq!(colors, 10);
    }
}
