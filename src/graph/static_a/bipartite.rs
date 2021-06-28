use std::{collections::LinkedList, ops::Deref, str::FromStr};

use super::super::*;

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default,
    W: Hash + Eq + Clone + Default,
{
    fn is_bipartite(&self) -> bool {
        let Self {
            adjacency_list,
            vertices,
            ..
        } = self;

        let first = adjacency_list.keys().next();

        if first.is_none() {
            return false;
        };
        let first = first.unwrap();

        let mut visiting = LinkedList::<&T>::new();
        visiting.push_back(first);

        let mut sides = HashMap::<&T, bool>::new();

        loop {
            if let Some(current) = visiting.pop_back() {
                if !sides.contains_key(current) {
                    sides.insert(current, false);
                };
                let current_side = sides.get(current).cloned().unwrap();
                if let Some(neighbors) = self.get_neighbors(current) {
                    for neighbor in neighbors.iter() {
                        if let Some(old_side) = sides.insert(neighbor, !current_side) {
                            if old_side == current_side {
                                return false;
                            }
                        } else {
                            visiting.push_back(neighbor);
                        }
                    }
                }
            } else {
                if let Some(next) = vertices.iter().find(|v| !sides.contains_key(v)) {
                    visiting.push_back(next)
                } else {
                    return true;
                }
            }
        }
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + Copy + std::fmt::Debug + Default + From<String>,
    W: Hash + Eq + Clone + Default + PartialOrd,
{
    fn bipartite_augmenting_path(&self, matching: HashSet<Edge<T, W>>) -> Result<Vec<T>, String> {
        let Self {
            vertices,
            edges,
            adjacency_list,
        } = self;
        let mut sides = HashMap::<&T, bool>::new();

        let first = vertices.iter().next().ok_or("Empty Graph".to_string())?;
        let mut visiting = LinkedList::<&T>::new();
        visiting.push_back(first);

        loop {
            if let Some(current) = visiting.pop_back() {
                if !sides.contains_key(current) {
                    sides.insert(current, false);
                };
                let current_side = sides
                    .get(current)
                    .cloned()
                    .ok_or("Error fetching Current Side")?;
                if let Some(neighbors) = self.get_neighbors(current) {
                    for neighbor in neighbors {
                        if let Some(old_side) = sides.insert(neighbor, !current_side) {
                            if old_side == current_side {
                                return Err("Graph is not bipartite".into());
                            }
                        } else {
                            visiting.push_back(neighbor);
                        }
                    }
                }
            } else {
                if let Some(next) = self.vertices.iter().find(|v| !sides.contains_key(v)) {
                    visiting.push_back(next)
                } else {
                    break;
                }
            }
        }

        let source = T::from("source".to_string());
        let sink = T::from("sink".to_string());

        let vertices = vertices
            .clone()
            .into_iter()
            .chain(vec![source, sink])
            .collect();

        let mut added_edges = HashSet::<Edge<T, W>>::new();
        sides.iter().for_each(|(vertex, side)| {
            if *side {
                added_edges.insert(Edge {
                    v1: source.clone(),
                    v2: vertex.clone().clone(),
                    directed: Some(EdgeDirection::V1ToV2),
                    weight: None,
                });
            } else {
                added_edges.insert(Edge {
                    v1: vertex.clone().clone(),
                    v2: sink.clone(),
                    directed: Some(EdgeDirection::V1ToV2),
                    weight: None,
                });
            }
        });

        let mut edges: HashSet<Edge<T, W>> = edges
            .into_iter()
            .map(|edge| {
                let edge = edge.clone();
                if matching.contains(&edge) {
                    added_edges.remove_edges(&edge.v1);
                    added_edges.remove_edges(&edge.v2);
                    Edge {
                        directed: Some(EdgeDirection::V1ToV2),
                        ..edge
                    }
                } else {
                    Edge {
                        directed: Some(EdgeDirection::V2ToV1),
                        ..edge
                    }
                }
            })
            .collect();

        edges.extend(added_edges);

        let new_graph = Self {
            edges,
            vertices,
            adjacency_list: adjacency_list.clone(),
        };

        let shortest_path: Vec<_> = new_graph
            .shorted_path_to(&source, &sink)
            .ok_or("Could not get shorted path for some reason".to_string())?
            .into_iter()
            .cloned()
            .collect();

        Ok(shortest_path[1..shortest_path.len()].into())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_graph_not_bipartite() -> Graph<i32, i32> {
        let mut graph = HashMap::new();
        graph.insert(0, vec![1]);
        graph.insert(1, vec![0]);
        graph.insert(2, vec![3, 4]);
        graph.insert(3, vec![2, 4]);
        graph.insert(4, vec![2, 3]);
        Graph::from_adj_list(graph, None)
    }

    fn test_graph_bipartite() -> Graph<i32, i32> {
        let mut graph = HashMap::new();
        graph.insert(0, vec![1]);
        graph.insert(1, vec![0]);
        graph.insert(2, vec![3, 4]);
        graph.insert(3, vec![2, 5]);
        graph.insert(4, vec![2, 5]);
        graph.insert(5, vec![3, 4]);
        Graph::from_adj_list(graph, None)
    }

    #[test]
    fn test() {
        assert_eq!(test_graph_not_bipartite().is_bipartite(), false);
    }

    #[test]
    fn test_is() {
        assert_eq!(test_graph_bipartite().is_bipartite(), true)
    }
}
