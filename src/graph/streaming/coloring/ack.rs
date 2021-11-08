// //! Coloring Algorithm as defined in the ACK paper
// /// This is a Work In Progress, and by no means correct or complete yet.

use super::compute_s;
use crate::{
    graph::{
        static_a::{
            coloring::{Colorer, Coloring},
            matching::MatchingT,
            search::{ConnectedComponents, Search},
        },
        streaming::{
            sparse_recovery::s_sparse::{SparseRecovery, SparseRecoveryOutput},
            Query, Stream,
        },
        Edge, Graph, GraphWithRecaller, Graphed,
    },
    utils::hash_function::PowerFiniteFieldHasher,
};
use itertools::Itertools;
use num_integer::binomial;
use num_traits::Pow;
use rand::{distributions::Bernoulli, prelude::Distribution};
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    f32::EPSILON,
    fmt::Debug,
};

type Color = usize;
pub struct PairQuerier {
    // Data
    pub inner: HashMap<u32, SparseRecovery<PowerFiniteFieldHasher>>,

    // Metadata
    n: u32,
    delta: u64,
    p: f32,
    del: f64,
}

impl Debug for PairQuerier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "delta: {:?}
            p: {:?}",
            self.delta, self.p
        )
    }
}

impl PairQuerier {
    // We might be able to cut down on data if we only store inner on the vertices and not on all n
    fn init(n: u32, delta: u64, del: f64) -> Self {
        let p = (10.0 * (n as f32).log2()) / (del.pow(2) as f32 * (delta as f32));
        // let p = 1.0;

        println!("[Pair Querier]: {:?}", p);
        let bern = Bernoulli::new(p as f64)
            .unwrap_or_else(|_| panic!("[PairQuerier] Invalid Probability: {}", p));

        let mut rng = rand::thread_rng();

        println!("{:?}, {}, {}", del, delta, p);

        println!(
            "[Pair Querier] Threshold: {:?}",
            (1.0 - (1.5 * del)) as f64 * (delta as f64) * (p as f64)
        );

        // Pick a set S of vertices at the beginning of the stream by choosing each vertex
        // independently with probability p.
        //
        // For any chosen vertex in S, run the algorithm in Proposition 4.2(Sparse Recovery) with P
        // being the set of all edge slots incident to the vertex and k = delta
        let base = SparseRecovery::init(n.into(), delta, 0.01);
        let inner: HashMap<u32, _> = (0..n)
            .into_iter()
            .filter(|_| bern.sample(&mut rng))
            .map(|v| (v, base.clone()))
            .collect();

        println!("[Pair Querier]: Completed Initialization");

        Self {
            n,
            inner,
            delta,
            p,
            del,
        }
    }

    fn feed(&mut self, token: (Edge<u32, ()>, bool)) {
        let (u, v) = token.0.vertices();

        self.inner
            .entry(*u)
            .and_modify(|recovery| recovery.feed((*v as u64, token.1)));

        self.inner
            .entry(*v)
            .and_modify(|recovery| recovery.feed((*u as u64, token.1)));
    }

    fn query(&mut self) -> Graph<u32, ()> {
        let Self { n, .. } = self;
        let queried: HashMap<u32, HashSet<u32>> = self
            .inner
            .iter_mut()
            .map(|(k, stream)| {
                (
                    *k,
                    stream
                        .clone()
                        .query()
                        .unwrap_or_default()
                        .keys()
                        .copied()
                        .filter_map(|e| e.try_into().ok())
                        .collect(),
                )
            })
            .collect();

        let mut graph = Graph::default();
        let mut d_prelim: HashMap<u32, u32> = HashMap::default();

        for (v, neighbors) in queried.iter() {
            for u in neighbors {
                if u == v {
                    continue;
                }
                let u_neighbors = queried.get(u);
                let v_neighbors = queried.get(v);

                let thresh =
                    (1.0 - (1.5 * self.del)) as f64 * (self.delta as f64) * (self.p as f64);

                let overlap = u_neighbors
                    .zip(v_neighbors)
                    .map(|(u, v)| u.intersection(&v).count() as f64)
                    .unwrap_or_default();
                if overlap >= thresh {
                    // Answer is YES
                    *d_prelim.entry(*u).or_default() += 1;
                    *d_prelim.entry(*v).or_default() += 1;
                    graph.add_edge(Edge::init(*u, *v));
                }
            }
        }

        let graph2 = graph.clone();
        for v in graph2.vertices() {
            let thresh = (1.0 - self.del) * (compute_s(*n) / *n as f64);
            if (graph.get_neighbors(v).map(|s| s.len()).unwrap_or_default() as f64) < thresh {
                graph.remove_vertex(v)
            }
        }

        graph
    }
}

type Vertex = u32;

type ColorSampling = HashSet<Color>;
pub struct StreamColoring {
    color_batches: HashMap<Vertex, (ColorSampling, ColorSampling, ColorSampling)>,
    chi: HashMap<Color, HashSet<Vertex>>,
    recovery: SparseRecovery<PowerFiniteFieldHasher>,
    pair_querier: PairQuerier,
    // Values
    vertices: HashSet<u32>,
    delta: u32,
}

impl Debug for StreamColoring {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "
            delta: {:?}
            recovery: {:?}",
            self.delta, self.recovery
        )
    }
}

impl StreamColoring {
    const EPSILON: f64 = 40.0;
    const ALPHA: f64 = 10000.0;
    /// Initiate a new StreamColoring instance under the ACK paper
    ///
    /// - *n* : Size of the graph (|V|)
    /// - *delta* : Maximum degree within the graph
    pub fn init(vertices: HashSet<&u32>, delta: u32) -> Self {
        let n = **(vertices.iter().max().unwrap_or(&&0));

        println!(
            "Minimum component size: {}",
            ((1.0 - Self::EPSILON / 10.0) * delta as f64)
        );
        let mut rng = rand::thread_rng();
        let bern = {
            let p = (Self::ALPHA as f64 * (n as f64).log2())
                / (3_f64 * Self::EPSILON.pow(2) * (delta as f64 + 1_f64));
            println!("[Stream Coloring]: {:?}", p);
            Bernoulli::new(p)
                .unwrap_or_else(|_| panic!("[StreamColoring] Bernoulli p value invalid: {}", p))
        };

        let pair_querier = PairQuerier::init(n, delta as u64, Self::EPSILON / 10.0);

        let mut color_batches: HashMap<u32, _> = Default::default();
        let mut chi = HashMap::<Color, HashSet<Vertex>>::default();

        for vertex in vertices.iter() {
            let mut sample = || -> HashSet<Color> {
                (0..(delta + 1))
                    .into_iter()
                    .filter(|color| {
                        if bern.sample(&mut rng) {
                            chi.entry((*color) as Color).or_default().insert(**vertex);
                            return true;
                        }
                        false
                    })
                    .map(|i| i as Color)
                    .collect()
            };

            color_batches.insert(**vertex, (sample(), sample(), sample()));
        }

        // Recovery data structure used to recover a subset of the edges
        let s = compute_s(n);
        println!("Creating sparse recovery");
        let recovery = SparseRecovery::init(binomial(n.into(), 2), s.ceil() as u64, 0.01);

        println!("[Stream Coloring]: Completed Initialization");

        Self {
            color_batches,
            chi,
            recovery,
            pair_querier,
            vertices: vertices.into_iter().copied().collect(),
            delta,
        }
    }

    pub fn feed(&mut self, token: (Edge<u32, ()>, bool)) {
        let (u, v) = token.0.vertices();
        let (batch1, batch2, batch3) = self
            .color_batches
            .get(u)
            .expect("This stream includes vertices that are not present in the graph");

        let check_match = |batch: &HashSet<Color>| -> bool {
            batch.iter().any(|c| {
                self.chi
                    .get(c)
                    .map(|set| set.contains(v))
                    .unwrap_or_default()
            })
        };

        if check_match(batch1) || check_match(batch2) || check_match(batch3) {
            self.recovery.feed((token.0.to_d1(), token.1));
        }
        self.pair_querier.feed(token);
    }

    pub fn query(self, actual_graph: &Graph<u32, ()>) -> Option<Coloring<u32>> {
        // Find a proper list coloring, where any color for v \in L(v)
        let Self {
            pair_querier,
            delta,
            recovery,
            color_batches,
            chi,
            vertices,
            ..
        } = self;

        let mut pair_querier = pair_querier;

        let result = recovery.query();

        if let SparseRecoveryOutput::Pass(result) = result {
            let conflict_graph = {
                let mut tmp: Graph<u32, ()> = Graph::default();
                for e in result.keys().map(|k| Edge::from_d1(*k)) {
                    tmp.add_edge(e)
                }
                tmp
            };

            println!("{}", &conflict_graph);
            let h = pair_querier.query();
            let del = Self::EPSILON / 10.0;

            let (v_sparse, conn_comp) = {
                let min_comp_size = ((1.0 - del) * delta as f64) as usize;

                let mut connected_components = ConnectedComponents::<u32, ()>::default();

                if let Some(start) = h.vertices().iter().next() {
                    h.breadth_first(&mut connected_components, vec![start]);
                }

                let comp_verts: HashSet<_> = connected_components
                    .data
                    .iter()
                    .filter(|g| g.vertices().len() >= min_comp_size)
                    .flat_map(|g| g.vertices())
                    .collect();

                let v_sparse: HashSet<_> = vertices
                    .into_iter()
                    .filter(|v| !comp_verts.contains(v))
                    .collect();

                println!("Sparse vertices: {:?}", v_sparse.len());
                println!(
                    "Connected Components: (min size: {}) {:?}",
                    min_comp_size,
                    connected_components.data.len()
                );

                (v_sparse, connected_components)
            };

            let coloring_sparse_vertices = {
                let batch1s: HashMap<_, HashSet<_>> = color_batches
                    .iter()
                    .map(|(k, batches)| (*k, batches.0.clone()))
                    .collect();

                // Something isn't right here, we SHOULD always be able to color with the sampled colors, maybe our probabilities are off.
                // Should copy code from Constraint Problem
                let mut coloring = HashMap::<u32, Color>::default();
                for v in v_sparse {
                    let color = batch1s.get(&v).cloned().and_then(|batch| {
                        let mut batch = batch;
                        if let Some(neighbors) = conflict_graph.get_neighbors(&v) {
                            for n in neighbors {
                                if let Some(neighbor_c) = coloring.get(&n.destination) {
                                    batch.remove(neighbor_c);
                                }
                            }
                        }
                        batch.into_iter().next()
                    });

                    if let Some(color) = color {
                        coloring.insert(v, color);
                    } else {
                        panic!("NO COLOR AVAILABLE")
                    }
                }
                coloring
            };

            let mut coloring = coloring_sparse_vertices;

            assert!(actual_graph.is_partial(&coloring));

            println!("{:?}", coloring);

            // ALmost CLiques Initial Coloring
            {
                // For each color (c), if we can find a pair of vertices u,v such that (u,v) is not in G_conflict,
                // u and v are not in the colorful matching yet, and L(u) and L(v) both contain (c),
                // then we add (u,v) with this color to the colorful matching. Hence, this phase also takes O(n) time.
                for comp in conn_comp.data.iter() {
                    // This loop takes O(∆) time, since each almost clique has size O(∆)
                    let vertices: &HashSet<_> = &comp.vertices().into_iter().copied().collect();
                    let mut colors_used: HashSet<_> = HashSet::new();

                    for v in vertices {
                        let (_, batch2, _) = color_batches.get(v).unwrap();

                        'inner: for (c, opts) in batch2
                            .iter()
                            .filter(|c| !colors_used.contains(c))
                            .map(|c| (c, chi.get(c).unwrap()))
                        {
                            for u in opts {
                                let edge: Edge<_, ()> = Edge::init(*u, *v);
                                if !conflict_graph.has_edge(&edge)
                                    && !coloring.contains_key(u)
                                    && !coloring.contains_key(v)
                                {
                                    coloring.insert(*u, *c);
                                    coloring.insert(*v, *c);
                                    colors_used.insert(c);
                                    break 'inner;

                                    // 30: 24,27,36,42,48,52,58,63,65,66,67,68,69,71,82,95,98
                                    // 30: 24,27,36,42,48,52,58,63,65,66,67,68,69,71,82,95,98
                                }
                            }
                        }
                    }

                    // O(∆) Time
                    // for c in 0..(delta as Color + 1) {
                    //     // If each vertex samples O(log n) colors, then this runs in O(log n) Time, and collects O(log n) vertices.
                    //     // A given vertex samples a given color with probability log 2 / delta, since the
                    //     let nodes = chi
                    //         .get(&c)
                    //         .unwrap_or_else(|| panic!("Bad Data"))
                    //         .intersection(vertices);

                    //     // This runs in polylog time, since nodes has size O(log n), this will run in O(log^2 n) time.
                    //     let mut edges_to_check: HashSet<_> =
                    //         nodes.clone().cartesian_product(nodes.clone()).collect();

                    //     'inner: while let Some((u, v)) = edges_to_check.iter().next().cloned() {
                    //         edges_to_check.remove(&(u, v));
                    //         let edge: Edge<_, ()> = Edge::init(*u, *v);

                    //         if !conflict_graph.has_edge(&edge)
                    //             && !coloring.contains_key(u)
                    //             && !coloring.contains_key(v)
                    //         {
                    //             coloring.insert(*u, c);
                    //             coloring.insert(*v, c);
                    //             break 'inner;

                    //             // 30: 24,27,36,42,48,52,58,63,65,66,67,68,69,71,82,95,98
                    //             // 30: 24,27,36,42,48,52,58,63,65,66,67,68,69,71,82,95,98
                    //         }
                    //     }
                    // }
                }
            };

            assert!(actual_graph.is_partial(&coloring));

            let complete = {
                for almost_clique in conn_comp.data.iter() {
                    let mut palette_graph = Graph::<u32, ()>::default();

                    // This takes O(∆) time since each almost_cliques has no more that (1 + 6*del) * delta vertices
                    let uncolored_vertices: HashSet<_> = almost_clique
                        .vertices()
                        .into_iter()
                        .filter(|v| !coloring.contains_key(v))
                        .copied()
                        .collect();

                    println!("{:?}", uncolored_vertices.len());

                    uncolored_vertices.iter().for_each(|v| {
                        color_batches.get(&v).unwrap().2.iter().for_each(|c| {
                            palette_graph.add_edge(Edge::init(*v, (*c).try_into().unwrap()))
                        })
                    });

                    for edge in conflict_graph.clone() {
                        let (u, v) = edge.vertices();

                        if let Some(c) = coloring.get(u) {
                            palette_graph.remove_edge(Edge::init(*v, *c as u32))
                        }
                        if let Some(c) = coloring.get(v) {
                            palette_graph.remove_edge(Edge::init(*u, *c as u32))
                        }
                    }
                    // Creating the Pallette Graph therefore takes O(∆ log2 n)
                    let matching = palette_graph.hopkroft_karp(Some(uncolored_vertices));

                    for edge in matching {
                        let (v, c) = edge.vertices();
                        coloring.insert(*v, *c as Color);
                    }
                }
                coloring
            };

            assert!(actual_graph.is_partial(&complete));

            return Some(complete);
        }

        None
    }
}

#[cfg(test)]
mod test {

    use std::fs;

    use super::*;
    use crate::random_graph::bernoulli::BernoulliGraphDistribution;

    fn test_graph() -> Graph<u32, ()> {
        let mut rng = rand::thread_rng();
        // Test for p = log n
        let n: f64 = 100.0;
        let p = 1.0 / n.log2();
        BernoulliGraphDistribution::<u32>::init(n as u32, p)
            .unwrap()
            .sample(&mut rng)
    }

    #[test]
    fn tester() {
        let graph = test_graph();

        // fs::write("./big_graph_100", format!("{}", graph)).unwrap();

        // let graph = fs::read_to_string("./big_graph_100").unwrap();
        // let graph: Graph<u32, ()> = graph.as_str().parse().unwrap();

        let delta = graph
            .adj_list()
            .iter()
            .map(|(_, n)| n.len())
            .max()
            .unwrap_or_default() as u32;

        println!("Delta: {:?}", &delta);

        let mut colorer = StreamColoring::init(
            // This should change, we should pass in the graph and it should deal with converting this into an "n"
            graph.vertices(),
            delta,
        );

        graph
            .clone()
            .into_iter()
            .for_each(|e| colorer.feed((e, true)));

        println!("Completed Stream");

        let coloring = colorer.query(&graph).unwrap();

        println!(
            "Colors Used: {:?}",
            coloring.values().into_iter().unique().count()
        );

        assert!(graph.is_proper(&coloring));
    }
}
