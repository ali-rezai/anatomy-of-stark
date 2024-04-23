use crate::{
    consts::{ONE, ZERO},
    field::Field,
};
use primitive_types::U256;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct FieldElement {
    pub value: U256,
    pub field: Field,
}

impl FieldElement {
    pub fn new(value: U256, field: Field) -> Self {
        FieldElement { value, field }
    }

    pub fn inv(&self) -> FieldElement {
        self.field.inv(&self)
    }

    pub fn is_zero(&self) -> bool {
        self.value == ZERO
    }
}

impl std::ops::Add<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn add(self, rhs: &FieldElement) -> FieldElement {
        self.field.add(self, rhs)
    }
}

impl std::ops::Sub<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn sub(self, rhs: &FieldElement) -> FieldElement {
        self.field.sub(self, rhs)
    }
}

impl std::ops::Mul<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn mul(self, rhs: &FieldElement) -> FieldElement {
        self.field.mul(self, rhs)
    }
}

impl std::ops::Div<&FieldElement> for &FieldElement {
    type Output = FieldElement;

    fn div(self, rhs: &FieldElement) -> FieldElement {
        self.field.div(self, rhs)
    }
}

impl std::ops::Neg for &FieldElement {
    type Output = FieldElement;

    fn neg(self) -> FieldElement {
        self.field.neg(self)
    }
}

impl std::ops::BitXor<U256> for &FieldElement {
    type Output = FieldElement;

    fn bitxor(self, rhs: U256) -> FieldElement {
        let mut acc = self.field.one();

        let mut i: U256 = 128.into();
        while i > ZERO {
            i -= ONE;
            if (rhs >> i) & ONE == ONE {
                break;
            }
        }

        i += ONE;
        while i > ZERO {
            i -= ONE;
            acc = &acc * &acc;
            if (ONE << i) & rhs != ZERO {
                acc = &acc * &self;
            }
        }

        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn element_test() {
        let f = Field::new(7.into());
        let e1 = FieldElement::new(ONE, f);
        let e2 = FieldElement::new(3.into(), f);
        let e3 = FieldElement::new(3.into(), f);
        assert_eq!(e1.field, e2.field);
        assert_eq!(e1.value, ONE);
        assert_eq!(e2.value, 3.into());
        assert_eq!(e2, e3);
        assert_ne!(e2, e1);
    }

    #[test]
    fn arithmetic_test() {
        let f = Field::new(7.into());
        let e1 = FieldElement::new(ONE, f);
        let e2 = FieldElement::new(3.into(), f);
        assert_eq!((&e1 + &e2).value, 4.into());
        assert_eq!((&e1 - &e2).value, 5.into());
        assert_eq!((&e1 * &e2).value, 3.into());
        assert_eq!((&e1 / &e2).value, 5.into());
        assert_eq!((-&e1).value, 6.into());
        assert_eq!((&e2.inv()).value, 5.into());
        assert_eq!((&e2 ^ 4.into()).value, 4.into());
        assert_eq!((&e2 ^ 2.into()).value, 2.into());
        assert_eq!((&e1 ^ 2.into()).value, 1.into());
    }
}
