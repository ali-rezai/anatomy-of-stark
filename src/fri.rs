use crate::{
    consts::*,
    element::FieldElement,
    field::Field,
    merkle::{self, Merkle},
    polynomial::Polynomial,
    proofstream::{Object, ProofStream},
};
use core::panic;

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
        if num_rounds == 1 && codeword_length > self.expansion_factor {
            num_rounds += 1;
        }
        num_rounds
    }

    pub fn eval_domain(&self) -> Vec<FieldElement> {
        (0..self.domain_length)
            .map(|i| &self.offset * &(&self.omega ^ i.into()))
            .collect()
    }

    pub fn commit(
        &self,
        mut codeword: Vec<FieldElement>,
        proof_stream: &mut ProofStream<Vec<FieldElement>>,
    ) -> Vec<Vec<FieldElement>> {
        let one = self.field.one();
        let two = FieldElement::new(*TWO, self.field);
        let mut omega = self.omega;
        let mut offset = self.offset;
        let mut codewords = vec![];

        for r in 0..self.num_rounds() {
            let root = Merkle::commit(&codeword);
            proof_stream.push_hash(root);

            if r == self.num_rounds() - 1 {
                break;
            }

            let alpha = self.field.sample(&proof_stream.prover_fiat_shamir(32));
            codewords.push(codeword.clone());
            codeword = (0..codeword.len() / 2)
                .map(|i| {
                    &(&(&(&one + &(&alpha / &(&offset * &(&omega ^ i.into())))) * &codeword[i])
                        + &(&(&one - &(&alpha / &(&offset * &(&omega ^ i.into()))))
                            * &codeword[codeword.len() / 2 + i]))
                        * &two.inv()
                })
                .collect();

            omega = &omega ^ two.value;
            offset = &offset ^ two.value;
        }

        proof_stream.push_obj(codeword.clone());
        codewords.push(codeword);
        codewords
    }

    pub fn query(
        &self,
        current_codeword: &Vec<FieldElement>,
        next_codeword: &Vec<FieldElement>,
        c_indices: &Vec<usize>,
        proof_stream: &mut ProofStream<Vec<FieldElement>>,
    ) -> Vec<usize> {
        let mut a_indices = c_indices.clone();
        let b_indices: Vec<usize> = c_indices
            .iter()
            .map(|i| i + current_codeword.len() / 2)
            .collect();

        for s in 0..self.num_colinearity_tests {
            let leafs = vec![
                current_codeword[a_indices[s]],
                current_codeword[b_indices[s]],
                next_codeword[c_indices[s]],
            ];
            proof_stream.push_leafs(leafs);
        }

        for s in 0..self.num_colinearity_tests {
            proof_stream.push_path(Merkle::open(a_indices[s], current_codeword));
            proof_stream.push_path(Merkle::open(b_indices[s], current_codeword));
            proof_stream.push_path(Merkle::open(c_indices[s], next_codeword));
        }

        a_indices.extend(b_indices);
        a_indices
    }

    pub fn sample_index(byte_array: &Vec<u8>, size: usize) -> usize {
        let mut acc = 0;
        byte_array.iter().for_each(|b| {
            acc = acc << 8 ^ (*b as usize);
        });
        acc % size
    }

    pub fn sample_indices(
        seed: &Vec<u8>,
        size: usize,
        reduced_size: usize,
        number: usize,
    ) -> Vec<usize> {
        assert!(number <= reduced_size);
        let mut indices = vec![];
        let mut reduced_indices = vec![];
        let mut counter = 0usize;

        let mut bytes = seed.clone();
        counter.to_be_bytes().iter().for_each(|b| {
            bytes.push(*b);
        });

        while indices.len() < number {
            let index = FRI::sample_index(&merkle::hash(&bytes), size);
            let reduced_index = index % reduced_size;

            counter += 1;
            let mut l = bytes.len();
            counter.to_le_bytes().iter().for_each(|b| {
                bytes[l - 1] = *b;
                l -= 1;
            });

            if !reduced_indices.contains(&reduced_index) {
                indices.push(index);
                reduced_indices.push(reduced_index);
            }
        }
        indices
    }

    pub fn prove(
        &self,
        codeword: &Vec<FieldElement>,
        proof_stream: &mut ProofStream<Vec<FieldElement>>,
    ) -> Vec<usize> {
        assert!(self.domain_length == codeword.len());
        let codewords = self.commit(codeword.clone(), proof_stream);
        let top_level_indices = FRI::sample_indices(
            &proof_stream.prover_fiat_shamir(32),
            codewords[1].len(),
            codewords.last().unwrap().len(),
            self.num_colinearity_tests,
        );
        let mut indices = top_level_indices.clone();

        codewords.iter().enumerate().for_each(|(i, codeword)| {
            if i < codewords.len() - 1 {
                indices = indices
                    .iter()
                    .map(|index| index % (codeword.len() / 2))
                    .collect();
                self.query(codeword, &codewords[i + 1], &indices, proof_stream);
            }
        });

        top_level_indices
    }

    pub fn verify(
        &self,
        proof_stream: &mut ProofStream<Vec<FieldElement>>,
        mut polynomial_values: Vec<(usize, FieldElement)>,
    ) -> bool {
        let two = FieldElement::new(*TWO, self.field);
        let mut omega = self.omega;
        let mut offset = self.offset;

        let mut roots = vec![];
        let mut alphas = vec![];
        for _ in 0..self.num_rounds() {
            if let Object::HASH(root) = proof_stream.pull() {
                roots.push(root);
            } else {
                panic!("Expected hash");
            }
            alphas.push(self.field.sample(&proof_stream.verifier_fiat_shamir(32)));
        }

        let last_codeword = match proof_stream.pull() {
            Object::OBJ(codeword) => codeword,
            _ => panic!("Expected object"),
        };

        if *roots.last().unwrap() != Merkle::commit(&last_codeword) {
            println!("Malformed last_codeword");
            return false;
        }

        let degree: i32 = (last_codeword.len() / self.expansion_factor - 1)
            .try_into()
            .unwrap();
        let mut last_omega = omega;
        let mut last_offset = offset;
        for _ in 0..self.num_rounds() - 1 {
            last_omega = &last_omega ^ two.value;
            last_offset = &last_offset ^ two.value;
        }
        assert!(last_omega.inv() == &last_omega ^ (last_codeword.len() - 1).into());

        let last_domain: Vec<FieldElement> = (0..last_codeword.len())
            .map(|i| &last_offset * &(&last_omega ^ i.into()))
            .collect();
        let poly = Polynomial::interpolate_domain(&last_domain, &last_codeword);
        assert!(poly.evaluate_domain(&last_domain) == last_codeword);

        if poly.degree() > degree {
            println!("last codeword does not correspond to polynomial of low enough degree");
            println!("observed degree: {}", poly.degree());
            println!("but should be: {}", degree);
            return false;
        }

        let top_level_indices = FRI::sample_indices(
            &proof_stream.verifier_fiat_shamir(32),
            self.domain_length >> 1,
            self.domain_length >> (self.num_rounds() - 1),
            self.num_colinearity_tests,
        );

        for r in 0..self.num_rounds() - 1 {
            let c_indices: Vec<usize> = top_level_indices
                .iter()
                .map(|index| *index % (self.domain_length >> (r + 1)))
                .collect();
            let a_indices = c_indices.clone();
            let b_indices: Vec<usize> = a_indices
                .iter()
                .map(|index| *index + (self.domain_length >> (r + 1)))
                .collect();

            let mut aa = vec![];
            let mut bb = vec![];
            let mut cc = vec![];
            for s in 0..self.num_colinearity_tests {
                let (ay, by, cy) = match proof_stream.pull() {
                    Object::LEAF(leafs) => (leafs[0], leafs[1], leafs[2]),
                    _ => panic!("Expected a leaf"),
                };

                aa.push(ay);
                bb.push(by);
                cc.push(cy);

                if r == 0 {
                    polynomial_values.push((a_indices[s], ay));
                    polynomial_values.push((b_indices[s], by));
                }

                let ax = &offset * &(&omega ^ a_indices[s].into());
                let bx = &offset * &(&omega ^ b_indices[s].into());
                let cx = alphas[r];

                if !Polynomial::test_colinearity(&vec![(ax, ay), (bx, by), (cx, cy)]) {
                    println!("Faild colinearity check");
                    return false;
                }
            }

            for i in 0..self.num_colinearity_tests {
                let path = match proof_stream.pull() {
                    Object::PATH(p) => p,
                    _ => panic!("Expected path"),
                };
                if !Merkle::verify(&roots[r], a_indices[i], &path, &aa[i]) {
                    println!("Auth path fail for aa");
                    return false;
                }

                let path = match proof_stream.pull() {
                    Object::PATH(p) => p,
                    _ => panic!("Expected path"),
                };
                if !Merkle::verify(&roots[r], b_indices[i], &path, &bb[i]) {
                    println!("Auth path fail for bb");
                    return false;
                }

                let path = match proof_stream.pull() {
                    Object::PATH(p) => p,
                    _ => panic!("Expected path"),
                };
                if !Merkle::verify(&roots[r + 1], c_indices[i], &path, &cc[i]) {
                    println!("Auth path fail for cc");
                    return false;
                }
            }

            omega = &omega ^ two.value;
            offset = &offset ^ two.value;
        }

        true
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn fri_test() {
        let f = Field::new(*PRIME);

        let fri = FRI::new(f.one(), f.generator(), 8, 2, 1);
        assert_eq!(fri.num_rounds(), 2);

        let fri = FRI::new(f.one(), f.generator(), 16, 2, 1);
        assert_eq!(fri.num_rounds(), 2);

        let fri = FRI::new(FieldElement::new(*TWO, f), f.generator(), 3, 2, 1);
        let two = FieldElement::new(*TWO, f);
        assert_eq!(
            fri.eval_domain(),
            vec![two, &two * &f.generator(), &two * &(&f.generator() ^ *TWO)]
        );
    }

    #[test]
    fn verification_test() {
        let f = Field::new(17.into());
        let fri = FRI::new(
            FieldElement::new(13.into(), f),
            FieldElement::new(7.into(), f),
            16,
            7,
            1,
        );
        let codeword = vec![
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
            f.one(),
            f.zero(),
        ];
        let mut ps = ProofStream::new();
        fri.prove(&codeword, &mut ps);
        assert!(!fri.verify(&mut ps, vec![]));

        let f = Field::new(7.into());
        let fri = FRI::new(
            FieldElement::new(1.into(), f),
            FieldElement::new(5.into(), f),
            6,
            1,
            1,
        );

        let p = Polynomial::new(vec![
            FieldElement::new(3.into(), f),
            FieldElement::new(4.into(), f),
            FieldElement::new(*TWO, f),
            f.one(),
        ]);
        let codeword = p.evaluate_domain(&vec![
            f.zero(),
            fri.omega,
            &fri.omega ^ 2.into(),
            &fri.omega ^ 3.into(),
            &fri.omega ^ 4.into(),
            &fri.omega ^ 5.into(),
        ]);
        let mut ps = ProofStream::new();
        fri.prove(&codeword, &mut ps);
        assert!(fri.verify(&mut ps, vec![]));
    }
}
