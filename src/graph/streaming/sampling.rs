/// Sampling Functions
///
/// This File is a WIP
///
/// Current includes:
///
/// 1. l-0 Sampling, an implementation over an iterator to consume it and sample a single element.
use rand::distributions::{Distribution, Uniform};

use crate::graph::streaming::sparse_recovery::{OneSparseRecovery, OneSparseRecoveryOutput};

/// Data Structure for storing a hash function
///
/// The hash functions are 2-universal and are F: {0,1}^n --> {0,1}^l
///
/// The functions are of the form f(x) = Ax + b
///
/// A := {0,1}^{n,l}
/// b :- {0,1}^l

#[derive(Debug)]
pub struct HashFunction {
    a: Vec<Vec<u32>>,
    b: Vec<u32>,
}

impl HashFunction {
    /// Initialize a new hash function. Practically this is simply creating an A and a b value, randomly
    fn init(n: u32, l: u32) -> Self {
        let (mut a, mut b) = (vec![], vec![]);
        let gen = Uniform::new_inclusive(0, 1);
        let mut rng = rand::thread_rng();

        for _ in 0..l {
            let mut a_row = vec![];
            for _ in 0..n {
                a_row.push(gen.sample(&mut rng));
            }
            a.push(a_row);
            b.push(gen.sample(&mut rng));
        }

        Self { a, b }
    }

    #[cfg(test)]
    fn test() -> Self {
        let a = vec![
            vec![0,1,0],
            vec![1,1,1],
        ];
        let b = vec![1, 1];

        Self {a, b}
    }
    /// Computes the boolean value of f(x) = *0*
    fn is_zero(&self, x: u32) -> bool {
        self.a
            .iter()
            .zip(self.b.iter())
            .find_map(|(a, b)| {
                let value = (a.get(x as usize).unwrap() + b) % 2;
                if value == 0 {
                    None
                } else {
                    Some(value)
                }
            })
            .is_none()
    }
}

/// L-0 Sampling
pub trait L0Sampling {
    /// Sample a coordinate at random from a high-demensional vector (vector of size n)
    ///
    /// In l-0-sampling we sample each coordinate with probability 1/||f||_0,
    /// meaning uniformly from the set of all distinct coordinates
    fn l_zero_sampling(self, n: u32, delta: f32) -> Option<(u32, i32)>;
}

impl<T> L0Sampling for T
where
    T: core::iter::Iterator<Item = (u32, bool)> + Sized,
{
    fn l_zero_sampling(self, n: u32, delta: f32) -> Option<(u32, i32)> {
        // Initialization
        let mut data_structure = vec![];
        let order = (n as f32).log2() * (1 as f32 / delta).log2();
        for l in 0..order.round() as u32 {
            let recover = OneSparseRecovery::init(n);
            let hash_function = HashFunction::init(n, l);

            data_structure.push((recover, hash_function));
        }

        // Process
        self.for_each(|(j, c)| {
            data_structure.iter_mut().for_each(|(recovery, hasher)| {
                if hasher.is_zero(j) {
                    recovery.feed((j, c))
                }
            })
        });

        // Output
        for (recovery, _) in data_structure {
            let query = recovery.query();
            match query {
                OneSparseRecoveryOutput::VeryLikely(l, i) => return Some((i, l)),
                _ => continue,
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    // Need testing, not sure what testing looks like thought

    use super::*;

    #[test]
    fn test_hash_computation() {
        let hash = HashFunction::test();

        assert!(hash.is_zero(1));
        assert!(!hash.is_zero(0));
        assert!(!hash.is_zero(2));
    }

    fn sampling() -> Option<bool> {
        let stream = vec![
            (0, true),
            (6, true),
            (7, true),
            (6, true),
            (7, true),
            (6, false),
            (3, true),
            (0, true),
            (3, false),
        ]
        .into_iter()
        .l_zero_sampling(10, 0.1);

        if let Some((cord, _)) = stream {
            Some([0, 6, 7].contains(&cord))
        } else {
            None
        }
    }

    #[test]
    fn sampling_test() {
        let mut fails = 0;
        let mut incorrect = 0;

        let n = 100;

        for _ in 0..n {
            let result = sampling();
            if let Some(b) = result {
                if !b {
                    incorrect += 1;
                }
            } else {
                fails += 1;
            }
        }

        println!(
            "Failed: {:?} / {n},\nIncorrect (due to incorrectness of 1-sparse-recover): {:?} / {n}",
            fails,
            incorrect,
            n = n
        )
    }
}
