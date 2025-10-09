#![no_main]
ziskos::entrypoint!(main);
use bincode;
use ream_consensus::electra::beacon_state::BeaconState;
use ream_lib::{input::OperationInput, ssz::from_ssz_bytes};
use tree_hash::TreeHash;
use ziskos::read_input;
use serde::{Serialize, Deserialize};
// use consenzisk_host::bin::ZiskInput;  // Import ZiskInput struct
#[derive(Serialize, Deserialize)]
pub struct ZiskInput {
    pre_state_ssz_bytes: Vec<u8>,
    operation_input: Vec<u8>, // or your specific operation type
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
    let mut state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();
    eprintln!("{}:{}", "deserialize-pre-state-ssz", "end");

    eprintln!("{}:{}", "deserialize-operation-input", "start");
    let operation_input_bytes = zisk_input.operation_input;
    let operation_input: OperationInput = bincode::deserialize(&operation_input_bytes).unwrap();
    eprintln!("{}:{}", "deserialize-operation-input", "end");

    // Main logic of the program.
    // State transition of the beacon state.
    eprintln!("{}:{}", "process-operation", "start");
    operation_input.process(&mut state);
    eprintln!("{}:{}", "process-operation", "end");

    // Merkleize the processed state
    eprintln!("{}:{}", "merkleize-operation", "start");
    let state_root = state.tree_hash_root();
    eprintln!("{}:{}", "merkleize-operation", "end");

    eprintln!("{}:{}", "output-state-root", "start");
    // Output state root as hex (for host to parse)
    for byte in state_root.as_ref() as &[u8] {
        print!("{:02x}", byte);
    }
    println!();
    eprintln!("{}:{}", "output-state-root", "end");
}
