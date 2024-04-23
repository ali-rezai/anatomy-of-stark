use crate::{consts::*, element::FieldElement, xgcd};
use primitive_types::U256;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Field {
    pub p: U256,
}

impl Field {
    pub fn new(p: U256) -> Self {
        Field { p }
    }

    pub fn zero(&self) -> FieldElement {
        FieldElement {
            value: ZERO,
            field: *self,
        }
    }

    pub fn one(&self) -> FieldElement {
        FieldElement {
            value: ONE,
            field: *self,
        }
    }

    pub fn generator(&self) -> FieldElement {
        assert!(self.p == *PRIME);
        return FieldElement::new(*GENERATOR, *self);
    }

    pub fn primitive_nth_root(&self, n: U256) -> FieldElement {
        assert!(self.p == *PRIME);
        assert!(n <= (1u128 << 119).into() && n & (n - 1) == ZERO);
        let mut root = self.generator();
        let mut order: U256 = (1u128 << 119).into();
        while order != n {
            root = &root ^ *TWO;
            order = order >> 1;
        }
        root
    }

    pub fn sample(&self, byte_array: &[u8]) -> FieldElement {
        let mut acc: U256 = ZERO;
        byte_array.iter().for_each(|b| {
            acc = (acc << 8) ^ (*b).into();
        });
        FieldElement::new(acc % self.p, *self)
    }

    pub fn add(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        FieldElement {
            value: (left.value + right.value) % self.p,
            field: *self,
        }
    }
    pub fn sub(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        FieldElement {
            value: (self.p + left.value - right.value) % self.p,
            field: *self,
        }
    }
    pub fn mul(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        FieldElement {
            value: (left.value * right.value) % self.p,
            field: *self,
        }
    }
    pub fn div(&self, left: &FieldElement, right: &FieldElement) -> FieldElement {
        assert!(right.value != ZERO);
        let (a, _, _, a_neg, _) = xgcd(right.value, self.p);
        FieldElement {
            value: if a_neg {
                self.p - (left.value * a) % self.p
            } else {
                left.value * a
            } % self.p,
            field: *self,
        }
    }
    pub fn neg(&self, operand: &FieldElement) -> FieldElement {
        FieldElement {
            value: (self.p - operand.value) % self.p,
            field: *self,
        }
    }

    pub fn inv(&self, operand: &FieldElement) -> FieldElement {
        let (a, _, _, a_neg, _) = xgcd(operand.value, self.p);
        FieldElement {
            value: if a_neg { self.p - a } else { a } % self.p,
            field: *self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_test() {
        let f = Field::new(*PRIME);
        assert_eq!(f.p, *PRIME);

        let root = f.primitive_nth_root((1u128 << 119).into());
        assert_eq!(root.value, *GENERATOR);

        let root = f.primitive_nth_root((1u128 << 118).into());
        assert_eq!(root.value, *GENERATOR * *GENERATOR % *PRIME);

        let root = f.primitive_nth_root((1u128 << 117).into());
        assert_eq!(
            root.value,
            (*GENERATOR * *GENERATOR % *PRIME) * (*GENERATOR * *GENERATOR % *PRIME) % *PRIME
        );

        let gen = f.generator();
        assert_eq!(gen.value, *GENERATOR);

        let s = f.sample(&[1u8, 2u8, 3u8]);
        assert_eq!(s.value, 66051.into());
    }
}
