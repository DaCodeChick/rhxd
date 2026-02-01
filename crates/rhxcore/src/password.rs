//! Password handling utilities (legacy XOR obfuscation)

/// Unscramble password using legacy XOR obfuscation (bitwise NOT)
///
/// The legacy Hotline protocol uses a simple XOR obfuscation where each byte
/// is inverted using the bitwise NOT operator (~).
pub fn unscramble_password(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| !b).collect()
}

/// Scramble password using legacy XOR obfuscation (bitwise NOT)
///
/// This is symmetric with unscramble_password.
pub fn scramble_password(data: &[u8]) -> Vec<u8> {
    data.iter().map(|&b| !b).collect()
}

/// Verify password against stored scrambled version
pub fn verify_password(stored_scrambled: &[u8], provided: &[u8]) -> bool {
    let provided_scrambled = scramble_password(provided);
    stored_scrambled == provided_scrambled.as_slice()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scramble_unscramble() {
        let password = b"test123";
        let scrambled = scramble_password(password);
        let unscrambled = unscramble_password(&scrambled);

        assert_eq!(password, unscrambled.as_slice());
    }

    #[test]
    fn test_verify_password() {
        let password = b"mypassword";
        let scrambled = scramble_password(password);

        assert!(verify_password(&scrambled, password));
        assert!(!verify_password(&scrambled, b"wrongpassword"));
    }

    #[test]
    fn test_scramble_inverse() {
        let data = b"Hello, World!";
        let scrambled = scramble_password(data);

        // Scrambling twice should give original
        let double_scrambled = scramble_password(&scrambled);
        assert_eq!(data, double_scrambled.as_slice());
    }
}
