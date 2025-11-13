use crate::error::BerError;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TagClass {
    Universal,
    Application,
    ContextSpecific,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tag {
    class: TagClass,
    constructed: bool,
    number: u32,
}

impl Tag {
    pub fn new(class: TagClass, constructed: bool, number: u32) -> Self {
        Tag {
            class,
            constructed,
            number,
        }
    }

    pub fn universal(number: u32, constructed: bool) -> Self {
        Tag::new(TagClass::Universal, constructed, number)
    }

    pub fn context_specific(number: u32, constructed: bool) -> Self {
        Tag::new(TagClass::ContextSpecific, constructed, number)
    }

    pub fn is_constructed(&self) -> bool {
        self.constructed
    }

    pub fn tag_number(&self) -> u32 {
        self.number
    }

    pub fn class(&self) -> TagClass {
        self.class
    }

    pub fn encode(&self, writer: &mut impl Write) -> Result<(), BerError> {
        let class_bits = match self.class {
            TagClass::Universal => 0b00,
            TagClass::Application => 0b01,
            TagClass::ContextSpecific => 0b10,
            TagClass::Private => 0b11,
        };

        let constructed_bit = if self.constructed { 0b1 } else { 0b0 };

        if self.number < 31 {
            let byte = (class_bits << 6) | (constructed_bit << 5) | (self.number as u8);
            writer.write_all(&[byte])?;
        } else {
            let first_byte = (class_bits << 6) | (constructed_bit << 5) | 0b11111;
            writer.write_all(&[first_byte])?;

            let mut number = self.number;
            let mut bytes = Vec::new();

            while number > 0 {
                let mut byte = (number & 0x7F) as u8;
                number >>= 7;
                if !bytes.is_empty() {
                    byte |= 0x80;
                }
                bytes.insert(0, byte);
            }

            writer.write_all(&bytes)?;
        }

        Ok(())
    }

    pub fn decode(reader: &mut impl Read) -> Result<Self, BerError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let first_byte = buf[0];

        const TAG_CLASSES: [TagClass; 4] = [
            TagClass::Universal,
            TagClass::Application,
            TagClass::ContextSpecific,
            TagClass::Private,
        ];
        let class = TAG_CLASSES[((first_byte >> 6) & 0b11) as usize];

        let constructed = ((first_byte >> 5) & 0b1) == 1;

        let number = if (first_byte & 0b11111) < 31 {
            (first_byte & 0b11111) as u32
        } else {
            let mut number: u32 = 0;
            loop {
                reader.read_exact(&mut buf)?;
                let byte = buf[0];

                number = (number << 7) | ((byte & 0x7F) as u32);

                if (byte & 0x80) == 0 {
                    break;
                }

                if number > (u32::MAX >> 7) {
                    return Err(BerError::InvalidTagNumber);
                }
            }
            number
        };

        Ok(Tag {
            class,
            constructed,
            number,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_form_tag() {
        let tag = Tag::universal(2, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x02]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_short_form_constructed() {
        let tag = Tag::universal(16, true);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x30]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_long_form_tag() {
        let tag = Tag::universal(127, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x1F, 0x7F]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_context_specific_tag() {
        let tag = Tag::context_specific(5, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x85]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_application_tag() {
        let tag = Tag::new(TagClass::Application, true, 10);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x6A]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_large_tag_number() {
        let tag = Tag::universal(200, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x1F, 0x81, 0x48]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_private_tag() {
        let tag = Tag::new(TagClass::Private, false, 15);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0xCF]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
        assert_eq!(decoded.class(), TagClass::Private);
    }

    #[test]
    fn test_tag_getters() {
        let tag = Tag::new(TagClass::Application, true, 42);
        assert_eq!(tag.class(), TagClass::Application);
        assert!(tag.is_constructed());
        assert_eq!(tag.tag_number(), 42);
    }

    #[test]
    fn test_invalid_tag_number_overflow() {
        let data = vec![0x1F, 0xFF, 0xFF, 0xFF, 0xFF, 0x7F];
        let result = Tag::decode(&mut &data[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_tag_number_31() {
        let tag = Tag::universal(31, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x1F, 0x1F]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_very_large_tag_number() {
        let tag = Tag::universal(16383, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_tag_number_0() {
        let tag = Tag::universal(0, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_tag_number_30() {
        let tag = Tag::universal(30, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x1E]);

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
    }

    #[test]
    fn test_tag_long_form_multibyte() {
        let tag = Tag::universal(1000, false);
        let mut buf = Vec::new();
        tag.encode(&mut buf).unwrap();

        let decoded = Tag::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, tag);
        assert_eq!(decoded.tag_number(), 1000);
    }

    #[test]
    fn test_all_tag_classes() {
        for &(class, shift) in &[
            (TagClass::Universal, 0b00),
            (TagClass::Application, 0b01),
            (TagClass::ContextSpecific, 0b10),
            (TagClass::Private, 0b11),
        ] {
            let tag = Tag::new(class, false, 5);
            let mut buf = Vec::new();
            tag.encode(&mut buf).unwrap();
            assert_eq!(buf[0] >> 6, shift);

            let decoded = Tag::decode(&mut &buf[..]).unwrap();
            assert_eq!(decoded.class(), class);
        }
    }
}
