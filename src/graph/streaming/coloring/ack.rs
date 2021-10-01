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
        streaming::{sparse_recovery::s_sparse::SparseRecovery, Query, Stream},
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
};

type Color = usize;

const EPSILON: f64 = 10.0;
const ALPHA: f64 = 1000.0;

// const EPSILON: f64 = 1_f64 / 5000_f64;
// const ALPHA: u64 = 5000_u64.pow(2);

#[derive(Debug)]
pub struct PairQuerier {
    // Data
    pub inner: HashMap<u32, SparseRecovery<PowerFiniteFieldHasher>>,

    // Metadata
    n: u32,
    delta: u64,
    p: f32,
}

impl PairQuerier {
    fn init(n: u32, p: f32, delta: u64) -> Self {
        let bern = Bernoulli::new(p as f64)
            .unwrap_or_else(|_| panic!("[PairQuerier] Invalid Probability: {}", p));

        let mut rng = rand::thread_rng();

        // Pick a set S of vertices at the beginning of the stream by choosing each vertex
        // independently with probability p.
        //
        // For any chosen vertex in S, run the algorithm in Proposition 4.2(Sparse Recovery) with P
        // being the set of all edge slots incident to the vertex and k = delta
        let inner: HashMap<u32, _> = (0..n)
            .into_iter()
            .filter(|_| bern.sample(&mut rng))
            .map(|v| (v, SparseRecovery::init(n.into(), delta, 0.01)))
            .collect();

        Self { n, inner, p, delta }
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
        let Self { n, delta, .. } = self;
        // This is E_s
        let mut queried: HashMap<u32, HashMap<u64, i64>> = self
            .inner
            .iter_mut()
            // Probably do not want to clone here
            .map(|(k, stream)| (*k, stream.clone().query().unwrap_or_default()))
            .collect();

        let mut graph = Graph::default();
        let mut d_prelim: HashMap<u32, u32> = HashMap::default();

        let del = EPSILON / 10.0;

        for v in queried.keys() {
            for u in queried.keys() {
                if u == v {
                    continue;
                }
                let u_neighbors = queried.get(u);
                let v_neighbors = queried.get(v);

                let thresh = (1.0 - (1.5 * del)) as f64 * (self.delta as f64) * (self.p as f64);
                // let thresh = 0.0;

                let add = u_neighbors
                    .zip(v_neighbors)
                    .map(|(u, v)| {
                        let u: HashSet<&u64> = u.keys().collect();
                        let v: HashSet<&u64> = v.keys().collect();

                        u.intersection(&v).count() as f64 >= thresh
                    })
                    .unwrap_or_default();
                if add {
                    *d_prelim.entry(*u).or_default() += 1;
                    *d_prelim.entry(*v).or_default() += 1;
                    graph.add_edge(Edge::init(*u, *v));
                }
            }
        }

        let graph2 = graph.clone();
        for v in graph2.vertices() {
            // let thresh = (1.0 - del) * (compute_s(*n) / *n as f64);
            let thresh = 0_f64;
            if (graph.get_neighbors(v).map(|s| s.len()).unwrap_or_default() as f64) < thresh {
                graph.remove_vertex(v)
            }
        }

        graph
    }
}

type Vertex = u32;

type ColorSampling = HashSet<Color>;

#[derive(Debug)]
pub struct StreamColoring {
    color_batches: HashMap<Vertex, (ColorSampling, ColorSampling, ColorSampling)>,
    chi: HashMap<Color, HashSet<Vertex>>,
    recovery: SparseRecovery<PowerFiniteFieldHasher>,
    pair_querier: PairQuerier,
    // Values
    vertices: HashSet<u32>,
    p: f32,
    delta: u32,
}

impl StreamColoring {
    /// Initiate a new StreamColoring instance under the ACK paper
    ///
    /// - *n* : Size of the graph (|V|)
    /// - *delta* : Maximum degree within the graph
    pub fn init(vertices: HashSet<&u32>, delta: u32) -> Self {
        let n = **(vertices.iter().max().unwrap_or(&&0));
        let mut rng = rand::thread_rng();
        let bern = {
            // This needs to be a toggled value
            // This probability value makes no sense
            let p = (ALPHA as f64 * (n as f64).log2())
                / (3_f64 * EPSILON.pow(2) * (delta as f64 + 1_f64));
            // let p = 1_f64 / (n as f64).log2();
            // let p = 1.0;
            Bernoulli::new(p)
                .unwrap_or_else(|_| panic!("[StreamColoring] Bernoulli p value invalid: {}", p))
        };

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
        let recovery = SparseRecovery::init(binomial(n.into(), 2), s.ceil() as u64, 0.01);

        let del = EPSILON / 10.0;

        let p = (10.0 * (n as f32).log2()) / (del.pow(2) as f32 * (delta as f32));
        // let p = 1_f32 / (n as f32).log2();

        let pair_querier = PairQuerier::init(n, p, delta as u64);

        println!(
            r"Initialization: {{
                n: {:?},
                delta: {:?},
                p: {:?}
            }}",
            n, delta, p
        );

        Self {
            color_batches,
            chi,
            recovery,
            pair_querier,
            vertices: vertices.into_iter().copied().collect(),
            p,
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

    pub fn query(self) -> Option<Coloring<u32>> {
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

        if let Some(result) = recovery.query() {
            let result_graph = {
                let mut tmp: Graph<u32, ()> = Graph::default();
                for e in result.keys().map(|k| Edge::from_d1(*k)) {
                    tmp.add_edge(e)
                }
                tmp
            };

            let h = pair_querier.query();
            let del = EPSILON / 10.0;

            let (v_sparse, conn_comp) = {
                let max_comp_size = ((1.0 - del) * delta as f64) as usize;

                let mut connected_components =
                    ConnectedComponents::<u32, ()>::default().with_max_size(max_comp_size);

                if let Some(start) = h.vertices().iter().next() {
                    h.breadth_first(&mut connected_components, vec![start]);
                }

                let comp_verts: HashSet<_> = connected_components
                    .data
                    .iter()
                    .flat_map(|g| g.vertices())
                    .collect();

                let v_sparse: HashSet<_> = vertices
                    .into_iter()
                    .filter(|v| !comp_verts.contains(v))
                    .collect();

                (v_sparse, connected_components)
            };

            let mut used_colors = HashSet::new();

            let coloring_sparse_vertices = {
                let batch1: HashMap<_, HashSet<_>> = color_batches
                    .iter()
                    .map(|(k, batches)| (*k, batches.0.clone()))
                    .collect();

                let mut coloring = HashMap::<u32, Color>::default();
                for v in v_sparse {
                    let color = batch1.get(&v).cloned().and_then(|batch| {
                        let mut batch = batch;
                        if let Some(neighbors) = result_graph.get_neighbors(&v) {
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
                        used_colors.insert(color);
                    } else {
                        panic!("NO COLOR AVAILABLE")
                    }
                }
                coloring
            };

            let almost_cliques_initial_coloring = {
                // For each color (c), if we can find a pair of vertices u,v such that (u,v) is not in G_conflict,
                // u and v are not in the colorful matching yet, and L(u) and L(v) both contain (c),
                // then we add (u,v) with this color to the colorful matching. Hence, this phase also takes O(n) time.
                let mut colorful_matching = HashMap::<Edge<u32, ()>, Color>::default();
                for comp in conn_comp.data.iter() {
                    let vertices = &comp.vertices().into_iter().copied().collect();
                    for c in 0..(delta as Color + 1) {
                        let nodes = chi
                            .get(&c)
                            .unwrap_or_else(|| panic!("Bad Data"))
                            .intersection(vertices);

                        if let Some(edge) =
                            nodes.clone().cartesian_product(nodes).find_map(|(u, v)| {
                                let edge: Edge<_, ()> = Edge::init(*u, *v);
                                (result_graph.has_edge(&edge)
                                    && !colorful_matching.contains_key(&edge))
                                .then(|| edge)
                            })
                        {
                            colorful_matching.insert(edge, c);
                            used_colors.insert(c);
                        }
                    }
                }
                colorful_matching
            };

            let mut coloring = {
                let mut coloring = coloring_sparse_vertices;
                for (edge, color) in almost_cliques_initial_coloring {
                    let (v1, v2) = edge.vertices();
                    coloring.insert(*v1, color);
                    coloring.insert(*v2, color);
                }
                coloring
            };

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

                    uncolored_vertices.iter().for_each(|v| {
                        color_batches
                            .get(&v)
                            .unwrap()
                            .2
                            .iter()
                            .filter(|c| !used_colors.contains(c))
                            // This loop takes O(log2 n) time since each batch samples O(log(n)) colors.
                            .for_each(|c| {
                                palette_graph.add_edge(Edge::init(*v, (*c).try_into().unwrap()))
                            });
                    });

                    // Creating the Pallette Graph therefore takes O(∆ log2 n)
                    let matching = palette_graph.hopkroft_karp(Some(uncolored_vertices));

                    for edge in matching {
                        let (v, c) = edge.vertices();
                        coloring.insert(*v, *c as Color);
                        used_colors.insert(*c as Color);
                    }
                }
                coloring
            };

            return Some(complete);
        }

        None
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::random_graph::bernoulli::BernoulliGraphDistribution;

    fn test_graph() -> Graph<u32, ()> {
        let mut rng = rand::thread_rng();
        let mut graph = Graph::default();
        for (edge, _) in BernoulliGraphDistribution::<u32>::init(500, 0.5)
            .unwrap()
            .sample(&mut rng)
        {
            graph.add_edge(edge)
        }
        graph
    }

    #[test]
    fn tester() {
        let graph = test_graph();

        let mut colorer = StreamColoring::init(
            // This should change, we should pass in the graph and it should deal with converting this into an "n"
            graph.vertices(),
            graph
                .adj_list()
                .iter()
                .map(|(_, n)| n.len())
                .max()
                .unwrap_or_default() as u32,
        );

        graph
            .clone()
            .into_iter()
            .for_each(|e| colorer.feed((e, true)));

        let coloring = colorer.query();

        assert!(graph.is_proper(coloring.unwrap()));
    }
}
