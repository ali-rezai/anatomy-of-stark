use blake2::Blake2bVar;
use serde::Serialize;
use sha3::digest::{Update, VariableOutput};

pub fn hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(data);
    let mut out = vec![0; 32];
    hasher.finalize_variable(&mut out).unwrap();
    out
}

pub struct Merkle {}

impl Merkle {
    fn commit_(leafs: &[Vec<u8>]) -> Vec<u8> {
        let len = leafs.len();
        assert!(len & (len - 1) == 0);
        if len == 1 {
            return leafs[0].clone();
        }

        let mut combined = Vec::from(Merkle::commit_(&leafs[0..len / 2]));
        combined.extend(Merkle::commit_(&leafs[len / 2..len]));
        hash(&combined)
    }

    fn open_(index: usize, leafs: &[Vec<u8>]) -> Vec<Vec<u8>> {
        let len = leafs.len();
        assert!(len & (len - 1) == 0);
        assert!(index < len);
        if len == 2 {
            return vec![leafs[1 - index].clone()];
        } else if index < len / 2 {
            let mut combined = Merkle::open_(index, &leafs[0..len / 2]);
            combined.push(Merkle::commit_(&leafs[len / 2..len]));
            return combined;
        } else {
            let mut combined = Merkle::open_(index - len / 2, &leafs[len / 2..len]);
            combined.push(Merkle::commit_(&leafs[0..len / 2]));
            return combined;
        }
    }

    fn verify_(root: &[u8], index: usize, path: &[Vec<u8>], leaf: &[u8]) -> bool {
        let len = path.len();
        assert!(index < (1 << path.len()));
        let mut data;
        if index % 2 == 0 {
            data = Vec::from(leaf);
            data.extend(&path[0]);
        } else {
            data = path[0].clone();
            data.extend(leaf);
        }
        let hash = hash(&data);
        if len == 1 {
            return root == hash;
        } else {
            return Merkle::verify_(root, index >> 1, &path[1..], &hash);
        }
    }

    fn hash_data_array<T: Serialize>(data_array: &Vec<T>) -> Vec<Vec<u8>> {
        let mut hash_data: Vec<Vec<u8>> = data_array
            .iter()
            .map(|data| {
                let bytes = serde_pickle::to_vec(data, Default::default()).unwrap();
                hash(&bytes)
            })
            .collect();
        let len = hash_data.len();
        if len & (len - 1) != 0 {
            hash_data.resize_with(len.next_power_of_two(), || Vec::new());
        }
        hash_data
    }

    pub fn commit<T: Serialize>(data_array: &Vec<T>) -> Vec<u8> {
        Merkle::commit_(&Merkle::hash_data_array(data_array))
    }

    pub fn open<T: Serialize>(index: usize, data_array: &Vec<T>) -> Vec<Vec<u8>> {
        Merkle::open_(index, &Merkle::hash_data_array(data_array))
    }

    pub fn verify<T: Serialize>(
        root: &[u8],
        index: usize,
        path: &[Vec<u8>],
        data_element: &T,
    ) -> bool {
        let bytes = serde_pickle::to_vec(data_element, Default::default()).unwrap();
        let leaf = hash(&bytes);
        Merkle::verify_(root, index, path, &leaf)
    }
}

#[cfg(test)]
mod tests {
    use super::{hash, Merkle};

    fn combine(a: &[u8], b: &[u8]) -> Vec<u8> {
        let mut combined = Vec::from(a);
        combined.extend(b);
        combined
    }

    #[test]
    fn commit_test() {
        let leafs = vec![vec![1], vec![2], vec![3], vec![4]];
        let root = Merkle::commit(&leafs);

        let hashed_leafs = Merkle::hash_data_array(&leafs);

        let mut expected_root = hash(&combine(&hashed_leafs[0], &hashed_leafs[1]));
        expected_root.extend(hash(&combine(&hashed_leafs[2], &hashed_leafs[3])));

        assert_eq!(root, hash(&expected_root));
    }

    #[test]
    fn open_test() {
        let leafs = vec![vec![1], vec![2], vec![3], vec![4]];
        let path = Merkle::open(1, &leafs);

        let hashed_leafs = Merkle::hash_data_array(&leafs);

        let mut expected_path = vec![hashed_leafs[0].clone()];
        expected_path.push(hash(&combine(&hashed_leafs[2], &hashed_leafs[3])));

        assert_eq!(path, expected_path);
    }

    #[test]
    fn verify_test() {
        let leafs = vec![vec![1], vec![2], vec![3], vec![4]];

        let root = Merkle::commit(&leafs);

        let path = Merkle::open(0, &leafs);
        assert!(Merkle::verify(&root, 0, &path, &vec![1]));

        let path = Merkle::open(1, &leafs);
        assert!(Merkle::verify(&root, 1, &path, &vec![2]));
        assert!(!Merkle::verify(&root, 2, &path, &vec![2]));
    }
}
