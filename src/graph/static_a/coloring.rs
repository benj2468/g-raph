use rand::Rng;

use super::super::*;
use std::collections::HashSet;
use std::collections::LinkedList;

#[derive(Debug)]
pub struct Coloring<T>(HashMap<T, usize>);

impl<T> AsRef<HashMap<T, usize>> for Coloring<T> {
    fn as_ref(&self) -> &HashMap<T, usize> {
        &self.0
    }
}

impl<T, W> Graph<T, W>
where
    T: Hash + Eq + Clone + std::fmt::Debug + Default,
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

            coloring.insert(v.clone().clone(), color);
        });

        Coloring(coloring)
    }

    /// Randomized Coloring Algorithm
    ///
    /// Data Structure
    /// --------
    ///
    /// HashMap<Vertex, Color>
    /// HashMap<Vertex, Monochromatic Edges> // This should start as ALL edges (copy the adjacency list, and update by removing vertices from each set as we give them colors)
    ///
    /// Steps
    ///
    /// 1. Give each vertex a color, uniformly at random
    /// 2. Select a vertex that is incident to a monochromatic edge uniformly at random from all such vertices (build a set in step 1)
    /// 3. Increment that vertex's color by 1, and recompute the set
    fn color(&self) -> Coloring<T> {
        // let mut thread = rand::thread_rng();
        // let max = self
        //     .adjacency_list
        //     .values()
        //     .into_iter()
        //     .map(|set| set.len())
        //     .max()
        //     .unwrap_or(0)
        //     + 1;

        // // Structure Setup
        // let mut monochromatic_edges = self.adjacency_list.clone();

        // let mut coloring = HashMap::<&T, usize>::new();
        // for vertex in monochromatic_edges.keys() {
        //     let color = thread.gen_range(0..max);
        //     coloring.insert(vertex, color);
        // }

        // for vertex in self.adjacency_list.keys() {
        //     let random_color = thread.gen_range(0..max);
        //     monochromatic_edges.get_mut(vertex).map(|set| {
        //         set.retain(|neighbor| {
        //             coloring
        //                 .get(&neighbor.destination)
        //                 .map(|color| color == &random_color)
        //                 .unwrap_or(true)
        //         });
        //     });
        //     let current_neighbors = monochromatic_edges.get(vertex).unwrap().clone();
        //     for neighbor in current_neighbors.iter() {
        //         monochromatic_edges
        //             .get_mut(&neighbor.destination)
        //             .map(|set| {
        //                 set.retain(|neighbor| {
        //                     coloring
        //                         .get(&neighbor.destination)
        //                         .map(|color| color == &random_color)
        //                         .unwrap_or(true)
        //                 });
        //             });
        //     }
        //     // monochromatic_edges.get(vertex).map(|neighbors| {
        //     //     neighbors.iter().for_each(|neighbor| {
        //     //         monochromatic_edges
        //     //             .get_mut(&neighbor.destination)
        //     //             .map(|set| {
        //     //                 set.retain(|n_neighbor| {
        //     //                     coloring
        //     //                         .get(&n_neighbor.destination)
        //     //                         .map(|color| color == &random_color)
        //     //                         .unwrap_or(true)
        //     //                 });
        //     //             });
        //     //     })
        //     // });
        // }

        // println!("{:?}", coloring);

        todo!()
    }
}
