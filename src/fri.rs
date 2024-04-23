use crate::{element::FieldElement, field::Field};

pub struct FRI {
    pub offset: FieldElement,
    pub omega: FieldElement,
    pub domain_length: usize,
    pub field: Field,
    pub expansion_factor: usize,
    pub num_colinearity_tests: usize,
}

impl FRI {
    pub fn new(
        offset: FieldElement,
        omega: FieldElement,
        initial_domain_length: usize,
        expansion_factor: usize,
        num_colinearity_tests: usize,
    ) -> Self {
        FRI {
            offset,
            omega,
            domain_length: initial_domain_length,
            field: omega.field,
            expansion_factor,
            num_colinearity_tests,
        }
    }

    pub fn num_rounds(&self) -> usize {
        let mut codeword_length = self.domain_length;
        let mut num_rounds = 0;
        while codeword_length > self.expansion_factor
            && 4 * self.num_colinearity_tests < codeword_length
        {
            codeword_length /= 2;
            num_rounds += 1;
        }
        num_rounds
    }

    pub fn eval_domain(&self) -> Vec<FieldElement> {
        (0..self.domain_length)
            .map(|i| &self.offset * &(&self.omega ^ i.into()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::*;

    #[test]
    fn fri_test() {
        let f = Field::new(*PRIME);

        let fri = FRI::new(f.one(), f.generator(), 8, 2, 1);
        assert_eq!(fri.num_rounds(), 1);

        let fri = FRI::new(f.one(), f.generator(), 16, 2, 1);
        assert_eq!(fri.num_rounds(), 2);

        let fri = FRI::new(FieldElement::new(*TWO, f), f.generator(), 3, 2, 1);
        let two = FieldElement::new(*TWO, f);
        assert_eq!(
            fri.eval_domain(),
            vec![two, &two * &f.generator(), &two * &(&f.generator() ^ *TWO)]
        );
    }
}
