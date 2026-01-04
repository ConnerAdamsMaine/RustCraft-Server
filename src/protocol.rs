use std::io::{Cursor, Read};

use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

/// Minecraft protocol packet structure
pub struct Packet {
    pub id:   i32,
    pub data: Bytes,
}

impl Packet {
    pub fn new(id: i32, data: Bytes) -> Self {
        Self { id, data }
    }
}

/// Read a Minecraft varint from bytes
pub fn read_varint(cursor: &mut Cursor<&[u8]>) -> std::io::Result<i32> {
    let mut result = 0;
    let mut bytes_read = 0;
    let mut byte = [0u8; 1];

    loop {
        cursor.read_exact(&mut byte)?;
        let b = byte[0];
        result |= ((b & 0x7F) as i32) << (7 * bytes_read);

        if (b & 0x80) == 0 {
            break;
        }

        bytes_read += 1;
        if bytes_read >= 5 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "VarInt is too big"));
        }
    }

    Ok(result)
}

/// Write a Minecraft varint to bytes
pub fn write_varint(value: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut v = value as u32;

    loop {
        let mut temp = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            temp |= 0x80;
        }
        result.push(temp);
        if v == 0 {
            break;
        }
    }

    result
}

pub struct PacketWriter {
    data: BytesMut,
}

impl Default for PacketWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub trait ByteWritable {
    fn write_varint<N: Into<i32>>(&mut self, value: N);

    fn write_string<S: AsRef<str>>(&mut self, s: S);

    fn write_byte<N: Into<u8>>(&mut self, value: N);

    fn write_short<N: Into<i16>>(&mut self, value: N);

    fn write_int<N: Into<i32>>(&mut self, value: N);

    fn write_long<N: Into<i64>>(&mut self, value: N);

    fn write_float<N: Into<f32>>(&mut self, value: N);

    fn write_double<N: Into<f64>>(&mut self, value: N);

    fn write_bool<B: Into<bool>>(&mut self, value: B);

    // TODO: @check : check the constraints on how we want to do this -
    //  May want something like: AsRef + AsBytes or something else
    fn write_uuid<U: AsRef<Uuid>>(&mut self, uuid: U);

    fn write_bytes<A: AsRef<[u8]>>(&mut self, bytes: A);

    fn finish(self) -> BytesMut;
}

impl ByteWritable for PacketWriter {
    fn write_varint<N: Into<i32>>(&mut self, value: N) {
        self.data.extend_from_slice(&write_varint(value.into()))
    }

    fn write_string<S: AsRef<str>>(&mut self, s: S) {
        let bytes = s.as_ref().as_bytes();
        self.write_varint(bytes.len() as i32);
        self.data.extend_from_slice(bytes);
    }

    fn write_byte<N: Into<u8>>(&mut self, value: N) {
        self.data.put_u8(value.into());
    }

    fn write_short<N: Into<i16>>(&mut self, value: N) {
        self.data.put_i16_le(value.into());
    }

    fn write_int<N: Into<i32>>(&mut self, value: N) {
        self.data.put_i32_le(value.into());
    }

    fn write_long<N: Into<i64>>(&mut self, value: N) {
        self.data.put_i64_le(value.into());
    }

    fn write_float<N: Into<f32>>(&mut self, value: N) {
        self.data.put_f32_le(value.into());
    }

    fn write_double<N: Into<f64>>(&mut self, value: N) {
        self.data.put_f64_le(value.into());
    }

    fn write_bool<B: Into<bool>>(&mut self, value: B) {
        self.data.put_u8(if value.into() { 1 } else { 0 });
    }

    fn write_uuid<U: AsRef<Uuid>>(&mut self, uuid: U) {
        self.data.extend_from_slice(uuid.as_ref().as_bytes());
    }

    fn write_bytes<A: AsRef<[u8]>>(&mut self, bytes: A) {
        self.data.extend_from_slice(bytes.as_ref());
    }

    fn finish(self) -> BytesMut {
        self.data
    }
}

impl PacketWriter {
    pub fn new() -> Self {
        Self {
            data: BytesMut::new(),
        }
    }
}

pub struct PacketReader<'a> {
    cursor: Cursor<&'a [u8]>,
}

impl<'a> PacketReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            cursor: Cursor::new(data),
        }
    }

    pub fn read_varint(&mut self) -> std::io::Result<i32> {
        read_varint(&mut self.cursor)
    }

    pub fn read_string(&mut self) -> std::io::Result<String> {
        let len = self.read_varint()? as usize;
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    pub fn read_byte(&mut self) -> std::io::Result<u8> {
        let mut buf = [0u8; 1];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_short(&mut self) -> std::io::Result<i16> {
        let mut buf = [0u8; 2];
        self.cursor.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    pub fn read_int(&mut self) -> std::io::Result<i32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    pub fn read_long(&mut self) -> std::io::Result<i64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(i64::from_le_bytes(buf))
    }

    pub fn read_float(&mut self) -> std::io::Result<f32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    pub fn read_double(&mut self) -> std::io::Result<f64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(f64::from_le_bytes(buf))
    }

    pub fn read_bool(&mut self) -> std::io::Result<bool> {
        Ok(self.read_byte()? != 0)
    }

    pub fn read_uuid(&mut self) -> std::io::Result<Uuid> {
        let mut buf = [0u8; 16];
        self.cursor.read_exact(&mut buf)?;
        Ok(Uuid::from_bytes(buf))
    }

    pub fn read_bytes(&mut self, len: usize) -> std::io::Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.cursor.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn remaining(&self) -> usize {
        let pos = self.cursor.position() as usize;
        self.cursor.get_ref().len() - pos
    }
}
