use std::fmt;
use std::io;

#[derive(Debug)]
pub enum BerError {
    IoError(io::Error),
    InvalidTag,
    InvalidLength,
    UnexpectedEof,
    InvalidValue(String),
    InvalidTagNumber,
    InvalidLengthEncoding,
    TagMismatch { expected: u8, found: u8 },
}

impl fmt::Display for BerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BerError::IoError(e) => write!(f, "IO error: {}", e),
            BerError::InvalidTag => write!(f, "Invalid BER tag encoding"),
            BerError::InvalidLength => write!(f, "Invalid BER length encoding"),
            BerError::UnexpectedEof => write!(f, "Unexpected end of input"),
            BerError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            BerError::InvalidTagNumber => write!(f, "Invalid tag number"),
            BerError::InvalidLengthEncoding => write!(f, "Invalid length encoding"),
            BerError::TagMismatch { expected, found } => {
                write!(f, "Tag mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for BerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BerError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BerError {
    fn from(error: io::Error) -> Self {
        BerError::IoError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_display_io_error() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "test");
        let err = BerError::from(io_err);
        assert!(format!("{}", err).contains("IO error"));
    }

    #[test]
    fn test_display_invalid_tag() {
        let err = BerError::InvalidTag;
        assert_eq!(format!("{}", err), "Invalid BER tag encoding");
    }

    #[test]
    fn test_display_invalid_length() {
        let err = BerError::InvalidLength;
        assert_eq!(format!("{}", err), "Invalid BER length encoding");
    }

    #[test]
    fn test_display_unexpected_eof() {
        let err = BerError::UnexpectedEof;
        assert_eq!(format!("{}", err), "Unexpected end of input");
    }

    #[test]
    fn test_display_invalid_value() {
        let err = BerError::InvalidValue("test message".to_string());
        assert_eq!(format!("{}", err), "Invalid value: test message");
    }

    #[test]
    fn test_display_invalid_tag_number() {
        let err = BerError::InvalidTagNumber;
        assert_eq!(format!("{}", err), "Invalid tag number");
    }

    #[test]
    fn test_display_invalid_length_encoding() {
        let err = BerError::InvalidLengthEncoding;
        assert_eq!(format!("{}", err), "Invalid length encoding");
    }

    #[test]
    fn test_display_tag_mismatch() {
        let err = BerError::TagMismatch {
            expected: 5,
            found: 10,
        };
        assert_eq!(format!("{}", err), "Tag mismatch: expected 5, found 10");
    }

    #[test]
    fn test_error_source() {
        let io_err = io::Error::new(io::ErrorKind::UnexpectedEof, "test");
        let err = BerError::from(io_err);
        assert!(err.source().is_some());

        let err = BerError::InvalidTag;
        assert!(err.source().is_none());
    }
}
