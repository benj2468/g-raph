use std::{
    collections::{HashMap, HashSet, LinkedList},
    fmt::{Debug, Display},
    hash::Hash,
};

use super::search::{Search, Searcher};
use crate::graph::{Edge, Graphed};

type Matching<T, W> = HashSet<Edge<T, W>>;
type SideMatching<T> = HashMap<T, T>;

pub trait MatchingT<T, W> {
    fn hopkroft_karp(&self, left: Option<HashSet<T>>) -> Matching<T, W>;
}

pub trait AugmentingPath<'m, T, W> {
    fn find_augmenting_paths(
        &self,
        left_side: &HashSet<T>,
        matching: (&'m SideMatching<T>, &'m SideMatching<T>),
    ) -> HashSet<Vec<T>>;
}

impl<'m, G, T, W> AugmentingPath<'m, T, W> for G
where
    G: Graphed<T, W>,
    T: Hash + Eq + PartialEq + Clone + Debug,
{
    fn find_augmenting_paths(
        &self,
        left_side: &HashSet<T>,
        matching: (&'m SideMatching<T>, &'m SideMatching<T>),
    ) -> HashSet<Vec<T>> {
        let (left, right) = matching;

        let is_matched = |v: &T| left.contains_key(v) || right.contains_key(v);
        let edge_status = |u: &T, v: &T| {
            left.get(v)
                .or_else(|| right.get(v))
                .map(|m| m == u)
                .unwrap_or_default()
        };

        let mut not_visited: HashSet<&T> = self.vertices();
        let first = left_side.iter().find(|v| !is_matched(v));
        if first.is_none() || not_visited.is_empty() {
            return Default::default();
        }

        let mut to_visit: LinkedList<&T> = vec![first.unwrap()].into_iter().collect();
        let mut backtracking = HashMap::<T, T>::new();
        let mut paths = HashSet::<Vec<T>>::new();

        loop {
            if let Some(current) = to_visit.pop_front() {
                if let Some(neighbors) = self.get_neighbors(&current) {
                    for neighbor in neighbors.iter() {
                        let next = &neighbor.destination;
                        let next_edge_status = edge_status(current, next);

                        if let Some(previous) = backtracking.get(current) {
                            if next == previous {
                                continue;
                            }
                            let prev_edge_status = edge_status(previous, current);
                            if prev_edge_status != next_edge_status && not_visited.contains(next) {
                                backtracking.insert(next.clone(), current.clone());
                                to_visit.push_front(next);
                            } else if !prev_edge_status {
                                let mut node = Some(current.clone());
                                let mut path = vec![];
                                while let Some(cur) = node {
                                    path.push(cur.clone());
                                    node = backtracking.get(&cur).cloned();
                                }
                                paths.insert(path);
                                to_visit.clear();
                                backtracking.clear();
                                break;
                            }
                        } else if !next_edge_status && not_visited.contains(next) {
                            backtracking.insert(next.clone(), current.clone());
                            to_visit.push_front(next);
                        }
                    }
                    if neighbors.len() == 1 && backtracking.contains_key(current) {
                        if let Some(neigh) = neighbors.iter().next() {
                            let previous = &neigh.destination;
                            let previous_edge_value = edge_status(previous, current);
                            if !previous_edge_value {
                                let mut node = Some(current.clone());
                                let mut path = vec![];
                                while let Some(cur) = node {
                                    path.push(cur.clone());
                                    node = backtracking.get(&cur).cloned();
                                }
                                path.reverse();
                                paths.insert(path);
                                to_visit.clear();
                                backtracking.clear();
                            }
                        }
                    }
                    not_visited.remove(current);
                }
            } else if let Some(next) = left_side
                .iter()
                .find(|v| !is_matched(v) && not_visited.contains(v))
            {
                to_visit.push_back(next);
            } else {
                break;
            }
        }
        paths
    }
}

#[derive(Debug)]
pub struct Bipartite<T> {
    left: HashSet<T>,
    right: HashSet<T>,
}

impl<T, W> Searcher<T, W> for Bipartite<T>
where
    T: Clone + Debug + Eq + Hash,
{
    fn new_component(&mut self, node: &T) {
        self.left.insert(node.clone());
    }
    fn visit(&mut self, source: &T, node: &crate::graph::EdgeDestination<T, W>) {
        if self.left.get(source).is_some() {
            self.right.insert(node.destination.clone());
        } else {
            self.left.insert(node.destination.clone());
        }
    }
}

pub trait Nil {
    fn nil() -> Self;
}

impl<G, T, W> MatchingT<T, W> for G
where
    G: Graphed<T, W> + Display,
    T: Hash + Eq + PartialOrd + Clone + Debug,
    W: Hash + Eq + Clone + Default,
{
    fn hopkroft_karp(&self, left: Option<HashSet<T>>) -> Matching<T, W> {
        let mut left_matching = HashMap::<T, T>::default();
        let mut right_matching = HashMap::<T, T>::default();

        let left = match left {
            Some(l) => l,
            None => {
                let mut bipartite = Bipartite {
                    right: Default::default(),
                    left: Default::default(),
                };

                self.breadth_first(&mut bipartite, vec![self.vertices().iter().next().unwrap()]);
                bipartite.left
            }
        };

        loop {
            let augmenting_paths =
                self.find_augmenting_paths(&left, (&left_matching, &right_matching));
            if augmenting_paths.is_empty() {
                break;
            }
            for path in augmenting_paths.iter() {
                for edge in path.rchunks(2) {
                    let v_left = &edge[0];
                    let v_right = &edge[1];
                    left_matching.insert(v_left.clone(), v_right.clone());
                    right_matching.insert(v_right.clone(), v_left.clone());
                }
            }
        }

        left_matching
            .into_iter()
            .map(|(k, v)| Edge::init(k, v))
            .collect()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::graph::Graph;

    fn test_graph() -> Graph<u32, ()> {
        r"0: 3,4
        1: 3
        2: 4,7
        3: 0,1
        4: 0,2
        5: 6
        6: 5,7
        7: 2,6"
            .parse()
            .unwrap()
    }

    #[test]
    fn test() {
        let graph = test_graph();
        println!("{}", graph);

        let matching = graph.hopkroft_karp(None);

        println!("Matching: {:?}", matching);
    }
}
