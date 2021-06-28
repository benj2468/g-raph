use std::{
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
    iter::FromIterator,
};

#[derive(Debug)]
pub struct Graph<T, W> {
    vertices: HashSet<T>,
    edges: HashSet<Edge<T, W>>,
    adjacency_list: HashMap<T, HashSet<T>>,
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
    W: Hash + Eq + Clone + Default,
{
    pub fn from_adj_list(matrix: HashMap<T, Vec<T>>, directed: Option<EdgeDirection>) -> Self {
        let mut edges = HashSet::new();

        let adjacency_list: HashMap<T, HashSet<T>> = matrix
            .clone()
            .into_iter()
            .map(|(v, edges)| (v, edges.into_iter().collect()))
            .collect();

        let vertices = adjacency_list.keys().cloned().collect();

        adjacency_list.iter().for_each(|(v1, value)| {
            value.iter().for_each(|v2| {
                edges.insert(Edge {
                    v1: v1.clone(),
                    v2: v2.clone(),
                    directed,
                    ..Default::default()
                });
            })
        });

        Self {
            vertices,
            edges,
            adjacency_list,
        }
    }

    pub fn get_neighbors(&self, vertex: &T) -> Option<&HashSet<T>> {
        self.adjacency_list.get(vertex)
    }
}

pub trait ClearVertex<T, W> {
    fn remove_edges(&mut self, vertex: &T);
}

impl<T, W> ClearVertex<T, W> for HashSet<Edge<T, W>>
where
    T: PartialEq + Eq + Hash + Clone,
    W: Eq + Hash + Clone,
{
    fn remove_edges(&mut self, vertex: &T) {
        let removing_edges: Vec<_> = self
            .clone()
            .into_iter()
            .filter(|edge| !edge.on_vertex(vertex))
            .collect();

        for edge in removing_edges {
            self.remove(&edge);
        }
    }
}

mod static_a;
mod streaming;
