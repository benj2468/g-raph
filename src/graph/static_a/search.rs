//! Different Graph Search Algorithms

use crate::graph::Graph;
use std::{
    collections::{HashMap, LinkedList},
    fmt::Debug,
    hash::Hash,
};

/// Allows for accomplishing various different actions through a DFS/BFS search algorithm
pub trait Searcher<T>: Default {
    /// Called when there are no more vertices in the current search scope, and we need to look for a new, unconnected unvisited vertex
    fn new_component(&mut self);
    /// Called before visiting a node. This is called BEFORE a node is processed. It is called when we find a new node from a source node.
    ///
    /// i.e. when we pop it from the stack/queue
    ///
    /// - *source*: The node that was just popped from the list
    /// - *node*: The node that is a neighbor of source.
    fn visit(&mut self, source: &T, node: &T);
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + PartialOrd + Clone + Debug,
    W: Hash + Eq + Clone,
{
    /// Performs Breadth First Search on a Graph
    fn breadth_first<S>(&self) -> S
    where
        S: Searcher<T>,
    {
        let mut visited: HashMap<&T, bool> =
            self.adjacency_list.keys().map(|key| (key, false)).collect();

        let mut to_visit = LinkedList::<&T>::new();

        let mut searcher = S::default();

        loop {
            if let Some(current) = to_visit.pop_front() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        let destination = neighbor.destination();
                        if *visited.get(destination).unwrap() {
                            searcher.visit(current, destination);
                            to_visit.push_back(destination);
                        }
                    }
                }

                visited.insert(current, true);
            } else {
                if let Some((next, _)) = visited.iter().find(|(_, visited)| !**visited) {
                    to_visit.push_back(next);
                    searcher.new_component();
                } else {
                    break;
                }
            }
        }

        searcher
    }

    /// Performs Depth First Search on a Graph
    fn depth_first<S>(&self) -> S
    where
        S: Searcher<T>,
    {
        let mut visited: HashMap<&T, bool> =
            self.adjacency_list.keys().map(|key| (key, false)).collect();

        let mut to_visit = LinkedList::<&T>::new();

        let mut searcher = S::default();

        loop {
            if let Some(current) = to_visit.pop_back() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        let destination = neighbor.destination();
                        if *visited.get(destination).unwrap() {
                            searcher.visit(current, destination);
                            to_visit.push_back(destination);
                        }
                    }
                }

                visited.insert(current, true);
            } else {
                if let Some((next, _)) = visited.iter().find(|(_, visited)| !**visited) {
                    to_visit.push_back(next);
                    searcher.new_component();
                } else {
                    break;
                }
            }
        }

        searcher
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use super::*;

    use crate::graph::EdgeDestination;
    fn test_graph() -> Graph<u32, ()> {
        let mut adj: HashMap<u32, _> = HashMap::new();
        adj.insert(
            0,
            vec![
                EdgeDestination::init(1, Some(())),
                EdgeDestination::init(2, Some(())),
            ]
            .into_iter()
            .collect(),
        );
        adj.insert(
            1,
            vec![
                EdgeDestination::init(0, Some(())),
                EdgeDestination::init(2, Some(())),
            ]
            .into_iter()
            .collect(),
        );
        adj.insert(
            1,
            vec![
                EdgeDestination::init(0, Some(())),
                EdgeDestination::init(2, Some(())),
            ]
            .into_iter()
            .collect(),
        );
        adj.insert(
            2,
            vec![EdgeDestination::init(3, Some(()))]
                .into_iter()
                .collect(),
        );
        adj.insert(
            3,
            vec![EdgeDestination::init(2, Some(()))]
                .into_iter()
                .collect(),
        );
        adj.insert(
            4,
            vec![EdgeDestination::init(5, Some(()))]
                .into_iter()
                .collect(),
        );
        adj.insert(
            5,
            vec![EdgeDestination::init(4, Some(()))]
                .into_iter()
                .collect(),
        );
        Graph::new(adj)
    }
}
