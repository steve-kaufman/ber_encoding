use crate::error::BerError;
use std::io::{Read, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    Definite(usize),
    Indefinite,
}

impl Length {
    pub fn encode(&self, writer: &mut impl Write) -> Result<(), BerError> {
        match self {
            Length::Definite(len) if *len <= 127 => {
                writer.write_all(&[*len as u8])?;
            }
            Length::Definite(len) => {
                let mut bytes = Vec::new();
                let mut remaining = *len;

                while remaining > 0 {
                    bytes.insert(0, (remaining & 0xFF) as u8);
                    remaining >>= 8;
                }

                let num_octets = bytes.len();
                // usize max is 8 octets (64-bit) or 16 octets (128-bit), always < 126
                writer.write_all(&[0x80 | num_octets as u8])?;
                writer.write_all(&bytes)?;
            }
            Length::Indefinite => {
                writer.write_all(&[0x80])?;
            }
        }

        Ok(())
    }

    pub fn decode(reader: &mut impl Read) -> Result<Self, BerError> {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let first_byte = buf[0];

        if first_byte & 0x80 == 0 {
            Ok(Length::Definite(first_byte as usize))
        } else if first_byte == 0x80 {
            Ok(Length::Indefinite)
        } else {
            let num_octets = (first_byte & 0x7F) as usize;

            if num_octets == 0 || num_octets > 8 {
                return Err(BerError::InvalidLengthEncoding);
            }

            let mut length: usize = 0;
            for _ in 0..num_octets {
                reader.read_exact(&mut buf)?;
                length = (length << 8) | (buf[0] as usize);
            }

            Ok(Length::Definite(length))
        }
    }

    pub fn value(&self) -> Option<usize> {
        match self {
            Length::Definite(len) => Some(*len),
            Length::Indefinite => None,
        }
    }

    pub fn is_definite(&self) -> bool {
        matches!(self, Length::Definite(_))
    }

    pub fn is_indefinite(&self) -> bool {
        matches!(self, Length::Indefinite)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_form_length() {
        let length = Length::Definite(42);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![42]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_short_form_zero() {
        let length = Length::Definite(0);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_short_form_max() {
        let length = Length::Definite(127);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![127]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_long_form_one_octet() {
        let length = Length::Definite(200);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x81, 0xC8]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_long_form_two_octets() {
        let length = Length::Definite(1000);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x82, 0x03, 0xE8]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_indefinite_length() {
        let length = Length::Indefinite;
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x80]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_length_value() {
        assert_eq!(Length::Definite(42).value(), Some(42));
        assert_eq!(Length::Indefinite.value(), None);
    }

    #[test]
    fn test_length_predicates() {
        assert!(Length::Definite(100).is_definite());
        assert!(!Length::Definite(100).is_indefinite());
        assert!(Length::Indefinite.is_indefinite());
        assert!(!Length::Indefinite.is_definite());
    }

    #[test]
    fn test_invalid_length_encoding_zero_octets() {
        let data = vec![0x80 | 0];
        let result = Length::decode(&mut &data[..]);
        assert!(matches!(result, Ok(Length::Indefinite)));
    }

    #[test]
    fn test_invalid_length_encoding_too_many_octets() {
        let data = vec![0x80 | 9, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let result = Length::decode(&mut &data[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_length() {
        let length = Length::Definite(0xFFFFFF);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_length_128() {
        let length = Length::Definite(128);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x81, 128]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_length_255() {
        let length = Length::Definite(255);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x81, 255]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_length_256() {
        let length = Length::Definite(256);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();
        assert_eq!(buf, vec![0x82, 0x01, 0x00]);

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_length_long_form_max_octets() {
        let length = Length::Definite(0xFFFFFFFF);
        let mut buf = Vec::new();
        length.encode(&mut buf).unwrap();

        let decoded = Length::decode(&mut &buf[..]).unwrap();
        assert_eq!(decoded, length);
    }

    #[test]
    fn test_length_long_form_boundaries() {
        for len in [128, 255, 256, 65535, 65536] {
            let length = Length::Definite(len);
            let mut buf = Vec::new();
            length.encode(&mut buf).unwrap();

            let decoded = Length::decode(&mut &buf[..]).unwrap();
            assert_eq!(decoded, length);
        }
    }

}
