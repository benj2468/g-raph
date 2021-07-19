//! Contains all things related to graphs

use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use priority_queue::PriorityQueue;

pub mod edge;

use edge::*;
#[derive(Debug, Clone)]
pub struct Graph<T, W>
where
    T: Hash + Eq,
{
    adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>,
    vertex_heap: Option<PriorityQueue<T, Reverse<usize>>>,
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default,
    W: Hash + Eq + Clone + Default + std::fmt::Debug,
{
    pub fn from_adj_list(matrix: HashMap<T, HashSet<(T, Option<W>)>>) -> Self {
        let adjacency_list = matrix
            .into_iter()
            .map(|(v, edges)| {
                (
                    v,
                    edges
                        .into_iter()
                        .map(|(dest, label)| EdgeDestination::init(dest, label))
                        .collect(),
                )
            })
            .collect();

        Self {
            adjacency_list,
            vertex_heap: None,
        }
    }

    pub fn with_vertex_recaller(self) -> Self {
        let mut queue = PriorityQueue::new();

        self.adjacency_list.iter().for_each(|(v, edges)| {
            queue.push(v.clone(), Reverse(edges.len()));
        });

        let mut new = self;

        new.vertex_heap.replace(queue);

        new
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
        let Self {
            adjacency_list,
            vertex_heap,
        } = self;
        if let Some(heap) = vertex_heap {
            let min = heap.pop();

            if let Some((vertex, _)) = min {
                let neighbors = adjacency_list.get(&vertex);
                if let Some(neighbors) = neighbors {
                    neighbors.iter().for_each(|neighbor| {
                        let destination = neighbor.destination();
                        let current = heap.get_priority(destination).unwrap().0;
                        heap.change_priority(destination, Reverse(current - 1));
                    })
                }
                self.remove_vertex(&vertex);

                return Some(vertex);
            };
        } else if let Some((vertex, _)) = self.min_degree() {
            self.remove_vertex(&vertex);
        }

        None
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
