

/// Bits is a struct that holds an arbitrary amount of binary data. The data is stored
/// in a Vec<u8> but does not need to be a multiple of 8 bits. A bit offset and a bit length
/// are stored.
pub struct Bits {
    data: Vec<u8>,
    offset: u64,
    length: u64,
}

impl Bits {
    pub fn new(data: Vec<u8>, offset: u64, length: u64) -> Result<Self, String> {
        if offset + length > (data.len() as u64) * 8 {
            return Err(format!(
                "Offset + length ({} bits) is greater than data length ({} bits).",
                offset + length,
                (data.len() as u64) * 8
            ));
        }
        Ok(Bits {
            data,
            offset,
            length,
        })
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let bitlength = (data.len() as u64) * 8;
        Bits {
            data,
            offset: 0,
            length: bitlength,
        }
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let data = match hex::decode(hex) {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };
        Ok(Bits::from_bytes(data))
    }

    // pub fn from_bin(bin: &str) -> Result<Self, String> {
    //
    // }

    fn binary_string_to_vec_u8(binary_string: &str) -> Result<Vec<u8>, String> {
        let mut data: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        let mut bit_count = 0;
        for c in binary_string.chars() {
            if c == '1' {
                byte |= 1 << (7 - bit_count);
            }
            if c != '0' {
                return Err(format!("Invalid character in binary string: {}", c));
            }
            bit_count += 1;
            if bit_count == 8 {
                data.push(byte);
                byte = 0;
                bit_count = 0;
            }
        }
        Ok(data)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let data: Vec<u8> = vec![10, 20, 30];
        let bits = Bits::new(data, 0, 24).unwrap();
        assert_eq!(bits.data, vec![10, 20, 30]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 24);
    }

    #[test]
    fn from_bytes() {
        let data: Vec<u8> = vec![10, 20, 30];
        let bits = Bits::from_bytes(data);
        assert_eq!(bits.data, vec![10, 20, 30]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 24);
    }

    #[test]
    fn from_hex() {
        let bits = Bits::from_hex("0a141e").unwrap();
        assert_eq!(bits.data, vec![10, 20, 30]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 24);
    }

    #[test]
    fn binary_string_to_vec_u8() {
        let mut data = Bits::binary_string_to_vec_u8("00001010").unwrap();
        assert_eq!(data, vec![10]);
        data = Bits::binary_string_to_vec_u8("").unwrap();
        assert_eq!(data, vec![]);
    }

    #[test]
    #[should_panic]
    fn binary_string_to_vec_errors() {
        let _data = Bits::binary_string_to_vec_u8("hello").unwrap();
    }

}
