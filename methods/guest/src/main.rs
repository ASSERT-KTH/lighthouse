use std::io::Read;

use risc0_zkvm::guest::env;

fn main() {
    let mut input_key = String::new();
    env::stdin().read_to_string(&mut input_key).unwrap();

    env::commit(&input_key);
}




