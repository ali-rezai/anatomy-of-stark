use primitive_types::U256;

use crate::{element::FieldElement, field::Field, polynomial::Polynomial, ONE, ZERO};
use std::{collections::HashMap, vec};

#[derive(PartialEq, Debug, Clone)]
pub struct MPolynomial {
    pub coefficients: HashMap<Vec<U256>, FieldElement>,
}

impl MPolynomial {
    pub fn new(coefficients: HashMap<Vec<U256>, FieldElement>) -> Self {
        MPolynomial { coefficients }
    }

    pub fn constant(element: FieldElement) -> Self {
        let mut map = HashMap::new();
        map.insert(vec![ZERO], element);
        MPolynomial::new(map)
    }

    pub fn is_zero(&self) -> bool {
        if self.coefficients.is_empty() {
            return true;
        } else {
            self.coefficients.values().all(|v| v.is_zero())
        }
    }

    pub fn variables(num_variables: usize, field: &Field) -> Vec<MPolynomial> {
        let mut variables = vec![];
        for i in 0..num_variables {
            let mut exponent = vec![ZERO; i];
            exponent.push(ONE);
            for _ in 0..(num_variables - i - 1) {
                exponent.push(ZERO);
            }
            let mut map = HashMap::new();
            map.insert(exponent, field.one());
            variables.push(MPolynomial::new(map))
        }
        variables
    }

    pub fn lift(polynomial: &Polynomial, variable_index: usize) -> Self {
        let map = HashMap::new();
        if polynomial.is_zero() {
            return MPolynomial::new(map);
        }
        let field = polynomial.coefficients[0].field;
        let variables = MPolynomial::variables(variable_index + 1, &field);
        let x = variables.last().unwrap();
        let mut acc = MPolynomial::new(map);
        polynomial
            .coefficients
            .iter()
            .enumerate()
            .for_each(|(i, c)| {
                acc = &acc + &(&MPolynomial::constant(*c) * &(x ^ i.into()));
            });
        acc
    }

    pub fn evaluate(&self, point: &Vec<FieldElement>) -> FieldElement {
        let mut acc = point[0].field.zero();
        self.coefficients.iter().for_each(|(k, v)| {
            let mut prod = *v;
            for i in 0..k.len() {
                prod = &prod * &(&point[i] ^ k[i]);
            }
            acc = &acc + &prod;
        });
        acc
    }

    pub fn evaluate_symbolic(&self, point: &Vec<Polynomial>) -> Polynomial {
        let mut acc = Polynomial::new(vec![]);
        self.coefficients.iter().for_each(|(k, v)| {
            let mut prod = Polynomial::new(vec![*v]);
            for i in 0..k.len() {
                prod = &prod * &(&point[i] ^ k[i]);
            }
            acc = &acc + &prod;
        });
        acc
    }
}

impl std::ops::Add<&MPolynomial> for &MPolynomial {
    type Output = MPolynomial;

    fn add(self, rhs: &MPolynomial) -> MPolynomial {
        let mut map = HashMap::new();
        let self_keys = self
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap_or(&vec![])
            .len();
        let rhs_keys = rhs
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap_or(&vec![])
            .len();
        let num_variables = usize::max(self_keys, rhs_keys);

        self.coefficients.iter().for_each(|e| {
            let mut v = e.0.clone();
            for _ in 0..(num_variables - e.0.len()) {
                v.push(ZERO);
            }
            map.insert(v, *e.1);
        });
        rhs.coefficients.iter().for_each(|e| {
            let mut v = e.0.clone();
            for _ in 0..(num_variables - e.0.len()) {
                v.push(ZERO);
            }
            if map.contains_key(&v) {
                let element = &map[&v] + e.1;
                map.insert(v, element);
            } else {
                map.insert(v, *e.1);
            }
        });

        MPolynomial::new(map)
    }
}

impl std::ops::Neg for &MPolynomial {
    type Output = MPolynomial;

    fn neg(self) -> MPolynomial {
        let mut map = HashMap::new();
        self.coefficients.iter().for_each(|e| {
            map.insert(e.0.clone(), -e.1);
        });
        MPolynomial::new(map)
    }
}

impl std::ops::Sub<&MPolynomial> for &MPolynomial {
    type Output = MPolynomial;

    fn sub(self, rhs: &MPolynomial) -> MPolynomial {
        self + &(-rhs)
    }
}

impl std::ops::Mul<&MPolynomial> for &MPolynomial {
    type Output = MPolynomial;

    fn mul(self, rhs: &MPolynomial) -> MPolynomial {
        let mut map = HashMap::new();
        let self_keys = self
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap_or(&vec![])
            .len();
        let rhs_keys = rhs
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap_or(&vec![])
            .len();
        let num_variables = usize::max(self_keys, rhs_keys);
        self.coefficients.iter().for_each(|(k0, v0)| {
            rhs.coefficients.iter().for_each(|(k1, v1)| {
                let mut exponent = vec![ZERO; num_variables];
                for i in 0..k0.len() {
                    exponent[i] += k0[i];
                }
                for i in 0..k1.len() {
                    exponent[i] += k1[i];
                }
                if map.contains_key(&exponent) {
                    let element = &map[&exponent] + &(v0 * v1);
                    map.insert(exponent, element);
                } else {
                    map.insert(exponent, v0 * v1);
                }
            });
        });
        MPolynomial::new(map)
    }
}

impl std::ops::BitXor<U256> for &MPolynomial {
    type Output = MPolynomial;

    fn bitxor(self, rhs: U256) -> MPolynomial {
        let mut map = HashMap::new();
        if self.is_zero() {
            return MPolynomial::new(map);
        }
        let field = self.coefficients.values().nth(0).unwrap().field;
        let num_variables = self.coefficients.keys().nth(0).unwrap().len();
        let exp = vec![ZERO; num_variables];

        map.insert(exp, field.one());
        let mut acc = MPolynomial::new(map);

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
            if (rhs >> i) & ONE == ONE {
                acc = &acc * &self;
            }
        }

        acc
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{consts::*, field::Field};

    #[test]
    fn mpolynomial_test() {
        let f = Field::new(*PRIME);
        let mut coefficients = HashMap::new();
        coefficients.insert(vec![*TWO, ONE], f.one());
        coefficients.insert(vec![ONE, *TWO], f.generator());
        coefficients.insert(vec![ZERO, ZERO], f.zero());

        let mp = MPolynomial::new(coefficients);
        assert_eq!(mp.is_zero(), false);
        assert_eq!(*mp.coefficients.get(&vec![*TWO, ONE]).unwrap(), f.one());
        assert_eq!(
            *mp.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            f.generator()
        );
        assert_eq!(*mp.coefficients.get(&vec![ZERO, ZERO]).unwrap(), f.zero());

        let cp = MPolynomial::constant(f.one());
        assert_eq!(cp.is_zero(), false);
        assert_eq!(*cp.coefficients.get(&vec![ZERO]).unwrap(), f.one());

        let zp = MPolynomial::constant(f.zero());
        assert_eq!(zp.is_zero(), true);

        let vars = MPolynomial::variables(3, &f);
        assert_eq!(vars.len(), 3);
        assert!(vars.iter().enumerate().all(|(i, v)| {
            if v.coefficients.keys().len() != 1 {
                return false;
            }
            let k = v.coefficients.keys().nth(0).unwrap();
            let mut expected_k = vec![ZERO; 3];
            expected_k[i] = ONE;
            *k == expected_k && *v.coefficients.get(k).unwrap() == f.one()
        }));
    }

    #[test]
    fn arithmetic_test() {
        let f = Field::new(*PRIME);
        let three: U256 = 3.into();
        let four: U256 = 4.into();

        let mut coefficients = HashMap::new();
        coefficients.insert(vec![*TWO, ONE], f.one());
        coefficients.insert(vec![ONE, *TWO], f.generator());
        coefficients.insert(vec![ZERO, ZERO], FieldElement::new(*TWO, f));
        let mp = MPolynomial::new(coefficients);
        let cp = MPolynomial::constant(f.one());

        let sum = &mp + &cp;
        assert_eq!(sum.coefficients.keys().len(), 3);
        assert_eq!(
            *sum.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            f.generator()
        );
        assert_eq!(*sum.coefficients.get(&vec![*TWO, ONE]).unwrap(), f.one());
        assert_eq!(
            *sum.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(three, f)
        );

        let sum2 = &mp + &mp;
        assert_eq!(sum2.coefficients.keys().len(), 3);
        assert_eq!(
            *sum2.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            &f.generator() * &FieldElement::new(*TWO, f)
        );
        assert_eq!(
            *sum2.coefficients.get(&vec![*TWO, ONE]).unwrap(),
            &f.one() * &FieldElement::new(*TWO, f)
        );
        assert_eq!(
            *sum2.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(four, f)
        );

        assert_eq!(&mp * &cp, mp);
        let mul = &mp * &mp;
        assert_eq!(mul.coefficients.keys().len(), 6);
        assert_eq!(*mul.coefficients.get(&vec![four, *TWO]).unwrap(), f.one());
        assert_eq!(
            *mul.coefficients.get(&vec![three, three]).unwrap(),
            &f.generator() * &FieldElement::new(*TWO, f)
        );
        assert_eq!(
            *mul.coefficients.get(&vec![*TWO, ONE]).unwrap(),
            FieldElement::new(four, f)
        );
        assert_eq!(
            *mul.coefficients.get(&vec![*TWO, four]).unwrap(),
            &f.generator() ^ *TWO
        );
        assert_eq!(
            *mul.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            &f.generator() * &FieldElement::new(four, f)
        );
        assert_eq!(
            *mul.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(four, f)
        );

        let exp = &mp ^ *TWO;
        assert_eq!(exp, mul);

        let mul3 = &(&mp * &mp) * &mp;
        let exp3 = &mp ^ 3.into();
        assert_eq!(mul3, exp3);

        let sub = &mul - &mp;
        assert_eq!(sub.coefficients.keys().len(), 6);
        assert_eq!(*sub.coefficients.get(&vec![four, *TWO]).unwrap(), f.one());
        assert_eq!(
            *sub.coefficients.get(&vec![three, three]).unwrap(),
            &f.generator() * &FieldElement::new(*TWO, f)
        );
        assert_eq!(
            *sub.coefficients.get(&vec![*TWO, ONE]).unwrap(),
            FieldElement::new(three, f)
        );
        assert_eq!(
            *sub.coefficients.get(&vec![*TWO, four]).unwrap(),
            &f.generator() ^ *TWO
        );
        assert_eq!(
            *sub.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            &f.generator() * &FieldElement::new(three, f)
        );
        assert_eq!(
            *sub.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(*TWO, f)
        );
    }

    #[test]
    fn lift_test() {
        let f = Field::new(*PRIME);
        let poly = Polynomial::new(vec![f.generator(), f.one(), FieldElement::new(*TWO, f)]);
        let mut coefficients = HashMap::new();
        coefficients.insert(vec![ZERO, ZERO, *TWO], FieldElement::new(*TWO, f));
        coefficients.insert(vec![ZERO, ZERO, ONE], f.one());
        coefficients.insert(vec![ZERO, ZERO, ZERO], f.generator());
        let lifted_expected = MPolynomial::new(coefficients);

        let lifted = MPolynomial::lift(&poly, 2);
        assert_eq!(lifted_expected, lifted);
    }

    #[test]
    fn evaluate_test() {
        let f = Field::new(*PRIME);
        let mut coefficients = HashMap::new();
        coefficients.insert(vec![*TWO, ONE, ONE], f.one());
        coefficients.insert(vec![ONE, *TWO, ONE], f.generator());
        coefficients.insert(vec![ZERO, ZERO, *TWO], FieldElement::new(*TWO, f));
        coefficients.insert(vec![ZERO, ZERO, ZERO], FieldElement::new(*TWO, f));
        let mp = MPolynomial::new(coefficients);

        assert_eq!(
            mp.evaluate(&vec![f.one(), f.generator(), f.zero()]),
            FieldElement::new(*TWO, f)
        );
        assert_eq!(
            mp.evaluate(&vec![f.one(), f.generator(), f.generator()]),
            &(&(&(&f.generator() ^ 2.into()) + &(&f.generator() ^ 4.into()))
                + &(&(&f.generator() ^ *TWO) * &FieldElement::new(*TWO, f)))
                + &FieldElement::new(*TWO, f)
        );

        let mut coefficients = HashMap::new();
        coefficients.insert(vec![*TWO, ONE], f.one());
        coefficients.insert(vec![ONE, *TWO], f.generator());
        coefficients.insert(vec![ZERO, *TWO], FieldElement::new(*TWO, f));
        coefficients.insert(vec![ZERO, ZERO], FieldElement::new(*TWO, f));
        let mp = MPolynomial::new(coefficients);

        let poly0 = Polynomial::new(vec![FieldElement::new(*TWO, f), f.generator(), f.one()]);
        let poly1 = Polynomial::new(vec![f.zero(), f.one()]);
        let polys = vec![poly0, poly1];
        assert_eq!(
            mp.evaluate_symbolic(&polys),
            Polynomial::new(vec![
                FieldElement::new(*TWO, f),
                FieldElement::new(4.into(), f),
                &(&FieldElement::new(6.into(), f) * &f.generator()) + &FieldElement::new(*TWO, f),
                &(&(&f.generator() ^ 2.into()) * &FieldElement::new(*TWO, f))
                    + &FieldElement::new(4.into(), f),
                &f.generator() * &FieldElement::new(3.into(), f),
                f.one()
            ])
        );
    }
}
