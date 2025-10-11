#![no_main]
ziskos::entrypoint!(main);
use bincode;
use ream_consensus::electra::beacon_state::BeaconState;
use ream_lib::{input::OperationInput, ssz::from_ssz_bytes};
use tree_hash::TreeHash;
use ziskos::read_input;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ZiskInput {
    pre_state_ssz_bytes: Vec<u8>,
    operation_input: Vec<u8>, // or your specific operation type
}

/// Timing markers for per-operation cycle counting
/// Clean implementation without hex output

/// Start timing for an operation
fn start_timing(operation: &str) {
    eprintln!("TIMING_START:{}", operation);
}

/// End timing for an operation
fn end_timing(operation: &str) {
    eprintln!("TIMING_END:{}", operation);
}

fn main() {
    // Read inputs to the program.
    eprintln!("{}:{}", "read-inputs", "start");
    let input_bytes=read_input();
    eprintln!("{}:{}", "read-inputs", "end");

    eprintln!("{}:{}", "deserialize-inputs", "start");
    let zisk_input: ZiskInput = bincode::deserialize(&input_bytes).expect("Failed to deserialize input");
    eprintln!("{}:{}", "deserialize-inputs", "end");

    let pre_state_ssz_bytes = zisk_input.pre_state_ssz_bytes;
    eprintln!("{}:{}", "deserialize-pre-state-ssz", "start");
    start_timing("deserialize-pre-state-ssz");
    let mut state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();
    end_timing("deserialize-pre-state-ssz");
    eprintln!("{}:{}", "deserialize-pre-state-ssz", "end");

    eprintln!("{}:{}", "deserialize-operation-input", "start");
    start_timing("deserialize-operation-input");
    let operation_input_bytes = zisk_input.operation_input;
    let operation_input: OperationInput = bincode::deserialize(&operation_input_bytes).unwrap();
    end_timing("deserialize-operation-input");
    eprintln!("{}:{}", "deserialize-operation-input", "end");

    // Main logic of the program.
    // State transition of the beacon state.
    eprintln!("{}:{}", "process-operation", "start");
    start_timing("process-operation");
    operation_input.process(&mut state);
    end_timing("process-operation");
    eprintln!("{}:{}", "process-operation", "end");

    // Merkleize the processed state
    eprintln!("{}:{}", "merkleize-operation", "start");
    start_timing("merkleize-operation");
    let state_root = state.tree_hash_root();
    end_timing("merkleize-operation");
    eprintln!("{}:{}", "merkleize-operation", "end");

    eprintln!("{}:{}", "output-state-root", "start");
    start_timing("output-state-root");
    // Output state root as hex (for host to parse)
    for byte in state_root.as_ref() as &[u8] {
        print!("{:02x}", byte);
    }
    println!();
    end_timing("output-state-root");
    eprintln!("{}:{}", "output-state-root", "end");
}