use crate::runtime::is_unconstrained;

// Implementation of SHA-256 mapping a byte array of variable length to
// 32 bytes.

// Deprecated in favour of `sha256_var`
#[foreign(sha256)]
// docs:start:sha256
pub fn sha256<let N: u32>(input: [u8; N]) -> [u8; 32]
// docs:end:sha256
{}

#[foreign(sha256_compression)]
pub fn sha256_compression(_input: [u32; 16], _state: [u32; 8]) -> [u32; 8] {}

// SHA-256 hash function
#[no_predicates]
pub fn digest<let N: u32>(msg: [u8; N]) -> [u8; 32] {
    sha256_var(msg, N as u64)
}

// Convert 64-byte array to array of 16 u32s
fn msg_u8_to_u32(msg: [u8; 64]) -> [u32; 16] {
    let mut msg32: [u32; 16] = [0; 16];

    for i in 0..16 {
        let mut msg_field: Field = 0;
        for j in 0..4 {
            msg_field = msg_field * 256 + msg[64 - 4*(i + 1) + j] as Field;
        }
        msg32[15 - i] = msg_field as u32;
    }

    msg32
}

unconstrained fn build_msg_block_iter<let N: u32>(msg: [u8; N], message_size: u64, msg_start: u32) -> ([u8; 64], u64) {
    let mut msg_block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    let mut msg_byte_ptr: u64 = 0; // Message byte pointer
    let mut msg_end = msg_start + BLOCK_SIZE;
    if msg_end > N {
        msg_end = N;
    }
    for k in msg_start..msg_end {
        if k as u64 < message_size {
            msg_block[msg_byte_ptr] = msg[k];
            msg_byte_ptr = msg_byte_ptr + 1;
        }
    }
    (msg_block, msg_byte_ptr)
}

// Verify the block we are compressing was appropriately constructed
fn verify_msg_block<let N: u32>(
    msg: [u8; N],
    message_size: u64,
    msg_block: [u8; 64],
    msg_start: u32
) -> u64 {
    let mut msg_byte_ptr: u64 = 0; // Message byte pointer
    let mut msg_end = msg_start + BLOCK_SIZE;
    let mut extra_bytes = 0;
    if msg_end > N {
        msg_end = N;
        extra_bytes = msg_end - N;
    }

    for k in msg_start..msg_end {
        if k as u64 < message_size {
            msg_byte_ptr = msg_byte_ptr + 1;
        }
    }

    for i in 0..BLOCK_SIZE {
        if i as u64 >= msg_byte_ptr {
            assert_eq(msg_block[i], 0);
        } else {
            assert_eq(msg_block[i], msg[msg_start + i - extra_bytes]);
        }
    }

    msg_byte_ptr
}

global BLOCK_SIZE = 64;
global ZERO = 0;

// Variable size SHA-256 hash
pub fn sha256_var<let N: u32>(msg: [u8; N], message_size: u64) -> [u8; 32] {
    let num_blocks = N / BLOCK_SIZE;
    let mut msg_block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
    let mut h: [u32; 8] = [1779033703, 3144134277, 1013904242, 2773480762, 1359893119, 2600822924, 528734635, 1541459225]; // Intermediate hash, starting with the canonical initial value
    let mut msg_byte_ptr = 0; // Pointer into msg_block

    for i in 0..num_blocks {
        let (new_msg_block, new_msg_byte_ptr) = unsafe {
            build_msg_block_iter(msg, message_size, BLOCK_SIZE * i)
        };
        msg_block = new_msg_block;

        if !is_unconstrained() {
            // Verify the block we are compressing was appropriately constructed
            msg_byte_ptr = verify_msg_block(msg, message_size, msg_block, BLOCK_SIZE * i);
        } else {
            msg_byte_ptr = new_msg_byte_ptr;
        }

        // Compress the block
        h = sha256_compression(msg_u8_to_u32(msg_block), h);
    }

    let modulo = N % BLOCK_SIZE;
    // Handle setup of the final msg block.
    // This case is only hit if the msg is less than the block size, 
    // or our message cannot be evenly split into blocks. 
    if modulo != 0 {
        let (new_msg_block, new_msg_byte_ptr) = unsafe {
            build_msg_block_iter(msg, message_size, BLOCK_SIZE * num_blocks)
        };
        msg_block = new_msg_block;

        if !is_unconstrained() {
            msg_byte_ptr = verify_msg_block(msg, message_size, msg_block, BLOCK_SIZE * num_blocks);
        } else {
            msg_byte_ptr = new_msg_byte_ptr;
        }
    }

    if msg_byte_ptr == BLOCK_SIZE as u64 {
        msg_byte_ptr = 0;
    }

    // This variable is used to get around the compiler under-constrained check giving a warning.
    // We want to check against a constant zero, but if it does not come from the circuit inputs
    // or return values the compiler check will issue a warning.
    let zero = msg_block[0] - msg_block[0];

    // Pad the rest such that we have a [u32; 2] block at the end representing the length
    // of the message, and a block of 1 0 ... 0 following the message (i.e. [1 << 7, 0, ..., 0]).    
    msg_block[msg_byte_ptr] = 1 << 7;
    let last_block = msg_block;
    msg_byte_ptr = msg_byte_ptr + 1;

    unsafe {
        let (new_msg_block, new_msg_byte_ptr) = pad_msg_block(msg_block, msg_byte_ptr);
        msg_block = new_msg_block;
        if crate::runtime::is_unconstrained() {
            msg_byte_ptr = new_msg_byte_ptr;
        }
    }

    if !crate::runtime::is_unconstrained() {
        for i in 0..64 {
            assert_eq(msg_block[i], last_block[i]);
        }

        // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
        // the 1 and 0s fill up the current block, which we then compress accordingly.
        // Not enough bits (64) to store length. Fill up with zeros.
        for _i in 57..64 {
            if msg_byte_ptr <= 63 & msg_byte_ptr >= 57 {
                assert_eq(msg_block[msg_byte_ptr], zero);
                msg_byte_ptr += 1;
            }
        }
    }

    if msg_byte_ptr >= 57 {
        h = sha256_compression(msg_u8_to_u32(msg_block), h);

        msg_byte_ptr = 0;
    }

    msg_block = unsafe {
        attach_len_to_msg_block(msg_block, msg_byte_ptr, message_size)
    };

    if !crate::runtime::is_unconstrained() {
        for i in 0..56 {
            if i < msg_byte_ptr {
                assert_eq(msg_block[i], last_block[i]);
            } else {
                assert_eq(msg_block[i], zero);
            }
        }

        let len = 8 * message_size;
        let len_bytes = (len as Field).to_be_bytes(8);
        for i in 56..64 {
            assert_eq(msg_block[i], len_bytes[i - 56]);
        }
    }

    hash_final_block(msg_block, h)
}

unconstrained fn pad_msg_block(mut msg_block: [u8; 64], mut msg_byte_ptr: u64) -> ([u8; 64], u64) {
    // If i >= 57, there aren't enough bits in the current message block to accomplish this, so
    // the 1 and 0s fill up the current block, which we then compress accordingly.
    if msg_byte_ptr >= 57 {
        // Not enough bits (64) to store length. Fill up with zeros.
        if msg_byte_ptr < 64 {
            for _ in 57..64 {
                if msg_byte_ptr <= 63 {
                    msg_block[msg_byte_ptr] = 0;
                    msg_byte_ptr += 1;
                }
            }
        }
    }
    (msg_block, msg_byte_ptr)
}

unconstrained fn attach_len_to_msg_block(mut msg_block: [u8; 64], mut msg_byte_ptr: u64, message_size: u64) -> [u8; 64] {
    let len = 8 * message_size;
    let len_bytes = (len as Field).to_be_bytes(8);
    for _i in 0..64 {
        // In any case, fill blocks up with zeros until the last 64 (i.e. until msg_byte_ptr = 56).
        if msg_byte_ptr < 56 {
            msg_block[msg_byte_ptr] = 0;
            msg_byte_ptr = msg_byte_ptr + 1;
        } else if msg_byte_ptr < 64 {
            for j in 0..8 {
                msg_block[msg_byte_ptr + j] = len_bytes[j];
            }
            msg_byte_ptr += 8;
        }
    }
    msg_block
}

fn hash_final_block(msg_block: [u8; 64], mut state: [u32; 8]) -> [u8; 32] {
    let mut out_h: [u8; 32] = [0; 32]; // Digest as sequence of bytes

    // Hash final padded block
    state = sha256_compression(msg_u8_to_u32(msg_block), state);

    // Return final hash as byte array
    for j in 0..8 {
        let h_bytes = (state[7 - j] as Field).to_le_bytes(4);
        for k in 0..4 {
            out_h[31 - 4*j - k] = h_bytes[k];
        }
    }

    out_h
}

