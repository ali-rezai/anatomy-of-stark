use primitive_types::U256;

use crate::{element::FieldElement, field::Field, ONE, ZERO};
use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub struct MPolynomial<'a> {
    pub coefficients: HashMap<Vec<U256>, FieldElement<'a>>,
}

impl<'a> MPolynomial<'a> {
    pub fn new(coefficients: HashMap<Vec<U256>, FieldElement<'a>>) -> Self {
        MPolynomial { coefficients }
    }

    pub fn constant(element: FieldElement<'a>) -> Self {
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

    pub fn variables(num_variables: usize, field: &'a Field) -> Vec<MPolynomial<'a>> {
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
}

impl<'a> std::ops::Add<&MPolynomial<'a>> for &MPolynomial<'a> {
    type Output = MPolynomial<'a>;

    fn add(self, rhs: &MPolynomial<'a>) -> MPolynomial<'a> {
        let mut map = HashMap::new();
        let self_keys = self
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap()
            .len();
        let rhs_keys = rhs
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap()
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

impl<'a> std::ops::Neg for &MPolynomial<'a> {
    type Output = MPolynomial<'a>;

    fn neg(self) -> MPolynomial<'a> {
        let mut map = HashMap::new();
        self.coefficients.iter().for_each(|e| {
            map.insert(e.0.clone(), -e.1);
        });
        MPolynomial::new(map)
    }
}

impl<'a> std::ops::Sub<&MPolynomial<'a>> for &MPolynomial<'a> {
    type Output = MPolynomial<'a>;

    fn sub(self, rhs: &MPolynomial<'a>) -> MPolynomial<'a> {
        self + &(-rhs)
    }
}

impl<'a> std::ops::Mul<&MPolynomial<'a>> for &MPolynomial<'a> {
    type Output = MPolynomial<'a>;

    fn mul(self, rhs: &MPolynomial<'a>) -> MPolynomial<'a> {
        let mut map = HashMap::new();
        let self_keys = self
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap()
            .len();
        let rhs_keys = rhs
            .coefficients
            .keys()
            .max_by_key(|k| k.len())
            .unwrap()
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

impl<'a> std::ops::BitXor<U256> for &MPolynomial<'a> {
    type Output = MPolynomial<'a>;

    fn bitxor(self, rhs: U256) -> MPolynomial<'a> {
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
        coefficients.insert(vec![ZERO, ZERO], FieldElement::new(*TWO, &f));
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
            FieldElement::new(three, &f)
        );

        let sum2 = &mp + &mp;
        assert_eq!(sum2.coefficients.keys().len(), 3);
        assert_eq!(
            *sum2.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            &f.generator() * &FieldElement::new(*TWO, &f)
        );
        assert_eq!(
            *sum2.coefficients.get(&vec![*TWO, ONE]).unwrap(),
            &f.one() * &FieldElement::new(*TWO, &f)
        );
        assert_eq!(
            *sum2.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(four, &f)
        );

        assert_eq!(&mp * &cp, mp);
        let mul = &mp * &mp;
        assert_eq!(mul.coefficients.keys().len(), 6);
        assert_eq!(*mul.coefficients.get(&vec![four, *TWO]).unwrap(), f.one());
        assert_eq!(
            *mul.coefficients.get(&vec![three, three]).unwrap(),
            &f.generator() * &FieldElement::new(*TWO, &f)
        );
        assert_eq!(
            *mul.coefficients.get(&vec![*TWO, ONE]).unwrap(),
            FieldElement::new(four, &f)
        );
        assert_eq!(
            *mul.coefficients.get(&vec![*TWO, four]).unwrap(),
            &f.generator() ^ *TWO
        );
        assert_eq!(
            *mul.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            &f.generator() * &FieldElement::new(four, &f)
        );
        assert_eq!(
            *mul.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(four, &f)
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
            &f.generator() * &FieldElement::new(*TWO, &f)
        );
        assert_eq!(
            *sub.coefficients.get(&vec![*TWO, ONE]).unwrap(),
            FieldElement::new(three, &f)
        );
        assert_eq!(
            *sub.coefficients.get(&vec![*TWO, four]).unwrap(),
            &f.generator() ^ *TWO
        );
        assert_eq!(
            *sub.coefficients.get(&vec![ONE, *TWO]).unwrap(),
            &f.generator() * &FieldElement::new(three, &f)
        );
        assert_eq!(
            *sub.coefficients.get(&vec![ZERO, ZERO]).unwrap(),
            FieldElement::new(*TWO, &f)
        );
    }
}
