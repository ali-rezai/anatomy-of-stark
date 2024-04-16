use crate::{element::FieldElement, ONE, ZERO};
use primitive_types::U256;

#[derive(PartialEq, Debug, Clone)]
pub struct Polynomial<'a> {
    pub coefficients: Vec<FieldElement<'a>>,
}

fn divide<'a>(
    numerator: &Polynomial<'a>,
    denominator: &Polynomial<'a>,
) -> Option<(Polynomial<'a>, Polynomial<'a>)> {
    if denominator.degree() == -1 {
        return None;
    }
    if numerator.degree() < denominator.degree() {
        return Some((Polynomial::new(vec![]), numerator.clone()));
    }

    let degree = numerator.degree() - denominator.degree() + 1;

    let field = denominator.coefficients[0].field;
    let mut remainder = numerator.clone();
    let mut quotient_coefficients = vec![field.zero(); degree.try_into().unwrap()];

    for _ in 0..degree {
        if remainder.degree() < denominator.degree() {
            break;
        }
        let coefficient = &(remainder.leading_coefficient()) / &(denominator.leading_coefficient());
        let shift: usize = (remainder.degree() - denominator.degree())
            .try_into()
            .unwrap();

        let mut coeffs = vec![field.zero(); shift];
        coeffs.push(coefficient.clone());

        let subtractee = &Polynomial::new(coeffs) * denominator;

        quotient_coefficients[shift] = coefficient;
        remainder = &remainder - &subtractee;
    }
    let quotient = Polynomial::new(quotient_coefficients);
    return Some((quotient, remainder));
}

impl<'a> Polynomial<'a> {
    pub fn new(coefficients: Vec<FieldElement<'a>>) -> Self {
        Polynomial { coefficients }
    }

    pub fn degree(&self) -> i32 {
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

    pub fn leading_coefficient(&self) -> FieldElement<'a> {
        let index: usize = self.degree().try_into().unwrap();
        FieldElement::new(
            self.coefficients[index].value,
            self.coefficients[index].field,
        )
    }

    pub fn evaluate(&self, point: &FieldElement<'a>) -> FieldElement<'a> {
        let mut xi: FieldElement<'a> = point.field.one();
        let mut value: FieldElement<'a> = point.field.zero();
        self.coefficients.iter().for_each(|c| {
            value = &value + &(c * &xi);
            xi = &xi * point;
        });
        value
    }

    pub fn evaluate_domain(&self, domain: &Vec<FieldElement<'a>>) -> Vec<FieldElement<'a>> {
        domain.iter().map(|point| self.evaluate(point)).collect()
    }

    pub fn interpolate_domain(
        domain: &Vec<FieldElement<'a>>,
        values: &Vec<FieldElement<'a>>,
    ) -> Self {
        assert!(domain.len() == values.len());
        assert!(domain.len() > 0);
        let field = domain[0].field;
        let x = Polynomial::new(vec![field.zero(), field.one()]);
        let mut acc = Polynomial::new(vec![]);
        for i in 0..domain.len() {
            let mut prod = Polynomial::new(vec![values[i]]);
            for j in 0..domain.len() {
                if j == i {
                    continue;
                }
                prod = &(&prod * &(&x - &Polynomial::new(vec![domain[j]])))
                    * &Polynomial::new(vec![(&domain[i] - &domain[j]).inv()]);
            }
            acc = &acc + &prod;
        }
        acc
    }

    pub fn zerofier_domain(domain: &Vec<FieldElement<'a>>) -> Self {
        assert!(domain.len() > 0);
        let field = domain[0].field;
        let x = Polynomial::new(vec![field.zero(), field.one()]);
        let mut acc = Polynomial::new(vec![field.one()]);
        for i in 0..domain.len() {
            acc = &acc * &(&x - &Polynomial::new(vec![domain[i]]));
        }
        acc
    }

    pub fn scale(&self, factor: FieldElement<'a>) -> Self {
        Polynomial::new(
            self.coefficients
                .iter()
                .enumerate()
                .map(|(index, c)| &(&factor ^ index.into()) * c)
                .collect(),
        )
    }

    pub fn test_colinearity(points: &Vec<(FieldElement, FieldElement)>) -> bool {
        let domain: Vec<FieldElement<'_>> = points.iter().map(|p| p.0).collect();
        let values: Vec<FieldElement<'_>> = points.iter().map(|p| p.1).collect();
        let poly = Polynomial::interpolate_domain(&domain, &values);
        poly.degree() <= 1
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

impl<'a> std::ops::Div<&Polynomial<'a>> for &Polynomial<'a> {
    type Output = Polynomial<'a>;

    fn div(self, rhs: &Polynomial<'a>) -> Polynomial<'a> {
        if let Some((quotient, remainder)) = divide(self, rhs) {
            assert!(remainder.degree() != -1);
            return quotient;
        } else {
            panic!("[Polynomial] Division error");
        }
    }
}

impl<'a> std::ops::BitXor<U256> for &Polynomial<'a> {
    type Output = Polynomial<'a>;

    fn bitxor(self, rhs: U256) -> Polynomial<'a> {
        if self.degree() == -1 {
            return Polynomial::new(vec![]);
        }
        if rhs == ZERO {
            return Polynomial::new(vec![self.coefficients[0].field.one()]);
        }

        let mut acc = Polynomial::new(vec![self.coefficients[0].field.one()]);

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

        return acc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{consts::*, field::Field};

    #[test]
    fn polynomial_test() {
        let f = Field::new(*PRIME);
        let coefficients = vec![f.one(), f.zero(), f.generator()];
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
        assert_eq!(
            (-&poly1).coefficients,
            vec![
                FieldElement::new(*PRIME - ONE, &f),
                FieldElement::new(*PRIME - *GENERATOR, &f)
            ]
        );

        let poly2 = Polynomial::new(vec![f.generator(), f.one()]);
        assert_eq!(
            (&poly1 + &poly2).coefficients,
            vec![
                FieldElement::new(*GENERATOR + ONE, &f),
                FieldElement::new(*GENERATOR + ONE, &f)
            ]
        );
        assert_eq!(
            (&poly1 - &poly2).coefficients,
            vec![
                FieldElement::new(*PRIME + ONE - *GENERATOR, &f),
                FieldElement::new(*GENERATOR - ONE, &f)
            ]
        );
        assert_eq!(
            (&poly1 * &poly2).coefficients,
            vec![
                f.generator(),
                FieldElement::new((*GENERATOR * *GENERATOR) % f.p + ONE, &f),
                f.generator()
            ]
        );

        assert_eq!(&poly1 ^ ONE, poly1);

        assert_eq!(&poly1 ^ *TWO, &poly1 * &poly1);

        assert_eq!(
            (&poly1 / &poly2).coefficients,
            vec![&poly1.leading_coefficient() / &poly2.leading_coefficient()]
        );
    }

    #[test]
    fn evaluate_test() {
        let f = Field::new(*PRIME);
        let poly1 = Polynomial::new(vec![f.zero(), f.zero()]);
        let poly2 = Polynomial::new(vec![f.generator(), f.one(), FieldElement::new(*TWO, &f)]);

        let point1 = FieldElement::new(134.into(), &f);
        let point2 = FieldElement::new(1932.into(), &f);
        assert_eq!(poly1.evaluate(&point1), f.zero(),);

        assert_eq!(
            poly2.evaluate(&point1),
            &(&(&FieldElement::new(*TWO, &f) * &(&point1 ^ *TWO)) + &point1) + &f.generator(),
        );

        assert_eq!(
            poly2.evaluate_domain(&vec![point1, point2]),
            vec![
                &(&(&FieldElement::new(*TWO, &f) * &(&point1 ^ *TWO)) + &point1) + &f.generator(),
                &(&(&FieldElement::new(*TWO, &f) * &(&point2 ^ *TWO)) + &point2) + &f.generator()
            ]
        );
    }

    #[test]
    fn interpolate_test() {
        let f = Field::new(*PRIME);
        let point1 = FieldElement::new(134.into(), &f);
        let point2 = FieldElement::new(1932.into(), &f);

        let interpolated =
            Polynomial::interpolate_domain(&vec![point1, point2], &vec![f.one(), f.generator()]);
        assert_eq!(
            interpolated,
            Polynomial::new(vec![
                FieldElement::new(156715821677969870210199381849610144059u128.into(), &f),
                FieldElement::new(144172632631064309698331206458044765549u128.into(), &f)
            ])
        );
        assert_eq!(interpolated.evaluate(&point1), f.one());
        assert_eq!(interpolated.evaluate(&point2), f.generator());

        let zero_interpolated = Polynomial::zerofier_domain(&vec![point1, point2]);
        assert_eq!(
            zero_interpolated,
            Polynomial::new(vec![
                FieldElement::new(258888.into(), &f),
                FieldElement::new(270497897142230380135924736767050119151u128.into(), &f),
                f.one()
            ])
        );
        assert_eq!(zero_interpolated.evaluate(&point1), f.zero());
        assert_eq!(zero_interpolated.evaluate(&point2), f.zero());
    }

    #[test]
    fn scale_test() {
        let f = Field::new(*PRIME);
        let point1 = FieldElement::new(134.into(), &f);
        let point2 = FieldElement::new(1932.into(), &f);
        let poly = Polynomial::zerofier_domain(&vec![point1, point2]);

        let scale = FieldElement::new(*TWO, &f);
        let scaled_poly = poly.scale(scale);

        assert_eq!(scaled_poly.coefficients[0], poly.coefficients[0]);
        assert_eq!(scaled_poly.coefficients[1], &poly.coefficients[1] * &scale);
        assert_eq!(
            scaled_poly.coefficients[2],
            &(&poly.coefficients[2] * &scale) * &scale
        );

        assert_eq!(
            scaled_poly.evaluate(&(&point1 / &scale)),
            poly.evaluate(&point1)
        );

        assert_eq!(
            scaled_poly.evaluate(&(&point1 / &scale)),
            poly.evaluate(&point2)
        );

        assert_eq!(
            scaled_poly.evaluate(&(&f.generator() / &scale)),
            poly.evaluate(&f.generator())
        );
    }

    #[test]
    fn colinearity_test() {
        let f = Field::new(*PRIME);
        let point1 = (f.one(), f.zero());
        let point2 = (FieldElement::new(*TWO, &f), f.one());
        let point3 = (FieldElement::new(3.into(), &f), FieldElement::new(*TWO, &f));
        let point4 = (f.generator(), f.one());

        assert_eq!(Polynomial::test_colinearity(&vec![point1, point2]), true);
        assert_eq!(Polynomial::test_colinearity(&vec![point1, point4]), true);
        assert_eq!(
            Polynomial::test_colinearity(&vec![point1, point2, point4]),
            false
        );
        assert_eq!(
            Polynomial::test_colinearity(&vec![point1, point2, point3]),
            true
        );
    }
}
