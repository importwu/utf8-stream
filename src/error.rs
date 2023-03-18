use std::{
    fmt,
    io,
    error
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Utf8Error {
    pub(crate) err_len: u8,
    pub(crate) bytes: [u8; 3]
}

impl Utf8Error {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[0..self.err_len as usize]
    }
}

impl fmt::Debug for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Utf8Error")
            .field("err_len", &self.err_len)
            .field("bytes", &&self.bytes[0..self.err_len as usize])
            .finish()
    }
}


impl fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.err_len {
            1 => write!(f, "invalid utf-8 sequence [0x{:x}]", self.bytes[0]),
            2 => write!(f, "invalid utf-8 sequence [0x{:x}, 0x{:x}]", self.bytes[0], self.bytes[1]),
            3 => write!(f, "invalid utf-8 sequence [0x{:x}, 0x{:x}, 0x{:x}]", self.bytes[0], self.bytes[1], self.bytes[2]),
            _ => unreachable!()
        }
    }
}

impl error::Error for Utf8Error {}

#[derive(Debug)]
pub enum Error {
    Utf8Error(Utf8Error),
    IoError(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Utf8Error(e) => e.fmt(f),
            Error::IoError(e) => e.fmt(f)
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Utf8Error(e) => Some(e),
            Error::IoError(e) => Some(e)
        }
    }
}
