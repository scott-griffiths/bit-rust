

/// Bits is a struct that holds an arbitrary amount of binary data. The data is stored
/// in a Vec<u8> but does not need to be a multiple of 8 bits. A bit offset and a bit length
/// are stored.
#[derive(Debug)]
pub struct Bits {
    data: Vec<u8>,
    offset: u64,
    length: u64,
}

impl Clone for Bits {
    fn clone(&self) -> Self {
        Bits {
            data: self.data.clone(),
            offset: self.offset,
            length: self.length,
        }
    }
}

impl PartialEq for Bits {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }
        let other_offset: &Bits = if other.offset != self.offset {
            &other.copy_with_new_offset(self.offset).unwrap()
        } else {
            other
        };
        for i in 0..self.data.len() {
            if self.data[i] != other_offset.data[i] {
                return false;
            }
        }
        true
    }
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
        if offset < 8 && (offset + length + 7) / 8 == data.len() as u64 {
            return Ok(Bits {
                data,
                offset,
                length,
            });
        }
        let start_byte = (offset / 8) as usize;
        let end_byte = ((offset + length + 7) / 8) as usize;
        Ok(Bits {
            data: data[start_byte..end_byte].to_vec(),
            offset: offset % 8,
            length,
        })
    }
    
    pub fn get_offset(&self) -> u64 {
        self.offset
    }
    
    pub fn get_length(&self) -> u64 {
        self.length
    }
    
    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
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

    pub fn get_index(&self, bit_index: u64) -> Result<bool, String> {
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

    pub fn get_slice(&self, start_bit: Option<u64>, end_bit: Option<u64>) -> Result<Self, String> {
        let start_bit = start_bit.unwrap_or_else(|| 0);
        let end_bit = end_bit.unwrap_or_else(|| self.length);
        if start_bit > end_bit {
            return Err(format!("start_bit is greater than end_bit: {} > {}.", start_bit, end_bit));
        }
        if end_bit > self.length {
            return Err(format!("end_bit ({}) is greater than the length ({}).", end_bit, self.length));
        }
        let start_byte = (start_bit + self.offset) / 8;
        let end_byte = (end_bit + self.offset + 7) / 8;
        let new_offset = (start_bit + self.offset) % 8;
        debug_assert!(end_bit >= start_bit);
        let new_length = end_bit - start_bit;
        let x = Bits {
            data: self.data[start_byte as usize..end_byte as usize].to_vec(),
            offset: new_offset,
            length: new_length,
        };
        Ok(x)
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

    pub fn to_bin(&self) -> String {
        let x = self.data.iter()
            .map(|byte| format!("{:08b}", byte))
            .fold(String::new(), |mut acc, bin| {
                acc.push_str(&bin);
                acc
            });
        x[self.offset as usize..(self.offset + self.length) as usize].to_string()
    }

    pub fn to_hex(&self) -> Result<String, String> {
        if self.length % 4 != 0 {
            return Err(format!("Bits must be a multiple of 4 to convert to hex, got length of {}", self.length));
        }
        let nibble_offset_data: &Vec<u8> =
            if self.offset == 0 || self.offset == 4
            { &self.data }
            else
            { &self.copy_with_new_offset(0)?.data };
        if self.offset == 0 || self.offset == 4 {
            let x = nibble_offset_data.iter()
                .map(|byte| format!("{:02x}", byte))
                .fold(String::new(), |mut acc, hex| {
                    acc.push_str(&hex);
                    acc
                });
            if self.offset == 0 {
                if self.length % 8 == 0 {
                    return Ok(x);
                }
                debug_assert_eq!(self.length % 8, 4);
                return Ok(x[..x.len()-1].to_string());
            }
            debug_assert_eq!(self.offset, 4);
            if self.length % 8 == 0 {
                return Ok(x[1..x.len()-1].to_string());
            }
            return Ok(x[1..].to_string());
        }
        return Ok("TODO".to_string());
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

    pub fn join(bits_vec: &Vec<&Bits>) -> Bits {
        if bits_vec.len() == 0 {
            return Bits::from_zeros(0);
        }
        if bits_vec.len() == 1 {
            return bits_vec[0].clone();
        }
        let mut data = bits_vec[0].data.clone();
        let offset: u64 = bits_vec[0].offset;
        let mut length: u64 = bits_vec[0].length;
        // Go though the vec of Bits and set the offset of each to the number of bits in the final byte of the previous one
        for bits in &bits_vec[1..] {
            if bits.length == 0 {
                continue;
            }
            let extra_bits = (length + offset) % 8;
            let offset_bits = bits.copy_with_new_offset(extra_bits).unwrap();
            if extra_bits == 0 {
                data.extend(offset_bits.data);
            }
            else {
                // Combine last byte of data with first byte of offset_bits.data.
                // The first extra_bits come from the last byte of data, the rest from the first byte of offset_bits.data.
                let last_byte = data.pop().unwrap() & !(0xff >> extra_bits);
                let first_byte = offset_bits.data[0] & (0xff >> extra_bits);
                data.push(last_byte + first_byte);
                data.extend(&offset_bits.data[1..]);
            }
            length += bits.length;
        }
        Bits {
            data,
            offset,
            length,
        }
    }

    fn copy_with_new_offset(&self, offset: u64) -> Result<Bits, String> {
        // Create a new Bits object with the same data but a different offset.
        // Each byte will in general have to be bit shifted to the left or right.
        if self.data.len() == 0 {
            debug_assert_eq!(self.length, 0);
            if offset != 0 {
                return Err("The offset of an empty Bits can only be zero.".to_string());
            }
            return Ok(Bits {
                data: vec![],
                offset: 0,
                length: 0,
            });
        }
        if offset % 8 == self.offset % 8 {
            // No bit shifts to do.
            if offset < 8 {
                return Ok(Bits {
                    data: self.data.clone(),
                    offset: self.offset,
                    length: self.length,
                });
            }
            else {
                if ((offset + self.length + 7) / 8) as usize > self.data.len() {
                    return Err(format!("Can't change the offset to {} with a length of {} and only {} bytes of data",
                    offset, self.length, self.data.len()));
                }
                let start_byte = (offset / 8) as usize;
                let end_byte = ((offset + self.length + 7) / 8) as usize;
                return Ok(Bits {
                    data: self.data[start_byte..end_byte].to_vec(),
                    offset: offset % 8,
                    length: self.length,
                });
            }
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
                debug_assert_eq!(new_byte_length, self.data.len() - 1);
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
            new_data[0] = self.data[0] >> right_shift;
            if self.data.len() > 1 {
                for i in 1..self.data.len() {
                    new_data[i] = (self.data[i] >> right_shift) + (self.data[i - 1] << (8 - right_shift));
                }
            }
            if new_byte_length > self.data.len() {
                new_data[new_byte_length - 1] = self.data[self.data.len() - 1] << (8 - right_shift);
            }
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

// Tests for internal methods only here.
#[cfg(test)]
mod tests {
    use super::*;
    
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
    fn copy_with_new_offset() {
        let bits = Bits::from_bin("001100").unwrap();
        assert_eq!(bits.to_bin(), "001100");
        let new_bits = bits.copy_with_new_offset(2).unwrap();
        assert_eq!(new_bits.to_bin(), "001100");
        assert_eq!(new_bits.data, vec![0b00001100]);
        assert_eq!(new_bits.offset, 2);
        assert_eq!(new_bits.length, 6);
        let new_bits = bits.copy_with_new_offset(0).unwrap();
        assert_eq!(new_bits.to_bin(), "001100");
        assert_eq!(new_bits.data, vec![0b00110000]);
        assert_eq!(new_bits.offset, 0);
        assert_eq!(new_bits.length, 6);
        let new_bits = bits.copy_with_new_offset(4).unwrap();
        assert_eq!(new_bits.to_bin(), "001100");
        assert_eq!(new_bits.data, vec![0b00000011, 0b00000000]);
        assert_eq!(new_bits.offset, 4);
        assert_eq!(new_bits.length, 6);
        let left_shifted_bits = new_bits.copy_with_new_offset(2).unwrap();
        assert_eq!(left_shifted_bits.to_bin(), "001100");
        assert_eq!(left_shifted_bits.data, vec![0b00001100]);
        assert_eq!(left_shifted_bits.offset, 2);
        assert_eq!(left_shifted_bits.length, 6);
    }

}
