/// Errors returned from `unhexlify`.
#[derive(Debug, PartialEq)]
pub enum FromHexError {
    InvalidLength(usize),
    InvalidHexCharacter(usize, u8),
}

/// Convert an array of bytes to a string formed of lowercase hexadecimal characters.
pub fn hexlify(bytes: &[u8]) -> String {
    const BYTES: [u8; 16] = *b"0123456789abcdef";

    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(BYTES[usize::from(byte >> 4)] as char);
        result.push(BYTES[usize::from(byte & 0xf)] as char);
    }
    return result;
}

/// Convert a string formed of lowecase hexadecimal characters to an array of bytes.
///
/// If the string contains a odd number of characters, `InvalidLength` is returned.
/// If the string contains non lowercase hexadecimal characters, `InvalidHexCharacter`
/// is returned.
pub fn unhexlify(input: &str) -> Result<Vec<u8>, FromHexError> {
    if (input.len() % 2) != 0 {
        return Err(FromHexError::InvalidLength(input.len()));
    }

    let mut result = Vec::with_capacity(input.len() / 2);
    for (i, chars) in input.as_bytes().chunks(2).enumerate() {
        let high = match chars[0] {
            b'0'..=b'9' => Ok(chars[0] - b'0'),
            b'a'..=b'f' => Ok(chars[0] - b'a' + 10),
            _ => Err(FromHexError::InvalidHexCharacter(i * 2, chars[0])),
        }?;
        let low = match chars[1] {
            b'0'..=b'9' => Ok(chars[1] - b'0'),
            b'a'..=b'f' => Ok(chars[1] - b'a' + 10),
            _ => Err(FromHexError::InvalidHexCharacter((i * 2) + 1, chars[1])),
        }?;

        result.push((high << 4) + low);
    }

    return Ok(result);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexlify_all() {
        assert_eq!(hexlify(b""), "");
        assert_eq!(hexlify(b"\x01\x11\x10\xAA\xA1\xA0\x0A"), "011110aaa1a00a");
    }

    #[test]
    fn unhexlify_valid() {
        assert_eq!(unhexlify("").unwrap(), b"");
        assert_eq!(
            unhexlify("011110aaa1a00a").unwrap(),
            b"\x01\x11\x10\xAA\xA1\xA0\x0A"
        );
    }

    #[test]
    fn unhexlify_invalid() {
        assert_eq!(
            unhexlify("011").unwrap_err(),
            FromHexError::InvalidLength(3)
        );
        assert_eq!(
            unhexlify("01k1").unwrap_err(),
            FromHexError::InvalidHexCharacter(2, b'k')
        );
        assert_eq!(
            unhexlify("010k").unwrap_err(),
            FromHexError::InvalidHexCharacter(3, b'k')
        );
        assert_eq!(
            unhexlify("010A").unwrap_err(),
            FromHexError::InvalidHexCharacter(3, b'A')
        );
    }
}
