use clap::Parser;
use ream_lib::{file::ssz_from_file, input::OperationInput, ssz::from_ssz_bytes};
use std::path::PathBuf;
use tracing::info;
use tree_hash::{Hash256, TreeHash};
mod cli;
use cli::{
    fork::Fork,
    operation::{Operation, OperationHandler},
};
use ream_consensus::electra::beacon_state::BeaconState;
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    fork: cli::fork::ForkArgs,
    #[clap(flatten)]
    operation: cli::operation::OperationArgs,
    #[clap(long, default_value_t = true)]
    compare_specs: bool,
    #[clap(long, default_value_t = false)]
    compare_recompute: bool,
    #[clap(long)]
    excluded_cases: Vec<String>,
}
#[derive(Serialize, Deserialize)]
pub struct ZiskInput {
    pre_state_ssz_bytes: Vec<u8>,
    operation_input: Vec<u8>, // or your specific operation type
}

fn main() {
    setup_log();
    let (fork, operation, excluded_cases, compare_specs, compare_recompute) = parse_args();
    let build_guest_result = Command::new("cargo-zisk")
        .args(["build", "--release"])
        .current_dir("guest")
        .status()
        .expect("Failed to build guest code");
    if !build_guest_result.success() {
        eprintln!("Guest code build failed!");
        std::process::exit(1);
    }
    match operation {
        Operation::Block {
            operation: block_op,
        } => {
            run_tests(
                &fork,
                &block_op,
                excluded_cases,
                compare_specs,
                compare_recompute,
            );
        }
        Operation::Epoch {
            operation: epoch_op,
        } => {
            run_tests(
                &fork,
                &epoch_op,
                excluded_cases,
                compare_specs,
                compare_recompute,
            );
        }
    }
}

fn run_tests<T: OperationHandler>(
    fork: &Fork,
    operation: &T,
    excluded_cases: Vec<String>,
    compare_specs: bool,
    compare_recompute: bool,
) {
    let (base_dir, test_cases) = operation.load_test_cases(fork);
    for test_case in test_cases {
        if excluded_cases.contains(&test_case) {
            info!("Skipping test case: {test_case}");
            continue;
        }
        info!("[{operation}] Test case: {test_case}");
        let case_dir = &base_dir.join(&test_case);
        let input = operation.prepare_input(&case_dir);
        let pre_state_ssz_bytes: Vec<u8> = ssz_from_file(&case_dir.join("pre.ssz_snappy"));
        // Write input to file for ZISK guest
        let build_dir = PathBuf::from("guest/build");
        if !build_dir.exists() {
            info!("Creating build directory at {:?}", build_dir);
            fs::create_dir_all(&build_dir).expect("Failed to create build directory");
        }
        let input_path = build_dir.join("input.bin");

        let _written = write_zisk_input(
            pre_state_ssz_bytes.clone(),
            bincode::serialize(&input).unwrap(),
            input_path.to_str().unwrap(),
        );
        info!("Input written to {:?}", input_path);
        info!("----- Cycle Tracker Start -----");

        let output = Command::new("ziskemu")
            .args([
                "-i",
                input_path.to_str().unwrap(),
                "-e",
                "target/riscv64ima-zisk-zkvm-elf/release/consenzisk_guest",
            ])
            .output()
            .expect("Failed to run ZISK VM");
        if !output.status.success() {
            eprintln!(
                "ZISK execution failed for test case {}!\n{}",
                test_case,
                String::from_utf8_lossy(&output.stderr)
            );
            std::process::exit(1);
        }

        // Parse output from ZISK guest
        let zisk_output = String::from_utf8_lossy(&output.stdout);
        info!("ZISK output for {}: {}", test_case, zisk_output);

        let new_state_root = parse_state_root_from_hex(&zisk_output);
        if compare_specs {
            assert_state_root_matches_specs(&new_state_root, &pre_state_ssz_bytes, &case_dir);
        }
        if compare_recompute {
            assert_state_root_matches_recompute(&new_state_root, &pre_state_ssz_bytes, &input);
        }

        info!("----- Cycle Tracker End -----");
    }
}

fn setup_log() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    // Initialize tracing. In order to view logs, run `RUST_LOG=info cargo run`
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::filter::EnvFilter::from_default_env())
        .init();
}
fn write_zisk_input(
    pre_state_ssz_bytes: Vec<u8>,
    operation_input: Vec<u8>,
    input_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let zisk_input = ZiskInput {
        pre_state_ssz_bytes,
        operation_input,
    };

    let serialized_input = bincode::serialize(&zisk_input)?;
    std::fs::write(input_path, serialized_input)?;
    Ok(())
}
fn parse_args() -> (Fork, Operation, Vec<String>, bool, bool) {
    let args = Args::parse();

    (
        args.fork.fork,
        args.operation.operation,
        args.excluded_cases,
        args.compare_specs,
        args.compare_recompute,
    )
}

fn assert_state_root_matches_specs(
    new_state_root: &Hash256,
    pre_state_ssz_bytes: &[u8],
    case_dir: &PathBuf,
) {
    let post_state_opt: Option<BeaconState> = {
        if case_dir.join("post.ssz_snappy").exists() {
            let ssz_bytes: Vec<u8> = ssz_from_file(&case_dir.join("post.ssz_snappy"));
            Some(from_ssz_bytes(&ssz_bytes).unwrap())
        } else {
            None
        }
    };

    match post_state_opt {
        // If the specs provide post_state, compare the computed root against post_state's root
        Some(post_state) => {
            info!("post_state provided. The state root should be mutated.");
            assert_eq!(*new_state_root, post_state.tree_hash_root());
            info!("Execution is correct! State mutated and the roots match.");
        }
        // If the specs does not contain a post_state, compare the computed root against pre_state's root
        None => {
            info!("post_state not provided. The state root should not be mutated.");
            let pre_state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();
            assert_eq!(*new_state_root, pre_state.tree_hash_root());
            info!("Execution is correct! State should not be mutated and the roots match.");
        }
    }
}

fn parse_state_root_from_hex(hex_output: &str) -> Hash256 {
    // The guest program outputs the state root as hex string
    // Find the hex string in the output (it should be a 64-character hex string)
    let lines: Vec<&str> = hex_output.lines().collect();

    for line in lines {
        let trimmed = line.trim();
        // Look for a 64-character hex string (32 bytes * 2 chars per byte)
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            // Parse hex string to bytes
            let mut bytes = [0u8; 32];
            for (i, chunk) in trimmed.as_bytes().chunks(2).enumerate() {
                let hex_str = std::str::from_utf8(chunk).unwrap();
                bytes[i] = u8::from_str_radix(hex_str, 16).unwrap();
            }
            return Hash256::from(bytes);
        }
    }

    panic!(
        "Could not find valid hex state root in output: {}",
        hex_output
    );
}

fn assert_state_root_matches_recompute(
    new_state_root: &Hash256,
    pre_state_ssz_bytes: &[u8],
    input: &OperationInput,
) {
    let mut state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();

    input.process(&mut state);

    let recomputed_state_root = state.tree_hash_root();

    println!("recomputed_state_root: {}", recomputed_state_root);
    println!("new_state_root: {}", new_state_root);

    assert_eq!(*new_state_root, recomputed_state_root);
    info!("Execution is correct! State roots match host's recomputed state root.");
}
