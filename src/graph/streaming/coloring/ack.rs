//! Coloring Algorithm as defined in the ACK paper
/// This is a Work In Progress, and by no means correct or complete yet.
use super::compute_s;
use crate::{
    graph::{streaming::sparse_recovery::s_sparse::SparseRecovery, Edge, Graph, Graphed},
    utils::hash_function::PowerFiniteFieldHasher,
};
use itertools::Itertools;
use num_traits::Pow;
use rand::{distributions::Bernoulli, prelude::Distribution, Rng};
use std::collections::{HashMap, HashSet};

type Color = u32;

#[derive(Debug)]
pub struct StreamColoring {
    color_sample: HashMap<u32, HashSet<Color>>,
    // We might not actually need this...
    chromatic: HashMap<Color, HashSet<u32>>,
    recovery: SparseRecovery<PowerFiniteFieldHasher>,
    pair_querier: HashMap<u32, SparseRecovery<PowerFiniteFieldHasher>>,
    // Values
    n: u32,
    p: f32,
    delta: u32,
    del: f32,
}

impl StreamColoring {
    pub fn init(n: u32, delta: u32, del: f32) -> Self {
        let mut color_sample: HashMap<u32, HashSet<Color>> = Default::default();
        let mut chromatic: HashMap<Color, HashSet<u32>> = Default::default();
        let logn = (n as f64).log2().ceil() as u32;
        let mut rng = rand::thread_rng();
        for vertex in 0..n {
            let sample = (0..logn)
                .into_iter()
                .map(|_| {
                    let color = rng.gen_range(0..(delta + 1));
                    chromatic.entry(color).or_default().insert(vertex);
                    color
                })
                .collect();

            color_sample.insert(vertex, sample);
        }

        let s = compute_s(n);

        let recovery = SparseRecovery::init(n.into(), s.ceil() as u64, 0.01);

        // This probability checking value is incorrect
        let p = (10.0 * (n as f32).log2()) / (delta.pow(2) as f32 * (del as f32));

        println!("{:?}", p);

        let pair_querier = {
            let bern = Bernoulli::new(p as f64).unwrap();

            let mut rng = rand::thread_rng();

            // I think we can do this with making n way smaller, rather than O(n^2) where n is number of vertices, we can make this O(n),
            // since we know that each edge sampled for each of these structures will have v as an incident vertex.
            (0..n)
                .into_iter()
                .filter(|_| bern.sample(&mut rng))
                .map(|v| (v, SparseRecovery::init(n.into(), del as u64, 0.01)))
                .collect()
        };

        Self {
            n,
            color_sample,
            chromatic,
            recovery,
            pair_querier,
            p,
            del,
            delta,
        }
    }

    pub fn feed(&mut self, token: (Edge<u32, ()>, bool)) {
        let (u, v) = token.0.vertices();
        let has_overlap = {
            let u = self
                .color_sample
                .get(u)
                .unwrap_or_else(|| panic!("That vertex is not in the graph: {:?}", u));
            let v = self
                .color_sample
                .get(v)
                .unwrap_or_else(|| panic!("That vertex is not in the graph: {:?}", u));

            u.intersection(v).next().is_some()
        };

        let token = (token.0.to_d1(), token.1);

        if has_overlap {
            self.recovery.feed(token)
        }

        self.pair_querier
            .entry(*u)
            .and_modify(|recovery| recovery.feed((*v as u64, token.1)));

        self.pair_querier
            .entry(*v)
            .and_modify(|recovery| recovery.feed((*u as u64, token.1)));
    }

    pub fn query(self) -> Option<HashMap<u32, Color>> {
        // Find a proper list coloring, where any color for v \in L(v)

        let mut pair_querier = self.pair_querier;

        let sparse_queries: HashMap<u32, HashMap<u64, i64>> = pair_querier
            .into_iter()
            .filter_map(|(v, recovery)| recovery.query().map(|res| (v, res)))
            .collect();

        let Self {
            n, delta, del, p, ..
        } = self;

        let query_pair = |edge: Edge<u32, ()>| -> bool {
            let (u, v) = edge.vertices();
            let u_neighbors = sparse_queries.get(&u);
            let v_neighbors = sparse_queries.get(&v);

            u_neighbors
                .zip(v_neighbors)
                .map(|(u, v)| {
                    let u = u.keys().collect::<HashSet<_>>();
                    let v = v.keys().collect::<HashSet<_>>();

                    (u.intersection(&v).count() as f32) < (1.0 - (1.5 * del) * (delta as f32) * p)
                })
                .unwrap_or_default()
        };

        if let Some(stream) = self.recovery.query() {
            let mut graph: Graph<u32, ()> = Graph::new(Default::default());
            stream
                .iter()
                .filter_map(|(edge, _)| {
                    let edge = Edge::<u32, ()>::from_d1(*edge);

                    query_pair(edge).then(|| edge)
                })
                .for_each(|e| graph.add_edge(e));

            let mut d = HashSet::new();

            for v in graph.vertices() {
                if graph.get_neighbors(v).map(|e| e.len()).unwrap_or_default() as f32
                    > (1.0 - del) * ((compute_s(n) / n as f64) as f32)
                {
                    d.insert(v);
                }
            }
            println!("{:?}", d);
        }

        todo!()
    }
}

pub struct QueryStructure(HashMap<(u32, u32), bool>);

impl QueryStructure {
    pub fn init<G: Graphed<u32, ()>>(graph: G, del: f32) -> Self {
        let vertices = graph.vertices();
        let n = vertices.len() as u32;
        let max_degree = graph
            .adj_list()
            .iter()
            .map(|(_, v)| v.len())
            .max()
            .unwrap_or_default();
        let p = (10.0 * (n as f32).log2()) / (del.pow(2) * (max_degree as f32));

        let set: HashSet<u32> = {
            let bern = Bernoulli::new(p as f64).unwrap();

            let mut rng = rand::thread_rng();

            (0..n)
                .into_iter()
                .filter(|_| bern.sample(&mut rng))
                .collect()
        };

        QueryStructure(
            (0..n)
                .into_iter()
                .cartesian_product(0..n)
                .map(|(u, v)| {
                    let c = graph
                        .get_neighbors(&u)
                        .zip(graph.get_neighbors(&v))
                        .map(|(a, b)| {
                            let a_b = a.intersection(b);
                            a_b.map(|e| e.destination)
                                .collect::<HashSet<_>>()
                                .intersection(&set)
                                .collect::<HashSet<_>>()
                                .len() as f32
                                > ((1.0_f32 - (1.5 * del)) * (max_degree as f32) * p)
                        })
                        .unwrap_or_default();
                    ((u, v), c)
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod test {

    use crate::graph::Graph;

    fn test_graph() -> Graph<u32, ()> {
        r"0: 1,2,3
            1: 0,2,3
            2: 0,1,3
            3: 0,1,2"
            .parse()
            .unwrap()
    }
}
