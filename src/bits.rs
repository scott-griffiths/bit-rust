use std::fmt;
use std::sync::Arc;
use pyo3::{pyclass, pymethods, PyRef, PyResult};
use pyo3::exceptions::{PyIndexError, PyValueError};


/// Bits is a struct that holds an arbitrary amount of binary data. The data is stored
/// in a Vec<u8> but does not need to be a multiple of 8 bits. A bit offset and a bit length
/// are stored.
/// 
#[pyclass]
pub struct Bits {
    data: Arc<Vec<u8>>,
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
            data: Arc::clone(&self.data),
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

// Things not part of the Python interface.
impl Bits {
    fn bitwise_op<F>(&self, other: &Bits, op: F) -> Result<Self, ()>
    where F: Fn(u8, u8) -> u8 {
        if self.length != other.length {
            return Err(());
        }
        let other_offset = other.copy_with_new_offset(self.offset % 8);
        let mut data: Vec<u8> = Vec::new();
        for i in 0..other_offset.data.len() {
            data.push(op(self.data[i + self.start_byte()], other.data[i]));
        }
        Ok(Bits {
            data: Arc::new(data),
            length: other.length,
            offset: other.offset,
        })
    }
    
    fn count(&self) -> u64 {
        let mut count: u64 = 0;
        let clipped = self.copy_with_new_offset(0);
        for byte in clipped.data.iter() {
            count += byte.count_ones() as u64;
        }
        count
    }

    /// Returns the byte index of the start of the binary data.
    fn start_byte(&self) -> usize {
        (self.offset / 8) as usize
    }

    /// Returns the byte index of the end of the binary data.
    fn end_byte(&self) -> usize {
        ((self.offset + self.length + 7) / 8) as usize
    }
    
    /// Return copy with a new offset (< 8). Any excess bytes will be trimmed.
    fn copy_with_new_offset(&self, new_offset: u64) -> Self {
        assert!(new_offset < 8);
        // Create a new Bits object with the same value but a different offset.
        // Each byte will in general have to be bit shifted to the left or right.
        if self.length == 0 {
            return Bits {
                data: Arc::new(vec![]),
                offset: 0,
                length: 0,
            }
        }
        let byte_offset = (self.offset / 8) as usize;
        let bit_offset = self.offset % 8;
        if new_offset == bit_offset {
            return Bits {
                data: Arc::new(self.data[self.start_byte()..self.end_byte()].to_vec()),
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
            data: Arc::new(new_data),
            offset: new_offset,
            length: self.length,
        }
    }
    
    /// Slice used internally without bounds checking.
    fn slice(&self, start_bit: u64, end_bit: u64) -> Self {
        assert!(start_bit <= end_bit);
        assert!(end_bit <= self.length);
        let new_length = end_bit - start_bit;
        Bits {
            data: Arc::clone(&self.data),
            offset: start_bit + self.offset,
            length: new_length,
        }
    }

    // Return a new Bits with any excess stored bytes trimmed.
    pub fn trim(&self) -> Self {
        if self.offset < 8 && self.end_byte() == self.data.len() {
            return Bits {
                data: Arc::clone(&self.data),
                offset: self.offset,
                length: self.length,
            }
        }
        Bits {
            data: Arc::new(self.data[self.start_byte()..self.end_byte()].to_vec()),
            offset: self.offset % 8,
            length: self.length,
        }
    }
    // I think this works as a Rust version. Keeping this copy for reference.
    pub fn find_all_rust<'a>(&'a self, b: &'a Bits, bytealigned: bool) -> impl Iterator<Item = u64> + 'a {
        // Use the find fn to find all instances of b in self and return as an iterator
        let mut start: u64 = 0;
        std::iter::from_fn(move || {
            let found = self.slice(start, self.length).find(b, bytealigned);
            match found {
                Some(x) => {
                    start = x + 1;
                    Some(x)
                }
                None => None,
            }
        })
    }

}

#[pymethods]
impl Bits {
    
    #[staticmethod]
    pub fn from_zeros(length: u64) -> Self {
        Bits {
            data: Arc::new(vec![0; ((length + 7) / 8) as usize]),
            offset: 0,
            length,
        }
    }

    #[staticmethod]
    pub fn from_ones(length: u64) -> Self {
        Bits {
            data: Arc::new(vec![0xff; ((length + 7) / 8) as usize]),
            offset: 0,
            length,
        }
    }

    #[staticmethod]
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let bitlength = (data.len() as u64) * 8;
        Bits {
            data: Arc::new(data),
            offset: 0,
            length: bitlength,
        }
    }

    #[staticmethod]
    pub fn from_bin(binary_string: &str) -> PyResult<Self> {
        let mut data: Vec<u8> = Vec::new();
        let mut byte: u8 = 0;
        for chunk in binary_string.as_bytes().chunks(8) {
            for (i, &c) in chunk.iter().enumerate() {
                if c == b'1' {
                    byte |= 1 << (7 - i);
                } else if c != b'0' {
                    return Err(PyValueError::new_err("Invalid character"));
                }
            }
            data.push(byte);
            byte = 0;
        }
        Ok(Bits {
            data: Arc::new(data),
            offset: 0,
            length: binary_string.len() as u64,
        })
    }

    #[staticmethod]
    pub fn from_hex(hex: &str) -> PyResult<Self> {
        let mut new_hex = hex.to_string();
        let is_odd_length: bool = hex.len() % 2 != 0;
        if is_odd_length {
            new_hex.push('0');
        }
        let data = match hex::decode(new_hex) {
            Ok(d) => d,
            Err(_) => return Err(PyValueError::new_err("Invalid character")),
        };
        Ok(Bits {
            data: Arc::new(data),
            offset: 0,
            length: hex.len() as u64 * 4,
        })
    }

    #[staticmethod]
    pub fn from_oct(oct: &str) -> PyResult<Self> {
        let mut bin_str = String::new();
        for ch in oct.chars() {
            // Convert each ch to an integer
            let digit = match ch.to_digit(8) {
                Some(d) => d,
                None => return Err(PyValueError::new_err("Invalid character")),
            };
            bin_str.push_str(&format!("{:03b}", digit)); // Format as 3-bit binary
        }
        Ok(Bits::from_bin(&bin_str).unwrap())
    }
    
    // Don't need - rewrite Python to convert int to bytes
    // pub fn from_int
    
    #[staticmethod]
    pub fn join(bits_vec: Vec<PyRef<Bits>>) -> Bits {
        if bits_vec.is_empty() {
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
            data: Arc::new(data),
            offset: new_offset,
            length: new_length,
        }
    }
    
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO offset!
        self.data[self.start_byte()..self.end_byte()].to_vec()
    }
    
    // Don't need - rewrite Python to convert bytes to int
    // pub fn to uint(&self) 
    // pub fn to_int(&self)

    pub fn to_hex(&self) -> PyResult<String> {
        if self.length % 4 != 0 {
            return Err(PyValueError::new_err("Not a multiple of 4 bits long."));
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

    pub fn to_bin(&self) -> String {
        let x = self.data.iter()
            .map(|byte| format!("{:08b}", byte))
            .fold(String::new(), |mut bin_str, bin| {
                bin_str.push_str(&bin);
                bin_str
            });
        x[self.offset as usize..(self.offset + self.length) as usize].to_string()
    }

    pub fn to_oct(&self) -> PyResult<String> {
        if self.length % 3 != 0 {
            return Err(PyValueError::new_err("Not a multiple of 3 bits long."));
        }
        let bin_str = self.to_bin();
        let mut oct_str: String = String::new();

        for chunk in bin_str.as_bytes().chunks(3) {
            let binary_chunk = std::str::from_utf8(chunk).unwrap();
            let value = u8::from_str_radix(binary_chunk, 2).unwrap();
            oct_str.push(std::char::from_digit(value as u32, 8).unwrap());
        }
        Ok(oct_str)
    }

    pub fn and_(&self, other: &Bits) -> PyResult<Bits> {
        match self.bitwise_op(other, |a, b| a & b) {
            Ok(b) => Ok(b),
            Err(_) => Err(PyValueError::new_err("Lengths do not match.")),
        }
    }
    pub fn or_(&self, other: &Bits) -> PyResult<Bits> {
        match self.bitwise_op(other, |a, b| a | b) {
            Ok(b) => Ok(b),
            Err(_) => Err(PyValueError::new_err("Lengths do not match.")),
        }
    }
    pub fn xor_(&self, other: &Bits) -> PyResult<Bits> {
        match self.bitwise_op(other, |a, b| a ^ b) {
            Ok(b) => Ok(b),
            Err(_) => Err(PyValueError::new_err("Lengths do not match.")),
        }
    }
    
    pub fn find(&self, b: &Bits, bytealigned: bool) -> Option<u64> {
        if b.length > self.length {
            return None;
        }
        let step = if bytealigned { 8 } else { 1 };
        (0..self.length - b.length).step_by(step).find(|&sb| self.slice(sb, sb + b.length) == *b)
    }
    
    
    // pub fn rfind(&self, b: &Bits, bytealigned: bool) -> Option<u64> {}

    // pub fn find_all<'a>(&'a self, b: &'a Bits, bytealigned: bool) -> impl Iterator<Item = u64> + 'a {
    //     // Use the find fn to find all instances of b in self and return as an iterator
    //     let mut start: u64 = 0;
    //     std::iter::from_fn(move || {
    //         let found = self.slice(start, self.length).find(b, bytealigned);
    //         match found {
    //             Some(x) => {
    //                 start = x + 1;
    //                 Some(x)
    //             }
    //             None => None,
    //         }
    //     })
    // }

    // pub fn rfind_all(&self, b: &Bits, bytealigned: bool) -> iter<u64> {}

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
            data: Arc::new(data),
            offset: new_offset,
            length: self.length,
        }
    }
    
    // TODO
    // pub fn iter(&self) -> iter<bool> {}

    // TODO: rename to getindex ? 
    pub fn index(&self, bit_index: u64) -> PyResult<bool> {
        if bit_index >= self.length {
            return Err(PyIndexError::new_err("Out of range."));
        }
        let p: u64 = bit_index + self.offset;
        let byte = self.data[(p / 8) as usize];
        Ok(byte & (128 >> (p % 8)) != 0)
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

    pub fn getslice(&self, start_bit: u64, end_bit: u64) -> PyResult<Self> {
        assert!(start_bit <= end_bit);
        if end_bit > self.length {
            return Err(PyValueError::new_err("end bit goes past the end"));
        }
        let new_length = end_bit - start_bit;
        Ok(Bits {
            data: Arc::clone(&self.data),
            offset: start_bit + self.offset,
            length: new_length,
        })
    }

    pub fn invert(&self) -> Self {
        let mut data: Vec<u8> = Vec::new();
        for byte in self.data[self.start_byte()..self.end_byte()].iter() {
            data.push(byte ^ 0xff);
        }
        Bits {
            data: Arc::new(data),
            offset: self.offset,
            length: self.length,
        }
    }

}


// #[test]
// fn new1() {
//     let data: Vec<u8> = vec![10, 20, 30];
//     let bits = Bits::new(data, 0, 24).unwrap();
//     assert_eq!(*bits.data(), vec![10, 20, 30]);
//     assert_eq!(bits.offset(), 0);
//     assert_eq!(bits.length(), 24);
// }

// #[test]
// fn new2() {
//     let bits = Bits::new(vec![], 0, 0).unwrap();
//     assert_eq!(bits.length(), 0);
// }

#[test]
fn from_bytes() {
    let data: Vec<u8> = vec![10, 20, 30];
    let bits = Bits::from_bytes(data);
    assert_eq!(*bits.data(), vec![10, 20, 30]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 24);
}

#[test]
fn from_hex() {
    let bits = Bits::from_hex("0a141e").unwrap();
    assert_eq!(*bits.data(), vec![10, 20, 30]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 24);
    let bits = Bits::from_hex("").unwrap();
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 0);
    let bits = Bits::from_hex("hello");
    assert!(bits.is_err());
    let bits = Bits::from_hex("1").unwrap();
    assert_eq!(*bits.data(), vec![16]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 4);
}

#[test]
fn from_bin() {
    let bits = Bits::from_bin("00001010").unwrap();
    assert_eq!(*bits.data(), vec![10]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 8);
    let bits = Bits::from_bin("").unwrap();
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 0);
    let bits = Bits::from_bin("hello");
    assert!(bits.is_err());
    let bits = Bits::from_bin("1").unwrap();
    assert_eq!(*bits.data(), vec![128]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 1);
}

#[test]
fn from_zeros() {
    let bits = Bits::from_zeros(8);
    assert_eq!(*bits.data(), vec![0]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 8);
    assert_eq!(bits.to_hex().unwrap(), "00");
    let bits = Bits::from_zeros(9);
    assert_eq!(*bits.data(), vec![0, 0]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 9);
    let bits = Bits::from_zeros(0);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 0);
}

#[test]
fn from_ones() {
    let bits = Bits::from_ones(8);
    assert_eq!(*bits.data(), vec![255]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 8);
    assert_eq!(bits.to_hex().unwrap(), "ff");
    let bits = Bits::from_ones(9);
    assert_eq!(bits.to_bin(), "111111111");
    assert!(bits.to_hex().is_err());
    assert_eq!((*bits.data())[0], 0xff);
    assert_eq!((*bits.data())[1] & 0x80, 0x80);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 9);
    let bits = Bits::from_ones(0);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 0);
}

#[test]
fn get_index() {
    let bits = Bits::from_bin("001100").unwrap();
    assert_eq!(bits.index(0).unwrap(), false);
    assert_eq!(bits.index(1).unwrap(), false);
    assert_eq!(bits.index(2).unwrap(), true);
    assert_eq!(bits.index(3).unwrap(), true);
    assert_eq!(bits.index(4).unwrap(), false);
    assert_eq!(bits.index(5).unwrap(), false);
    assert!(bits.index(6).is_err());
    assert!(bits.index(60).is_err());
}

// #[test]
// fn join_whole_byte() {
//     let b1 = Bits::new(vec![5, 10, 20], 0, 24).unwrap();
//     let b2 = Bits::from_bytes(vec![30, 40, 50]);
//     let j = Bits::join(&vec![&b1, &b2, &b1]);
//     assert_eq!(*j.data(), vec![5, 10, 20, 30, 40, 50, 5, 10, 20]);
//     assert_eq!(j.offset(), 0);
//     assert_eq!(j.length(), 72);
// }

// #[test]
// fn join_single_bits() {
//     let b1 = Bits::from_bin("1").unwrap();
//     let b2 = Bits::from_bin("0").unwrap();
//     let bits = Bits::join(&vec![&b1, &b2, &b1]);
//     assert_eq!(bits.offset(), 0);
//     assert_eq!(bits.length(), 3);
//     assert_eq!(*bits.data(), vec![0b10100000]);
//     let b3 = Bits::from_bin("11111111").unwrap();
//     let j = Bits::join(&vec![&b2, &b3]);
//     assert_eq!(j.offset(), 0);
//     assert_eq!(j.length(), 9);
//     assert_eq!(*j.data(), vec![0b01111111, 0b10000000]);
//     let j = Bits::join(&vec![&b3, &b2, &b3]);
//     assert_eq!(j.offset(), 0);
//     assert_eq!(j.length(), 17);
//     assert_eq!(j, Bits::from_bin("11111111011111111").unwrap());
// }

#[test]
fn hex_edge_cases() {
    let b1 = Bits::from_hex("0123456789abcdef").unwrap();
    let b2 = b1.getslice(12, b1.length()).unwrap();
    assert_eq!(b2.to_hex().unwrap(), "3456789abcdef");
    assert_eq!(b2.offset(), 12);
    assert_eq!(b2.length(), 52);
    assert_eq!(b2.data().len(), 8);
    let bp = b2.trim();
    assert_eq!(bp, b2);
    assert_eq!(bp.offset(), 4);
    assert_eq!(bp.length(), 52);
    assert_eq!(bp.data().len(), 7);

    // let b2 = Bits::new(vec![0x01, 0x23, 0x45, 0x67], 12, 12).unwrap();
    // assert_eq!(b2.to_hex().unwrap(), "345");
}

// #[test]
// fn a_few_things() {
//     let b1 = Bits::from_hex("abcdef").unwrap();
//     let b2 = Bits::from_bin("01").unwrap();
//     let b4 = Bits::join(&vec![&b1, &b2]).trim();
//     assert_eq!(b4.length(), 26);
//     assert_eq!(b4.to_bin(), "10101011110011011110111101");
//     let b5 = Bits::join(&vec![&b1, &b1]);
//     assert_eq!(b5.length(), 48);
//     assert_eq!(b5.to_hex().unwrap(), "abcdefabcdef");
//     let b6 = Bits::join(&vec![&b2, &b2, &b1]);
//     assert_eq!(b6.length(), 28);
//     assert_eq!(b6.to_bin(), "0101101010111100110111101111");
//     assert_eq!(b6.to_hex().unwrap(), "5abcdef");
//     let b3 = Bits::join(&vec![&b1, &b2, &b1, &b2]);
//     assert_eq!(b3.length(), 52);
//     assert_eq!(b3.to_hex().unwrap(), "abcdef6af37bd");
//     // assert_eq!(b3.get_slice(Some(b1.get_length() + 2), Some(b3.get_length() - 2)).unwrap().to_hex().unwrap(), "abcdef");
// }

#[test]
fn test_count() {
    let x = vec![1, 2, 3];
    let b = Bits::from_bytes(x);
    assert_eq!(b.count_ones(), 4);
    assert_eq!(b.count_zeros(), 20);
}

#[test]
fn test_reverse() {
    let b = Bits::from_bin("11110000").unwrap();
    let bp = b.reverse();
    assert_eq!(bp.to_bin(), "00001111");
    let b = Bits::from_bin("1").unwrap();
    let bp = b.reverse();
    assert_eq!(bp.to_bin(), "1");
    let empty = Bits::from_bin("").unwrap();
    let empty_p = empty.reverse();
    assert_eq!(empty_p.to_bin(), "");
    let b = Bits::from_bin("11001").unwrap();
    let bp = b.reverse();
    assert_eq!(bp.to_bin(), "10011");
    let hex_str = "98798379287592836521000cbdbeff";
    let long = Bits::from_hex(hex_str).unwrap();
    let rev = long.reverse();
    assert_eq!(rev.reverse(), long);
}

#[test]
fn test_invert() {
    let b = Bits::from_bin("0").unwrap();
    assert_eq!(b.invert().to_bin(), "1");
    let b = Bits::from_bin("01110").unwrap();
    assert_eq!(b.invert().to_bin(), "10001");
    let hex_str = "abcdef8716258765162548716258176253172635712654714";
    let long = Bits::from_hex(hex_str).unwrap();
    let temp = long.invert();
    assert_eq!(long.length(), temp.length());
    assert_eq!(temp.invert(), long);
}

// #[test]
// fn test_join_again() {
//     let b1 = Bits::from_hex("0123456789").unwrap();
//     let b2 = b1.slice(12, 24);
//     let b3 = Bits::join(&vec![&b2, &b2]);
//     assert_eq!(b3.to_hex().unwrap(), "345345");
//     let b3 = Bits::join(&vec![&b2, &b2, &b1]);
//     assert_eq!(b3.to_hex().unwrap(), "3453450123456789");
// }

#[test]
fn test_find() {
    let b1 = Bits::from_zeros(10);
    let b2 = Bits::from_ones(2);
    assert_eq!(b1.find(&b2, false), None);
    let b3 = Bits::from_bin("00001110").unwrap();
    let b4 = Bits::from_bin("01").unwrap();
    assert_eq!(b3.find(&b4, false), Some(3));
    assert_eq!(b3.slice(2, b3.length()).find(&b4, false), Some(1));
}

Add#[test]
fn test_and() {
    let a1 = Bits::from_hex("f0f").unwrap();
    let a2 = Bits::from_hex("123").unwrap();
    let a3 = a1.and_(&a2).unwrap();
    assert_eq!(a3, Bits::from_hex("103").unwrap());
}