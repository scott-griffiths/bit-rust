use std::fmt;
use std::rc::Rc;
use crate::BitsError;

/// Bits is a struct that holds an arbitrary amount of binary data. The data is stored
/// in a Vec<u8> but does not need to be a multiple of 8 bits. A bit offset and a bit length
/// are stored.
pub struct Bits {
    data: Rc<Vec<u8>>,
    offset: u64,
    length: u64,
}

impl fmt::Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.length > 100 {
            return f.debug_struct("Bits")
                .field("hex", &self.slice(0, 100).to_hex().unwrap())
                .field("length", &self.length)
                .finish();
        }
        if self.length % 4 == 0 {
            return f.debug_struct("Bits")
                .field("hex", &self.to_hex().unwrap())
                .field("length", &self.length)
                .finish();
        }
        f.debug_struct("Bits")
            .field("bin", &self.to_bin())
            .field("length", &self.length)
            .finish()
    }
}

impl Clone for Bits {
    fn clone(&self) -> Self {
        Bits {
            data: Rc::clone(&self.data),
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
        self.to_bin() == other.to_bin()
    }
}

impl Bits {
    pub fn new(data: Vec<u8>, offset: u64, length: u64) -> Result<Self, BitsError> {
        if offset + length > (data.len() as u64) * 8 {
            return Err(BitsError::InvalidLength(offset + length));
        }
        if offset < 8 && (offset + length + 7) / 8 == data.len() as u64 {
            return Ok(Bits {
                data: Rc::new(data),
                offset,
                length,
            });
        }
        let start_byte = (offset / 8) as usize;
        let end_byte = ((offset + length + 7) / 8) as usize;
        Ok(Bits {
            data: Rc::new(data[start_byte..end_byte].to_vec()),
            offset: offset % 8,
            length,
        })
    }

    /// Returns the bit offset to the data in the Bits object.
    pub fn offset(&self) -> u64 {
        self.offset
    }

    /// Returns the length of the Bits object in bits.
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Returns a reference to the raw data in the Bits object.
    /// Note that the offset and length values govern which part of this raw buffer is the actual
    /// binary data.
    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn index(&self, bit_index: u64) -> Result<bool, BitsError> {
        if bit_index >= self.length {
            return Err(BitsError::OutOfBounds(bit_index, self.length));
        }
        let p: u64 = bit_index + self.offset;
        let byte = self.data[(p / 8) as usize];
        Ok(byte & (128 >> (p % 8)) != 0)
    }

    pub fn slice(&self, start_bit: u64, end_bit: u64) -> Self {
        assert!(start_bit <= end_bit);
        assert!(end_bit <= self.length);
        let new_length = end_bit - start_bit;
        Bits {
            data: Rc::clone(&self.data),
            offset: start_bit + self.offset,
            length: new_length,
        }
    }

    // Return a new Bits with any excess stored bytes trimmed.
    pub fn trim(&self) -> Self {
        if self.offset < 8 && self.end_byte() == self.data.len() {
            return Bits {
                data: Rc::clone(&self.data),
                offset: self.offset,
                length: self.length,
            }
        }
        Bits {
            data: Rc::new(self.data[self.start_byte()..self.end_byte()].to_vec()),
            offset: self.offset % 8,
            length: self.length,
        }
    }

    pub fn from_bytes(data: Vec<u8>) -> Self {
        let bitlength = (data.len() as u64) * 8;
        Bits {
            data: Rc::new(data),
            offset: 0,
            length: bitlength,
        }
    }

    pub fn from_hex(hex: &str) -> Result<Self, BitsError> {
        let mut new_hex = hex.to_string();
        let is_odd_length: bool = hex.len() % 2 != 0;
        if is_odd_length {
            new_hex.push('0');
        }
        let data = match hex::decode(new_hex) {
            Ok(d) => d,
            Err(e) => return Err(BitsError::HexDecodeError(e)),
        };
        Ok(Bits {
            data: Rc::new(data),
            offset: 0,
            length: hex.len() as u64 * 4,
        })

    }

    pub fn from_bin(binary_string: &str) -> Result<Self, BitsError> {
        let mut data: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        for chunk in binary_string.as_bytes().chunks(8) {
            for (i, &c) in chunk.iter().enumerate() {
                if c == b'1' {
                    byte |= 1 << (7 - i);
                } else if c != b'0' {
                    return Err(BitsError::InvalidCharacter(c as char));
                }
            }
            data.push(byte);
            byte = 0;
        }
        Ok(Bits {
            data: Rc::new(data),
            offset: 0,
            length: binary_string.len() as u64,
        })
    }

    pub fn to_bin(&self) -> String {
        let x = self.data.iter()
            .map(|byte| format!("{:08b}", byte))
            .fold(String::new(), |mut bin_str, bin| {
                bin_str.push_str(&bin);
                bin_str
            });
        x[self.offset as usize..(self.offset + self.length) as usize].to_string()
    }

    pub fn to_hex(&self) -> Result<String, BitsError> {
        if self.length % 4 != 0 {
            return Err(BitsError::InvalidLength(self.length));
        }
        let bit_offset = self.offset % 8;
        let nibble_offset_data: &Vec<u8> = if bit_offset == 0 || bit_offset == 4 {
            &self.data[self.start_byte()..self.end_byte()].to_vec()
        } else {
            &self.copy_with_new_offset(0).data
        };
        let x = nibble_offset_data.iter()
            .map(|byte| format!("{:02x}", byte))
            .fold(String::new(), |mut acc, hex| {
                acc.push_str(&hex);
                acc
            });
        if bit_offset == 4 {
            if self.length % 8 == 0 {
                return Ok(x[1..x.len()-1].to_string());
            }
            return Ok(x[1..].to_string());
        }
        if self.length % 8 == 0 {
            return Ok(x);
        }
        debug_assert_eq!(self.length % 8, 4);
        Ok(x[..x.len()-1].to_string())
    }

    pub fn from_zeros(length: u64) -> Self {
        Bits {
            data: Rc::new(vec![0; ((length + 7) / 8) as usize]),
            offset: 0,
            length,
        }
    }

    pub fn from_ones(length: u64) -> Self {
        Bits {
            data: Rc::new(vec![0xff; ((length + 7) / 8) as usize]),
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
        let mut data = bits_vec[0].data[bits_vec[0].start_byte()..bits_vec[0].end_byte()].to_vec();
        let new_offset: u64 = bits_vec[0].offset % 8;
        let mut new_length: u64 = bits_vec[0].length;
        // Go though the vec of Bits and set the offset of each to the number of bits in the final byte of the previous one
        for bits in &bits_vec[1..] {
            if bits.length == 0 {
                continue;
            }
            let extra_bits = (new_length + new_offset) % 8;
            let offset_bits = bits.copy_with_new_offset(extra_bits);
            if extra_bits == 0 {
                data.extend(offset_bits.data[offset_bits.start_byte()..offset_bits.end_byte()].to_vec());
            }
            else {
                // Combine last byte of data with first byte of offset_bits.data.
                // The first extra_bits come from the last byte of data, the rest from the first byte of offset_bits.data.
                let last_byte = data.pop().unwrap() & !(0xff >> extra_bits);
                let first_byte = offset_bits.data[0] & (0xff >> extra_bits);
                data.push(last_byte + first_byte);
                data.extend(&offset_bits.data[1..]);
            }
            new_length += bits.length;
        }
        Bits {
            data: Rc::new(data),
            offset: new_offset,
            length: new_length,
        }
    }

    pub fn find(&self, b: &Bits) -> Option<u64> {
        if b.length > self.length {
            return None;
        }
        for sb in 0..self.length - b.length {
            if self.slice(sb, sb + b.length) == *b {
                return Some(sb);
            }
        }
        None
    }

    pub fn find_aligned(&self, b: &Bits, step: usize) -> Option<u64> {
        if b.length > self.length {
            return None;
        }
        for sb in (0..self.length - b.length).step_by(step) {
            if self.slice(sb, sb + b.length) == *b {
                return Some(sb);
            }
        }
        None
    }

    fn count(&self) -> u64 {
        let mut count: u64 = 0;
        let clipped = self.copy_with_new_offset(0);
        for byte in clipped.data.iter() {
            count += byte.count_ones() as u64;
        }
        count
    }

    pub fn count_ones(&self) -> u64 {
        self.count()
    }

    pub fn count_zeros(&self) -> u64 {
        self.length - self.count()
    }

    pub fn reverse(&self) -> Self {
        let mut data: Vec<u8> = Vec::new();
        for byte in self.data[self.start_byte()..self.end_byte()].iter() {
            data.push(byte.reverse_bits());
        }
        let final_bits = (self.offset + self.length) % 8;
        let new_offset = if final_bits == 0 { 0 } else { 8 - final_bits };
        Bits {
            data: Rc::new(data),
            offset: new_offset,
            length: self.length,
        }
    }

    /// Returns the byte index of the start of the binary data.
    fn start_byte(&self) -> usize {
        (self.offset / 8) as usize
    }

    /// Returns the byte index of the end of the binary data.
    fn end_byte(&self) -> usize {
        ((self.offset + self.length + 7) / 8) as usize
    }

    pub fn invert(&self) -> Self {
        let mut data: Vec<u8> = Vec::new();
        for byte in self.data[self.start_byte()..self.end_byte()].iter() {
            data.push(byte ^ 0xff);
        }
        Bits {
            data: Rc::new(data),
            offset: self.offset,
            length: self.length,
        }
    }

    // Return copy with a new offset (< 8). Any excess bytes will be trimmed.
    fn copy_with_new_offset(&self, new_offset: u64) -> Self {
        assert!(new_offset < 8);
        // Create a new Bits object with the same value but a different offset.
        // Each byte will in general have to be bit shifted to the left or right.
        if self.length == 0 {
            return Bits {
                data: Rc::new(vec![]),
                offset: 0,
                length: 0,
            }
        }
        let byte_offset = (self.offset / 8) as usize;
        let bit_offset = self.offset % 8;
        if new_offset == bit_offset {
            return Bits {
                data: Rc::new(self.data[self.start_byte()..self.end_byte()].to_vec()),
                offset: new_offset,
                length: self.length,
            }
        }
        let old_byte_length = ((self.length + self.offset + 7)/ 8) as usize;
        let new_byte_length = ((self.length + new_offset + 7) / 8) as usize;
        let mut new_data: Vec<u8> = vec![0; new_byte_length];
        if new_offset < bit_offset {
            let left_shift = bit_offset - new_offset;
            debug_assert!(left_shift < 8);
            // Do everything up to the final byte
            for i in 0..new_byte_length - 1 {
                new_data[i] = (self.data[i + byte_offset] << left_shift) + (self.data[i + 1 + byte_offset] >> (8 - left_shift));
            }
            // The final byte
            new_data[new_byte_length - 1] = self.data[byte_offset + new_byte_length - 1] << left_shift;
        }
        else {
            let right_shift: u64 = new_offset - bit_offset;
            debug_assert!(right_shift < 8);
            new_data[0] = self.data[0] >> right_shift;
            for i in 1..old_byte_length {
                new_data[i] = (self.data[i + byte_offset] >> right_shift) + (self.data[i + byte_offset - 1] << (8 - right_shift));
            }
            if new_byte_length > old_byte_length {
                new_data[new_byte_length - 1] = self.data[byte_offset + old_byte_length - 1] << (8 - right_shift);
            }
        }
        Bits {
            data: Rc::new(new_data),
            offset: new_offset,
            length: self.length,
        }
    }
}