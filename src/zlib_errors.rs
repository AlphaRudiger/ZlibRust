use thiserror::Error;

#[derive(Error, Debug)]
pub enum ZlibError {
    #[error("Invalid Zlib header (expected {expected:?}, got {found:?})")]
    InvalidZlibHeader {
        expected: u8,
        found: u8,
    },
    #[error("Zlib data was corrupted")]
    ZlibCorrupted(),
    #[error("Invalid Zlib Action: {action}")]
    UnsupportedZlibAction {
        action: &'static str
    },
    #[error("Zlib data was corrupted(Adler32 missmatch)")]
    ZlibChecksumMissmatch,
    #[error("Zlib data left: {left} bytes")]
    ZlibStreamNotConsumed {
        left: usize
    }
}

#[derive(Error, Debug)]
pub enum InflateError {
    #[error("NLEN check failed")]
    FailedNLenCheck,
    #[error("Unsupported Inflate Method: {method}")]
    UnsupportedMethod {
        method: u8
    }
}

