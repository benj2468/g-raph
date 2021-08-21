//! Supporting Finite Field Arithmetic

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

use algebraics::{
    mod_int::{Mod2, ModularInteger},
    polynomial::{Polynomial, PolynomialCoefficient},
};
use galois_2p8::*;
use num_bigint::BigInt;
use num_bigint::ToBigUint;
use num_traits::{Pow, ToPrimitive};
use rand::Rng;

pub struct Primitive;

impl Primitive {
    fn of_degree(deg: &u8) -> Polynomial<ModularInteger<u8, Mod2>> {
        let map: HashMap<u8, Polynomial<ModularInteger<u8, Mod2>>> = vec![
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
        .map(|(i, j)| (i as u8, bijection(j)))
        .collect();

        map.get(deg).unwrap().clone()
    }
}

fn bits(val: &u64) -> u64 {
    (*val as f64).log2().ceil() as u64
}

pub fn find_primitive(degree: &u8) -> u64 {
    // Randomly generate a bit string of size degree - 1
    // Check if it is primitive by factoring it
    // If there is only one factor, then it is primitive.
    let poly = Primitive::of_degree(degree);
    // let mut potential_polys: HashSet<Polynomial<ModularInteger<u8, Mod2>>> =
    //     (2_u32.pow(*degree as u32)..2_u32.pow(*degree as u32 + 1))
    //         .into_iter()
    //         .filter(|a| a % 2 == 1)
    //         .map(bijection)
    //         .collect();

    // let max: u32 = 2_u32.pow(*degree as u32);
    // for a in 0..(max / 2) - 1 {
    //     for b in a..(max / 2) - 1 {
    //         let a = bijection(2 * a + 1);
    //         let b = bijection(2 * b + 1);
    //         potential_polys.remove(&(a * b));
    //     }
    // }

    // potential_polys.iter().for_each(|e| println!("{}", e));

    // let poly = potential_polys.into_iter().next().unwrap();
    poly.iter().enumerate().fold(0, |res, (i, val)| {
        if *val.value() == 1 {
            res + (2_u32).pow(i as u32) as u64
        } else {
            res
        }
    })
}

fn bijection(x: u32) -> Polynomial<ModularInteger<u8, Mod2>> {
    Polynomial::from(
        BigInt::from(x)
            .to_radix_le(2)
            .1
            .into_iter()
            .map(|i| ModularInteger::new(i, Mod2 {}))
            .collect::<Vec<_>>(),
    )
}

fn reverse(poly: Polynomial<ModularInteger<u8, Mod2>>) -> u32 {
    poly.iter().enumerate().fold(0, |res, (i, val)| {
        if *val.value() == 1 {
            res + (2_u32).pow(i as u32) as u32
        } else {
            res
        }
    })
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

        Self::init_with_irreducible(order, find_primitive(&degree))
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

#[cfg(test)]
mod test {

    use super::*;

    fn helper(s: &str) {
        let res = s.split(" + ").fold(0, |res, cur| {
            if cur == "1" {
                return res + 1_u32;
            } else {
                return res + 2_u32.pow(cur.split("^").last().unwrap().parse::<u32>().unwrap());
            }
        });

        println!(", {},", res);
    }
    #[test]
    fn parse() {
        helper("x^15 + x^1 + 1");
        helper("x^14 + x^8 + x^6 + x^1 + 1");
    }

    #[test]
    fn find_primitive_test() {
        // (2..20)
        //     .into_iter()
        //     .for_each(|i| println!("({}, {}),", i, find_primitive(&i)))
    }

    #[test]
    fn validity_test() {
        let field = GeneralField::new(IrreducablePolynomial::Poly84320);

        println!("{}", field.add(2, 10));
        println!("{}", field.mult(2, 10));
    }

    #[test]
    fn add() {
        let field = FField::init(256);
        let x = field.elem(2);
        let y = field.elem(10);

        assert_eq!(field.add(x, y).value, 8);
        assert_eq!(field.mult(x, y).value, 20);
    }

    #[test]
    fn bits_test() {
        assert_eq!(bits(&6), 3_u64)
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

// #[cfg(test)]
// mod test {
//     use super::*;

//     fn test_field() -> FiniteField {
//         FiniteField::new(23)
//     }

//     #[test]
//     fn test_mod_p_i64_positive() {
//         let field = test_field();
//         let val = 30;

//         let result = field.mod_p_i64(val);

//         assert_eq!(result, FieldElement(7))
//     }

//     #[test]
//     fn test_mod_p_i64_negative() {
//         let field = test_field();
//         let val = -20;

//         let result = field.mod_p_i64(val);

//         assert_eq!(result, 3)
//     }

//     #[test]
//     fn test_power() {
//         let field = test_field();
//         let base = field.mod_p_i64(-20);

//         let result = field.pow(base, 100);

//         assert_eq!(result, 3)
//     }

//     #[test]
//     fn test_multiply() {
//         let field = test_field();
//         let v1 = field.mod_p_i64(-20);
//         let v2 = field.mod_p_i64(5);

//         let result = field.mul(v1, v2);

//         assert_eq!(result, 15)
//     }

//     #[test]
//     fn test_addition() {
//         let field = test_field();
//         let v1 = field.mod_p_i64(-20);
//         let v2 = field.mod_p_i64(5);

//         let result = field.add(v1, v2);

//         assert_eq!(result, 8)
//     }

//     #[test]
//     fn test_negate_neg() {
//         let field = test_field();
//         let v1 = field.mod_p_i64(-20);

//         let result = field.neg(v1);

//         assert_eq!(result, 20)
//     }

//     #[test]
//     fn test_negate_pos() {
//         let field = test_field();
//         let v1 = field.mod_p_i64(20);

//         let result = field.neg(v1);

//         assert_eq!(result, 3)
//     }
// }
