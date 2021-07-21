//! Module for Edge Definitions

use std::ops::Not;

/// An Edge is defined as a relation between two vertices. It may, or may not, have a direction, and may or may not have a label.
#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub struct Edge<T, W> {
    /// If directed, the source
    v1: T,
    /// If directed, the destination
    v2: T,
    /// Whether or not the vertex is directed
    directed: bool,
    /// The weight, or any label associated with the edge
    label: Option<W>,
}

impl<T, W> Not for Edge<T, W> {
    type Output = Edge<T, W>;
    fn not(self) -> Self::Output {
        Self {
            v1: self.v2,
            v2: self.v1,
            directed: self.directed,
            label: self.label,
        }
    }
}

impl<T, W> Edge<T, W>
where
    T: Eq + PartialEq + PartialOrd,
{
    /// Creates an edge given two vertices
    pub fn init(v1: T, v2: T) -> Self {
        Self {
            v1,
            v2,
            directed: false,
            label: None,
        }
    }

    /// Creates a directed edge between two vertices
    pub fn init_directed(v1: T, v2: T) -> Self {
        Self {
            v1,
            v2,
            directed: true,
            label: None,
        }
    }

    /// Updates the label, or places in a label if none exists
    pub fn update_label(&mut self, new: W) {
        self.label.replace(new);
    }

    /// Determines whether a vertex is incident to an edge
    pub fn is_incident(&self, vertex: &T) -> bool {
        self.v1 == *vertex || self.v2 == *vertex
    }

    pub fn is_directed(&self) -> bool {
        self.directed
    }

    /// Returns the vertices incident to an edge, (source, dest) if directed
    pub fn vertices(&self) -> (&T, &T) {
        if self.v1 <= self.v2 {
            (&self.v1, &self.v2)
        } else {
            (&self.v2, &self.v1)
        }
    }
}

impl<W> Edge<u32, W> {
    /// Creates an edge from a 1-dimensional space value, assuming a total possible number of edges being n^2
    pub fn from_d1(d1: u64) -> Self {
        let (mut min, mut max): (u32, u32) = (0, 0);
        loop {
            if Self::formula(&min, &max) == d1 {
                break;
            }
            if min == max {
                max += 1;
                min = 0;
            } else {
                min += 1;
            }
        }

        Self {
            v1: min as u32,
            v2: max as u32,
            directed: false,
            label: None,
        }
    }

    /// Converts an edge in `N^2` space to `N` space, provided a number of vertices in the graph
    pub fn to_d1(&self) -> u64 {
        let (min, max) = self.vertices();

        Self::formula(min, max)
    }

    fn formula(min: &u32, max: &u32) -> u64 {
        if *max == 0 {
            return 0;
        }
        *max as u64 * ((*max as u64) - 1) / 2 + *min as u64
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn from_d1() {
        let edge = Edge::<u32, ()> {
            v1: 4,
            v2: 5,
            directed: false,
            label: None,
        };

        let d1 = edge.to_d1();

        assert_eq!(d1, 14);

        let edge = Edge::<u32, ()>::from_d1(14);

        assert_eq!(edge.vertices(), (&4, &5));
    }
}

/// The destination of an edge, used in an adjacency list representation
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct EdgeDestination<T, W> {
    destination: T,
    label: Option<W>,
}

impl<T, W> EdgeDestination<T, W> {
    pub fn init(destination: T, label: Option<W>) -> Self {
        Self { destination, label }
    }

    pub fn destination(&self) -> &T {
        &self.destination
    }
}

impl<T, W> From<&Edge<T, W>> for EdgeDestination<T, W>
where
    T: Clone,
    W: Clone,
{
    fn from(edge: &Edge<T, W>) -> Self {
        EdgeDestination {
            destination: edge.v2.clone(),
            label: edge.label.clone(),
        }
    }
}
