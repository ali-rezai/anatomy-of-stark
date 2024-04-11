use crate::element::FieldElement;

#[derive(PartialEq, Debug, Clone)]
pub struct Polynomial<'a> {
    pub coefficients: Vec<FieldElement<'a>>,
}

impl<'a> Polynomial<'a> {
    fn new(coefficients: Vec<FieldElement<'a>>) -> Self {
        Polynomial { coefficients }
    }

    fn degree(&self) -> i32 {
        let len = self.coefficients.len();
        if len == 0 {
            return -1;
        }
        let zero = self.coefficients[0].field.zero();
        if self.coefficients.iter().all(|e| e == &zero) {
            return -1;
        }
        let mut max_index = 0;
        self.coefficients.iter().enumerate().for_each(|(index, e)| {
            if e != &zero {
                max_index = index
            }
        });
        return max_index.try_into().unwrap();
    }

    fn leading_coefficient(self) -> FieldElement<'a> {
        let index: usize = self.degree().try_into().unwrap();
        FieldElement::new(
            self.coefficients[index].value,
            self.coefficients[index].field,
        )
    }
}

impl<'a> std::ops::Add<&Polynomial<'a>> for &Polynomial<'a> {
    type Output = Polynomial<'a>;

    fn add(self, rhs: &Polynomial<'a>) -> Polynomial<'a> {
        if self.degree() == -1 {
            return rhs.clone();
        } else if rhs.degree() == -1 {
            return self.clone();
        }
        let field = self.coefficients[0].field;
        let size = if self.coefficients.len() > rhs.coefficients.len() {
            self.coefficients.len()
        } else {
            rhs.coefficients.len()
        };
        let mut new_coeffs = vec![field.zero(); size];
        self.coefficients.iter().enumerate().for_each(|(index, e)| {
            new_coeffs[index] = &new_coeffs[index] + e;
        });
        rhs.coefficients.iter().enumerate().for_each(|(index, e)| {
            new_coeffs[index] = &new_coeffs[index] + e;
        });
        Polynomial::new(new_coeffs)
    }
}

impl<'a> std::ops::Neg for &Polynomial<'a> {
    type Output = Polynomial<'a>;

    fn neg(self) -> Polynomial<'a> {
        let new_coeffs: Vec<FieldElement<'a>> = self.coefficients.iter().map(|e| -e).collect();
        Polynomial::new(new_coeffs)
    }
}

impl<'a> std::ops::Sub<&Polynomial<'a>> for &Polynomial<'a> {
    type Output = Polynomial<'a>;

    fn sub(self, rhs: &Polynomial<'a>) -> Polynomial<'a> {
        self + &(-rhs)
    }
}

impl<'a> std::ops::Mul<&Polynomial<'a>> for &Polynomial<'a> {
    type Output = Polynomial<'a>;

    fn mul(self, rhs: &Polynomial<'a>) -> Polynomial<'a> {
        if self.coefficients.len() == 0 || rhs.coefficients.len() == 0 {
            return Polynomial::new(vec![]);
        }
        let zero = self.coefficients[0].field.zero();
        let size = rhs.coefficients.len() + self.coefficients.len() - 1;
        let mut new_coeffs = vec![zero; size];
        let zero = self.coefficients[0].field.zero();
        self.coefficients.iter().enumerate().for_each(|(i, e)| {
            if e != &zero {
                rhs.coefficients.iter().enumerate().for_each(|(j, er)| {
                    new_coeffs[i + j] = &new_coeffs[i + j] + &(e * er);
                });
            }
        });
        Polynomial::new(new_coeffs)
    }
}

#[cfg(test)]
mod tests {
    use crate::{field::Field, consts::*};
    use super::*;

    #[test]
    fn polynomial_test() {
        let f = Field::new(*PRIME);
        let coefficients = vec![
            f.one(),
            f.zero(),
            f.generator()
        ];
        let poly = Polynomial::new(coefficients);
        assert_eq!(poly.degree(), 2);
        assert_eq!(poly.leading_coefficient(), f.generator());

        let poly = Polynomial::new(vec![]);
        assert_eq!(poly.degree(), -1);

        let poly = Polynomial::new(vec![f.zero(), f.zero()]);
        assert_eq!(poly.degree(), -1);
    }

    #[test]
    fn arithmetic_test() {
        let f = Field::new(*PRIME);
        let poly = Polynomial::new(vec![f.zero(), f.zero()]);
        assert_eq!((-&poly).coefficients, vec![f.zero(), f.zero()]);

        let poly1 = Polynomial::new(vec![f.one(), f.generator()]);
        assert_eq!((-&poly1).coefficients, vec![FieldElement::new(*PRIME - ONE, &f), FieldElement::new(*PRIME - *GENERATOR, &f)]);

        let poly2 = Polynomial::new(vec![f.generator(), f.one()]);
        assert_eq!((&poly1 + &poly2).coefficients, vec![FieldElement::new(*GENERATOR + ONE, &f), FieldElement::new(*GENERATOR + ONE, &f)]);
        assert_eq!((&poly1 - &poly2).coefficients, vec![FieldElement::new(*PRIME + ONE - *GENERATOR, &f), FieldElement::new(*GENERATOR - ONE, &f)]);
        assert_eq!((&poly1 * &poly2).coefficients, vec![f.generator(), FieldElement::new((*GENERATOR * *GENERATOR) % f.p + ONE, &f), f.generator()]);
    }
}
