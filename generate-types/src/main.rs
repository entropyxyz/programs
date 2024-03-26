use schemars::schema_for;
use std::fs;

fn main() {
    let schema_config_device_key_proxy = schema_for!(device_key_proxy::UserConfig);
    fs::write(
        "./device_key_proxy_config_type.txt",
        format!(
            "{:?}",
            serde_json::to_vec(&schema_config_device_key_proxy)
                .expect("error converting user config for device key proxy")
        ),
    )
    .expect("Failed to write to device key proxy config");

    let schema_aux_data_device_key_proxy = schema_for!(device_key_proxy::AuxData);
    fs::write(
        "./device_key_proxy_aux_data_type.txt",
        format!(
            "{:?}",
            serde_json::to_vec(&schema_aux_data_device_key_proxy)
                .expect("error converting user aux_data for device key proxy")
        ),
    )
    .expect("Failed to write to device key proxy aux_data");
}
