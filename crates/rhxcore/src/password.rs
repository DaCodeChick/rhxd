//! Password handling utilities (legacy XOR obfuscation)

/// Transform password using legacy XOR obfuscation (bitwise NOT)
///
/// The legacy Hotline protocol uses a simple XOR obfuscation where each byte
/// is inverted using the bitwise NOT operator (~). This operation is symmetric:
/// applying it twice returns the original data.
///
/// This can be used for both scrambling and unscrambling since they are identical operations.
#[inline]
pub fn xor_password(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| !b).collect()
}

/// Alias for xor_password for compatibility
#[deprecated(
    since = "0.1.0",
    note = "Use xor_password instead - scramble and unscramble are the same operation"
)]
#[inline]
pub fn scramble_password(data: &[u8]) -> Vec<u8> {
    xor_password(data)
}

/// Alias for xor_password for compatibility
#[deprecated(
    since = "0.1.0",
    note = "Use xor_password instead - scramble and unscramble are the same operation"
)]
#[inline]
pub fn unscramble_password(data: &[u8]) -> Vec<u8> {
    xor_password(data)
}

/// Verify password against stored scrambled version
#[inline]
pub fn verify_password(stored_scrambled: &[u8], provided: &[u8]) -> bool {
    let provided_scrambled = xor_password(provided);
    stored_scrambled == provided_scrambled.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xor_password_symmetric() {
        let password = b"test123";
        let xored = xor_password(password);
        let double_xored = xor_password(&xored);

        // XOR twice should return original
        assert_eq!(password, double_xored.as_slice());
        // Single XOR should be different
        assert_ne!(password, xored.as_slice());
    }

    #[test]
    fn test_verify_password() {
        let password = b"mypassword";
        let scrambled = xor_password(password);

        assert!(verify_password(&scrambled, password));
        assert!(!verify_password(&scrambled, b"wrongpassword"));
    }
}
