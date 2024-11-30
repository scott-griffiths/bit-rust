

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

    fn from_bytes_with_offsets(data: Vec<u8>, offset: u64, padding: u64) -> Result<Self, String> {
        let bitlength = (data.len() as u64) * 8;
        if bitlength < offset + padding {
            return Err(format!(
                "Offset + padding ({} bits) is greater than data length ({} bits).",
                offset + padding,
                bitlength
            ));
        }
        let length: u64 = bitlength - offset - padding;
        Ok(Bits {
            data,
            offset,
            length,
        })
    }

    fn get_index(&self, bit_index: u64) -> Result<bool, String> {
        if bit_index >= self.length {
            return Err(format!(
                "Bit index {} is greater than length {}.",
                bit_index, self.length
            ));
        }
        let p: u64 = bit_index + self.offset;
        let byte = self.data[(p / 8) as usize];
        Ok(byte & (128 >> (p % 8)) != 0)
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, String> {
        let bitlength = (data.len() as u64) * 8;
        Ok(Bits {
            data,
            offset: 0,
            length: bitlength,
        })
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let mut hex = hex.to_string();
        let is_odd_length: bool = hex.len() % 2 != 0;
        if is_odd_length {
            hex.push('0');
        }
        let data = match hex::decode(hex) {
            Ok(d) => d,
            Err(e) => return Err(e.to_string()),
        };
        let padding = if is_odd_length { 4 } else { 0 };
        Bits::from_bytes_with_offsets(data, 0, padding)
    }

    pub fn from_bin(bin: &str) -> Result<Self, String> {
        let data = Bits::binary_string_to_vec_u8(bin)?;
        let padding = (8 - ((bin.len() as u64) % 8)) % 8;
        Bits::from_bytes_with_offsets(data, 0, padding)
    }

    pub fn from_zeros(length: u64) -> Self {
        Bits {
            data: vec![0; ((length + 7) / 8) as usize],
            offset: 0,
            length,
        }
    }

    pub fn from_ones(length: u64) -> Self {
        Bits {
            data: vec![0xff; ((length + 7) / 8) as usize],
            offset: 0,
            length,
        }
    }

    fn copy_with_new_offset(&self, offset: u64) -> Result<Bits, String> {
        // Create a new Bits object with the same data but a different offset.
        // Each byte will in general have to be bit shifted to the left or right.
        if self.data.len() == 0 {
            assert_eq!(self.length, 0);
            if offset != 0 {
                return Err("The offset of an empty Bits can only be zero.".to_string());
            }
            return Ok(Bits {
                data: vec![],
                offset: 0,
                length: 0,
            });
        }
        if offset == self.offset {
            // Nothing to do - just make a copy.
            return Ok(Bits {
                data: self.data.clone(),
                offset: self.offset,
                length: self.length,
            });
        }
        let new_byte_length = ((self.length + offset + 7) / 8) as usize;
        let mut new_data: Vec<u8> = vec![0; new_byte_length];
        if offset < self.offset {
            let left_shift: u64 = self.offset - offset;
            if self.data.len() == 1 {
                new_data[0] = self.data[0] << left_shift;
                return Ok(Bits {
                    data: new_data,
                    offset,
                    length: self.length,
                });
            }
            assert!(self.data.len() > 1);
            for i in 0..new_byte_length - 1 {
                new_data[i] = (self.data[i] << left_shift) + (self.data[i + 1] >> (8 - left_shift));
            }
            // Do final byte of new_data.
            if new_byte_length == self.data.len() {
                new_data[new_byte_length - 1] = self.data[self.data.len() - 1] << left_shift;
            }
            else {
                assert_eq!(new_byte_length, self.data.len() - 1);
                new_data[new_byte_length - 1] = self.data[self.data.len() - 2] << left_shift;
                new_data[new_byte_length - 1] += self.data[self.data.len() - 1] >> (8 - left_shift);
            }
            Ok(Bits {
                data: new_data,
                offset,
                length: self.length,
            })
        }
        else {
            let right_shift: u64 = offset - self.offset;
            for i in 1..self.data.len() {
                new_data[i] = (self.data[i] >> right_shift) + (self.data[i - 1] << (8 - right_shift));
            }
            new_data[0] = self.data[0] >> right_shift;
            Ok(Bits {
                data: new_data,
                offset,
                length: self.length,
            })
        }
    }

    fn binary_string_to_vec_u8(binary_string: &str) -> Result<Vec<u8>, String> {
        let mut data: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        let mut bit_count = 0;
        for c in binary_string.chars() {
            if c == '1' {
                byte |= 1 << (7 - bit_count);
            }
            else if c != '0' {
                return Err(format!("Invalid character in binary string: {}", c));
            }
            bit_count += 1;
            if bit_count == 8 {
                data.push(byte);
                byte = 0;
                bit_count = 0;
            }
        }
        if bit_count != 0 {
            data.push(byte);
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
        let bits = Bits::from_bytes(data).unwrap();
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
        let bits = Bits::from_hex("").unwrap();
        assert_eq!(bits.data, vec![]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 0);
        let bits = Bits::from_hex("hello");
        assert!(bits.is_err());
        let bits = Bits::from_hex("1").unwrap();
        assert_eq!(bits.data, vec![16]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 4);
    }

    #[test]
    fn from_bin() {
        let bits = Bits::from_bin("00001010").unwrap();
        assert_eq!(bits.data, vec![10]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 8);
        let bits = Bits::from_bin("").unwrap();
        assert_eq!(bits.data, vec![]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 0);
        let bits = Bits::from_bin("hello");
        assert!(bits.is_err());
        let bits = Bits::from_bin("1").unwrap();
        assert_eq!(bits.data, vec![128]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 1);
    }

    #[test]
    fn binary_string_to_vec_u8() {
        let mut data = Bits::binary_string_to_vec_u8("00001010").unwrap();
        assert_eq!(data, vec![10]);
        data = Bits::binary_string_to_vec_u8("").unwrap();
        assert_eq!(data, vec![]);
    }

    #[test]
    fn binary_string_to_vec_errors() {
        let data = Bits::binary_string_to_vec_u8("hello");
        assert!(data.is_err());
    }

    #[test]
    fn from_zeros() {
        let bits = Bits::from_zeros(8);
        assert_eq!(bits.data, vec![0]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 8);
        let bits = Bits::from_zeros(9);
        assert_eq!(bits.data, vec![0, 0]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 9);
        let bits = Bits::from_zeros(0);
        assert_eq!(bits.data, vec![]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 0);
    }

    #[test]
    fn from_ones() {
        let bits = Bits::from_ones(8);
        assert_eq!(bits.data, vec![255]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 8);
        let bits = Bits::from_ones(9);
        assert_eq!(bits.data[0], 0xff);
        assert_eq!(bits.data[1] & 0x80, 0x80);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 9);
        let bits = Bits::from_ones(0);
        assert_eq!(bits.data, vec![]);
        assert_eq!(bits.offset, 0);
        assert_eq!(bits.length, 0);
    }

    #[test]
    fn get_index() {
        let bits = Bits::from_bin("001100").unwrap();
        assert_eq!(bits.get_index(0).unwrap(), false);
        assert_eq!(bits.get_index(1).unwrap(), false);
        assert_eq!(bits.get_index(2).unwrap(), true);
        assert_eq!(bits.get_index(3).unwrap(), true);
        assert_eq!(bits.get_index(4).unwrap(), false);
        assert_eq!(bits.get_index(5).unwrap(), false);
        assert!(bits.get_index(6).is_err());
        assert!(bits.get_index(60).is_err());
    }

    #[test]
    fn copy_with_new_offset() {
        let bits = Bits::from_bin("001100").unwrap();
        let new_bits = bits.copy_with_new_offset(2).unwrap();
        assert_eq!(new_bits.data, vec![0b00001100]);
        assert_eq!(new_bits.offset, 2);
        assert_eq!(new_bits.length, 6);
        let new_bits = bits.copy_with_new_offset(0).unwrap();
        assert_eq!(new_bits.data, vec![0b00110000]);
        assert_eq!(new_bits.offset, 0);
        assert_eq!(new_bits.length, 6);
        let new_bits = bits.copy_with_new_offset(4).unwrap();
        assert_eq!(new_bits.data, vec![0b00000011, 0b00000000]);
        assert_eq!(new_bits.offset, 4);
        assert_eq!(new_bits.length, 6);
        let left_shifted_bits = new_bits.copy_with_new_offset(2).unwrap();
        assert_eq!(left_shifted_bits.data, vec![0b00001100]);
        assert_eq!(left_shifted_bits.offset, 2);
        assert_eq!(left_shifted_bits.length, 6);
    }

}
