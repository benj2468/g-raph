//! L0 Sampling

use crate::graph::streaming::sparse_recovery::one_sparse::{
    OneSparseRecovery, OneSparseRecoveryOutput,
};

use crate::utils::hash_function::{FieldHasher, HashFunction};

/// L-0 Sampling Data Str
pub trait L0Sampling {
    /// Sample a coordinate at random from a high-demensional vector (vector of size n)
    ///
    /// In l-0-sampling we sample each coordinate with probability 1/||f||_0,
    /// meaning uniformly from the set of all distinct coordinates
    fn l_zero_sampling(self, n: u64, delta: f32) -> Option<(u64, i64)>;
}

impl<T> L0Sampling for T
where
    T: core::iter::Iterator<Item = (u64, bool)> + Sized,
{
    fn l_zero_sampling(self, n: u64, delta: f32) -> Option<(u64, i64)> {
        // Initialization
        let mut data_structure = vec![];
        let order = (n as f32).log2() * (1_f32 / delta).log2();
        for l in 0..order.round() as u64 {
            let recover = OneSparseRecovery::init(n);
            let hash_function = FieldHasher::init(n, l);

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
    use rand::seq::SliceRandom;
    use rand::Rng;

    use super::*;

    fn random_sampling() -> Option<bool> {
        let mut rng = rand::thread_rng();
        let n: u64 = 100;

        let survivors: Vec<(u64, bool)> = (0..50)
            .into_iter()
            .map(|_| (rng.gen_range(0..n), true))
            .collect();

        let mut noise: Vec<(u64, bool)> = (0..50)
            .into_iter()
            .flat_map(|_| {
                let v = rng.gen_range(0..n);
                vec![(v, true), (v, false)]
            })
            .collect();

        let mut stream = survivors.clone();
        stream.append(&mut noise);
        stream.shuffle(&mut rng);

        let sample = stream.into_iter().l_zero_sampling(n, 0.1);

        if let Some((cord, _)) = sample {
            Some(survivors.into_iter().any(|(e, _)| e == cord))
        } else {
            None
        }
    }

    #[test]
    fn sampling_test() {
        let mut fails = 0;
        let mut incorrect = 0;

        let n = 10;

        for _ in 0..n {
            let result = random_sampling();
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
