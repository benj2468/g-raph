use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    hash::Hash,
    time,
};

use algebraics::traits::{CeilLog2, FloorLog2};
use edge::EdgeDestination;
use itertools::Itertools;
use num_integer::{binomial, Roots};
use num_traits::Pow;
use rand::{distributions::Bernoulli, prelude::Distribution};

use crate::{
    graph::{
        edge,
        streaming::{
            sampling::l0_sampling::L0Sampler,
            sparse_recovery::s_sparse::{SparseRecovery, SparseRecoveryOutput},
        },
        Edge, Graph, Graphed,
    },
    utils::hash_function::HashFunction,
};

#[derive(Clone, Debug)]
struct KSampler<H>
where
    H: HashFunction + Clone,
{
    samplers: Vec<L0Sampler<H>>,
}

impl<H> KSampler<H>
where
    H: HashFunction + Clone,
{
    pub fn init(n: u64, k: u64, delta: f32) -> Self {
        let samplers = (0..k)
            .into_iter()
            .map(|_| L0Sampler::init(n, delta))
            .collect();

        Self { samplers }
    }

    pub fn feed(&mut self, token: (u64, bool)) {
        self.samplers
            .iter_mut()
            .for_each(|sampler| sampler.feed(token));
    }

    pub fn query(self) -> Vec<u64> {
        self.samplers
            .into_iter()
            .filter_map(|s| s.query())
            .map(|(e, _)| e)
            .collect()
    }
}

struct HSSDecomp<H>
where
    H: HashFunction + Clone,
{
    // Data
    pub inner: HashMap<u32, KSampler<H>>,
    edges: KSampler<H>,

    // Metadata
    n: u32,
    delta: u64,
    p: f32,
    del: f64,
}

impl<H> HSSDecomp<H>
where
    H: HashFunction + Clone,
{
    fn all(graph: Graph<u32, ()>) -> Graph<u32, ()> {
        let n = graph.vertices().len() as u32;
        let delta = graph
            .adj_list()
            .iter()
            .map(|(_, n)| n.len())
            .max()
            .unwrap_or_default() as u64;

        let eps = 20.0;

        let mut str = Self::init(n, delta, eps);

        for edge in graph.clone() {
            str.feed((edge, true));
        }

        str.query(&graph)
    }
    fn init(n: u32, delta: u64, eps: f32) -> Self {
        let del = eps / 10.0;

        let p = (5.0 * (n as f32).log2()) / (del.pow(2) as f32 * (delta as f32));

        let thresh = (1.0 - (1.5 * del)) as f64 * (delta as f64) * (p as f64);

        let k = n as f64 * (n as f64).log2() / (del.pow(2) as f64);
        let vertex_threshold = (1.0 - del as f64) * (k / n as f64);

        println!(
            "p: {}, n: {}, delta: {}, eps: {}, thresh: {}, k: {}, vertex_threshold: {}",
            p, n, delta, eps, thresh, k, vertex_threshold
        );

        let bern = Bernoulli::new(p as f64)
            .unwrap_or_else(|_| panic!("[PairQuerier] Invalid Probability: {}", p));

        let mut rng = rand::thread_rng();

        let base = KSampler::init(n as u64, delta, 0.01);
        let inner: HashMap<u32, KSampler<H>> = (0..n)
            .into_iter()
            .filter(|_| bern.sample(&mut rng))
            .map(|v| (v, base.clone()))
            .collect();

        let edges = KSampler::init(binomial(n.into(), 2), k as u64, 0.01);
        Self {
            inner,
            edges,
            del: del.into(),
            delta,
            n,
            p,
        }
    }

    pub fn feed(&mut self, token: (Edge<u32, ()>, bool)) {
        let (j, c) = token;
        let (u, v) = j.vertices();

        self.edges.feed((j.to_d1(), c));

        self.inner.entry(*u).and_modify(|e| {
            e.feed((*v as u64, c));
        });
        self.inner.entry(*v).and_modify(|e| {
            e.feed((*u as u64, c));
        });
    }

    pub fn query(self, actual_graph: &Graph<u32, ()>) -> Graph<u32, ()> {
        // PairQuery Phase

        let Self { inner, edges, .. } = self;

        let graph: Graph<u32, ()> = Graph::new(
            inner
                .into_iter()
                .map(|(k, s)| {
                    (
                        k,
                        s.query()
                            .iter()
                            .map(|e| EdgeDestination::init(*e as u32))
                            .collect(),
                    )
                })
                .collect(),
        );
        let edges = edges.query();

        // let mut graph = Graph::default();

        // let thresh = (1.0 - (1.5 * self.del)) as f64 * (self.delta as f64) * (self.p as f64);

        // println!("Thresh: {}", thresh);

        // let mut incidency: HashMap<u32, u32> = HashMap::new();

        // for (u, neighbors) in queried.iter() {
        //     for v in neighbors {
        //         let edge = Edge::<u32, ()>::init(min(*u, *v), max(*u, *v));
        //         let (u, v) = edge.vertices();
        //         if u == v {
        //             panic!("This should never happen");
        //         }
        //         let u_neighbors = queried.get(u);
        //         let v_neighbors = queried.get(v);

        //         let overlap = u_neighbors
        //             .and_then(|u| v_neighbors.map(|v| u.intersection(&v).count() as f64))
        //             .unwrap_or_default();

        //         if overlap >= thresh {
        //             graph.add_edge(edge);
        //             incidency.entry(*u).and_modify(|u| *u += 1);
        //             incidency.entry(*v).and_modify(|u| *u += 1);
        //         }
        //     }
        // }

        // let k = 100.0 * self.n as f64 * (self.n as f64).log2() / (self.del.pow(2));

        // let vertex_threshold = 1.0 - self.del as f64 * (k / self.n as f64);

        // for (u, count) in incidency {
        //     if (count as f64) < vertex_threshold {
        //         graph.remove_vertex(&u);
        //     }
        // }

        // println!("{}", graph);

        Default::default()
    }
}

#[cfg(test)]
mod test {
    use crate::{
        random_graph::bernoulli::BernoulliGraphDistribution,
        utils::hash_function::PowerFiniteFieldHasher,
    };

    use super::*;

    #[test]
    fn tester() {
        HSSDecomp::<PowerFiniteFieldHasher>::all(test_graph(100));
    }

    fn test_graph(n: u32) -> Graph<u32, ()> {
        let mut rng = rand::thread_rng();
        let p = 0.7 / (n as f64).log2();
        BernoulliGraphDistribution::<u32>::init(n as u32, p)
            .unwrap()
            .sample(&mut rng)
    }
}
