mod error;

pub use error::{
    Error,
    Utf8Error
};

mod stream;

pub use stream::Stream;