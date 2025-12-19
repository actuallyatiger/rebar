// Function for const-time integer parsing from string literals
// This is the most "generic" solution possible in const context
const fn parse_unsigned(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut result: usize = 0;
    let mut i = 0;
    while i < bytes.len() {
        let digit = (bytes[i] - b'0') as usize;
        result = result * 10 + digit;
        i += 1;
    }
    result
}

// Now we can use the macro directly for both constants
pub const FILE_SIZE_LIMIT: usize = parse_unsigned(env!("FILE_SIZE_LIMIT"));

// Not dynamic as the hashing algorithm is fixed to SHA256
pub const HASH_SIZE: u8 = 64;

pub const COMPRESSION_LEVEL: u8 = parse_unsigned(env!("COMPRESSION_LEVEL")) as u8;
