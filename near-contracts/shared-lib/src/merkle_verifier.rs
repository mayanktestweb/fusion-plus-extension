use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::env;

/// Sort and hash two byte slices using Keccak256
fn hash_pair(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut combined = vec![];
    if a <= b {
        combined.extend_from_slice(a);
        combined.extend_from_slice(b);
    } else {
        combined.extend_from_slice(b);
        combined.extend_from_slice(a);
    }
    env::keccak256(&combined)
}

/// Verifies a Merkle proof (leaf is already hashed once)
pub fn verify_proof(leaf: &[u8], proof: &[Vec<u8>], root: &[u8]) -> bool {
    let mut computed_hash = leaf.to_vec(); // NOTE: no hash here!

    for proof_element in proof {
        computed_hash = hash_pair(&computed_hash, proof_element);
    }

    computed_hash == root
}

#[derive(Default, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct MerkleVerifier;

impl MerkleVerifier {
    /// Verifies Merkle proof. All inputs are hex strings (no 0x prefix)
    pub fn verify(leaf_hex: &str, proof_hex: Vec<String>, root_hex: &str) -> bool {

        // strip 0x if they exist
        let leaf_hex = Self::strip_0x(leaf_hex);
        let root_hex = Self::strip_0x(root_hex);
        let proof_hex = proof_hex.iter().map(|p| Self::strip_0x(&p.clone())).collect::<Vec<String>>();


        let leaf = hex::decode(&leaf_hex).expect("Invalid leaf hex");
        let root = hex::decode(&root_hex).expect("Invalid root hex");
        let proof: Vec<Vec<u8>> = proof_hex
            .iter()
            .map(|p| hex::decode(p).expect("Invalid proof hex"))
            .collect();

        verify_proof(&leaf, &proof, &root)
    }

    /// Generate keccak256(index, keccak256(secret))
    pub fn indexed_secret_hash_string(index: u16, hashed_secret: &str) -> String {
        let mut combined = Vec::new();

        let hashed_secret = Self::strip_0x(hashed_secret);

        // Encode index as uint16 (big-endian)
        combined.extend_from_slice(&index.to_be_bytes());

        // Decode hashed_secret from hex 
        let hash_bytes = hex::decode(&hashed_secret).expect("Invalid hex");
        combined.extend_from_slice(&hash_bytes);

        let value = env::keccak256(&combined);
        let value = hex::encode(value);

        value
    }

    fn strip_0x(hex_string: &str) -> String {
        if hex_string.starts_with("0x") {
            hex_string[2..].to_string()
        } else {
            hex_string.to_string()
        }
    }
}
