//! Contains all things related to graphs

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use priority_queue::PriorityQueue;

pub mod edge;

use edge::*;

/// A graph is, conceptually, a tuple G = (V, E), where:
///
/// V \subset R i.e. The graphs set of vertices
/// E = {(u,v,w) : u,v \in V, w \in W} i.e. The graphs set of edges
/// W = Set of possible weights
///
/// In our implementation, a graph is stored as an adjacency list form, or a HashMap of vertices to a set of EdgeDestinations. A graph is generic over the type of vertex, `T`, and the type of the edges weight: `W`
#[derive(Debug, Clone)]
pub struct Graph<T, W>
where
    T: Hash + Eq,
{
    adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>,
}

pub struct GraphWithRecaller<T, W>
where
    T: Hash + Eq,
{
    graph: Graph<T, W>,
    /// Component of the graph that keeps track of degree orderings, not instantialized unless requested
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
    /// ## Example
    /// ```
    /// let mut map = HashMap::new();
    /// map.insert(0, vec![1,2].into_iter().collect());
    /// map.insert(1, vec![0].into_iter().collect());
    /// map.insert(2, vec![0].into_iter().collect());
    /// let graph = Graph::new(map).with_vertex_recaller();
    ///
    /// let min = graph.
    /// ```
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

impl<T, W> GraphWithRecaller<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default + PartialOrd,
    W: Hash + Eq + Clone + Default + std::fmt::Debug,
{
    fn remove_min(&mut self) -> Option<T> {
        let Self { graph, vertex_heap } = self;
        let min = vertex_heap.pop();

        if let Some((vertex, _)) = min {
            let neighbors = graph.adjacency_list.get(&vertex);
            if let Some(neighbors) = neighbors {
                neighbors.iter().for_each(|neighbor| {
                    let destination = neighbor.destination();
                    let current = vertex_heap.get_priority(destination).unwrap().0;
                    vertex_heap.change_priority(destination, Reverse(current - 1));
                })
            }
            graph.remove_vertex(&vertex);

            return Some(vertex);
        };

        None
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + PartialOrd,
    W: Hash + Eq + Clone + Default,
{
    /// Create a new graph from an Adjacency List
    pub fn new(adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>) -> Self {
        Self { adjacency_list }
    }

    pub fn get_neighbors(&self, vertex: &T) -> Option<&HashSet<EdgeDestination<T, W>>> {
        self.adjacency_list.get(vertex)
    }

    pub fn remove_vertex(&mut self, vertex: &T) {
        self.adjacency_list.remove(vertex);

        self.adjacency_list
            .iter_mut()
            .for_each(|(vertex, edges)| edges.retain(|edge| edge.destination() == vertex));
    }

    /// This is an O(n), if we were to back the graph with a priority queue we could speed this up, but for now who cares
    /// Returns None if the graph has no vertices
    pub fn min_degree(&self) -> Option<(T, usize)> {
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

    pub fn max_degree(&self) -> Option<(&T, usize)> {
        let (mut vertex, mut min) = (
            self.adjacency_list.keys().next(),
            self.adjacency_list.values().next().map(|e| e.len()),
        );

        self.adjacency_list.iter().for_each(|(ver, edges)| {
            let len = edges.len();
            if let Some(min_value) = min {
                if len > min_value {
                    vertex.replace(ver);
                    min.replace(len);
                };
            }
        });
        vertex.and_then(|v| min.map(|m| (v, m)))
    }

    pub fn is_empty(&self) -> bool {
        self.adjacency_list.is_empty()
    }

    pub fn remove_min(&mut self) -> Option<T> {
        self.min_degree().map(|(vertex, _)| {
            self.remove_vertex(&vertex);
            vertex
        })
    }

    pub fn add_edge(&mut self, edge: Edge<T, W>) {
        let graph = &mut self.adjacency_list;
        let (u, v) = edge.vertices();

        if !graph.contains_key(&u) {
            graph.insert(u.clone(), HashSet::new());
        }
        graph.get_mut(&u).unwrap().insert((&edge).into());

        if !edge.is_directed() {
            if !graph.contains_key(&v) {
                graph.insert(v.clone(), HashSet::new());
            }
            graph.get_mut(&v).unwrap().insert((&!edge).into());
        }
    }
}

#[doc(hidden)]
pub mod static_a;
pub mod streaming;
