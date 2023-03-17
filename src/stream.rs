use std::io::{
    Read,
    Bytes
};

use crate::{
    Error,
    Utf8Error
};

pub struct Stream<R> {
    bytes: Bytes<R>,
    buf: Option<u8>
}

impl <R: Read> Stream<R> {
    pub fn new(reader: R) -> Self {
        Self {
            bytes: reader.bytes(),
            buf: None
        }
    }
}

impl<R: Read> Iterator for Stream<R> {
    type Item = Result<char, Error>;

    fn next(&mut self) -> Option<Self::Item> {

        let mut repr = [0u8; 4];
        let mut index = 0;

        macro_rules! store {
            ($b: expr) => {
                {
                    index += 1;
                    repr[0] = index as u8;
                    repr[index] = $b;
                }
            };
        }

        macro_rules! next {
            () => {
                match self.bytes.next() {
                    Some(Ok(b)) => b,
                    Some(Err(e)) => return Some(Err(Error::IoError(e))),
                    None => return Some(Err(Error::Utf8Error(Utf8Error {repr})))
                }
            };
        }

        let x = match self.buf.take() {
            None => match self.bytes.next()? {
                Ok(b) => b,
                Err(e) => return Some(Err(Error::IoError(e)))
            }
            Some(b) => b
        };
        
        store!(x);

        // https://tools.ietf.org/html/rfc3629
        // UTF8-1      = %x00-7F
        // UTF8-2      = %xC2-DF UTF8-tail
        // UTF8-3      = %xE0 %xA0-BF UTF8-tail / %xE1-EC 2( UTF8-tail ) /
        //               %xED %x80-9F UTF8-tail / %xEE-EF 2( UTF8-tail )
        // UTF8-4      = %xF0 %x90-BF 2( UTF8-tail ) / %xF1-F3 3( UTF8-tail ) /
        //               %xF4 %x80-8F 2( UTF8-tail )
        match x {
            (0..=0x7F) => Some(Ok(x as char)),
            (0xC2..=0xF4) => {
                let y = next!();
                match (x, y) {
                    (0xC2..=0xDF, 0x80..=0xBF) => {
                        let code_point = acc_cont_byte((x & 0x1F) as u32, y);

                        Some(Ok(unsafe {char::from_u32_unchecked(code_point)}))
                    }
                    (0xE0, 0xA0..=0xBF)
                    | (0xE1..=0xEC, 0x80..=0xBF)
                    | (0xED, 0x80..=0x9F)
                    | (0xEE..=0xEF, 0x80..=0xBF) => {
                        store!(y);

                        let code_point = acc_cont_byte((x & 0xF) as u32, y);

                        let z = next!();
                        
                        if z < 0x80 || z > 0xBF {
                            self.buf = Some(z);
                            return Some(Err(Error::Utf8Error(Utf8Error {repr})))
                        }

                        let code_point = acc_cont_byte(code_point, z);

                        Some(Ok(unsafe {char::from_u32_unchecked(code_point)}))
                    }
                    (0xF0, 0x90..=0xBF) 
                    | (0xF1..=0xF3, 0x80..=0xBF) 
                    | (0xF4, 0x80..=0x8F) => {

                        store!(y);

                        let code_point = acc_cont_byte((x & 0x7) as u32, y);

                        let z = next!();
                        
                        if z < 0x80 || z > 0xBF {
                            self.buf = Some(z);
                            return Some(Err(Error::Utf8Error(Utf8Error {repr})))
                        }
                        
                        store!(z);

                        let code_point = acc_cont_byte(code_point, z);

                        let w = next!();

                        if w < 0x80 || w > 0xBF {
                            self.buf = Some(w);
                            return Some(Err(Error::Utf8Error(Utf8Error {repr})))
                        }

                        let code_point = acc_cont_byte(code_point, w);

                        Some(Ok(unsafe {char::from_u32_unchecked(code_point)}))
                    }
                    _ => {
                        self.buf = Some(y);
                        return Some(Err(Error::Utf8Error(Utf8Error {repr})))
                    }
                }
            }
            _ => Some(Err(Error::Utf8Error(Utf8Error {repr})))
        }
    }
}

fn acc_cont_byte(point: u32, byte: u8) -> u32 {
    (point << 6) | (byte as u32 & 0x3F)
}

mod test {

use super::*;

#[test]
    fn test() {
        let stream = Stream::new(b"Hello \xF0\x90\x80World".as_slice());

        let mut stream = stream
            .map(|r| match r {
                Ok(ch) => Ok(ch),
                Err(Error::Utf8Error(e)) => Err(e),
                Err(_) => unreachable!()
            });

        assert_eq!(stream.next(), Some(Ok('H')));
        assert_eq!(stream.next(), Some(Ok('e')));
        assert_eq!(stream.next(), Some(Ok('l')));
        assert_eq!(stream.next(), Some(Ok('l')));
        assert_eq!(stream.next(), Some(Ok('o')));
        assert_eq!(stream.next(), Some(Ok(' ')));
        assert_eq!(stream.next(), Some(Err(Utf8Error { repr: [3, 0xF0, 0x90, 0x80] })));
        assert_eq!(stream.next(), Some(Ok('W')));
        assert_eq!(stream.next(), Some(Ok('o')));
        assert_eq!(stream.next(), Some(Ok('r')));
        assert_eq!(stream.next(), Some(Ok('l')));
        assert_eq!(stream.next(), Some(Ok('d')));
        assert_eq!(stream.next(), None);
    }
}