//! More Generalized s-Sparse Recovery

use super::one_sparse::{OneSparseRecovery, OneSparseRecoveryOutput};
use crate::utils::hash_function::HashFunction;
use num_primes::Generator;
use std::{collections::HashMap, fmt::Debug};

/// S-Sparse Recovery Data Structure
///
/// Algorithm for recovery and detection is based off of [Algorithm 15](https://www.cs.dartmouth.edu/~ac/Teach/CS35-Spring20/Notes/lecnotes.pdf)
///
/// Storage: O(tlog(t) + tlog(n))
#[derive(Clone)]
pub struct SparseRecovery<F: HashFunction> {
    /// Sparsity Parameter
    ///
    /// Constant Space
    s: u64,
    /// The Sparse Recovery Data Structures
    ///
    /// Stores O(2st) = O(slog(s/del))
    structures: Vec<Vec<OneSparseRecovery>>,
    /// Hash Functions for hashing to the Sparse recovery systems
    /// Store O(t * HF bits)
    functions: Vec<F>,
}

impl<F: HashFunction> Debug for SparseRecovery<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-Sparse Recovery Structure", self.s)
    }
}

impl<F> SparseRecovery<F>
where
    F: HashFunction,
{
    /// Initialize a new S-Sparse Detection and Recovery Data Structure
    ///
    /// - *n* : Universe Size
    /// - *s* : Sparsity we wish to detect
    /// - *del* : Error probability controller
    pub fn init(n: u64, s: u64, del: f32) -> Self {
        let t = (s as f32 / del).log2().ceil() as u64;

        let order = {
            let prime_bits = (3_f64 * (n as f64).log2()).ceil() as u64 + 1;
            let prime = Generator::new_prime(prime_bits);
            prime
                .to_u32_digits()
                .into_iter()
                .enumerate()
                .fold(0, |val, (i, next)| {
                    let digit_value = 32u64.pow(i as u32) * next as u64;
                    val + digit_value
                })
        };

        let s = s.next_power_of_two();

        let structures = (0..t)
            .into_iter()
            .map(|_| {
                (0..(2 * s))
                    .into_iter()
                    .map(|_| OneSparseRecovery::init_with_order(n, order))
                    .collect()
            })
            .collect();

        let functions = (0..t).into_iter().map(|_| F::init(n, 2 * s)).collect();

        Self {
            s,
            structures,
            functions,
        }
    }

    /// Feed a token into the Structure
    pub fn feed(&mut self, token: (u64, bool)) {
        let Self {
            structures,
            functions,
            ..
        } = self;
        let (j, _) = token;
        structures
            .iter_mut()
            .zip(functions.iter())
            .enumerate()
            .for_each(|(_, (recoveries, hasher))| {
                let hashed_index = hasher.compute(j);
                if let Some(recovery) = recoveries.get_mut(hashed_index) {
                    recovery.feed(token);
                } else {
                    panic!(
                        "ERROR HASHED INDEX OUT OF BOUNDS: {} \\in [{}]. Using hasher: {:?}",
                        hashed_index,
                        recoveries.len(),
                        hasher
                    );
                }
            });
    }

    /// Query the Structure for detection and recovery
    ///
    /// The HashMap contains a mapping from indices which are part of the recovery to the values they contained.
    ///
    /// If the stream was not s-sparse, or if one of the one-sparse recovery systems got an answer wrong, then we return `None`.
    pub fn query(self) -> Option<HashMap<u64, i64>> {
        let mut recovery = HashMap::new();

        let mut can_return = false;

        for (_, row) in self.structures.into_iter().enumerate() {
            for (_, cell) in row.into_iter().enumerate() {
                let res = cell.clone().query();
                match res {
                    OneSparseRecoveryOutput::VeryLikely(lambda, i) => {
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
                        can_return = true
                    }
                    _ => continue,
                }
            }
        }
        if can_return {
            Some(recovery)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use std::{cmp::max, collections::HashSet};

    use rand::prelude::Distribution;

    use crate::{
        random_graph::uniform::UniformGraphDistribution, utils::hash_function::FieldHasher,
    };

    use super::*;

    fn tiny_sparse() -> bool {
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

        let mut recovery = SparseRecovery::<FieldHasher>::init(10, 3, 0.01);

        stream.into_iter().for_each(|token| recovery.feed(token));

        let mut expected: HashSet<(u64, i64)> = HashSet::new();
        expected.insert((0, 1));
        expected.insert((6, 1));
        expected.insert((7, 3));

        recovery
            .query()
            .map(|actual| {
                actual
                    .into_iter()
                    .collect::<HashSet<(u64, i64)>>()
                    .is_subset(&expected)
            })
            .unwrap_or_default()
    }

    fn tiny_not_sparse() -> Option<HashMap<u64, i64>> {
        let stream: Vec<u64> = vec![1, 2, 3, 4, 5, 6, 7, 8, 3, 5, 4, 6, 7, 3, 2, 5, 7, 5];

        let mut recovery = SparseRecovery::<FieldHasher>::init(10, 3, 0.01);

        stream
            .into_iter()
            .for_each(|token| recovery.feed((token, true)));

        recovery.query()
    }

    #[test]
    fn not_sparse_probability() {
        let n = 10;

        let mut incorrect = 0;

        for _ in 0..n {
            let res = tiny_not_sparse();
            if res.is_some() {
                incorrect += 1;
            }
        }

        let probability = incorrect as f32 / n as f32;
        assert!(probability <= 0.01);
    }

    #[test]
    fn sparse_probability() {
        let n = 1000;

        let mut incorrect = 0;

        for _ in 0..n {
            if !tiny_sparse() {
                incorrect += 1;
            }
        }

        let probability = incorrect as f32 / n as f32;
        assert!(probability <= 0.01);
    }

    fn test_graph() -> Option<HashMap<u64, i64>> {
        let distribution = UniformGraphDistribution::init(500, 200_000);
        let mut rng = rand::thread_rng();
        let graph_edges: Vec<_> = distribution.sample(&mut rng);
        let mut recovery = SparseRecovery::<FieldHasher>::init(124750, 17932, 0.01);

        graph_edges
            .into_iter()
            .for_each(|t| recovery.feed((t.to_d1(), true)));

        recovery.query()
    }

    #[test]
    fn large_vec() {
        let n = 1;

        for _ in 0..n {
            let res = test_graph();
            assert!(res.is_none())
        }
    }
}
