//! Supporting Edge Definitions

use roots::find_roots_quadratic;
use std::fmt::Debug;

/// Undirected Edge
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct Edge<T, W> {
    /// If directed, the source
    v1: T,
    /// If directed, the destination
    v2: T,
    /// The weight, or any label associated with the edge
    label: W,
    // Whether or not the edge is directed
    pub directed: bool,
}

impl<T, W> Edge<T, W>
where
    T: Eq + PartialOrd,
    W: Default,
{
    /// Creates an edge given two vertices
    pub fn init(v1: T, v2: T) -> Self {
        Self {
            v1,
            v2,
            label: W::default(),
            directed: false,
        }
    }

    /// Creates a directed edge given two vertices
    pub fn init_directed(v1: T, v2: T) -> Self {
        Self {
            v1,
            v2,
            label: W::default(),
            directed: true,
        }
    }

    /// Updates the label, or places in a label if none exists
    pub fn update_label(&mut self, new: W) {
        self.label = new;
    }

    /// Determines whether a vertex is incident to an edge
    pub fn is_incident(&self, vertex: &T) -> bool {
        self.v1 == *vertex || self.v2 == *vertex
    }

    /// Returns the vertices indicent to an edge
    pub fn vertices(&self) -> (&T, &T) {
        (&self.v1, &self.v2)
    }

    /// Reverse the direction of the edge, but swapping v1 and v2
    pub fn reverse(self) -> Self {
        Self {
            v1: self.v2,
            v2: self.v1,
            label: self.label,
            directed: self.directed,
        }
    }
}

impl<W> Edge<u32, W>
where
    W: Default,
{
    /// Creates an edge from a 1-dimensional space value, assuming a total possible number of edges being n Choose 2
    ///
    /// Assumes default weight
    pub fn from_d1(d1: u64) -> Self {
        let roots = find_roots_quadratic(0.5, -0.5, -(d1 as f64));
        match roots {
            roots::Roots::Two([_, root]) => {
                let max = (root / 1.0) as u64;
                let min = d1 - ((max * (max - 1)) / 2);
                Self {
                    v1: min as u32,
                    v2: max as u32,
                    label: W::default(),
                    directed: false,
                }
            }
            _ => panic!(
                "The quadratic didn't have two roots, while it should have {:?}",
                roots
            ),
        }
    }

    /// Converts an edge in `n Choose 2` space to `n` space, provided a number of vertices in the graph
    pub fn to_d1(&self) -> u64 {
        let (min, max) = self.vertices_ord();

        Self::formula(min, max)
    }

    #[doc(hidden)]
    pub fn vertices_ord(&self) -> (&u32, &u32) {
        if self.v1 <= self.v2 {
            (&self.v1, &self.v2)
        } else {
            (&self.v2, &self.v1)
        }
    }

    #[doc(hidden)]
    fn formula(min: &u32, max: &u32) -> u64 {
        if *max == 0 {
            return 0;
        }
        (*max as u64 * ((*max as u64) - 1) / 2) + *min as u64
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn from_d1() {
        for i in 0..100 {
            let edge = Edge::<u32, ()>::from_d1(i);
            let d1 = edge.to_d1();
            assert_eq!(i, d1);
        }
    }
}

/// The destination of an edge, used in an adjacency list representation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EdgeDestination<T, W> {
    pub destination: T,
    pub label: W,
}

impl<T, W> EdgeDestination<T, W>
where
    W: Default,
{
    pub fn init_with_label(destination: T, label: W) -> Self {
        Self { destination, label }
    }
    pub fn init(destination: T) -> Self {
        Self {
            destination,
            label: W::default(),
        }
    }
}

impl<T, W> From<&Edge<T, W>> for EdgeDestination<T, W>
where
    T: Clone + Eq + PartialOrd,
    W: Clone + Default,
{
    fn from(edge: &Edge<T, W>) -> Self {
        EdgeDestination {
            destination: edge.v2.clone(),
            label: edge.label.clone(),
        }
    }
}
