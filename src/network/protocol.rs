use std::io::{Cursor, Read};

use anyhow::{anyhow, Result};
use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;

/// Validate a Minecraft identifier (resource location)
/// Ensures the identifier contains no null bytes and only valid characters
fn validate_identifier(id: &str) -> Result<()> {
    if id.contains('\0') {
        return Err(anyhow!("Identifier contains null byte: {:?}", id));
    }
    if !id.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | ':')) {
        return Err(anyhow!("Invalid identifier characters: {}", id));
    }
    Ok(())
}

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
        self.data.extend_from_slice(&value.into().to_be_bytes());
    }

    fn write_int<N: Into<i32>>(&mut self, value: N) {
        self.data.extend_from_slice(&value.into().to_be_bytes());
    }

    fn write_long<N: Into<i64>>(&mut self, value: N) {
        self.data.put_i64_ne(value.into());
    }

    fn write_float<N: Into<f32>>(&mut self, value: N) {
        self.data.extend_from_slice(&value.into().to_be_bytes());
    }

    fn write_double<N: Into<f64>>(&mut self, value: N) {
        self.data.put_f64_ne(value.into());
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
        Ok(i16::from_ne_bytes(buf))
    }

    pub fn read_int(&mut self) -> std::io::Result<i32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(i32::from_ne_bytes(buf))
    }

    pub fn read_long(&mut self) -> std::io::Result<i64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(i64::from_ne_bytes(buf))
    }

    pub fn read_float(&mut self) -> std::io::Result<f32> {
        let mut buf = [0u8; 4];
        self.cursor.read_exact(&mut buf)?;
        Ok(f32::from_ne_bytes(buf))
    }

    pub fn read_double(&mut self) -> std::io::Result<f64> {
        let mut buf = [0u8; 8];
        self.cursor.read_exact(&mut buf)?;
        Ok(f64::from_ne_bytes(buf))
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

// Helper functions for Prefixed Optional encoding
pub fn write_optional_bytes<A: AsRef<[u8]>>(writer: &mut PacketWriter, data: Option<A>) {
    match data {
        Some(bytes) => {
            let bytes_ref = bytes.as_ref();
            writer.write_varint(bytes_ref.len() as i32);
            writer.write_bytes(bytes_ref);
        }
        None => {
            writer.write_varint(-1); // -1 indicates no data
        }
    }
}

// Simple NBT encoder for registry data
pub struct NBTBuilder {
    data: BytesMut,
}

impl NBTBuilder {
    pub fn new() -> Self {
        Self {
            data: BytesMut::new(),
        }
    }

    /// Create an empty compound (root compound with no tags)
    pub fn empty_compound() -> Vec<u8> {
        vec![0x0A, 0x00, 0x00, 0x00] // TAG_Compound, empty name, TAG_End
    }

    /// Create a dimension type compound with minimal properties
    pub fn dimension_compound(
        name: &str,
        height: i32,
        min_y: i32,
        has_skylight: bool,
        has_ceiling: bool,
        ultrawarm: bool,
        natural: bool,
        coordinate_scale: f32,
    ) -> Vec<u8> {
        let mut bytes = BytesMut::new();

        // TAG_Compound
        bytes.put_u8(0x0A);

        // Root compound name (empty)
        bytes.put_i16(0);

        // bed_works: TAG_Byte = 1
        bytes.put_u8(0x01); // TAG_Byte
        bytes.extend_from_slice(b"\x00\x09bed_works"); // name length + name
        bytes.put_u8(if name.contains("nether") || name.contains("end") {
            0
        } else {
            1
        });

        // has_ceiling: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x0bhas_ceiling");
        bytes.put_u8(if has_ceiling { 1 } else { 0 });

        // has_skylight: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x0bhas_skylight");
        bytes.put_u8(if has_skylight { 1 } else { 0 });

        // has_raids: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x09has_raids");
        bytes.put_u8(if name.contains("end") { 0 } else { 1 });

        // height: TAG_Int
        bytes.put_u8(0x03);
        bytes.extend_from_slice(b"\x00\x06height");
        bytes.put_i32(height);

        // logical_height: TAG_Int
        bytes.put_u8(0x03);
        bytes.extend_from_slice(b"\x00\x0elogical_height");
        bytes.put_i32(height);

        // min_y: TAG_Int
        bytes.put_u8(0x03);
        bytes.extend_from_slice(b"\x00\x05min_y");
        bytes.put_i32(min_y);

        // ultrawarm: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x08ultrawarm");
        bytes.put_u8(if ultrawarm { 1 } else { 0 });

        // natural: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x07natural");
        bytes.put_u8(if natural { 1 } else { 0 });

        // coordinate_scale: TAG_Float
        bytes.put_u8(0x05);
        bytes.extend_from_slice(b"\x00\x10coordinate_scale");
        bytes.put_f32(coordinate_scale);

        // piglin_safe: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x0bpiglin_safe");
        bytes.put_u8(0);

        // respawn_anchor_works: TAG_Byte
        bytes.put_u8(0x01);
        bytes.extend_from_slice(b"\x00\x14respawn_anchor_works");
        bytes.put_u8(if name.contains("nether") { 1 } else { 0 });

        // TAG_End
        bytes.put_u8(0x00);

        bytes.to_vec()
    }

    /// Create a damage type compound
    pub fn damage_type_compound(message_id: &str, scaling: &str, exhaustion: f32) -> Vec<u8> {
        let mut bytes = BytesMut::new();
        
        // Validate identifiers before processing
        validate_identifier(message_id).expect("Invalid message_id identifier");
        validate_identifier(scaling).expect("Invalid scaling identifier");
        
        bytes.put_u8(0x0A); // TAG_Compound
        bytes.extend_from_slice(&(0i16).to_be_bytes());   // empty root name
        
        // exhaustion: TAG_Float
        bytes.put_u8(0x05);
        bytes.extend_from_slice(b"\x00\x0bexhaustion");
        bytes.extend_from_slice(&exhaustion.to_be_bytes());
        
        // message_id: TAG_String
        bytes.put_u8(0x08);
        bytes.extend_from_slice(b"\x00\x0amessage_id");
        bytes.extend_from_slice(&(message_id.len() as i16).to_be_bytes());
        bytes.extend_from_slice(message_id.as_bytes());

        // scaling: TAG_String
        bytes.put_u8(0x08);
        bytes.extend_from_slice(b"\x00\x07scaling");
        bytes.extend_from_slice(&(scaling.len() as i16).to_be_bytes());
        bytes.extend_from_slice(scaling.as_bytes());

        // TAG_End
        bytes.put_u8(0x00);

        bytes.to_vec()
    }
}
