use super::super::*;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::vec;

#[derive(Debug)]
pub struct ConnectedComponents<T>(Vec<HashSet<T>>);

impl<T> AsRef<Vec<HashSet<T>>> for ConnectedComponents<T> {
    fn as_ref(&self) -> &Vec<HashSet<T>> {
        &self.0
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default,
    W: Hash + Eq + Clone + Default,
{
    pub fn connected_components(&self) -> Result<ConnectedComponents<&T>, String> {
        let Self {
            adjacency_list,
            vertices,
            ..
        } = self;

        let mut visited = HashSet::<&T>::new();

        let first = adjacency_list
            .keys()
            .next()
            .ok_or("Empty Graph".to_string())?;

        let mut visiting = LinkedList::<&T>::new();
        visiting.push_back(first);

        let mut components = vec![HashSet::<&T>::new()];

        loop {
            let current_component = components.last_mut().unwrap();
            if let Some(current) = visiting.pop_back() {
                current_component.insert(current);
                visited.insert(current);
                if let Some(neighbors) = self.get_neighbors(current) {
                    neighbors.into_iter().for_each(|vertex| {
                        if !visited.contains(vertex) {
                            visiting.push_back(vertex)
                        }
                    })
                }
            } else {
                if let Some(next) = vertices.iter().find(|v| !visited.contains(v)) {
                    components.push(HashSet::<&T>::new());
                    visiting.push_back(next)
                } else {
                    break;
                }
            }
        }

        Ok(ConnectedComponents(components))
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
    fn test_size() {
        let graph = test_graph();

        let connected_components = graph.connected_components().unwrap();

        assert_eq!(connected_components.as_ref().len(), 2)
    }

    #[test]
    fn test_content() {
        let graph = test_graph();

        let connected_components = graph.connected_components().unwrap();

        let mut components = connected_components.as_ref().iter();
        let component1 = components.next().unwrap();
        let component2 = components.next().unwrap();

        let expected1 = vec![2, 3, 4];
        let expected2 = vec![0, 1];
        let expected: Vec<HashSet<&i32>> =
            vec![expected1.iter().collect(), expected2.iter().collect()];
        let expected1 = expected.get(0).unwrap();
        let expected2 = expected.get(1).unwrap();

        if component1
            .difference(&expected1)
            .into_iter()
            .next()
            .is_none()
            && component2
                .difference(&expected2)
                .into_iter()
                .next()
                .is_none()
        {
            assert!(true)
        } else if component2
            .difference(&expected1)
            .into_iter()
            .next()
            .is_none()
            && component1
                .difference(&expected2)
                .into_iter()
                .next()
                .is_none()
        {
            assert!(true)
        } else {
            assert!(false)
        }
    }
}
