use schemars::schema_for;
use std::fs;
use program::{UserConfig, AuxData, ORACLE_DATA};
use codec::Encode; 

pub fn generate_types() {
    let schema_config = schema_for!(UserConfig);
    fs::write(
        "./tests_serialized_config_type.txt",
        format!(
            "{:?}",
            serde_json::to_vec(&schema_config)
                .expect("error converting user config")
        ),
    )
    .expect("Failed to write to config");

    let schema_aux_data = schema_for!(AuxData);
    fs::write(
        "./tests_serialized_aux_data_type.txt",
        format!(
            "{:?}",
            serde_json::to_vec(&schema_aux_data)
                .expect("error converting user aux_data")
        ),
    )
    .expect("Failed to write to proxy aux_data");

    let oracle_data = ORACLE_DATA.iter().map(|x| x.encode()).collect::<Vec<_>>();
    fs::write(
        "./tests_serialized_oracle_data_type.txt",
        serde_json::to_vec(&oracle_data).expect("error converting user oracle_data")
    )
    .expect("Failed to write oracle_data");
}