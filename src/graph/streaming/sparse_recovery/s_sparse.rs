use std::{collections::HashMap, time::Instant};

use crate::graph::streaming::sampling::HashFunction;
use crate::printdur;

use super::one_sparse::{OneSparseRecovery, OneSparseRecoveryOutput};

#[derive(Clone)]
pub struct SparseRecovery {
    n: u32,
    t: u32,
    s: u32,
    structures: Vec<Vec<OneSparseRecovery>>,
    functions: Vec<HashFunction>,
}

impl SparseRecovery {
    pub fn init(n: u32, s: u32, del: f32) -> Self {
        let t = (s as f32 / del).log2().ceil() as u32;

        #[cfg(test)]
        println!("New Sparse Recovery: {:?} x {:?}", t, s * 2);
        let start = Instant::now();

        let structures = (0..t)
            .into_iter()
            .map(|_| {
                (0..(2 * s))
                    .into_iter()
                    .map(|_| OneSparseRecovery::init(n))
                    .collect()
            })
            .collect();

        // printdur!("Structures", start);

        let functions = (0..t)
            .into_iter()
            .map(|_| {
                // Need a different hash function here
                HashFunction::init(n, 2 * s)
            })
            .collect();

        // printdur!("Functions", start);

        Self {
            n,
            t,
            s,
            structures,
            functions,
        }
    }

    pub fn feed(&mut self, token: (u32, bool)) {
        let (j, _) = token;
        self.structures
            .iter_mut()
            .zip(self.functions.iter())
            .for_each(|(recoveries, hasher)| {
                let hashed_index: u32 = hasher.compute(j).iter().sum();
                if let Some(recovery) = recoveries.get_mut(hashed_index as usize) {
                    recovery.feed(token);
                };
            });
    }

    pub fn query(self) -> Option<Vec<i32>> {
        let mut a = HashMap::new();

        for row in self.structures {
            for cell in row {
                match cell.query() {
                    OneSparseRecoveryOutput::VeryLikely(lambda, i) => {
                        if lambda != 0 {
                            if a.get(&i).map(|val| val != &lambda).unwrap_or_default() {
                                return None;
                            }
                            a.insert(i, lambda);
                            if a.keys().len() > self.s as usize {
                                return None;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }

        let mut f: Vec<i32> = (0..self.n).into_iter().map(|_| 0).collect();

        a.into_iter().for_each(|(cord, val)| {
            if let Some(current) = f.get_mut(cord as usize) {
                *current = *current + val;
            };
        });

        Some(f)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_test() {
        let stream: Vec<(u32, bool)> = vec![
            (0, true),
            (9, true),
            (7, true),
            (6, true),
            (7, true),
            (9, true),
            (7, true),
            (9, false),
            (9, false),
        ];

        let mut recovery = SparseRecovery::init(10, 3, 0.5);

        stream.into_iter().for_each(|token| recovery.feed(token));

        let expected = Some(vec![1, 0, 0, 0, 0, 0, 1, 3, 0, 0]);

        assert_eq!(recovery.query(), expected)
    }
}
