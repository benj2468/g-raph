/// A structure for containing a finite field, and arithmetic within that field.
///
/// The value contained within the structure is the size of the field
#[derive(Clone, Copy)]
pub struct FiniteField {
    order: u64,
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

impl FiniteField {
    /// Generate a new field of size `size`.
    pub fn new(order: u64) -> Self {
        Self { order }
    }

    /// Converts an i32 into a field element of the current field
    pub fn mod_p_i32(&self, val: i32) -> FieldElement {
        if val >= 0 {
            self.mod_p(val as u64).into()
        } else {
            (self.order - val.abs() as u64).into()
        }
    }

    /// Converts a u64 into a field element of the current field
    pub fn mod_p(&self, val: u64) -> FieldElement {
        val.rem_euclid(self.order).into()
    }

    /// Compute base^expo within the field
    pub fn pow(&self, base: FieldElement, expo: u32) -> FieldElement {
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
    fn test_mod_p_i32_positive() {
        let field = test_field();
        let val = 30;

        let result = field.mod_p_i32(val);

        assert_eq!(result, FieldElement(7))
    }

    #[test]
    fn test_mod_p_i32_negative() {
        let field = test_field();
        let val = -20;

        let result = field.mod_p_i32(val);

        assert_eq!(result, 3)
    }

    #[test]
    fn test_power() {
        let field = test_field();
        let base = field.mod_p_i32(-20);

        let result = field.pow(base, 100);

        assert_eq!(result, 3)
    }

    #[test]
    fn test_multiply() {
        let field = test_field();
        let v1 = field.mod_p_i32(-20);
        let v2 = field.mod_p_i32(5);

        let result = field.mul(v1, v2);

        assert_eq!(result, 15)
    }

    #[test]
    fn test_addition() {
        let field = test_field();
        let v1 = field.mod_p_i32(-20);
        let v2 = field.mod_p_i32(5);

        let result = field.add(v1, v2);

        assert_eq!(result, 8)
    }

    #[test]
    fn test_negate_neg() {
        let field = test_field();
        let v1 = field.mod_p_i32(-20);

        let result = field.neg(v1);

        assert_eq!(result, 20)
    }

    #[test]
    fn test_negate_pos() {
        let field = test_field();
        let v1 = field.mod_p_i32(20);

        let result = field.neg(v1);

        assert_eq!(result, 3)
    }
}
