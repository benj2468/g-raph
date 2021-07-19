use super::super::*;
use std::collections::HashSet;

#[derive(Debug)]
pub struct Coloring<T>(HashMap<T, usize>);

impl<T> AsRef<HashMap<T, usize>> for Coloring<T> {
    fn as_ref(&self) -> &HashMap<T, usize> {
        &self.0
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Copy + std::fmt::Debug + Default,
    W: Hash + Eq + Clone + Default + std::fmt::Debug,
{
    pub fn color_degeneracy(self) -> Coloring<T> {
        let mut ordering = vec![];

        let mut graph = self.clone().with_vertex_recaller();

        while let Some(min) = graph.remove_min() {
            ordering.push(min);
        }

        ordering.reverse();

        let mut coloring = HashMap::new();

        ordering.into_iter().for_each(|v| {
            let mut color: usize = 0;

            let neighbor_colors: HashSet<&usize> = self
                .adjacency_list
                .get(&v)
                .unwrap()
                .iter()
                .map(|e| &e.destination)
                .filter_map(|v| coloring.get(v))
                .collect();

            while neighbor_colors.contains(&color) {
                color += 1
            }

            coloring.insert(v, color);
        });

        Coloring(coloring)
    }
}
