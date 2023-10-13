//! This reads a text file of hex-encoded addresses, one per line
//! and puts them in the constant ADDRESSES
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("addresses.rs");
    let out_file = File::create(dest_path).unwrap();
    let mut writer = BufWriter::new(out_file);

    // First count the number of non-empty lines in the file
    let length = {
        let file = File::open(format!("addresses.txt")).unwrap();
        let reader = BufReader::new(file);
        let mut number_lines = 0;
        for line in reader.lines() {
            if !line.unwrap().is_empty() {
                number_lines += 1;
            }
        }
        number_lines
    };

    let file = File::open(format!("addresses.txt")).unwrap();
    let reader = BufReader::new(file);
    writer
        .write(format!("const ADDRESSES: [[u8; 20]; {}] = [", length).as_bytes())
        .unwrap();

    for line in reader.lines() {
        let line = line.unwrap();
        if line.is_empty() {
            continue;
        }
        let address: [u8; 20] = hex::decode(line).unwrap().try_into().unwrap();
        writer
            .write(format!("  {:?},", address).as_bytes())
            .unwrap();
    }
    writer.write("];".as_bytes()).unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=addresses.txt");
}
