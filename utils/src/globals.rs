// Macro for const-time integer parsing from string literals
// This is the most "generic" solution possible in const context
macro_rules! const_parse_unsigned {
    ($s:expr, $type:ty) => {{
        let bytes = $s.as_bytes();
        let mut result: $type = 0;
        let mut i = 0;
        while i < bytes.len() {
            let digit = (bytes[i] - b'0') as $type;
            result = result * 10 + digit;
            i += 1;
        }
        result
    }};
}

// Now we can use the macro directly for both constants
pub const FILE_SIZE_LIMIT: usize = const_parse_unsigned!(env!("FILE_SIZE_LIMIT"), usize);

// Not dynamic as the hashing algorithm is fixed to SHA256
pub const HASH_SIZE: u8 = 64;
