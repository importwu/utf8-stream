# utf8-stream
# Example
```rust
use utf8_stream::Stream;

fn main() {
    let stream = Stream::new("you and me 你和我".as_bytes());

    for ch in stream {
        println!("{:?}", ch)
    }
}
```
