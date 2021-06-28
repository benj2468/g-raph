use rand::Rng;

use crate::graph::streaming::sparse_recovery::{OneSparseRecovery, OneSparseRecoveryOutput};

pub trait L0Sampling {
    fn l_zero_sampling(self, n: u32) -> Option<(u32, i32)>;
}

impl<T> L0Sampling for T
where
    T: core::iter::Iterator<Item = (i32, i32)> + Sized,
{
    /// Sample a coordinate at random from a high-demensional vector
    ///
    /// In l0 sampling we sample each coordinate with probability 1/||f||_0, meaning uniformly from the set of all distinct coordinate
    fn l_zero_sampling(self, n: u32) -> Option<(u32, i32)> {
        // Initialization
        let mut data_structure = vec![];
        for _ in 0..(n as f32).ln().round() as u32 {
            let recover = OneSparseRecovery::init(n);

            data_structure.push(recover);
        }

        // Process
        let mut rng = rand::thread_rng();
        self.for_each(|(j, c)| {
            data_structure
                .iter_mut()
                .enumerate()
                .for_each(|(l, recovery)| {
                    if rng.gen_range(0..((2 as u32).pow(l as u32))) == 0 {
                        recovery.feed((j as u32, c))
                    }
                })
        });

        #[cfg(test)]
        println!("Current State:");

        #[cfg(test)]
        data_structure.iter().for_each(|recover| {
            println!("{:?}", recover);
        });

        // Output
        for recovery in data_structure {
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
}
