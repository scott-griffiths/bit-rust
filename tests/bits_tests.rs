use bit_rust::Bits;

#[test]
fn new1() {
    let data: Vec<u8> = vec![10, 20, 30];
    let bits = Bits::new(data, 0, 24).unwrap();
    assert_eq!(*bits.get_data(), vec![10, 20, 30]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 24);
}

#[test]
fn new2() {
    let bits = Bits::new(vec![], 0, 0).unwrap();
    assert_eq!(bits.get_length(), 0);
}

#[test]
fn from_bytes() {
    let data: Vec<u8> = vec![10, 20, 30];
    let bits = Bits::from_bytes(data);
    assert_eq!(*bits.get_data(), vec![10, 20, 30]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 24);
}

#[test]
fn from_hex() {
    let bits = Bits::from_hex("0a141e").unwrap();
    assert_eq!(*bits.get_data(), vec![10, 20, 30]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 24);
    let bits = Bits::from_hex("").unwrap();
    assert_eq!(*bits.get_data(), vec![]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 0);
    let bits = Bits::from_hex("hello");
    assert!(bits.is_err());
    let bits = Bits::from_hex("1").unwrap();
    assert_eq!(*bits.get_data(), vec![16]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 4);
}

#[test]
fn from_bin() {
    let bits = Bits::from_bin("00001010").unwrap();
    assert_eq!(*bits.get_data(), vec![10]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 8);
    let bits = Bits::from_bin("").unwrap();
    assert_eq!(*bits.get_data(), vec![]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 0);
    let bits = Bits::from_bin("hello");
    assert!(bits.is_err());
    let bits = Bits::from_bin("1").unwrap();
    assert_eq!(*bits.get_data(), vec![128]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 1);
}

#[test]
fn from_zeros() {
    let bits = Bits::from_zeros(8);
    assert_eq!(*bits.get_data(), vec![0]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 8);
    assert_eq!(bits.to_hex().unwrap(), "00");
    let bits = Bits::from_zeros(9);
    assert_eq!(*bits.get_data(), vec![0, 0]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 9);
    let bits = Bits::from_zeros(0);
    assert_eq!(*bits.get_data(), vec![]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 0);
}

#[test]
fn from_ones() {
    let bits = Bits::from_ones(8);
    assert_eq!(*bits.get_data(), vec![255]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 8);
    assert_eq!(bits.to_hex().unwrap(), "ff");
    let bits = Bits::from_ones(9);
    assert_eq!(bits.to_bin(), "111111111");
    assert!(bits.to_hex().is_err());
    assert_eq!((*bits.get_data())[0], 0xff);
    assert_eq!((*bits.get_data())[1] & 0x80, 0x80);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 9);
    let bits = Bits::from_ones(0);
    assert_eq!(*bits.get_data(), vec![]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 0);
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
fn join_whole_byte() {
    let b1 = Bits::new(vec![5, 10, 20], 0, 24).unwrap();
    let b2 = Bits::from_bytes(vec![30, 40, 50]);
    let j = Bits::join(&vec![&b1, &b2, &b1]);
    assert_eq!(*j.get_data(), vec![5, 10, 20, 30, 40, 50, 5, 10, 20]);
    assert_eq!(j.get_offset(), 0);
    assert_eq!(j.get_length(), 72);
}

#[test]
fn join_single_bits() {
    let b1 = Bits::from_bin("1").unwrap();
    let b2 = Bits::from_bin("0").unwrap();
    let bits = Bits::join(&vec![&b1, &b2, &b1]);
    assert_eq!(bits.get_offset(), 0);
    assert_eq!(bits.get_length(), 3);
    assert_eq!(*bits.get_data(), vec![0b10100000]);
    let b3 = Bits::from_bin("11111111").unwrap();
    let j = Bits::join(&vec![&b2, &b3]);
    assert_eq!(j.get_offset(), 0);
    assert_eq!(j.get_length(), 9);
    assert_eq!(*j.get_data(), vec![0b01111111, 0b10000000]);
    let j = Bits::join(&vec![&b3, &b2, &b3]);
    assert_eq!(j.get_offset(), 0);
    assert_eq!(j.get_length(), 17);
    assert_eq!(j, Bits::from_bin("11111111011111111").unwrap());
}

#[test]
fn hex_edge_cases() {
    let b1 = Bits::from_hex("0123456789abcdef").unwrap();
    let b2 = b1.get_slice(Some(12), None).unwrap();
    assert_eq!(b2.to_hex().unwrap(), "3456789abcdef");

    let b2 = Bits::new(vec![0x01, 0x23, 0x45, 0x67], 12, 12).unwrap();
    assert_eq!(b2.to_hex().unwrap(), "345");
}

#[test]
fn a_few_things() {
    let b1 = Bits::from_hex("abcdef").unwrap();
    let b2 = Bits::from_bin("01").unwrap();
    let b3 = Bits::join(&vec![&b1, &b2, &b1, &b2]);
    assert_eq!(b3.to_hex().unwrap(), "abcdef6af37bd");
    assert_eq!(b3.get_slice(Some(b1.get_length() + 2), Some(b3.get_length() - 2)).unwrap().to_hex().unwrap(), "abcdef");
}