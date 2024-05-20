use crate::error::{Error, Result};
use std::fmt::Write;

/// Converts an BER-encoded OID to it's string representation.
///
/// The limit for every arc value is 4294967295. If the BER-encoding
/// representation of the OID doesn't respect that, the function fails.
pub fn stringify(input: &[u8]) -> Result<String> {
    if input.is_empty() {
        return Err(Error("OID cannot have the length zero"));
    }

    let mut result = String::with_capacity(input.len());
    write!(&mut result, "{}.{}", input[0] / 40, input[0] % 40)
        .map_err(|_| Error("Failed to write a byte"))?;

    let mut value = 0;
    for byte in &input[1..] {
        if ((u32::MAX - 0x7F) >> 7) < value {
            return Err(Error("Value of an arc is too large"));
        }

        if (*byte == 0x80) && (value == 0) {
            return Err(Error("Illegal padding"));
        }

        value = value << 7;
        value = value + u32::from(byte & 0x7F);

        if (byte & 0x80) == 0 {
            write!(&mut result, ".{}", value).map_err(|_| Error("Failed to write a byte"))?;
            value = 0;
        }
    }

    Ok(result)
}

/// Converts a string encoded OID to it's BER-encoded representation.
///
/// The limit for every arc value is 4294967295.
pub fn parse_oid(input: &str) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(Error("OID cannot have the length zero"));
    }

    let mut result = Vec::new();
    let mut pieces = input.split('.');
    if let Some(p1) = pieces.next() {
        let p2 = pieces.next().ok_or("A valid OID can't have a single arc")?;

        let b1: u8 = p1.parse().map_err(|_| Error("Invalid uint32"))?;
        let b2: u8 = p2.parse().map_err(|_| Error("Invalid uint32"))?;

        if (5 < b1) || (39 < b2) {
            return Err(Error(
                "Invalid OID, the first two arcs do not respect the limitations",
            ));
        }

        result.push((b1 * 40) + b2);
        for rem in pieces {
            let value: u32 = rem.parse().map_err(|_| "Invalid uint32")?;

            // 7 bits are encoded (big-endian style) and all the groups except
            // the last group is written to the output with the highest order
            // bit set.
            // (32, 28], (28, 21], (21, 14], (14, 7], (7, 0]
            if (value & 0xf0000000) != 0 {
                result.push((((value & 0xf0000000) >> 28) as u8) | 0x80);
            }

            if (value & 0xfe00000) != 0 {
                result.push((((value & 0xfe00000) >> 21) as u8) | 0x80);
            }

            if (value & 0x1fc000) != 0 {
                result.push((((value & 0x1fc000) >> 14) as u8) | 0x80);
            }

            if (value & 0x3f80) != 0 {
                result.push((((value & 0x3f80) >> 7) as u8) | 0x80);
            }

            // Last 7 bits are not serialized with the highest-order bits set.
            // This indicate the end.
            result.push((value & 0x7F) as u8);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stringify_empty_oid() {
        assert!(stringify(b"").is_err());
    }

    #[test]
    fn stringify_single_byte_oid() {
        assert_eq!(stringify(b"\x2B").unwrap(), "1.3");
    }

    #[test]
    fn stringify_simple_oid() {
        assert_eq!(stringify(b"\x2B\x06\x01").unwrap(), "1.3.6.1");
    }

    #[test]
    fn stringify_multibytes_oid() {
        assert_eq!(stringify(b"\x2B\x82\x37").unwrap(), "1.3.311");
        assert_eq!(
            stringify(b"\x2B\x8F\xFF\xFF\xFF\x7F").unwrap(),
            "1.3.4294967295"
        );
    }

    #[test]
    fn stringify_oid_a_0x80() {
        assert_eq!(stringify(b"\x2B\x81\x80\x7F").unwrap(), "1.3.16511");
    }

    #[test]
    fn stringify_oid_with_overflow() {
        // Create an oid that would overflow the internal value.
        // We document the maximum allowed value, so we can rely on it.
        let oid = b"\x2B\x8F\xFF\xFF\xFF\x80\x01";
        assert!(stringify(oid.as_ref()).is_err());
    }

    #[test]
    fn stringify_oid_with_illegal_padding() {
        // This is failing, because "the subidentifier shall be encoded in the fewest possible octet".
        // The leading 0x80 are unnecessary, because they simply result in leading 0s.
        assert!(stringify(b"\x2B\x80\x7F").is_err());
    }

    #[test]
    fn parse_empty_oid() {
        assert!(&parse_oid("").is_err());
    }

    #[test]
    fn parse_root_oid() {
        assert_eq!(&parse_oid("1.3").unwrap(), b"\x2B");
    }

    #[test]
    fn parse_simple_oid() {
        assert_eq!(&parse_oid("1.3.54.23.21").unwrap(), b"\x2B\x36\x17\x15");
    }

    #[test]
    fn parse_largest_node_in_oid() {
        assert_eq!(
            &parse_oid("1.3.4294967295.5").unwrap(),
            b"\x2B\x8F\xFF\xFF\xFF\x7F\x05"
        );
    }

    #[test]
    fn parse_oid_with_node_too_large() {
        assert!(&parse_oid("1.3.42949672956").is_err());
    }
}
