//! Contains all things related to graphs

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    hash::Hash,
    str::FromStr,
};

use itertools::Itertools;
use priority_queue::PriorityQueue;

#[doc(hidden)]
pub mod edge;

#[doc(inline)]
pub use edge::*;

/// A graph is, conceptually, a tuple G = (V, E), where:
///
/// V \subset R i.e. The graphs set of vertices
/// E = {(u,v,w) : u,v \in V, w \in W} i.e. The graphs set of edges
/// W = Set of possible weights
///
/// In our implementation, a graph is stored as an adjacency list form, or a HashMap of vertices to a set of EdgeDestinations. A graph is generic over the type of vertex, `T`, and the type of the edges weight: `W`
pub trait Graphed<T, W>: Clone + Sized + Debug {
    /// Create a new Graph, given the adjacency list of that graph
    fn new(adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>) -> Self;
    /// Fetch the adjacency list of a graph
    fn adj_list(&self) -> &HashMap<T, HashSet<EdgeDestination<T, W>>>;
    /// Get all vertices in a graph
    fn vertices(&self) -> HashSet<&T>;
    /// Get the neighbors of a provided vertex
    fn get_neighbors(&self, vertex: &T) -> Option<&HashSet<EdgeDestination<T, W>>>;
    /// Add an edge to a graph, if the vertices of the edge do not exist, the edge is not added.
    fn add_edge(&mut self, edge: Edge<T, W>);
    /// Remove an edge from a graph
    fn remove_edge(&mut self, edge: Edge<T, W>);
    /// Remove a vertex, and all of it's incident edges from the graph
    fn remove_vertex(&mut self, vertex: &T);
    /// Fetch the minimum degree of a graph
    fn min_degree(&self) -> Option<(T, usize)>;
    /// Remove the vertex of minimum degree
    fn remove_min(&mut self) -> Option<T>;
    /// Check if the graph is empty.
    fn is_empty(&self) -> bool;
    /// Has edge
    fn has_edge(&self, edge: &Edge<T, W>) -> bool;
}

/// Simple Graph
///
/// Simplest version of a Graph that contains just the Adjacency list, where each destination may or may not have an edge weight associated with it.
#[derive(Debug, Clone, Default)]
pub struct Graph<T, W>
where
    T: Hash + Eq,
{
    adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>,
}

/// A more comprehensive Graph representation
///
/// This graph also holds a PriorityQueue to keep track of vertex degrees.

#[derive(Clone, Debug, Default)]
pub struct GraphWithRecaller<T, W>
where
    T: Hash + Eq,
{
    graph: Graph<T, W>,
    /// Component of the graph that keeps track of degree orderings
    vertex_heap: PriorityQueue<T, Reverse<usize>>,
}

impl<T, W> From<Graph<T, W>> for GraphWithRecaller<T, W>
where
    T: Hash + Eq + Clone,
    W: Clone,
{
    /// Add a vertex recaller structure to our Graph
    ///
    /// This enabled us to always know the vertex of minimum degree
    ///
    /// Runtime: `O(nlog(n))`
    fn from(graph: Graph<T, W>) -> Self {
        let mut queue = PriorityQueue::new();

        graph
            .adjacency_list
            .clone()
            .into_iter()
            .for_each(|(v, edges)| {
                queue.push(v, Reverse(edges.len()));
            });

        Self {
            graph,
            vertex_heap: queue,
        }
    }
}

impl<T, W> Graphed<T, W> for GraphWithRecaller<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd,
    W: Debug + Hash + Eq + Clone + Default,
{
    fn adj_list(&self) -> &HashMap<T, HashSet<EdgeDestination<T, W>>> {
        self.graph.adj_list()
    }
    /// Runtime: O(nlog(n))
    fn new(adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>) -> Self {
        let graph = Graph::new(adjacency_list);
        graph.into()
    }

    /// Runtime: O(1)
    fn vertices(&self) -> HashSet<&T> {
        self.graph.vertices()
    }

    /// Runtime: O(nlog(n))
    fn add_edge(&mut self, edge: Edge<T, W>) {
        self.graph.add_edge(edge.clone());
        let (v1, v2) = edge.vertices();

        self.vertex_heap.push_decrease(
            v1.clone(),
            Reverse(self.graph.get_neighbors(v1).unwrap().len()),
        );

        if !edge.directed {
            self.vertex_heap.push_decrease(
                v2.clone(),
                Reverse(self.graph.get_neighbors(v2).unwrap().len()),
            );
        }
    }

    fn remove_edge(&mut self, edge: Edge<T, W>) {
        self.graph.remove_edge(edge.clone());
        let (v1, v2) = edge.vertices();
        self.vertex_heap.push_increase(
            v1.clone(),
            Reverse(self.graph.get_neighbors(v1).unwrap().len()),
        );
        self.vertex_heap.push_increase(
            v2.clone(),
            Reverse(self.graph.get_neighbors(v2).unwrap().len()),
        );
    }
    /// Runtime: O(1)
    fn get_neighbors(&self, vertex: &T) -> Option<&HashSet<EdgeDestination<T, W>>> {
        self.graph.get_neighbors(vertex)
    }
    /// Runtime: O(1)
    fn is_empty(&self) -> bool {
        self.graph.is_empty()
    }
    /// Runtime: O(1)
    fn min_degree(&self) -> Option<(T, usize)> {
        self.vertex_heap.peek().map(|(v, r)| (v.clone(), r.0))
    }
    ///
    /// Runtime: O(nlog(n)); where n = number of neighbors
    fn remove_vertex(&mut self, vertex: &T) {
        let Self { graph, vertex_heap } = self;

        if let Some(neighbors) = graph.adjacency_list.get(&vertex) {
            neighbors.iter().for_each(|neighbor| {
                let destination = &neighbor.destination;
                if let Some(current) = vertex_heap.get_priority(destination).cloned() {
                    vertex_heap.change_priority(destination, Reverse(current.0 - 1));
                }
            })
        }
        self.graph.remove_vertex(&vertex);
    }
    /// Runtime: O(nlog(n))
    fn remove_min(&mut self) -> Option<T> {
        let Self { vertex_heap, .. } = self;
        if let Some((vertex, _)) = vertex_heap.pop() {
            self.remove_vertex(&vertex);
            return Some(vertex);
        }

        None
    }

    fn has_edge(&self, edge: &Edge<T, W>) -> bool {
        self.graph.has_edge(edge)
    }
}

impl<T, W> Graphed<T, W> for Graph<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd,
    W: Debug + Hash + Eq + Clone + Default,
{
    fn adj_list(&self) -> &HashMap<T, HashSet<EdgeDestination<T, W>>> {
        &self.adjacency_list
    }
    /// Runtime: O(1)
    fn new(adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>) -> Self {
        Self { adjacency_list }
    }

    /// Runtime: O(1)
    fn vertices(&self) -> HashSet<&T> {
        self.adjacency_list.keys().into_iter().collect()
    }

    /// Runtime: O(1)
    fn get_neighbors(&self, vertex: &T) -> Option<&HashSet<EdgeDestination<T, W>>> {
        self.adjacency_list.get(vertex)
    }

    /// Runtime: O(n^2)
    fn remove_vertex(&mut self, vertex: &T) {
        let neighbors = self.get_neighbors(vertex).cloned();
        let Self { adjacency_list } = self;

        adjacency_list.remove(vertex);

        if let Some(neighbors) = neighbors {
            neighbors.iter().for_each(|neighbor| {
                if let Some(edges) = adjacency_list.get_mut(&neighbor.destination) {
                    edges.retain(|edge| &edge.destination != vertex)
                };
            })
        }
    }

    /// Runtime: O(n)
    fn min_degree(&self) -> Option<(T, usize)> {
        let (mut vertex, mut min) = (
            self.adjacency_list.keys().next().cloned(),
            self.adjacency_list.values().next().map(|e| e.len()),
        );

        self.adjacency_list.iter().for_each(|(ver, edges)| {
            let len = edges.len();
            if let Some(min_value) = min {
                if len < min_value {
                    vertex.replace(ver.clone());
                    min.replace(len);
                };
            }
        });
        vertex.and_then(|v| min.map(|m| (v, m)))
    }

    /// Runtime: O(1)
    fn is_empty(&self) -> bool {
        self.adjacency_list.is_empty()
    }

    /// Runtime: O(1)
    fn add_edge(&mut self, edge: Edge<T, W>) {
        let graph = &mut self.adjacency_list;
        let (u, v) = edge.vertices();

        if !graph.contains_key(&u) {
            graph.insert(u.clone(), HashSet::new());
        }
        graph.get_mut(&u).unwrap().insert((&edge).into());

        if !edge.directed {
            if !graph.contains_key(&v) {
                graph.insert(v.clone(), HashSet::new());
            }
            graph.get_mut(&v).unwrap().insert((&edge.reverse()).into());
        }
    }

    fn remove_edge(&mut self, edge: Edge<T, W>) {
        let graph = &mut self.adjacency_list;
        let (u, v) = edge.vertices();

        graph
            .entry(u.clone())
            .and_modify(|set| set.retain(|dest| dest.destination != *v));

        graph
            .entry(v.clone())
            .and_modify(|set| set.retain(|dest| dest.destination != *u));

        if graph.get(u).map(|u| u.is_empty()).unwrap_or_default() {
            graph.remove(u);
        }
        if graph.get(v).map(|v| v.is_empty()).unwrap_or_default() {
            graph.remove(v);
        }
    }

    /// Runtime: O(n^2)
    fn remove_min(&mut self) -> Option<T> {
        self.min_degree().map(|(vertex, _)| {
            self.remove_vertex(&vertex);
            vertex
        })
    }

    /// Runtime: O(delta)
    fn has_edge(&self, edge: &Edge<T, W>) -> bool {
        let (u, v) = edge.vertices();
        self.adjacency_list
            .get(u)
            .and_then(|neighbors| neighbors.iter().find(|dest| dest.destination == *v))
            .is_some()
    }
}

fn from_str<G, T, W>(s: &str) -> Result<G, <T as FromStr>::Err>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
    G: Graphed<T, W>,
{
    let mut graph = G::new(Default::default());
    s.lines().into_iter().try_for_each(|line| {
        let mut split = line.split(':');
        split.next().unwrap().trim().parse().and_then(|vertex: T| {
            split
                .next()
                .unwrap()
                .trim()
                .split(',')
                .try_for_each(|neighbor| -> Result<(), _> {
                    neighbor.parse().map(|neighbor| {
                        let edge = Edge::<T, W>::init_directed(vertex.clone(), neighbor);
                        graph.add_edge(edge);
                    })
                })
        })
    })?;
    Ok(graph)
}

fn to_str<G, T, W>(graph: &G) -> String
where
    T: Debug + Hash + Eq + Clone + PartialOrd + Display + Ord,
    W: Debug + Hash + Eq + Clone,
    G: Graphed<T, W>,
{
    graph
        .adj_list()
        .iter()
        .sorted_by_key(|(a, _)| *a)
        .map(|(v, entry)| {
            let set = entry
                .iter()
                .map(|n| format!("{}", n.destination))
                .sorted()
                .join(",");
            format!("{}: {}", v, set)
        })
        .join("\n")
}

impl<T, W> std::str::FromStr for Graph<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str(s)
    }
}

impl<T, W> std::str::FromStr for GraphWithRecaller<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
{
    type Err = <T as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str(s)
    }
}

impl<T, W> Display for Graph<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + Display + Ord,
    W: Debug + Hash + Eq + Clone + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "--- Graph ---\n{}\n---\n", to_str(self))
    }
}

impl<T, W> Display for GraphWithRecaller<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + Display + Ord,
    W: Debug + Hash + Eq + Clone + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.graph)
    }
}

impl<T, W> Iterator for Graph<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
{
    type Item = Edge<T, W>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self { adjacency_list } = self;

        let v1 = adjacency_list.iter().next().map(|e| e.0.clone());

        if let Some(v1) = v1 {
            let v2 = adjacency_list.get(&v1).and_then(|set| set.iter().next());

            if let Some(v2) = v2 {
                let edge = Edge::init(v1, v2.destination.clone());
                self.remove_edge(edge.clone());
                Some(edge)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl<T, W> Iterator for GraphWithRecaller<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
{
    type Item = Edge<T, W>;

    fn next(&mut self) -> Option<Self::Item> {
        self.graph.next().map(|next| {
            self.remove_edge(next.clone());
            next
        })
    }
}

impl<T, W> PartialEq for Graph<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
{
    fn eq(&self, other: &Self) -> bool {
        let edges: HashSet<Edge<T, W>> = self.clone().into_iter().collect();
        let other: HashSet<Edge<T, W>> = other.clone().into_iter().collect();

        edges.difference(&other).count() == 0
    }
}

impl<T, W> Graph<T, W>
where
    T: Debug + Hash + Eq + Clone + PartialOrd + FromStr,
    W: Debug + Hash + Eq + Clone + Default,
{
    pub fn induce(self, vertices: HashSet<&T>) -> Self {
        let mut new = self.clone();

        for v in self.vertices().clone() {
            if !vertices.contains(v) {
                new.remove_vertex(v)
            }
        }

        new
    }
}

pub mod static_a;
pub mod streaming;

#[cfg(test)]
mod test {
    use std::default;

    use super::*;

    #[test]
    fn new_graph() {
        let mut graph = GraphWithRecaller::<u32, ()>::new(Default::default());

        graph.add_edge(Edge::init(1, 2));
        graph.add_edge(Edge::init(2, 3));
        graph.add_edge(Edge::init(1, 3));

        assert_eq!(
            graph.vertex_heap.get_priority(&1).unwrap(),
            &Reverse(2_usize)
        );
        assert_eq!(
            graph.vertex_heap.get_priority(&2).unwrap(),
            &Reverse(2_usize)
        );
        assert_eq!(
            graph.vertex_heap.get_priority(&3).unwrap(),
            &Reverse(2_usize)
        );
    }

    #[test]
    fn graph_to_iter() {
        let graph: Graph<u32, ()> = r"0: 1
        1: 0,2
        2: 1,5
        5: 2
        3: 4
        4: 3"
            .parse()
            .unwrap();
    }
}
