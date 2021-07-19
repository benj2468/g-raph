use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    f32::INFINITY,
    hash::Hash,
};

use priority_queue::PriorityQueue;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EdgeDestination<T, W> {
    pub destination: T,
    label: W,
}

#[derive(Debug, Clone)]
pub struct Graph<T, W>
where
    T: Hash + Eq,
{
    adjacency_list: HashMap<T, HashSet<EdgeDestination<T, W>>>,
    vertex_heap: Option<PriorityQueue<T, Reverse<usize>>>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum EdgeDirection {
    V1ToV2,
    V2ToV1,
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone)]
pub struct Edge<T, W> {
    v1: T,
    v2: T,
    directed: Option<EdgeDirection>,
    weight: Option<W>,
}

impl<T, W> Edge<T, W>
where
    T: Eq + PartialEq,
{
    pub fn on_vertex(&self, vertex: &T) -> bool {
        self.v1 == *vertex || self.v2 == *vertex
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default,
    W: Hash + Eq + Clone + Default + std::fmt::Debug,
{
    pub fn from_adj_list(
        matrix: HashMap<T, HashSet<(T, W)>>,
        directed: Option<EdgeDirection>,
    ) -> Self {
        let adjacency_list = matrix
            .clone()
            .into_iter()
            .map(|(v, edges)| {
                (
                    v,
                    edges
                        .into_iter()
                        .map(|(dest, label)| EdgeDestination {
                            destination: dest,
                            label,
                        })
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

        let mut new = self.clone();

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
            .for_each(|(vertex, edges)| edges.retain(|edge| edge.destination == *vertex));
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
                        let destination = &neighbor.destination;
                        let current = heap.get_priority(destination).unwrap().0;
                        heap.change_priority(destination, Reverse(current - 1));
                    })
                }
                self.remove_vertex(&vertex);

                return Some(vertex);
            };
        } else {
            if let Some((vertex, _)) = self.min_degree() {
                self.remove_vertex(&vertex);
            }
        }

        None
    }
}

pub mod static_a;
pub mod streaming;
