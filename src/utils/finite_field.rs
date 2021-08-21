//! Supporting Finite Field Arithmetic

use std::{collections::HashMap, fmt::Debug};

use algebraics::{
    mod_int::{Mod2, ModularInteger},
    polynomial::Polynomial,
};
use num_bigint::BigInt;
use num_bigint::ToBigUint;

fn bits(val: &u64) -> u64 {
    (*val as f64).log2().ceil() as u64
}

#[derive(Clone)]
pub struct TwoPowerFieldPoly(Polynomial<ModularInteger<u8, Mod2>>);

impl From<u64> for TwoPowerFieldPoly {
    fn from(input: u64) -> Self {
        Self(Polynomial::from(
            BigInt::from(input)
                .to_radix_le(2)
                .1
                .into_iter()
                .map(|i| ModularInteger::new(i, Mod2 {}))
                .collect::<Vec<_>>(),
        ))
    }
}

impl From<TwoPowerFieldPoly> for u64 {
    fn from(input: TwoPowerFieldPoly) -> Self {
        input.0.iter().enumerate().fold(0, |res, (i, val)| {
            if *val.value() == 1 {
                res + (2_u64).pow(i as u32) as u64
            } else {
                res
            }
        })
    }
}

pub struct Primitive;

impl Primitive {
    fn of_degree(deg: &u8) -> TwoPowerFieldPoly {
        let map: HashMap<u8, TwoPowerFieldPoly> = vec![
            (2, 7),
            (3, 9),
            (4, 25),
            (5, 37),
            (6, 73),
            (7, 185),
            (8, 355),
            (9, 623),
            (10, 1933),
            (11, 2091),
            (12, 5875),
            (13, 14513),
            (14, 32771),
            (15, 16707),
            (16, 66525),
            (17, 131081),
            (18, 262207),
            (19, 524327),
            (20, 1048585),
            (21, 2097157),
            (22, 4194307),
        ]
        .into_iter()
        .map(|(i, j)| (i as u8, j.into()))
        .collect();

        map.get(deg).unwrap().clone()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FField {
    order: u64,
    irreducible: u64,
}

impl FField {
    pub fn init_with_irreducible(order: u64, irreducible: u64) -> Self {
        if !order.is_power_of_two() {
            panic!("Order of FField must be a power of two: {}", order);
        }
        Self { order, irreducible }
    }
    pub fn init(order: u64) -> Self {
        if !order.is_power_of_two() {
            panic!("Order of FField must be a power of two: {}", order);
        }
        let degree = (order as f64).log2() as u8;
        println!("Degree: {:?}", degree);

        Self::init_with_irreducible(order, Primitive::of_degree(&degree).into())
    }

    pub fn reduce(self, value: u64) -> u64 {
        let mut value = value;
        while bits(&value) > bits(&self.order) {
            value ^= self.irreducible << (bits(&value) - bits(&self.irreducible))
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

/// An element of some field.
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
