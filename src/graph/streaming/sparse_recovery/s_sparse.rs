use std::collections::HashMap;

use crate::graph::streaming::sampling::HashFunction;

use super::one_sparse::{OneSparseRecovery, OneSparseRecoveryOutput};

/// S-Sparse Recovery Data Structure
///
/// Algorithm for recovery and detection is based off of [Algorithm 15](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf)
#[derive(Clone)]
pub struct SparseRecovery {
    /// Sparsity Parameter
    s: u64,
    /// The Sparse Recovery Data Structures
    structures: Vec<Vec<OneSparseRecovery>>,
    /// Hash Functions for hashing to the Sparse recovery systems
    functions: Vec<HashFunction>,
}

impl SparseRecovery {
    /// Initialize a new S-Sparse Detection and Recovery Data Structure
    ///
    /// - *n* : Universe Size
    /// - *s* : Sparsity we wish to detect
    /// - *del* : Error probability controller
    pub fn init(n: u64, s: u64, del: f32) -> Self {
        let t = (s as f32 / del).log2().ceil() as u64;

        #[cfg(test)]
        println!("New Sparse Recovery: {:?} x {:?}", t, s * 2);

        let structures = (0..t)
            .into_iter()
            .map(|_| {
                (0..(2 * s))
                    .into_iter()
                    .map(|_| OneSparseRecovery::init(n))
                    .collect()
            })
            .collect();

        let functions = (0..t)
            .into_iter()
            .map(|_| {
                // Need a different hash function here
                HashFunction::init(n, 2 * s)
            })
            .collect();

        Self {
            s,
            structures,
            functions,
        }
    }

    /// Feed a token into the Structure
    pub fn feed(&mut self, token: (u64, bool)) {
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

    /// Query the Structure for detection and recovery
    ///
    /// The HashMap contains a mapping from indices which are part of the recovery to the values they contained.
    ///
    /// If the stream was not s-sparse, or if one of the one-sparse recovery systems got an answer wrong, then we return `None`.
    pub fn query(self) -> Option<HashMap<u64, i32>> {
        let mut recovery = HashMap::new();

        for row in self.structures {
            for cell in row {
                match cell.query() {
                    OneSparseRecoveryOutput::VeryLikely(lambda, i) => {
                        if lambda != 0 {
                            if recovery
                                .get(&i)
                                .map(|val| val != &lambda)
                                .unwrap_or_default()
                            {
                                return None;
                            }
                            recovery.insert(i, lambda);
                            if recovery.keys().len() > self.s as usize {
                                return None;
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }
        Some(recovery)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_test() {
        let stream: Vec<(u64, bool)> = vec![
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

        let mut expected: HashMap<u64, i32> = HashMap::new();
        expected.insert(0, 1);
        expected.insert(6, 1);
        expected.insert(7, 3);

        assert_eq!(recovery.query(), Some(expected))
    }
}
