//! Different Graph Search Algorithms

use crate::graph::{edge::EdgeDestination, Graphed};
use std::{
    collections::{HashMap, HashSet, LinkedList},
    fmt::Debug,
    hash::Hash,
    ops::Add,
};

/// Allows for accomplishing various actions through DFS/BFS search algorithm
pub trait Searcher<T, W>: Default + Clone {
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
    fn breadth_first<S>(&self, searcher: &'s mut S, start: &T) -> &'s S
    where
        S: Searcher<T, W>;
    /// Standard Depth First Search
    ///
    /// Clearly described [here](https://www.geeksforgeeks.org/breadth-first-search-or-bfs-for-a-graph/)
    fn depth_first<S>(&self, searcher: &'s mut S, start: &T) -> &'s S
    where
        S: Searcher<T, W>;
}

impl<'s, G, T, W> Search<'s, T, W> for G
where
    G: Graphed<T, W>,
    T: Hash + Eq + PartialOrd + Clone + Debug,
    W: Hash + Eq + Clone + Default,
{
    fn breadth_first<S>(&self, searcher: &'s mut S, start: &T) -> &'s S
    where
        S: Searcher<T, W>,
    {
        let mut not_visited: HashSet<&T> = self.vertices();
        let mut to_visit: LinkedList<&T> = vec![start].into_iter().collect();

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

        searcher
    }

    fn depth_first<S>(&self, searcher: &'s mut S, start: &T) -> &'s S
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

        searcher
    }
}

/// Structure for maintaining backtracking data in a DFS or BFS search
#[derive(Default, Clone, Debug)]
pub struct BackTracking<T, W>(HashMap<T, (T, W)>);

impl<T, W> Searcher<T, W> for BackTracking<T, W>
where
    T: Default + Eq + Hash + Clone + Debug + Copy,
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

#[cfg(test)]
mod test {
    use std::collections::{HashMap, HashSet};

    use super::*;

    use crate::graph::{EdgeDestination, Graph};
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

        let mut backtracking = BackTracking {
            ..Default::default()
        };

        let back = Graph::new(adj).breadth_first(&mut backtracking, &3);

        let expected: Vec<u32> = vec![3, 2, 1, 0];

        assert_eq!(expected, back.shortest_path(0));
    }
}
