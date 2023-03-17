use std::{
    fmt,
    io,
    error
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Utf8Error {
    pub(crate) repr: [u8; 4]   //[len, code, code, code]
}

impl Utf8Error {
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let end = self.repr[0] as usize + 1;
        &self.repr[1..end]
    }
}

impl fmt::Debug for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let end = self.repr[0] as usize + 1;
        f.debug_struct("Utf8Error")
            .field("err_len", &self.repr[0])
            .field("invalid_sequence", &&self.repr[1..end])
            .finish()
    }
}


impl fmt::Display for Utf8Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr[0] {
            1 => write!(f, "invalid utf-8 sequence [0x{:x}]", self.repr[1]),
            2 => write!(f, "invalid utf-8 sequence [0x{:x}, 0x{:x}]", self.repr[1], self.repr[2]),
            3 => write!(f, "invalid utf-8 sequence [0x{:x}, 0x{:x}, 0x{:x}]", self.repr[1], self.repr[2], self.repr[3]),
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
