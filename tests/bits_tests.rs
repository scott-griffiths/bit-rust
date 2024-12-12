use bit_rust::bits::Bits;

#[test]
fn new1() {
    let data: Vec<u8> = vec![10, 20, 30];
    let bits = Bits::new(data, 0, 24).unwrap();
    assert_eq!(*bits.data(), vec![10, 20, 30]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 24);
}

#[test]
fn new2() {
    let bits = Bits::new(vec![], 0, 0).unwrap();
    assert_eq!(bits.length(), 0);
}

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
    assert_eq!(*bits.data(), vec![]);
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
    assert_eq!(*bits.data(), vec![]);
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
    assert_eq!(*bits.data(), vec![]);
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
    assert_eq!(*bits.data(), vec![]);
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

#[test]
fn join_whole_byte() {
    let b1 = Bits::new(vec![5, 10, 20], 0, 24).unwrap();
    let b2 = Bits::from_bytes(vec![30, 40, 50]);
    let j = Bits::join(&vec![&b1, &b2, &b1]);
    assert_eq!(*j.data(), vec![5, 10, 20, 30, 40, 50, 5, 10, 20]);
    assert_eq!(j.offset(), 0);
    assert_eq!(j.length(), 72);
}

#[test]
fn join_single_bits() {
    let b1 = Bits::from_bin("1").unwrap();
    let b2 = Bits::from_bin("0").unwrap();
    let bits = Bits::join(&vec![&b1, &b2, &b1]);
    assert_eq!(bits.offset(), 0);
    assert_eq!(bits.length(), 3);
    assert_eq!(*bits.data(), vec![0b10100000]);
    let b3 = Bits::from_bin("11111111").unwrap();
    let j = Bits::join(&vec![&b2, &b3]);
    assert_eq!(j.offset(), 0);
    assert_eq!(j.length(), 9);
    assert_eq!(*j.data(), vec![0b01111111, 0b10000000]);
    let j = Bits::join(&vec![&b3, &b2, &b3]);
    assert_eq!(j.offset(), 0);
    assert_eq!(j.length(), 17);
    assert_eq!(j, Bits::from_bin("11111111011111111").unwrap());
}

#[test]
fn hex_edge_cases() {
    let b1 = Bits::from_hex("0123456789abcdef").unwrap();
    let b2 = b1.slice(12, b1.length());
    assert_eq!(b2.to_hex().unwrap(), "3456789abcdef");
    assert_eq!(b2.offset(), 12);
    assert_eq!(b2.length(), 52);
    assert_eq!(b2.data().len(), 8);
    let bp = b2.trim();
    assert_eq!(bp, b2);
    assert_eq!(bp.offset(), 4);
    assert_eq!(bp.length(), 52);
    assert_eq!(bp.data().len(), 7);

    let b2 = Bits::new(vec![0x01, 0x23, 0x45, 0x67], 12, 12).unwrap();
    assert_eq!(b2.to_hex().unwrap(), "345");
}

#[test]
fn a_few_things() {
    let b1 = Bits::from_hex("abcdef").unwrap();
    let b2 = Bits::from_bin("01").unwrap();
    let b4 = Bits::join(&vec![&b1, &b2]).trim();
    assert_eq!(b4.length(), 26);
    assert_eq!(b4.to_bin(), "10101011110011011110111101");
    let b5 = Bits::join(&vec![&b1, &b1]);
    assert_eq!(b5.length(), 48);
    assert_eq!(b5.to_hex().unwrap(), "abcdefabcdef");
    let b6 = Bits::join(&vec![&b2, &b2, &b1]);
    assert_eq!(b6.length(), 28);
    assert_eq!(b6.to_bin(), "0101101010111100110111101111");
    assert_eq!(b6.to_hex().unwrap(), "5abcdef");
    let b3 = Bits::join(&vec![&b1, &b2, &b1, &b2]);
    assert_eq!(b3.length(), 52);
    assert_eq!(b3.to_hex().unwrap(), "abcdef6af37bd");
    // assert_eq!(b3.get_slice(Some(b1.get_length() + 2), Some(b3.get_length() - 2)).unwrap().to_hex().unwrap(), "abcdef");
}

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

#[test]
fn test_join_again() {
    let b1 = Bits::from_hex("0123456789").unwrap();
    let b2 = b1.slice(12, 24);
    let b3 = Bits::join(&vec![&b2, &b2]);
    assert_eq!(b3.to_hex().unwrap(), "345345");
    let b3 = Bits::join(&vec![&b2, &b2, &b1]);
    assert_eq!(b3.to_hex().unwrap(), "3453450123456789");
}

#[test]
fn test_find() {
    let b1 = Bits::from_zeros(10);
    let b2 = Bits::from_ones(2);
    assert_eq!(b1.find(&b2), None);
    let b3 = Bits::from_bin("00001110").unwrap();
    let b4 = Bits::from_bin("01").unwrap();
    assert_eq!(b3.find(&b4), Some(3));
    assert_eq!(b3.slice(2, b3.length()).find(&b4), Some(1));
}

#[test]
fn test_and() {
    let a1 = Bits::from_hex("f0f").unwrap();
    let a2 = Bits::from_hex("123").unwrap();
    let a3 = a1.and(&a2).unwrap();
    assert_eq!(a3, Bits::from_hex("103").unwrap());
}