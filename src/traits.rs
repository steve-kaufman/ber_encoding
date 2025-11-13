use crate::error::BerError;
use crate::tag::Tag;
use std::io::{Read, Write};

pub trait BerTag {
    fn tag() -> Tag;
}

pub trait BerEncode: BerTag {
    fn encode(&self, writer: &mut impl Write) -> Result<(), BerError>;

    fn encode_with_tag(&self, tag: Tag, writer: &mut impl Write) -> Result<(), BerError>;
}

pub trait BerDecode: BerTag + Sized {
    fn decode(reader: &mut impl Read) -> Result<Self, BerError>;

    fn decode_with_tag(tag: Tag, reader: &mut impl Read) -> Result<Self, BerError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tag::TagClass;

    struct TestType(i32);

    impl BerTag for TestType {
        fn tag() -> Tag {
            Tag::universal(2, false)
        }
    }

    impl BerEncode for TestType {
        fn encode(&self, writer: &mut impl Write) -> Result<(), BerError> {
            Self::tag().encode(writer)?;
            writer.write_all(&[0x01])?;
            writer.write_all(&[self.0 as u8])?;
            Ok(())
        }

        fn encode_with_tag(&self, tag: Tag, writer: &mut impl Write) -> Result<(), BerError> {
            tag.encode(writer)?;
            writer.write_all(&[0x01])?;
            writer.write_all(&[self.0 as u8])?;
            Ok(())
        }
    }

    impl BerDecode for TestType {
        fn decode(reader: &mut impl Read) -> Result<Self, BerError> {
            let tag = Tag::decode(reader)?;
            if tag != Self::tag() {
                return Err(BerError::TagMismatch {
                    expected: Self::tag().tag_number() as u8,
                    found: tag.tag_number() as u8,
                });
            }
            Self::decode_with_tag(tag, reader)
        }

        fn decode_with_tag(_tag: Tag, reader: &mut impl Read) -> Result<Self, BerError> {
            let mut buf = [0u8; 1];
            reader.read_exact(&mut buf)?;
            reader.read_exact(&mut buf)?;
            Ok(TestType(buf[0] as i32))
        }
    }

    #[test]
    fn test_ber_tag_trait() {
        assert_eq!(TestType::tag().tag_number(), 2);
        assert_eq!(TestType::tag().class(), TagClass::Universal);
        assert!(!TestType::tag().is_constructed());
    }

    #[test]
    fn test_ber_encode_trait() {
        let value = TestType(42);
        let mut buf = Vec::new();
        value.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x02, 0x01, 42]);
    }

    #[test]
    fn test_ber_decode_trait() {
        let data = vec![0x02, 0x01, 42];
        let value = TestType::decode(&mut &data[..]).unwrap();
        assert_eq!(value.0, 42);
    }

    #[test]
    fn test_ber_encode_with_tag() {
        let value = TestType(42);
        let custom_tag = Tag::context_specific(5, false);
        let mut buf = Vec::new();
        value.encode_with_tag(custom_tag, &mut buf).unwrap();
        assert_eq!(buf, vec![0x85, 0x01, 42]);
    }

    #[test]
    fn test_ber_decode_with_tag() {
        let custom_tag = Tag::context_specific(5, false);
        let data = vec![0x01, 42];
        let value = TestType::decode_with_tag(custom_tag, &mut &data[..]).unwrap();
        assert_eq!(value.0, 42);
    }

    #[test]
    fn test_ber_decode_tag_mismatch() {
        let data = vec![0x05, 0x00];
        let result = TestType::decode(&mut &data[..]);
        assert!(matches!(result, Err(BerError::TagMismatch { .. })));
    }

    #[test]
    fn test_round_trip() {
        let original = TestType(123);
        let mut buf = Vec::new();
        original.encode(&mut buf).unwrap();
        let decoded = TestType::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded.0, original.0);
    }
}
