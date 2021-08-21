//! Supporting Finite Field Arithmetic

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use algebraics::{
    mod_int::{Mod2, ModularInteger},
    polynomial::Polynomial,
    traits::FloorLog2,
};
use itertools::Itertools;
use num_bigint::BigInt;
use num_bigint::ToBigUint;

fn bits(val: &u64) -> u64 {
    (*val as f64).log2().ceil() as u64
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Wrapper for an element of F_(2^n)[X]
pub struct TwoPowerFieldPoly(u64);

impl From<Polynomial<ModularInteger<u8, Mod2>>> for TwoPowerFieldPoly {
    fn from(input: Polynomial<ModularInteger<u8, Mod2>>) -> Self {
        TwoPowerFieldPoly(input.iter().enumerate().fold(0, |res, (i, val)| {
            if *val.value() == 1 {
                res + (2_u64).pow(i as u32) as u64
            } else {
                res
            }
        }))
    }
}

impl From<TwoPowerFieldPoly> for Polynomial<ModularInteger<u8, Mod2>> {
    fn from(input: TwoPowerFieldPoly) -> Self {
        Polynomial::from(
            BigInt::from(input.0)
                .to_radix_le(2)
                .1
                .into_iter()
                .map(|i| ModularInteger::new(i, Mod2 {}))
                .collect::<Vec<_>>(),
        )
    }
}

/// A container for storing data regarding primitive polynomials of varying degrees.
pub struct Primitive {
    deg: u8,
    poly: TwoPowerFieldPoly,
}

impl Primitive {
    fn of_degree(deg: u8) -> Self {
        let poly = {
            let mut potential_polys: HashSet<u64> = (2_u64.pow(deg as u32)
                ..2_u64.pow(deg as u32 + 1))
                .into_iter()
                .collect();

            for i in 1..deg / 2 {
                for j in i..deg / 2 {
                    if i * j == deg {
                        let b_polys = 2_u64.pow(j as u32)..2_u64.pow(j as u32 + 1);
                        let a_polys = 2_u64.pow(i as u32)..2_u64.pow(i as u32 + 1);
                        a_polys
                            .into_iter()
                            .cartesian_product(b_polys)
                            .map(|(lhs, rhs)| lhs * rhs)
                            .for_each(|p| {
                                potential_polys.remove(&p);
                            })
                    }
                }
            }

            TwoPowerFieldPoly(potential_polys.into_iter().last().unwrap())
        };

        Self { deg, poly }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
// A finite field of order 2^n
pub struct PowerFiniteField {
    order: u64,
    irreducible: TwoPowerFieldPoly,
}

impl PowerFiniteField {
    /// Create a new Power Field, given an order and irreducible
    pub fn init_with_irreducible(order: u64, irreducible: Primitive) -> Self {
        if !order.is_power_of_two() {
            panic!("Order of FField must be a power of two: {}", order);
        }
        assert!(irreducible.deg == order.floor_log2().unwrap() as u8);
        Self {
            order,
            irreducible: irreducible.poly,
        }
    }
    pub fn init(order: u64) -> Self {
        if !order.is_power_of_two() {
            panic!("Order of FField must be a power of two: {}", order);
        }
        let degree = (order as f64).log2() as u8;
        println!("Degree: {:?}", degree);

        Self::init_with_irreducible(order, Primitive::of_degree(degree))
    }

    pub fn reduce(&self, value: u64) -> u64 {
        let mut value = value;
        while bits(&value) > bits(&self.order) {
            value ^= self.irreducible.0 << (bits(&value) - bits(&self.irreducible.0))
        }

        value
    }

    pub fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> PrimePowerFieldElement {
        let value = rng.gen_range(0..self.order);
        PrimePowerFieldElement { value }
    }

    pub fn elem(&self, value: u64) -> PrimePowerFieldElement {
        PrimePowerFieldElement {
            value: self.reduce(value),
        }
    }

    pub fn add(
        self,
        lhs: PrimePowerFieldElement,
        rhs: PrimePowerFieldElement,
    ) -> PrimePowerFieldElement {
        let value = lhs.value ^ rhs.value;

        PrimePowerFieldElement { value }
    }

    pub fn mult(
        self,
        lhs: PrimePowerFieldElement,
        rhs: PrimePowerFieldElement,
    ) -> PrimePowerFieldElement {
        let value = {
            let upper = lhs;
            let lower = rhs;
            let mut value = 0;

            for (loc, bit) in lower
                .value
                .to_biguint()
                .unwrap()
                .to_radix_le(2)
                .iter()
                .enumerate()
            {
                if *bit == 1_u8 {
                    value ^= upper.value << loc
                }
            }
            self.reduce(value)
        };

        PrimePowerFieldElement { value }
    }
}

#[derive(Clone, Copy)]
pub struct PrimePowerFieldElement {
    pub value: u64,
}

impl Debug for PrimePowerFieldElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

/// An element of some prime field.
///
/// Given some FieldElement, we do not know in fact, what field it is from. This computation would happen at runtime, so typing it is not an option.
/// What we can do, is semantically enforce that values passed into the FiniteField functions are FieldElements, rather than simply u64s.
///
/// Since we are using only one field at a time, this should suffice.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FieldElement(u64);

impl From<u64> for FieldElement {
    fn from(n: u64) -> Self {
        FieldElement(n)
    }
}

impl From<FieldElement> for u128 {
    fn from(element: FieldElement) -> Self {
        element.0 as u128
    }
}

impl From<FieldElement> for u64 {
    fn from(element: FieldElement) -> Self {
        element.0 as u64
    }
}

impl std::cmp::PartialEq<u64> for FieldElement {
    fn eq(&self, other: &u64) -> bool {
        self.0 == *other
    }
}

/// A structure for containing a finite field, and arithmetic within that field.
///
/// The value contained within the structure is the size of the field
#[derive(Clone, Copy)]
pub struct FiniteField {
    order: u64,
}

impl FiniteField {
    /// Generate a new field of size `size`.
    pub fn new(order: u64) -> Self {
        Self { order }
    }

    /// Converts an i32 into a field element of the current field
    pub fn mod_p_i64(&self, val: i64) -> FieldElement {
        if val >= 0 {
            self.mod_p(val as u64)
        } else {
            (self.order - val.abs() as u64).into()
        }
    }

    /// Converts a u64 into a field element of the current field
    pub fn mod_p(&self, val: u64) -> FieldElement {
        val.rem_euclid(self.order).into()
    }

    /// Compute base^expo within the field
    pub fn pow(&self, base: FieldElement, expo: u64) -> FieldElement {
        if expo == 0 {
            return 1.into();
        }
        // If the exponent is odd, get it to even, and continue
        if expo % 2 == 1 {
            self.mul(base, self.pow(base, expo - 1))
        } else {
            self.pow(self.mul(base, base), expo / 2)
        }
    }

    /// Computer v1 * v2 within the field
    pub fn mul(&self, v1: FieldElement, v2: FieldElement) -> FieldElement {
        let prod: u128 = u128::from(v1) * u128::from(v2);

        (prod.rem_euclid(self.order as u128) as u64).into()
    }

    /// Compute v1 + v2 within the field
    pub fn add(&self, v1: FieldElement, v2: FieldElement) -> FieldElement {
        let sum: u128 = u128::from(v1) + u128::from(v2);
        (sum.rem_euclid(self.order as u128) as u64).into()
    }

    /// Compute v1 - v2 within the field
    pub fn neg(&self, v1: FieldElement) -> FieldElement {
        (self.order - u64::from(v1)).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_field() -> FiniteField {
        FiniteField::new(23)
    }

    #[test]
    fn test_mod_p_i64_positive() {
        let field = test_field();
        let val = 30;

        let result = field.mod_p_i64(val);

        assert_eq!(result, FieldElement(7))
    }

    #[test]
    fn test_mod_p_i64_negative() {
        let field = test_field();
        let val = -20;

        let result = field.mod_p_i64(val);

        assert_eq!(result, 3)
    }

    #[test]
    fn test_power() {
        let field = test_field();
        let base = field.mod_p_i64(-20);

        let result = field.pow(base, 100);

        assert_eq!(result, 3)
    }

    #[test]
    fn test_multiply() {
        let field = test_field();
        let v1 = field.mod_p_i64(-20);
        let v2 = field.mod_p_i64(5);

        let result = field.mul(v1, v2);

        assert_eq!(result, 15)
    }

    #[test]
    fn test_addition() {
        let field = test_field();
        let v1 = field.mod_p_i64(-20);
        let v2 = field.mod_p_i64(5);

        let result = field.add(v1, v2);

        assert_eq!(result, 8)
    }

    #[test]
    fn test_negate_neg() {
        let field = test_field();
        let v1 = field.mod_p_i64(-20);

        let result = field.neg(v1);

        assert_eq!(result, 20)
    }

    #[test]
    fn test_negate_pos() {
        let field = test_field();
        let v1 = field.mod_p_i64(20);

        let result = field.neg(v1);

        assert_eq!(result, 3)
    }

    fn helper(s: &str) {
        let res = s.split(" + ").fold(0, |res, cur| {
            if cur == "1" {
                res + 1_u32
            } else {
                return res + 2_u32.pow(cur.split('^').last().unwrap().parse::<u32>().unwrap());
            }
        });

        println!(", {},", res);
    }
}
