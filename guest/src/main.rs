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
    let input_bytes=read_input();
    let zisk_input: ZiskInput = bincode::deserialize(&input_bytes).expect("Failed to deserialize input");  

    // eprintln!("{}:{}", "read-pre-state-len", "start");
    // let pre_state_len: Vec<u8> = read_input();
    // eprintln!("Received pre_state_len bytes: {:?}", pre_state_len);

    // The host sends a u64, so read it as u64 first
    // if pre_state_len.len() != 8 {
    //     panic!("Expected 8 bytes for length, got {}", pre_state_len.len());
    // }
    // let len_bytes: [u8; 8] = pre_state_len.try_into().unwrap();
    // let n: usize = u64::from_le_bytes(len_bytes) as usize;
    // eprintln!("{}:{} - length: {}", "read-pre-state-len", "end", n);

    // eprintln!("{}:{}", "read-pre-state-ssz", "start");
    // let mut pre_state_ssz_bytes = vec![0u8; n];
    let pre_state_ssz_bytes = zisk_input.pre_state_ssz_bytes;
    eprintln!(
        "{}:{} - bytes read: {}",
        "read-pre-state-ssz",
        "end",
        pre_state_ssz_bytes.len()
    );

    eprintln!("{}:{}", "deserialize-state", "start");
    let mut state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();
    eprintln!("{}:{}", "deserialize-state", "end");

    eprintln!("{}:{}", "read-operation-input", "start");
    let operation_input_bytes = zisk_input.operation_input;  
    let operation_input: OperationInput = bincode::deserialize(&operation_input_bytes).unwrap();
    eprintln!(
        "{}:{} - input bytes: {}",
        "read-operation-input",
        "end",
        operation_input_bytes.len()
    );

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
