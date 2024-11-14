//! This example shows how to write a contrieved and basic program: checking the length of the data to be signed.

#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{string::ToString, vec::Vec};

use entropy_programs_core::{bindgen::Error, bindgen::*, export_program, prelude::*};

use bincode;
use risc0_zkvm::Receipt;

// TODO confirm this isn't an issue for audit
register_custom_getrandom!(always_fail);

pub struct ZkVmVerificationProgram;

impl Program for ZkVmVerificationProgram {
    fn evaluate(signature_request: SignatureRequest, _config: Option<Vec<u8>>, _oracle_data: Option<Vec<Vec<u8>>>) -> Result<(), Error> {
        let image_id: [u32; 8] = bincode::deserialize(&signature_request.message)
            .map_err(|_| Error::InvalidSignatureRequest("Could not parse image_id".to_string()))?;

        let receipt: Receipt = match signature_request.auxilary_data {
            Some(serialized_receipt) => {
                bincode::deserialize(&serialized_receipt).map_err(|_| {
                    Error::InvalidSignatureRequest("Could not parse receipt".to_string())
                })?
            }
            None => {
                return Err(Error::InvalidSignatureRequest(
                    "No receipt provided".to_string(),
                ))
            }
        };

        receipt
            .verify(image_id)
            .map_err(|_| Error::Evaluation("Proof verification failed".to_string()))?;

        Ok(())
    }

    /// Since we don't use a custom hash function, we can just return `None` here.
    fn custom_hash(_data: Vec<u8>) -> Option<Vec<u8>> {
        None
    }
}

export_program!(ZkVmVerificationProgram);

// write a test that calls evaluate and passes it the proper parameters
#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::{create_dir_all, File};
    use std::io::{Read, Write};
    use std::path::Path;

    use json_methods::{SEARCH_JSON_ELF, SEARCH_JSON_ID};
    use risc0_zkvm::{default_prover, serde::to_vec, ExecutorEnv};

    use helpers::*;

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
        let signature_request = SignatureRequest {
            message: bincode::serialize(&read_test_image_id()).unwrap(),
            auxilary_data: Some(bincode::serialize(&read_test_receipt()).unwrap()),
        };

        assert!(ZkVmVerificationProgram::evaluate(signature_request, None, None).is_ok());
    }

    #[test]
    fn test_should_error_with_incorrect_image_id_for_receipt_image_pair() {
        let signature_request = SignatureRequest {
            message: bincode::serialize(&read_erronous_test_image_id()).unwrap(),
            auxilary_data: Some(bincode::serialize(&read_test_receipt()).unwrap()),
        };

        assert!(ZkVmVerificationProgram::evaluate(signature_request, None, None).is_err());
    }

    // Test helper functions
    mod helpers {
        use super::*;

        /// Read a file from disk and deserialize it using bincode
        pub fn read_struct_from_file<T>(filename: &str) -> T
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
        pub fn read_test_receipt() -> Receipt {
            read_struct_from_file("zkvm_receipt.bin")
        }

        /// Read the test image ID from disk
        pub fn read_test_image_id() -> [u32; 8] {
            read_struct_from_file("zkvm_image_id.bin")
        }

        // Read zkvm_wrong_image_id.bin from disk
        pub fn read_erronous_test_image_id() -> [u32; 8] {
            read_struct_from_file("zkvm_wrong_image_id.bin")
        }

        pub fn read_json_data() -> String {
            include_str!("../json/res/example.json").to_string()
        }

        pub fn generate_receipt(json_data: &str) -> Receipt {
            let env = ExecutorEnv::builder()
                .add_input(&to_vec(&json_data).unwrap())
                .build()
                .unwrap();
            let prover = default_prover();
            prover.prove_elf(env, SEARCH_JSON_ELF).unwrap()
        }

        pub fn write_struct_to_file<T: serde::Serialize>(path: &Path, data: &T) {
            create_dir_all(&path.parent().unwrap()).expect("Failed to create directory");
            let serialized_data = bincode::serialize(data).expect("Failed to serialize data");
            let mut file = File::create(path).expect("Failed to create file");
            file.write_all(&serialized_data)
                .expect("Failed to write data");
        }

        pub fn create_erroneous_commitment() -> [u32; 8] {
            let mut wrong_commitment = SEARCH_JSON_ID.clone();
            wrong_commitment[0] = 0x00;
            wrong_commitment
        }
    }
}
