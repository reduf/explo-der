use crate::error::{Error, Result};

pub const CONSTRUCTED: u8 = 0x20;
pub const APPLICATION: u8 = 0x40;
pub const CONTEXT_SPECIFIC: u8 = 0x80;

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Tag {
    Boolean = 0x01,
    Integer = 0x02,
    BitString = 0x03,
    OctetString = 0x04,
    Null = 0x05,
    OID = 0x06,
    Utf8String = 0x0C,
    PrintableString = 0x13,
    UTCTime = 0x17,
    GeneralizedTime = 0x18,
    UniversalString = 0x1C,
    BMPString = 0x1E,

    Sequence = CONSTRUCTED | 0x10, // 0x30
    Set = CONSTRUCTED | 0x11,      // 0x31

    ContextSpecific0 = CONTEXT_SPECIFIC | 0,
    ContextSpecific1 = CONTEXT_SPECIFIC | 1,

    ContextSpecificConstructed0 = CONTEXT_SPECIFIC | CONSTRUCTED | 0,
    ContextSpecificConstructed1 = CONTEXT_SPECIFIC | CONSTRUCTED | 1,
    ContextSpecificConstructed2 = CONTEXT_SPECIFIC | CONSTRUCTED | 2,
    ContextSpecificConstructed3 = CONTEXT_SPECIFIC | CONSTRUCTED | 3,

    ApplicationConstructed0 = APPLICATION | CONSTRUCTED | 0,
}

impl From<Tag> for u8 {
    fn from(tag: Tag) -> Self {
        return tag as Self;
    }
}

impl TryFrom<u8> for Tag {
    type Error = Error;

    fn try_from(tag: u8) -> Result<Self> {
        return match tag {
            x if x == Tag::Boolean as u8 => Ok(Tag::Boolean),
            x if x == Tag::Integer as u8 => Ok(Tag::Integer),
            x if x == Tag::BitString as u8 => Ok(Tag::BitString),
            x if x == Tag::OctetString as u8 => Ok(Tag::OctetString),
            x if x == Tag::Null as u8 => Ok(Tag::Null),
            x if x == Tag::OID as u8 => Ok(Tag::OID),
            x if x == Tag::Utf8String as u8 => Ok(Tag::Utf8String),
            x if x == Tag::Sequence as u8 => Ok(Tag::Sequence),
            x if x == Tag::Set as u8 => Ok(Tag::Set),
            x if x == Tag::PrintableString as u8 => Ok(Tag::PrintableString),
            x if x == Tag::UTCTime as u8 => Ok(Tag::UTCTime),
            x if x == Tag::GeneralizedTime as u8 => Ok(Tag::GeneralizedTime),
            x if x == Tag::UniversalString as u8 => Ok(Tag::UniversalString),
            x if x == Tag::BMPString as u8 => Ok(Tag::BMPString),
            x if x == Tag::ContextSpecific0 as u8 => Ok(Tag::ContextSpecific0),
            x if x == Tag::ContextSpecific1 as u8 => Ok(Tag::ContextSpecific1),
            x if x == Tag::ContextSpecificConstructed0 as u8 => Ok(Tag::ContextSpecificConstructed0),
            x if x == Tag::ContextSpecificConstructed1 as u8 => Ok(Tag::ContextSpecificConstructed1),
            x if x == Tag::ContextSpecificConstructed2 as u8 => Ok(Tag::ContextSpecificConstructed2),
            x if x == Tag::ContextSpecificConstructed3 as u8 => Ok(Tag::ContextSpecificConstructed3),
            x if x == Tag::ApplicationConstructed0 as u8 => Ok(Tag::ApplicationConstructed0),
            _ => Err(Error("Invalid or unsupported tag")),
        };
    }
}

/// A read-only, forward only, and zero-copy reader on a slice of bytes.
pub struct Reader<'a> {
    input: &'a [u8],
    index: usize,
}

const END_OF_INPUT: Error = Error("Reached end of input before completing operation");
impl<'a> Reader<'a> {
    /// Create a reader from a slice of bytes.
    pub fn new(input: &'a [u8]) -> Self {
        return Self { input, index: 0 };
    }

    /// Returns the number of remaining bytes in the stream.
    pub fn len(&self) -> usize {
        return self.input.len() - self.index;
    }

    /// Returns true if the input was completely read.
    pub fn at_end(&self) -> bool {
        return self.index == self.input.len();
    }

    /// Returns the bytes at the specified offset from the current position
    /// without consuming the stream.
    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        return self.input.get(self.index + offset).copied();
    }

    /// Peek and check if the next bytes is the expected value.
    pub fn peek_eq(&self, expected: u8) -> bool {
        return match self.input.get(self.index) {
            Some(&byte) => byte == expected,
            None => false,
        };
    }

    /// Try to consume and return the next byte.
    pub fn read_byte(&mut self) -> Result<u8> {
        return match self.input.get(self.index) {
            Some(&byte) => {
                self.index += 1;
                Ok(byte)
            }
            None => Err(END_OF_INPUT),
        };
    }

    /// Try to read the specified amount of bytes and fail if it's not possible.
    pub fn read_bytes(&mut self, amount: usize) -> Result<&'a [u8]> {
        let new_index = self.index.checked_add(amount).ok_or("Invalid amount")?;
        if self.input.len() < new_index {
            return Err(END_OF_INPUT);
        }
        let result = &self.input[self.index..new_index];
        self.index = new_index;
        return Ok(result);
    }

    /// Consume and returns the remaining bytes.
    pub fn read_bytes_to_end(&mut self) -> &'a [u8] {
        let result = &self.input[self.index..];
        self.index = self.input.len();
        return result;
    }

    /// Consume all the remaining bytes of the reader.
    pub fn skip_to_end(&mut self) {
        self.index = self.input.len();
    }

    /// Call the given `callback` and return the consumed bytes and the result.
    pub fn read_and_get_bytes_read<F, R>(&mut self, callback: F) -> Result<(&'a [u8], R)>
    where
        F: FnOnce(&mut Reader<'a>) -> Result<R>,
    {
        let index_at_start = self.index;
        let result = callback(self)?;
        return Ok((&self.input[index_at_start..self.index], result));
    }

    /// Create a new reader from the specified input and call the given callback
    /// with a this reader. If the reader isn't completely consumed by the
    /// callback, the function returns an error.
    pub fn read_all<F, R>(input: &'a [u8], callback: F) -> Result<R>
    where
        F: FnOnce(&mut Reader<'a>) -> Result<R>,
    {
        let mut reader = Reader::new(input);
        let result = callback(&mut reader)?;
        if reader.at_end() {
            return Ok(result);
        } else {
            return Err(Error("Incomplete Read"));
        }
    }
}

pub fn read_tag_and_get_value<'a>(input: &mut Reader<'a>) -> Result<(Tag, &'a [u8])> {
    let tag = input.read_byte()?;
    if (tag & 0x1F) == 0x1F {
        return Err(Error("High tag number form is not allowed."));
    }

    let tag = Tag::try_from(tag)?;

    // If the high order bit of the first byte is set to zero then the length
    // is encoded in the seven remaining bits of that byte. Otherwise, those
    // seven bits represent the number of bytes used to encode the length.
    let length = match input.read_byte()? {
        n if (n & 0x80) == 0 => usize::from(n),
        0x81 => {
            let second_byte = input.read_byte()?;
            if second_byte < 128 {
                return Err(Error("Not the canonical encoding."));
            }
            usize::from(second_byte)
        }
        0x82 => {
            let second_byte = usize::from(input.read_byte()?);
            let third_byte = usize::from(input.read_byte()?);
            let combined = (second_byte << 8) | third_byte;
            if combined < 256 {
                return Err(Error("Not the canonical encoding."));
            }
            combined
        }
        _ => {
            return Err(Error("We don't support longer lengths."));
        }
    };

    let inner = input.read_bytes(length)?;
    return Ok((tag, inner));
}
