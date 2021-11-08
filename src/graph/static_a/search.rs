//! Different Graph Search Algorithms

use crate::graph::{edge::EdgeDestination, Edge, Graph, Graphed};
use std::{
    collections::{HashMap, HashSet, LinkedList},
    f32::INFINITY,
    fmt::Debug,
    hash::Hash,
    ops::Add,
};

/// Allows for accomplishing various actions through DFS/BFS search algorithm
pub trait Searcher<T, W> {
    /// Called when there are no more vertices in the current search scope, and we need to look for a new, unconnected & unvisited vertex
    ///
    /// - *node*: The node that caused the new component
    fn new_component(&mut self, node: &T);
    /// This is called BEFORE a node is processed. It is called when we find a new node from a source node.
    ///
    /// i.e. when we pop it from the stack/queue
    ///
    /// - *source*: The node that was just popped from the stack/queue
    /// - *node*: The node that is a neighbor of source, including it's label
    fn visit(&mut self, source: &T, node: &EdgeDestination<T, W>);
}

/// Search functions on a graph
pub trait Search<'s, T, W> {
    /// Standard Breadth First Search
    ///
    /// Clearly described [here](https://www.geeksforgeeks.org/breadth-first-search-or-bfs-for-a-graph/)
    fn breadth_first<S>(&self, searcher: &'s mut S, start: Vec<&T>)
    where
        S: Searcher<T, W>;
    /// Standard Depth First Search
    ///
    /// Clearly described [here](https://www.geeksforgeeks.org/breadth-first-search-or-bfs-for-a-graph/)
    fn depth_first<S>(&self, searcher: &'s mut S, start: &T)
    where
        S: Searcher<T, W>;
}

impl<'s, G, T, W> Search<'s, T, W> for G
where
    G: Graphed<T, W>,
    T: Hash + Eq + PartialOrd + Clone + Debug,
    W: Hash + Eq + Clone + Default,
{
    fn breadth_first<S>(&self, searcher: &'s mut S, start: Vec<&T>)
    where
        S: Searcher<T, W>,
    {
        let mut not_visited: HashSet<&T> = self.vertices();
        let mut to_visit: LinkedList<&T> = start.into_iter().collect();

        loop {
            if let Some(current) = to_visit.pop_front() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        let destination = &neighbor.destination;
                        searcher.visit(current, neighbor);
                        if not_visited.get(destination).is_some() {
                            to_visit.push_back(destination);
                        }
                    }
                }
                not_visited.remove(current);
            } else if let Some(next) = not_visited.iter().next() {
                to_visit.push_back(next);
                searcher.new_component(next);
            } else {
                break;
            }
        }
    }

    fn depth_first<S>(&self, searcher: &'s mut S, start: &T)
    where
        S: Searcher<T, W>,
    {
        let mut not_visited: HashSet<&T> = self.vertices();
        let mut to_visit: LinkedList<&T> = vec![start].into_iter().collect();

        loop {
            if let Some(current) = to_visit.pop_back() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        let destination = &neighbor.destination;
                        searcher.visit(current, neighbor);
                        if not_visited.get(destination).is_some() {
                            to_visit.push_back(destination);
                        }
                    }
                }

                not_visited.remove(current);
            } else if let Some(next) = not_visited.iter().next() {
                to_visit.push_back(next);
                searcher.new_component(next);
            } else {
                break;
            }
        }
    }
}

/// Structure for maintaining backtracking data in a DFS or BFS search
#[derive(Default, Clone, Debug)]
pub struct BackTracking<T, W>(HashMap<T, (T, W)>);

impl<T, W> Searcher<T, W> for BackTracking<T, W>
where
    T: Eq + Hash + Clone + Debug + Copy,
    W: Default + Eq + Hash + Clone + Add<Output = W> + PartialOrd + Debug + Copy,
{
    fn new_component(&mut self, _node: &T) {}
    fn visit(&mut self, source: &T, node: &EdgeDestination<T, W>) {
        let current_label = self.0.get(source).map(|(_, w)| *w);

        let next_label = node.label;
        let destination = &node.destination;

        let next_weight = current_label.map(|c| c + next_label).unwrap_or(next_label);
        self.0
            .get_mut(&destination)
            .map(|(vertex, score)| {
                if next_weight < *score {
                    *vertex = *source;
                    *score = next_weight
                }
            })
            .unwrap_or_else(|| {
                self.0.insert(*destination, (*source, next_weight));
            });
    }
}
impl<T, W> BackTracking<T, W>
where
    T: Eq + Hash + Clone,
{
    pub fn shortest_path(&self, target: T) -> Vec<T> {
        let mut path = vec![];

        let mut current = &target;
        while let Some((node, _)) = self.0.get(current) {
            path.push(node.clone());
            current = node;
        }
        path.reverse();
        path.push(target);

        path
    }
}

#[derive(Clone, Debug)]
pub struct ConnectedComponents<T, W>
where
    T: Default + Clone + Eq + Hash + Debug,
    W: Default + Clone,
{
    pub data: Vec<Graph<T, W>>,
}

impl<T, W> Default for ConnectedComponents<T, W>
where
    T: Default + Clone + Eq + Hash + Debug,
    W: Default + Clone,
{
    fn default() -> Self {
        ConnectedComponents {
            data: vec![Default::default()],
        }
    }
}

impl<T, W> Searcher<T, W> for ConnectedComponents<T, W>
where
    T: Default + Clone + Eq + Hash + Debug + PartialOrd,
    W: Default + Clone + Hash + Eq + Debug,
{
    fn new_component(&mut self, _node: &T) {
        let Self { data, .. } = self;
        data.push(Default::default())
    }

    fn visit(&mut self, source: &T, node: &EdgeDestination<T, W>) {
        if let Some(last) = self.data.last_mut() {
            last.add_edge(Edge::init(source.clone(), node.destination.clone()));
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use super::*;

    use crate::graph::{EdgeDestination, Graph};
    #[test]
    fn test_graph() {
        let graph: Graph<u32, u32> = r"2: 1,0
        5: 4
        4: 5
        1: 0
        0: 2
        3: 2"
            .parse()
            .unwrap();

        let mut backtracking = BackTracking {
            ..Default::default()
        };

        graph.breadth_first(&mut backtracking, vec![&3]);

        let expected: Vec<u32> = vec![3, 2, 0];

        assert_eq!(expected, backtracking.shortest_path(0));
    }

    #[test]
    fn connected_components() {
        let graph: Graph<u32, ()> = r"0: 1
        1: 0,2
        2: 1,5
        5: 2
        3: 4
        4: 3"
            .parse()
            .unwrap();

        let mut conn = ConnectedComponents::default();

        graph.breadth_first(&mut conn, vec![&0]);

        let expected_subgraph: Graph<u32, ()> = r"4: 3
        3: 4"
            .parse()
            .unwrap();

        assert_eq!(conn.data[1], expected_subgraph);
    }
}
