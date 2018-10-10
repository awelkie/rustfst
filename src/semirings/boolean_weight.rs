use semirings::Semiring;
use std::ops::{Add, AddAssign, Mul, MulAssign};

#[derive(Clone, Debug, PartialEq, Default, Eq)]
pub struct BooleanWeight {
    value: bool,
}

impl BooleanWeight {
    pub fn new(value: bool) -> Self {
        BooleanWeight { value }
    }
}

impl Semiring for BooleanWeight {
    type Type = bool;
    fn plus(&self, rhs: &Self) -> Self {
        Self::new(self.value | rhs.value)
    }
    fn times(&self, rhs: &Self) -> Self {
        Self::new(self.value & rhs.value)
    }

    fn zero() -> Self {
        Self::new(false)
    }

    fn one() -> Self {
        Self::new(true)
    }

    fn value(&self) -> Self::Type {
        self.value
    }

    fn from_value(value: <Self as Semiring>::Type) -> Self {
        BooleanWeight { value }
    }

    fn set_value(&mut self, value: <Self as Semiring>::Type) {
        self.value = value
    }
}

add_mul_semiring!(BooleanWeight);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_weight() {
        let b_true = BooleanWeight::new(true);
        let b_false = BooleanWeight::new(false);

        // Test plus
        assert_eq!(b_true.plus(&b_true), b_true);
        assert_eq!(b_true.plus(&b_false), b_true);
        assert_eq!(b_false.plus(&b_true), b_true);
        assert_eq!(b_false.plus(&b_false), b_false);

        // Test times
        assert_eq!(b_true.times(&b_true), b_true);
        assert_eq!(b_true.times(&b_false), b_false);
        assert_eq!(b_false.times(&b_true), b_false);
        assert_eq!(b_false.times(&b_false), b_false);
    }

    #[test]
    fn test_boolean_weight_sum() {
        let b_true = BooleanWeight::new(true);
        let b_false = BooleanWeight::new(false);

        println!("LOL : {:?}", b_true.clone() + b_false.clone());
        println!("LOL : {:?}", b_true * b_false);
    }

}
