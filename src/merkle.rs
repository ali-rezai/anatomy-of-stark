use blake2::Blake2bVar;
use sha3::digest::{Update, VariableOutput};

fn hash(data: &[u8]) -> Vec<u8> {
    let mut hasher = Blake2bVar::new(32).unwrap();
    hasher.update(data);
    let mut out = vec![0; 32];
    hasher.finalize_variable(&mut out).unwrap();
    out
}

pub struct Merkle {}

impl Merkle {
    pub fn commit_(leafs: &[Vec<u8>]) -> Vec<u8> {
        let len = leafs.len();
        assert!(len & (len - 1) == 0);
        if len == 1 {
            return leafs[0].clone();
        }

        let mut combined = Vec::from(Merkle::commit_(&leafs[0..len / 2]));
        combined.extend(Merkle::commit_(&leafs[len / 2..len]));
        hash(&combined)
    }

    pub fn open_(index: usize, leafs: &[Vec<u8>]) -> Vec<Vec<u8>> {
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

    pub fn verify_(root: &[u8], index: usize, path: &[Vec<u8>], leaf: &[u8]) -> bool {
        let len = path.len();
        assert!(index < (1 << path.len()));
        let mut data;
        if index == 0 {
            data = Vec::from(leaf);
            data.extend(&path[0]);
            return root == hash(&data);
        } else {
            data = path[0].clone();
            data.extend(leaf);
        }
        let hash = hash(&data);
        if len == 1 {
            return root == hash;
        } else {
            if index % 2 == 0 {
                return Merkle::verify_(root, index >> 1, &path[1..], &hash);
            } else {
                return Merkle::verify_(root, index >> 1, &path[1..], &hash);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{hash, Merkle};

    #[test]
    fn commit_test() {
        let leafs = vec![vec![1], vec![2], vec![3], vec![4]];
        let root = Merkle::commit_(&leafs);

        let mut expected_root = hash(&[1, 2]);
        expected_root.extend(hash(&[3, 4]));
        assert_eq!(root, hash(&expected_root));
    }

    #[test]
    fn open_test() {
        let leafs = vec![vec![1], vec![2], vec![3], vec![4]];
        let path = Merkle::open_(1, &leafs);

        let mut expected_path = vec![vec![1]];
        expected_path.push(hash(&vec![3, 4]));

        assert_eq!(path, expected_path);
    }

    #[test]
    fn verify_test() {
        let leafs = vec![vec![1], vec![2], vec![3], vec![4]];

        let root = Merkle::commit_(&leafs);
        let path = Merkle::open_(1, &leafs);

        assert!(Merkle::verify_(&root, 1, &path, &vec![2]));
        assert!(!Merkle::verify_(&root, 2, &path, &vec![2]));
    }
}
