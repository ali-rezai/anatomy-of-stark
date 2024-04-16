use crate::{
    consts::{ONE, ZERO},
    field::Field,
};
use primitive_types::U256;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct FieldElement<'a> {
    pub value: U256,
    pub field: &'a Field,
}

impl<'a> FieldElement<'a> {
    pub fn new(value: U256, field: &'a Field) -> Self {
        FieldElement {
            value: value,
            field: field,
        }
    }

    pub fn inv(&self) -> FieldElement<'a> {
        self.field.inv(&self)
    }
}

impl<'a> std::ops::Add<&FieldElement<'a>> for &FieldElement<'a> {
    type Output = FieldElement<'a>;

    fn add(self, rhs: &FieldElement<'a>) -> FieldElement<'a> {
        self.field.add(self, rhs)
    }
}

impl<'a> std::ops::Sub<&FieldElement<'a>> for &FieldElement<'a> {
    type Output = FieldElement<'a>;

    fn sub(self, rhs: &FieldElement<'a>) -> FieldElement<'a> {
        self.field.sub(self, rhs)
    }
}

impl<'a> std::ops::Mul<&FieldElement<'a>> for &FieldElement<'a> {
    type Output = FieldElement<'a>;

    fn mul(self, rhs: &FieldElement<'a>) -> FieldElement<'a> {
        self.field.mul(self, rhs)
    }
}

impl<'a> std::ops::Div<&FieldElement<'a>> for &FieldElement<'a> {
    type Output = FieldElement<'a>;

    fn div(self, rhs: &FieldElement<'a>) -> FieldElement<'a> {
        self.field.div(self, rhs)
    }
}

impl<'a> std::ops::Neg for &FieldElement<'a> {
    type Output = FieldElement<'a>;

    fn neg(self) -> FieldElement<'a> {
        self.field.neg(self)
    }
}

impl<'a> std::ops::BitXor<U256> for &FieldElement<'a> {
    type Output = FieldElement<'a>;

    fn bitxor(self, rhs: U256) -> FieldElement<'a> {
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
        let e1 = FieldElement::new(ONE, &f);
        let e2 = FieldElement::new(3.into(), &f);
        let e3 = FieldElement::new(3.into(), &f);
        assert_eq!(e1.field, e2.field);
        assert_eq!(e1.value, ONE);
        assert_eq!(e2.value, 3.into());
        assert_eq!(e2, e3);
        assert_ne!(e2, e1);
    }

    #[test]
    fn arithmetic_test() {
        let f = Field::new(7.into());
        let e1 = FieldElement::new(ONE, &f);
        let e2 = FieldElement::new(3.into(), &f);
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
