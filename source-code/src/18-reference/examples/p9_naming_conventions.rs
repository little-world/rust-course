// Pattern 9: Conversion Naming Conventions
// | Prefix | Self | Returns | Cost | Example |
// |--------|------|---------|------|---------|
// | `as_` | `&self` | `&U` | O(1) | `as_str()`, `as_bytes()` |
// | `to_` | `&self` | `U` | O(n) | `to_string()`, `to_vec()` |
// | `into_` | `self` | `U` | O(1)* | `into_inner()`, `into_bytes()` |
//
// *`into_` may have O(n) cost for type conversions, but avoids cloning.

struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    fn new(data: Vec<u8>) -> Self {
        Buffer { data }
    }

    // as_: returns reference, no allocation
    fn as_slice(&self) -> &[u8] { &self.data }
    fn as_mut_slice(&mut self) -> &mut [u8] { &mut self.data }

    // to_: returns owned, may allocate/clone
    fn to_vec(&self) -> Vec<u8> { self.data.clone() }
    fn to_hex(&self) -> String {
        self.data.iter().map(|b| format!("{:02x}", b)).collect()
    }

    // into_: consumes self, returns owned
    fn into_vec(self) -> Vec<u8> { self.data }
    fn into_boxed_slice(self) -> Box<[u8]> { self.data.into_boxed_slice() }
}

fn main() {
    let mut buf = Buffer::new(vec![0xde, 0xad, 0xbe, 0xef]);

    // as_ methods - borrow
    println!("as_slice: {:?}", buf.as_slice());
    buf.as_mut_slice()[0] = 0xff;
    println!("after mutation: {:?}", buf.as_slice());

    // to_ methods - clone
    let cloned = buf.to_vec();
    println!("to_vec: {:?}", cloned);
    println!("to_hex: {}", buf.to_hex());

    // into_ methods - consume
    let boxed = buf.into_boxed_slice();
    println!("into_boxed_slice: {:?}", boxed);
    // buf is now consumed, can't use it anymore

    println!("Naming conventions example completed");
}
