use std::str::*;

pub struct ReadBufferCursor<'a> {
    index: usize,
    bytes: &'a [u8],
}

impl<'a> ReadBufferCursor<'a> {
    pub fn new(buffer: &'a [u8]) -> ReadBufferCursor {
        ReadBufferCursor {
            index: 0,
            bytes: buffer,
        }
    }

    /// Parses an unsigned 16 bit integer from the buffer assuming little endianness.
    pub fn parse_u16l(&mut self) -> Result<u16, String> {
        if self.bytes.len() - self.index + 1 < 2 {
            return Err(String::from("Not enough bytes remaining to parse u16."));
        }

        let result = parse_u16l(&self.bytes[self.index..(self.index + 2)]);
        self.index += 2;
        Ok(result)
    }

    /// Parses a signed 16 bit integer the buffer assuming little endianness.
    pub fn parse_i16l(&mut self) -> Result<i16, String> {
        if self.bytes.len() - self.index + 1 < 2 {
            return Err(String::from("Not enough bytes remaining to parse i16."));
        }

        let result = parse_i16l(&self.bytes[self.index..(self.index + 2)]);
        self.index += 2;
        Ok(result)
    }

    /// Parses a variable sized string from the buffer by reading a u16 length
    /// descriptor(little endian) then the string itself.
    pub fn parse_var_string(&mut self) -> Result<String, String> {
        match parse_var_string(&self.bytes[self.index..self.bytes.len()]) {
            Ok(t) => {
                self.index += t.len() + 2;
                Ok(t)
            }
            Err(e) => Err(e),
        }
    }

    /// Parses a fixed size string up to its capacity or null terminator.
    pub fn parse_string(&mut self, capacity: u16) -> Result<String, String> {
        let result = parse_string(
            &self.bytes[self.index..(self.index + (capacity as usize))],
            capacity,
        );
        self.index += capacity as usize;
        match result {
            Ok(t) => Ok(t.to_string()),
            Err(_) => Err(String::from("Failed to parse string.")),
        }
    }

    /// Get number of bytes remaining after cursor position.
    pub fn bytes_remaining(&self) -> usize {
        self.bytes.len() - self.index
    }

    /// Get the next single byte.
    pub fn get_byte(&mut self) -> Result<u8, String> {
        if self.bytes_remaining() == 0 {
            return Err(String::from("No byte remaining."));
        }

        self.index += 1;

        Ok(self.bytes[self.index - 1])
    }
}

pub fn parse_u16l(bytes: &[u8]) -> u16 {
    u16::from(bytes[0]) | (u16::from(bytes[1]) << 8)
}

fn parse_i16l(bytes: &[u8]) -> i16 {
    let val = parse_u16l(bytes);

    if val < 0x7fff {
        val as i16
    } else {
        -1 - (0xffffu32 - (u32::from(val))) as i16
    }
}

fn parse_var_string(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() < 2 {
        return Err(String::from("Not enough bytes to read a string."));
    }

    let size = parse_u16l(&bytes) as usize;

    if size > bytes.len() - 2 {
        return Err(format!(
            "Length descriptor {} asks for more bytes than present.",
            size
        ));
    }

    match from_utf8(&bytes[2..(2 + size)]) {
        Ok(t) => Ok(t.to_string()),
        Err(_) => Err(String::from("Failed to parse utf8 data.")),
    }
}

fn parse_string(bytes: &[u8], capacity: u16) -> Result<String, String> {
    let mut index: usize = capacity as usize;

    for i in 0..capacity {
        if bytes[i as usize] == 0 {
            index = i as usize;
            break;
        }
    }

    match from_utf8(&bytes[0..index]) {
        Ok(t) => Ok(t.to_string()),
        Err(_) => Err(String::from("Failed to parse utf8 data.")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_cursor_u16l_read() {
        let data = [0xf8u8, 0xffu8];
        let mut cursor = ReadBufferCursor::new(&data);

        assert_eq!(cursor.parse_u16l().unwrap(), 0xff_f8_u16);
    }

    #[test]
    fn test_cursor_i16l_read() {
        let data = [0xff, 0x00];
        let mut cursor = ReadBufferCursor::new(&data);

        assert_eq!(cursor.parse_i16l().unwrap(), 0x00_ff_i16);
    }

    #[test]
    fn test_cursor_var_string_read() {
        let data = [0x03, 0x00, 'c' as u8, 'a' as u8, 't' as u8];
        let mut cursor = ReadBufferCursor::new(&data);

        assert_eq!(cursor.parse_var_string().unwrap(), String::from("cat"));
    }

    #[test]
    fn test_cursor_string_read_null_terminated() {
        let data = ['c' as u8, 'a' as u8, 't' as u8, '\0' as u8];
        let mut cursor = ReadBufferCursor::new(&data);

        assert_eq!(cursor.parse_string(4).unwrap(), String::from("cat"));
    }

    #[test]
    fn test_cursor_string_read_full() {
        let data = ['c' as u8, 'a' as u8, 't' as u8];
        let mut cursor = ReadBufferCursor::new(&data);

        assert_eq!(cursor.parse_string(3).unwrap(), String::from("cat"));
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////

    #[test]
    fn test_u16l_read() {
        let data = [0xf8, 0xff];
        assert_eq!(parse_u16l(&data), 0xff_f8_u16);
    }

    #[test]
    fn test_i16l_read() {
        let data = [0xff, 0x00];
        assert_eq!(parse_i16l(&data), 0x00_ff_i16);
    }

    #[test]
    fn test_var_string_read() {
        let data = [0x03, 0x00, 'c' as u8, 'a' as u8, 't' as u8];
        assert_eq!(parse_var_string(&data).unwrap(), String::from("cat"));
    }

    #[test]
    fn test_string_read_null_terminated() {
        let data = ['c' as u8, 'a' as u8, 't' as u8, '\0' as u8];
        assert_eq!(parse_string(&data, 4).unwrap(), String::from("cat"));
    }

    #[test]
    fn test_string_read_full() {
        let data = ['c' as u8, 'a' as u8, 't' as u8];
        assert_eq!(parse_string(&data, 3).unwrap(), String::from("cat"));
    }

    ///////////////////////////////////////////////////////////////////////////////////////////////
}
