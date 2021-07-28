//! Different Graph Search Algorithms

use crate::graph::{edge::EdgeDestination, Graph};
use std::{
    collections::{HashMap, LinkedList},
    fmt::Debug,
    hash::Hash,
    ops::Add,
};

/// Allows for accomplishing various different actions through a DFS/BFS search algorithm
pub trait Searcher<T, W>: Default + Clone {
    fn start(&self) -> Option<&T> {
        None
    }
    /// Called when there are no more vertices in the current search scope, and we need to look for a new, unconnected & unvisited vertex
    fn new_component(&mut self);
    /// Called before visiting a node. This is called BEFORE a node is processed. It is called when we find a new node from a source node.
    ///
    /// i.e. when we pop it from the stack/queue
    ///
    /// - *source*: The node that was just popped from the list
    /// - *node*: The node that is a neighbor of source, including it's weight
    fn visit(&mut self, source: &T, node: &EdgeDestination<T, W>);
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + PartialOrd + Clone + Debug,
    W: Hash + Eq + Clone + Default,
{
    /// Performs Breadth First Search on a Graph
    fn breadth_first<S>(&self, mut searcher: S) -> S
    where
        S: Searcher<T, W>,
    {
        let mut visited: HashMap<&T, bool> =
            self.adjacency_list.keys().map(|key| (key, false)).collect();

        let mut to_visit = LinkedList::<&T>::new();

        let searcher2 = searcher.clone();

        if let Some(start) = searcher2.start() {
            to_visit.push_back(start);
        }

        loop {
            if let Some(current) = to_visit.pop_front() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        let destination = neighbor.destination();
                        searcher.visit(current, neighbor);
                        if !visited.get(destination).unwrap() {
                            to_visit.push_back(destination);
                        }
                    }
                }
                visited.insert(current, true);
            } else if let Some((next, _)) = visited.iter().find(|(_, visited)| !**visited) {
                to_visit.push_back(next);
                searcher.new_component();
            } else {
                break;
            }
        }

        searcher
    }

    /// Performs Depth First Search on a Graph
    fn depth_first<S>(&self, mut searcher: S) -> S
    where
        S: Searcher<T, W>,
    {
        let mut visited: HashMap<&T, bool> =
            self.adjacency_list.keys().map(|key| (key, false)).collect();

        let mut to_visit = LinkedList::<&T>::new();

        let searcher2 = searcher.clone();
        if let Some(start) = searcher2.start() {
            to_visit.push_back(start)
        }

        loop {
            if let Some(current) = to_visit.pop_back() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors {
                        let destination = neighbor.destination();
                        searcher.visit(current, neighbor);
                        if !visited.get(destination).unwrap() {
                            to_visit.push_back(destination);
                        }
                    }
                }

                visited.insert(current, true);
            } else if let Some((next, _)) = visited.iter().find(|(_, visited)| !**visited) {
                to_visit.push_back(next);
                searcher.new_component();
            } else {
                break;
            }
        }

        searcher
    }
}

#[derive(Default, Clone, Debug)]
pub struct BackTracking<T, W> {
    pub start: Option<T>,
    pub tracking: HashMap<T, (T, W)>,
}

impl<T, W> Searcher<T, W> for BackTracking<T, W>
where
    T: Default + Eq + Hash + Clone + Debug,
    W: Default + Eq + Hash + Clone + Add<Output = W> + PartialOrd + Debug,
{
    fn start(&self) -> Option<&T> {
        self.start.as_ref()
    }
    fn new_component(&mut self) {}
    fn visit(&mut self, source: &T, node: &EdgeDestination<T, W>) {
        let current = self
            .tracking
            .get(source)
            .map(|(_, w)| w.clone())
            .unwrap_or_default();

        let next_label = node.weight().clone();
        let destination = node.destination();

        let next_weight = current + next_label;
        if let Some((vertex, score)) = self.tracking.get_mut(destination) {
            if next_weight < *score {
                *vertex = source.clone();
                *score = next_weight
            }
        } else {
            self.tracking
                .insert(destination.clone(), (source.clone(), next_weight));
        }
    }
}

impl<T, W> BackTracking<T, W>
where
    T: Eq + Hash + Clone,
{
    pub fn shortest_path(&self, target: T) -> Vec<T> {
        let mut path = vec![];

        let mut current = &target;
        while let Some((node, _)) = self.tracking.get(current) {
            path.push(node.clone());
            current = node;
        }
        path.reverse();
        path.push(target);

        path
    }
}

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use super::*;

    use crate::graph::EdgeDestination;
    #[test]
    fn test_graph() {
        let mut adj: HashMap<u32, HashSet<EdgeDestination<u32, u32>>> = HashMap::new();
        adj.insert(
            0,
            vec![EdgeDestination::init_with_label(2, 4)]
                .into_iter()
                .collect(),
        );
        adj.insert(
            1,
            vec![EdgeDestination::init_with_label(0, 2)]
                .into_iter()
                .collect(),
        );
        adj.insert(
            2,
            vec![
                EdgeDestination::init_with_label(1, 1),
                EdgeDestination::init_with_label(0, 10),
            ]
            .into_iter()
            .collect(),
        );
        adj.insert(
            3,
            vec![EdgeDestination::init_with_label(2, 4)]
                .into_iter()
                .collect(),
        );
        adj.insert(
            4,
            vec![EdgeDestination::init_with_label(5, 0)]
                .into_iter()
                .collect(),
        );
        adj.insert(
            5,
            vec![EdgeDestination::init_with_label(4, 0)]
                .into_iter()
                .collect(),
        );

        let back = Graph::new(adj).breadth_first(BackTracking {
            start: Some(3),
            ..Default::default()
        });

        let expected: Vec<u32> = vec![3, 2, 1, 0];

        assert_eq!(expected, back.shortest_path(0));
    }
}
