use std::str;

pub struct OutputBuffer {
    bytes: Vec<u8>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        OutputBuffer { bytes: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_str(&mut self, value: &str) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

pub struct InputBuffer<'a> {
    bytes: &'a [u8],
}

impl<'a> InputBuffer<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        InputBuffer { bytes }
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn read_u8(&mut self) -> u8 {
        let (first, rest) = self.bytes.split_first().unwrap();
        self.bytes = rest;
        *first
    }

    pub fn read_u16(&mut self) -> u16 {
        let (value, rest) = self.bytes.split_at(2);
        self.bytes = rest;
        u16::from_le_bytes([value[0], value[1]])
    }

    pub fn read_u32(&mut self) -> u32 {
        let (value, rest) = self.bytes.split_at(4);
        self.bytes = rest;
        u32::from_le_bytes([value[0], value[1], value[2], value[3]])
    }

    pub fn read_str(&mut self, len: usize) -> &'a str {
        let (string, rest) = self.bytes.split_at(len);
        self.bytes = rest;
        str::from_utf8(string).unwrap()
    }
}
