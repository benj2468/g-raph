use rand::distributions::{Distribution, Uniform};

use crate::graph::streaming::sparse_recovery::{OneSparseRecovery, OneSparseRecoveryOutput};

#[derive(Debug)]
pub struct HashFunction {
    a: Vec<Vec<u32>>,
    b: Vec<u32>,
}

impl HashFunction {
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
    fn is_zero(&self, x: u32) -> bool {
        // Then, convert x, which should be in the range of [n], into the e_x matrix
        self.a
            .iter()
            .zip(self.b.iter())
            .find_map(|(a, b)| {
                let v = a.get(x as usize).unwrap();
                if *v == 0 {
                    None
                } else {
                    Some(v)
                }
            })
            .is_none()
    }
}

pub trait L0Sampling {
    fn l_zero_sampling(self, n: u32) -> Option<(u32, i32)>;
}

impl<T> L0Sampling for T
where
    T: core::iter::Iterator<Item = (u32, bool)> + Sized,
{
    /// Sample a coordinate at random from a high-demensional vector
    ///
    /// In l0 sampling we sample each coordinate with probability 1/||f||_0, meaning uniformly from the set of all distinct coordinate
    fn l_zero_sampling(self, n: u32) -> Option<(u32, i32)> {
        // Initialization
        let mut data_structure = vec![];
        for l in 0..(n as f32).log2().round() as u32 {
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
            match recovery.query() {
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
    fn hash_functions() {
        let n = 5;
        for i in 0..n {
            let hash = HashFunction::init(n, i);
        }
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
        .l_zero_sampling(10);

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

        let n = 1000;

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
            "Failed: {:?} / {n},\nIncorrect: {:?} / {n}",
            fails,
            incorrect,
            n = n
        )
    }
}
