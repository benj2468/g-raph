use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn zeros(p: u64) -> u64 {
    let (mut max, mut curr) = (0, 0);
    while curr <= p / 2 {
        if p % 2_u64.pow(curr as u32) == 0 {
            max = curr
        }
        curr += 1;
    }

    max
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub trait Distinct {
    fn tidemark(self) -> f32;
}

impl<T> Distinct for T
where
    T: core::iter::Iterator<Item = (i32, i32)> + Sized,
{
    fn tidemark(self) -> f32 {
        let mut z = 0;

        self.for_each(|token| {
            let hash = calculate_hash(&token);
            let zeros = zeros(hash);
            if zeros < z {
                z = zeros
            }
        });

        2_i32.pow(z as u32) as f32 * 2_f32.sqrt()
    }
}
