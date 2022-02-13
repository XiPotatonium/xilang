use std::iter::Peekable;
use std::mem::transmute;

pub trait IDeserializer {
    fn peek_byte(&mut self) -> u8;
    fn take_byte(&mut self) -> u8;
    fn take_bytes2(&mut self) -> [u8; 2];
    fn take_bytes4(&mut self) -> [u8; 4];
    fn take_bytes(&mut self, n: u32) -> Vec<u8>;
}

pub struct Deserializer<I: Iterator<Item = u8>> {
    stream: Peekable<I>,
    pub bytes_taken: u32,
}

impl<I: Iterator<Item = u8>> Deserializer<I> {
    pub fn new(stream: Peekable<I>) -> Deserializer<I> {
        Deserializer {
            stream,
            bytes_taken: 0,
        }
    }
}

impl<I: Iterator<Item = u8>> IDeserializer for Deserializer<I> {
    fn peek_byte(&mut self) -> u8 {
        *self.stream.peek().unwrap()
    }

    fn take_byte(&mut self) -> u8 {
        self.bytes_taken += 1;
        (&mut self.stream).next().unwrap()
    }

    fn take_bytes2(&mut self) -> [u8; 2] {
        self.bytes_taken += 2;
        let b1 = (&mut self.stream).next().unwrap();
        let b2 = (&mut self.stream).next().unwrap();

        [b1, b2]
    }

    fn take_bytes4(&mut self) -> [u8; 4] {
        self.bytes_taken += 4;
        let b1 = (&mut self.stream).next().unwrap();
        let b2 = (&mut self.stream).next().unwrap();
        let b3 = (&mut self.stream).next().unwrap();
        let b4 = (&mut self.stream).next().unwrap();

        [b1, b2, b3, b4]
    }

    fn take_bytes(&mut self, n: u32) -> Vec<u8> {
        self.bytes_taken += n;
        (&mut self.stream).take(n as usize).collect()
    }
}

pub trait ISerializable {
    fn serialize(&self, buf: &mut Vec<u8>);
    fn deserialize(buf: &mut dyn IDeserializer) -> Self;
}

impl ISerializable for u8 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push(*self)
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> u8 {
        buf.take_byte()
    }
}

impl ISerializable for u16 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push((self >> 8) as u8);
        buf.push(*self as u8);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> u16 {
        let v = buf.take_bytes2();
        ((v[0] as u16) << 8) + (v[1] as u16)
    }
}

impl ISerializable for u32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        buf.push((self >> 24) as u8);
        buf.push((self >> 16) as u8);
        buf.push((self >> 8) as u8);
        buf.push(*self as u8);
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> u32 {
        let v = buf.take_bytes4();
        ((v[0] as u32) << 24) + ((v[1] as u32) << 16) + ((v[2] as u32) << 8) + (v[3] as u32)
    }
}

impl ISerializable for i8 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        unsafe { buf.push(transmute(*self)) }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> i8 {
        unsafe { transmute(buf.take_byte()) }
    }
}

impl ISerializable for i16 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let bytes = self.to_be_bytes();
        for b in bytes.iter() {
            buf.push(*b);
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> i16 {
        let bytes = buf.take_bytes2();
        i16::from_be_bytes(bytes)
    }
}

impl ISerializable for i32 {
    fn serialize(&self, buf: &mut Vec<u8>) {
        let bytes = self.to_be_bytes();
        for b in bytes.iter() {
            buf.push(*b);
        }
    }

    fn deserialize(buf: &mut dyn IDeserializer) -> i32 {
        let bytes = buf.take_bytes4();
        i32::from_be_bytes(bytes)
    }
}
