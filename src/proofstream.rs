use serde::{Deserialize, Serialize};
use sha3::digest::ExtendableOutput;

#[derive(PartialEq, Debug)]
pub struct ProofStream<T> {
    pub objects: Vec<T>,
    pub read_index: usize,
}

impl<'a, T: Copy + Serialize + Deserialize<'a>> ProofStream<T> {
    pub fn new() -> Self {
        ProofStream {
            objects: vec![],
            read_index: 0,
        }
    }
    pub fn push(&mut self, obj: T) {
        self.objects.push(obj);
    }
    pub fn pull(&mut self) -> T {
        assert!(self.read_index < self.objects.len());
        let obj = self.objects[self.read_index];
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

    pub fn prove_fiat_shamir(&self, num_bytes: usize) -> Vec<u8> {
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
    use super::ProofStream;

    #[test]
    fn proofstream_test() {
        let mut ps = ProofStream::new();
        ps.push(1);
        ps.push(7);
        assert_eq!(ps.pull(), 1);
        ps.push(9);
        assert_eq!(ps.pull(), 7);
        assert_eq!(ps.pull(), 9);
    }

    #[test]
    fn serialization_test() {
        let mut ps: ProofStream<i32> = ProofStream::new();
        ps.push(1);
        ps.push(7);
        ps.push(9);

        let v = ps.serialize();
        let d: ProofStream<i32> = ProofStream::deserialize(&v);
        assert_eq!(ps, d);
    }

    #[test]
    fn verification_test() {
        let mut ps: ProofStream<i32> = ProofStream::new();
        ps.push(1);
        ps.push(7);
        ps.push(9);

        let prove0 = ps.prove_fiat_shamir(32);
        let verify0 = ps.verifier_fiat_shamir(32);
        assert_ne!(prove0, verify0);

        ps.pull();
        ps.pull();
        ps.pull();
        let prove1 = ps.prove_fiat_shamir(32);
        let verify1 = ps.verifier_fiat_shamir(32);
        assert_eq!(prove0, prove1);
        assert_eq!(prove1, verify1);
    }
}
