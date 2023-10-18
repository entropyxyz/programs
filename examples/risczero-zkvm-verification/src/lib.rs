//! This example shows how to write a contrieved and basic constraint: checking the length of the data to be signed.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::string::ToString;

use ec_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

use risc0_zkvm::Receipt;
use bincode;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub mod zkvm_verification {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct VerificationData {
        pub image_id: [u32; 8],
        pub receipt: Receipt,
    }
}
use zkvm_verification::*;

pub struct ZkVmVerificationProgram;

impl Program for ZkVmVerificationProgram {
    fn evaluate(signature_request: InitialState) -> Result<(), Error> {
        let VerificationData { receipt, image_id } = bincode::deserialize(&signature_request.data).map_err(|_| Error::InvalidTransactionRequest("Could not parse data into VerificationData".to_string()))?;

        receipt.verify(image_id).map_err(|_| Error::Evaluation("Proof verification failed".to_string()))?;

        Ok(())
    }
}

export_program!(ZkVmVerificationProgram);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;
    use zkvm_verification::VerificationData;

    use std::fs::{File, create_dir_all};
    use std::path::Path;
    use std::io::{Write, Read};

    use json_methods::{SEARCH_JSON_ELF, SEARCH_JSON_ID};
    use risc0_zkvm::{
        default_prover,
        serde::to_vec,
        ExecutorEnv,
    };

    /// Read a file from disk and deserialize it using bincode
    fn read_struct_from_file<T>(filename: &str) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let dest_path = std::path::Path::new("test_data").join(filename);

        // Read the serialized data from disk
        let mut f = File::open(dest_path).expect("Failed to open file");
        let mut serialized = Vec::new();
        f.read_to_end(&mut serialized).expect("Failed to read data");

        // Deserialize using bincode
        bincode::deserialize(&serialized).expect("Failed to deserialize data")
    }

    /// Read the test Receipt from disk
    fn read_test_receipt() -> Receipt {
        read_struct_from_file("zkvm_receipt.bin")
    }

    /// Read the test image ID from disk
    fn read_test_image_id() -> [u32; 8] {
        read_struct_from_file("zkvm_image_id.bin")
    }

    // Read zkvm_wrong_image_id.bin from disk
    fn read_erronous_test_image_id() -> [u32; 8] {
        read_struct_from_file("zkvm_wrong_image_id.bin")
    }

    /// Gets the test verification data for use in tests
    fn get_test_verification_data() -> VerificationData {
        VerificationData {
            image_id: read_test_image_id(),
            receipt: read_test_receipt(),
        }
    }

    // Gets the erronous test verification data for use in tests
    fn get_erronous_test_verification_data() -> VerificationData {
        VerificationData {
            image_id: read_erronous_test_image_id(),
            receipt: read_test_receipt(),
        }
    }

    fn read_json_data() -> String {
    include_str!("../json/res/example.json").to_string()
}

fn generate_receipt(json_data: &str) -> Receipt {
    let env = ExecutorEnv::builder()
        .add_input(&to_vec(&json_data).unwrap())
        .build()
        .unwrap();
    let prover = default_prover();
    prover.prove_elf(env, SEARCH_JSON_ELF).unwrap()
}

fn write_struct_to_file<T: serde::Serialize>(path: &Path, data: &T) {
    create_dir_all(&path.parent().unwrap()).expect("Failed to create directory");
    let serialized_data = bincode::serialize(data).expect("Failed to serialize data");
    let mut file = File::create(path).expect("Failed to create file");
    file.write_all(&serialized_data).expect("Failed to write data");
}

fn create_erroneous_commitment() -> [u32; 8] {
    let mut wrong_commitment = SEARCH_JSON_ID.clone();
    wrong_commitment[0] = 0x00;
    wrong_commitment
}

#[ignore = "this test is only used to reproducibly generate test data"]
#[test]
fn generate_and_save_receipt_for_test() {
    let json_data = read_json_data();

    // Generate receipt
    let receipt = generate_receipt(&json_data);

    // Write receipt to file
    let receipt_path = Path::new("test_data").join("zkvm_receipt.bin");
    write_struct_to_file(&receipt_path, &receipt);

    // Write commitment to file
    let commitment_path = Path::new("test_data").join("zkvm_image_id.bin");
    write_struct_to_file(&commitment_path, &SEARCH_JSON_ID);

    // Write an erroneous commitment to file
    let wrong_commitment = create_erroneous_commitment();
    let wrong_commitment_path = Path::new("test_data").join("zkvm_wrong_image_id.bin");
    write_struct_to_file(&wrong_commitment_path, &wrong_commitment);
}

    #[test]
    fn test_should_pass_valid_receipt_and_image_pair() {
        let test_verification_data = bincode::serialize(&get_test_verification_data()).unwrap();
        let signature_request = InitialState {
            data: test_verification_data
        };

        assert!(ZkVmVerificationProgram::evaluate(signature_request).is_ok());
    }

    #[test]
    fn test_should_error_with_incorrect_image_id_for_receipt_image_pair() {
        let erronous_test_verification_data = bincode::serialize(&get_erronous_test_verification_data()).unwrap();
        let signature_request = InitialState {
            data: erronous_test_verification_data
        };

        assert!(ZkVmVerificationProgram::evaluate(signature_request).is_err());
    }
}