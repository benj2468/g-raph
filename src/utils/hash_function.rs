//! Supporting randomized Hash Functions
use rand::thread_rng;
use std::{fmt::Debug, usize};

use super::finite_field::{FField, PrimePowerFieldElement};

/// Describes a Hashing Function from n bits to l bits
///
/// HashFunction the trait provides no guarantee for implementation.
/// As a result, universality of the functions are not consistent across different implementations.
pub trait HashFunction: Debug {
    /// Initialize a new hash function. This should
    fn init(n: u64, l: u64) -> Self;
    /// Computes the value of h(x), where h is the current hash function
    fn compute(&self, x: u64) -> usize;
    /// Computes the boolean value of h(x) = *0*, where h is the current hash function
    fn is_zero(&self, x: u64) -> bool {
        self.compute(x) == 0
    }
    /// Random copy
    fn random_copy(&self) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct FFieldHasher {
    field: FField,
    a: PrimePowerFieldElement,
    b: PrimePowerFieldElement,
    mask: u64,
}

impl FFieldHasher {
    fn init_a_b(
        field: FField,
        a: PrimePowerFieldElement,
        b: PrimePowerFieldElement,
        l: u64,
    ) -> Self {
        if !l.is_power_of_two() {
            panic!("Hash Function range MUST be a power of two: {}", l)
        }
        let mask = l.next_power_of_two() - 1;

        Self { field, a, b, mask }
    }
}

impl HashFunction for FFieldHasher {
    fn init(n: u64, l: u64) -> Self {
        let mut rng = thread_rng();
        let field = FField::init(n);

        Self::init_a_b(field, field.sample(&mut rng), field.sample(&mut rng), l)
    }

    fn compute(&self, x: u64) -> usize {
        let Self {
            a, b, field, mask, ..
        } = self;

        let x = field.elem(x);

        (field.add(field.mult(*a, x), *b).value & mask) as usize
    }
    fn random_copy(&self) -> Self {
        let mut rng = thread_rng();
        let field = self.field;
        Self {
            field,
            a: field.sample(&mut rng),
            b: field.sample(&mut rng),
            mask: self.mask,
        }
    }
}

#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use itertools::Itertools;
    use num_traits::Pow;

    use super::*;

    fn two_universal(n: u64, l: u64) -> Vec<(f32, f32)> {
        let n = n.next_power_of_two();
        let l = l.next_power_of_two();
        let field = FField::init(n);

        let mut results: Vec<_> = (0..n)
            .into_iter()
            .map(|_| HashMap::<(usize, usize), f32>::new())
            .collect();

        (0..n)
            .into_iter()
            .cartesian_product((0..n).into_iter())
            .into_iter()
            .map(|(a, b)| (field.elem(a), field.elem(b)))
            .for_each(|(a, b)| {
                let hasher = FFieldHasher::init_a_b(field, a, b, l);
                let one = hasher.compute(0);
                // let other = hasher.compute(2);
                // *results
                //     .get_mut(2 as usize)
                //     .unwrap()
                //     .entry((one, other))
                //     .or_default() += 1.0;
                for other in 1..n {
                    let val = hasher.compute(other);

                    *results
                        .get_mut(other as usize)
                        .unwrap()
                        .entry((one, val))
                        .or_default() += 1.0;
                }
            });

        let avgs: Vec<f32> = results
            .iter()
            .map(|map| map.values().into_iter().sum::<f32>() / map.len() as f32)
            .collect();

        let standard_deviations: Vec<f32> = results
            .iter()
            .enumerate()
            .map(|(i, map)| {
                (map.values().into_iter().fold(0_f32, |red, v| {
                    if i == 1 {
                        return red;
                    }
                    red + (*v as f32 - avgs.get(i).unwrap()).pow(2)
                }) / map.len() as f32)
                    .sqrt()
            })
            .collect();

        avgs.into_iter().zip(standard_deviations).collect()
    }

    #[test]
    fn close() {
        let res = two_universal(32, 16);
        println!("{:?}", res);
    }
}
