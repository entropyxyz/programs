use schemars::{schema_for, JsonSchema};
use device_key_proxy::UserConfig;

fn main() {
    let schema = schema_for!(UserConfig);

    println!("{:#?}", serde_json::to_string_pretty(&schema).unwrap());

}
