
from bit_rust import Bits
import pytest

def test_creation():
    b = Bits.from_zeros(10)
    assert b.length() == 10
    assert b.to_bin() == '0000000000'

    b2 = Bits.from_ones(8)
    assert b2.to_bin() == '11111111'
    assert b2.to_hex() == 'ff'

def test_creation_from_bytes():
    b3 = Bits.from_bytes(b'hello')
    assert b3.to_hex() == '68656c6c6f'
    assert b3.to_bytes() == b'hello'
    b4 = b3.getslice(8, 40)
    assert b4.to_hex() == '656c6c6f'
    assert b4.to_bytes() == b'ello'

def test_join():
    a = Bits.from_zeros(4)
    b = Bits.from_ones(4)
    c = Bits.join([a, b])
    assert c.to_bin() == '00001111'
    d = c.reverse()
    assert d.to_bin() == '11110000'
    e = c.and_(d)
    assert e.to_bin() == '00000000'

def test_find():
    a = Bits.from_bin('00000110001110')
    b = Bits.from_bin('11')
    assert a.find(b, False) == 5
    assert a.find(b, True) is None

def test_from_oct():
    a = Bits.from_oct('776')
    assert a.to_bin() == '111111110'
    with pytest.raises(ValueError):
        b = Bits.from_oct('abc')
    assert a.to_oct() == "776"