use blst::min_pk::SecretKey;
use risc0_zkvm::guest::env;

fn main() {
    let mut intro = [0u8; 4];
    env::read_slice(&mut intro);

    let mut secret_key = [0u8; 32];
    env::read_slice(&mut secret_key);

    let mut msg = [0u8; 32];
    env::read_slice(&mut msg);

    let sk = SecretKey::from_bytes(&secret_key).unwrap();

    let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

    let sig = sk.sign(&msg, dst, &[]);

    let serialized = sig.to_bytes();

    env::commit_slice(&serialized);
}

