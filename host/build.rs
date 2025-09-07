use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

// Define constants for the directory and input file name
const OUTPUT_DIR: &str = "build/";
const FILE_NAME: &str = "input.bin";

fn main() -> io::Result<()> {
    let n: u64 = 20; // Default value for iterations

    // Print cargo:rerun-if-changed to make cargo rebuild when input.rs changes
    println!("cargo:rerun-if-changed=../lib/src/input.rs");

    // We need the file in a predictable location for cargo-zisk
    let output_dir = Path::new(OUTPUT_DIR);
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }

    // Create the file and write the 'n' value in little-endian format
    let file_path = output_dir.join(FILE_NAME);
    let mut file = File::create(&file_path)?;
    file.write_all(&n.to_le_bytes())?;

    println!("cargo:warning=Generated input.bin with {} iterations", n);

    Ok(())
}
