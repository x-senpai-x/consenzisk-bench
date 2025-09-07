// ZISK guest for consensus logic
use ream_lib::input::OperationInput;
use ream_consensus::electra::beacon_state::BeaconState;
use ream_lib::ssz::from_ssz_bytes;
use tree_hash::TreeHash;
use ziskos::read_input;
use std::io::{Cursor, Read};
use bincode;

fn main() {
    // Read the input data as a byte array from ziskos
    let input: Vec<u8> = read_input();
    let mut cursor = Cursor::new(&input);
    // Read pre_state_ssz_bytes length (u64 LE)
    let mut len_buf = [0u8; 8];
    cursor.read_exact(&mut len_buf).unwrap();
    let pre_state_len = u64::from_le_bytes(len_buf) as usize;
    // Read pre_state_ssz_bytes
    let mut pre_state_ssz_bytes = vec![0u8; pre_state_len];
    cursor.read_exact(&mut pre_state_ssz_bytes).unwrap();
    // Read OperationInput (bincode)
    let mut op_bytes = Vec::new();
    cursor.read_to_end(&mut op_bytes).unwrap();
    let operation_input: OperationInput = bincode::deserialize(&op_bytes).unwrap();
    // Deserialize state
    let mut state: BeaconState = from_ssz_bytes(&pre_state_ssz_bytes).unwrap();
    // Process operation
    operation_input.process(&mut state);
    // Compute state root
    let state_root = state.tree_hash_root();
    // Output state root as hex (for host to parse)
    for byte in state_root.bytes() {
        print!("{:02x}", byte);
    }
    println!();
}
