use rand::Rng;
use sha2::{Digest, Sha256};

/// Device codes expire after 10 minutes.
pub const DEVICE_CODE_EXPIRY_SECONDS: i64 = 600;

/// Clients should poll no more frequently than every 5 seconds.
pub const DEVICE_POLL_INTERVAL_SECONDS: i64 = 5;

/// Alphabet for user codes — excludes ambiguous characters (0/O, 1/I/L, 5/S).
const USER_CODE_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRTUVWXYZ2346789";

/// Generate a random device code (40-char hex string).
/// This is the secret that only the device holds.
pub fn generate_device_code() -> String {
    let uuid1 = uuid::Uuid::new_v4();
    let uuid2 = uuid::Uuid::new_v4();
    format!(
        "{}{}",
        uuid1.as_simple(),
        &uuid2.as_simple().to_string()[..8]
    )
}

/// Generate a user-facing code in `XXXX-XXXX` format.
/// Uses an alphabet that avoids ambiguous characters.
pub fn generate_user_code() -> String {
    let mut rng = rand::thread_rng();
    let chars: String = (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..USER_CODE_ALPHABET.len());
            USER_CODE_ALPHABET[idx] as char
        })
        .collect();
    format!("{}-{}", &chars[..4], &chars[4..])
}

/// Hash a device code with SHA-256 (same pattern as `jwt::hash_token`).
pub fn hash_device_code(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Normalize user input: uppercase, strip all non-alphanumeric, re-format as `XXXX-XXXX`.
pub fn normalize_user_code(input: &str) -> Option<String> {
    let cleaned: String = input
        .to_uppercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    if cleaned.len() != 8 {
        return None;
    }

    Some(format!("{}-{}", &cleaned[..4], &cleaned[4..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_code_length() {
        let code = generate_device_code();
        assert_eq!(code.len(), 40);
        assert!(code.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn user_code_format() {
        let code = generate_user_code();
        assert_eq!(code.len(), 9); // XXXX-XXXX
        assert_eq!(code.chars().nth(4), Some('-'));
        // All characters should be from the alphabet
        for c in code.chars().filter(|c| *c != '-') {
            assert!(
                USER_CODE_ALPHABET.contains(&(c as u8)),
                "Unexpected char: {}",
                c
            );
        }
    }

    #[test]
    fn user_code_has_no_ambiguous_chars() {
        for _ in 0..100 {
            let code = generate_user_code();
            let ambiguous = ['0', 'O', '1', 'I', 'L', '5', 'S'];
            for c in code.chars().filter(|c| *c != '-') {
                assert!(
                    !ambiguous.contains(&c),
                    "Found ambiguous char '{}' in code {}",
                    c,
                    code
                );
            }
        }
    }

    #[test]
    fn hash_device_code_consistent() {
        let code = "test-device-code-12345";
        let h1 = hash_device_code(code);
        let h2 = hash_device_code(code);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn normalize_user_code_strips_formatting() {
        assert_eq!(
            normalize_user_code("abcd-efgh"),
            Some("ABCD-EFGH".to_string())
        );
        assert_eq!(
            normalize_user_code("ABCDEFGH"),
            Some("ABCD-EFGH".to_string())
        );
        assert_eq!(
            normalize_user_code("abcd efgh"),
            Some("ABCD-EFGH".to_string())
        );
        assert_eq!(
            normalize_user_code(" ab-cd ef-gh "),
            Some("ABCD-EFGH".to_string())
        );
    }

    #[test]
    fn normalize_user_code_rejects_wrong_length() {
        assert_eq!(normalize_user_code("ABC"), None);
        assert_eq!(normalize_user_code("ABCDEFGHI"), None);
        assert_eq!(normalize_user_code(""), None);
    }
}
