use automata_sgx_sdk::types::SgxStatus;
// For most of the cases, you can use the external library directly.
use std::fs::File;
use std::io::Write;
use bls::SecretKey;
// use blst::min_pk::SecretKey;

// Declare the OCALL function. The automata_sgx_sdk will link the OCALL to the mock_lib.
extern "C" {
    fn untrusted_execution(random_number: i32);
}

/**
 * This is an ECALL function defined in the edl file.
 * It will be called by the application.
 */
#[no_mangle]
pub unsafe extern "C" fn trusted_execution() -> SgxStatus {
    println!("=============== Trusted execution =================");

    //bls signature :)
    let ikm = [4u8; 32];
    let sk = SecretKey::random();
    //let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
    let dst = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";
    let msg = b"another message to be signed!!!!"; // exactly 32 bytes

    let sig = sk.sign(msg.into());
    let sig_bytes = sig.serialize();
    let extended: [u8; 64] = sig_bytes[..64].try_into().unwrap();
    //let extended = [0u8; 64];
    // println!("signature: {:#?}", sig);

    // The following code is used to generate an attestation report
    // Must be run on sgx-supported machine
    let attestation = automata_sgx_sdk::dcap::dcap_quote(extended);
    let result = match attestation {
        Ok(attestation) => {
            //println!("DCAP attestation: 0x{}", hex::encode(&attestation));
            let mut file = File::create("quote.dat").unwrap();
            let _ = file.write_all(&attestation);
            SgxStatus::Success
        }
        Err(e) => {
            println!("Generating attestation failed: {:?}", e);
            SgxStatus::Unexpected
        }
    };
    println!("=============== End of trusted execution =================");
    println!("status: {}", result);
    result
}
