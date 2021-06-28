use super::super::*;
use std::collections::{HashMap, LinkedList};

#[derive(Debug)]
pub struct ShortestPaths<T>(HashMap<T, Vec<T>>);

impl<T> AsRef<HashMap<T, Vec<T>>> for ShortestPaths<T> {
    fn as_ref(&self) -> &HashMap<T, Vec<T>> {
        &self.0
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default,
    W: Hash + Eq + Clone + Default,
{
    pub fn shorted_path_from<'a>(&'a self, source: &'a T) -> ShortestPaths<&'a T> {
        let mut paths = HashMap::<&T, Vec<&T>>::new();

        let mut visiting = LinkedList::<&T>::new();
        visiting.push_back(source);
        paths.insert(source, vec![]);

        loop {
            if let Some(current) = visiting.pop_back() {
                let current_path = paths.get(current).cloned().unwrap_or_default();
                if let Some(neighbors) = self.get_neighbors(current) {
                    neighbors.into_iter().for_each(|neighbor| {
                        if paths
                            .get(neighbor)
                            .map(|v| v.len() > current_path.len() + 1)
                            .unwrap_or(true)
                        {
                            let mut new_path = current_path.clone();
                            new_path.push(current);

                            paths.insert(neighbor, new_path);
                            visiting.push_back(neighbor);
                        }
                    })
                }
            } else {
                break;
            }
        }

        ShortestPaths(paths)
    }

    pub fn shorted_path_to<'a>(&'a self, source: &'a T, destination: &'a T) -> Option<Vec<&'a T>> {
        let mut previous = HashMap::<&T, Option<&T>>::new();

        let mut visiting = LinkedList::<&T>::new();
        visiting.push_back(source);
        previous.insert(source, None);

        'bfs: loop {
            if let Some(current) = visiting.pop_back() {
                if let Some(neighbors) = self.get_neighbors(current) {
                    for neighbor in neighbors {
                        if previous.contains_key(neighbor) {
                            continue;
                        }
                        previous.insert(neighbor, Some(current));
                        visiting.push_back(neighbor);
                        if neighbor == destination {
                            break 'bfs;
                        }
                    }
                }
            } else {
                return None;
            }
        }

        let mut path = vec![destination];
        while let Some(prev) = previous.get(path.last().unwrap()).unwrap() {
            path.push(prev)
        }

        Some(path)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    fn test_graph() -> Graph<i32, i32> {
        let mut graph = HashMap::new();
        graph.insert(0, vec![1]);
        graph.insert(1, vec![0]);
        graph.insert(2, vec![3, 4]);
        graph.insert(3, vec![2, 4]);
        graph.insert(4, vec![2, 3]);
        Graph::from_adj_list(graph, None)
    }

    #[test]
    fn test() {
        let graph = test_graph();
        let res = graph.shorted_path_from(&2);

        let shortest_paths = res.as_ref();

        assert_eq!(shortest_paths.get(&3).unwrap(), &vec![&2]);
        assert_eq!(shortest_paths.get(&4).unwrap(), &vec![&2]);

        let res = graph.shorted_path_from(&1);

        let shortest_paths = res.as_ref();

        assert_eq!(shortest_paths.get(&0).unwrap(), &vec![&1]);
        assert!(shortest_paths.get(&1).unwrap().is_empty());
    }

    #[test]
    fn shorted_path_to() {
        let mut graph = HashMap::new();
        graph.insert(0, vec![1]);
        graph.insert(1, vec![0]);
        graph.insert(2, vec![3, 4]);
        graph.insert(3, vec![2, 4]);
        graph.insert(4, vec![2, 3, 5]);
        graph.insert(5, vec![4]);
        let graph: Graph<i32, i32> = Graph::from_adj_list(graph, None);

        let shorted_path = graph.shorted_path_to(&2, &5);

        assert_eq!(shorted_path, Some(vec![&5, &4, &2]))
    }
}
