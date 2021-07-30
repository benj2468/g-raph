//! Contains all things related to graphs

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    hash::Hash,
};

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
pub trait Graphed<T, W>: Clone {
    fn new(adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>) -> Self;
    fn vertices(&self) -> HashSet<&T>;
    fn get_neighbors(&self, vertex: &T) -> Option<&HashSet<EdgeDestination<T, W>>>;
    fn add_edge(&mut self, edge: Edge<T, W>);
    fn remove_vertex(&mut self, vertex: &T);
    fn min_degree(&self) -> Option<(T, usize)>;
    fn remove_min(&mut self) -> Option<T>;
    fn is_empty(&self) -> bool;
}

/// Simple Graph
///
/// Simplest version of a Graph that contains just the Adjacency list, where each destination may or may not have an edge weight associated with it.
#[derive(Debug, Clone)]
pub struct Graph<T, W>
where
    T: Hash + Eq,
{
    adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>,
}

/// A more comprehensive Graph representation
///
/// This graph also holds a PriorityQueue to keep track of vertex degrees.

#[derive(Clone)]
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
    T: Hash + Eq + Clone + PartialOrd,
    W: Hash + Eq + Clone + Default,
{
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
                let current = vertex_heap.get_priority(destination).unwrap().0;
                vertex_heap.change_priority(destination, Reverse(current - 1));
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
}

impl<T, W> Graphed<T, W> for Graph<T, W>
where
    T: Hash + Eq + Clone + PartialOrd,
    W: Hash + Eq + Clone + Default,
{
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

        if !graph.contains_key(&v) {
            graph.insert(v.clone(), HashSet::new());
        }
        graph.get_mut(&v).unwrap().insert((&edge.reverse()).into());
    }

    /// Runtime: O(n^2)
    fn remove_min(&mut self) -> Option<T> {
        self.min_degree().map(|(vertex, _)| {
            self.remove_vertex(&vertex);
            vertex
        })
    }
}

pub mod static_a;
pub mod streaming;
