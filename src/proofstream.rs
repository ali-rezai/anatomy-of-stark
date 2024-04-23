use serde::{Deserialize, Serialize};
use sha3::digest::ExtendableOutput;

#[derive(PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum Object<T> {
    HASH(Vec<u8>),
    PATH(Vec<Vec<u8>>),
    LEAF(T),
    OBJ(T),
}

#[derive(PartialEq, Debug)]
pub struct ProofStream<T> {
    pub objects: Vec<Object<T>>,
    pub read_index: usize,
}

impl<'a, T: Clone + Serialize + Deserialize<'a>> ProofStream<T> {
    pub fn new() -> Self {
        ProofStream {
            objects: vec![],
            read_index: 0,
        }
    }
    pub fn push(&mut self, obj: Object<T>) {
        self.objects.push(obj);
    }

    pub fn push_hash(&mut self, hash: Vec<u8>) {
        self.objects.push(Object::HASH(hash));
    }

    pub fn push_obj(&mut self, obj: T) {
        self.objects.push(Object::OBJ(obj));
    }

    pub fn push_path(&mut self, path: Vec<Vec<u8>>) {
        self.objects.push(Object::PATH(path));
    }

    pub fn push_leafs(&mut self, leaf_index: T) {
        self.objects.push(Object::LEAF(leaf_index));
    }

    pub fn pull(&mut self) -> Object<T> {
        assert!(self.read_index < self.objects.len());
        let obj = self.objects[self.read_index].clone();
        self.read_index += 1;
        obj
    }

    pub fn serialize(&self) -> Vec<u8> {
        serde_pickle::to_vec(&self.objects, Default::default()).unwrap()
    }

    pub fn deserialize(data: &Vec<u8>) -> Self {
        ProofStream {
            objects: serde_pickle::from_slice(&data, Default::default()).unwrap(),
            read_index: 0,
        }
    }

    pub fn prover_fiat_shamir(&self, num_bytes: usize) -> Vec<u8> {
        let mut output = vec![0u8; num_bytes];
        sha3::Shake256::digest_xof(&self.serialize(), &mut output);
        output
    }

    pub fn verifier_fiat_shamir(&self, num_bytes: usize) -> Vec<u8> {
        let mut output = vec![0u8; num_bytes];

        let input = &self.objects[0..self.read_index];
        let input = serde_pickle::to_vec(&input, Default::default()).unwrap();

        sha3::Shake256::digest_xof(input, &mut output);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::{Object::OBJ, ProofStream};
    use crate::{consts::*, element::FieldElement, field::Field};

    #[test]
    fn proofstream_test() {
        let f = Field::new(*PRIME);
        let mut ps = ProofStream::new();
        ps.push_obj(f.one());
        ps.push_obj(f.zero());
        assert_eq!(ps.pull(), OBJ(f.one()));
        ps.push_obj(f.generator());
        assert_eq!(ps.pull(), OBJ(f.zero()));
        assert_eq!(ps.pull(), OBJ(f.generator()));
    }

    #[test]
    fn serialization_test() {
        let f = Field::new(*PRIME);
        let mut ps = ProofStream::new();
        ps.push_obj(f.one());
        ps.push_obj(f.zero());
        ps.push_obj(f.generator());

        let v = ps.serialize();
        let d: ProofStream<FieldElement> = ProofStream::deserialize(&v);
        assert_eq!(ps, d);
    }

    #[test]
    fn verification_test() {
        let f = Field::new(*PRIME);
        let mut ps = ProofStream::new();
        ps.push_obj(f.one());
        ps.push_obj(f.zero());
        ps.push_obj(f.generator());

        let prove0 = ps.prover_fiat_shamir(32);
        let verify0 = ps.verifier_fiat_shamir(32);
        assert_ne!(prove0, verify0);

        ps.pull();
        ps.pull();
        ps.pull();
        let prove1 = ps.prover_fiat_shamir(32);
        let verify1 = ps.verifier_fiat_shamir(32);
        assert_eq!(prove0, prove1);
        assert_eq!(prove1, verify1);
    }
}
