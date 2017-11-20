
pub struct OutputBuffer {
  pub data : Vec<u8>
}

impl OutputBuffer {
  pub fn new() -> OutputBuffer {
    OutputBuffer { data : vec![] }
  }

  pub fn write_byte(&mut self, val : u8) -> &mut Self {
    self.data.push(val);
    self
  }

  pub fn write_u16l(&mut self, val : u16) -> &mut Self {
    let extension = break_u16l(val);
    self.data.extend(extension.to_vec());
    self
  }

  pub fn write_i16l(&mut self, val : i16) -> &mut Self {
    let extension = break_i16l(val);
    self.data.extend(extension.to_vec());
    self
  }

  pub fn write_string(&mut self, text : &str) -> &mut Self {
    let extension = break_string(text);
    self.data.extend(extension);
    self
  }

  pub fn write_string_fixed(&mut self, text : &str, max_len : u16) -> &mut Self {
    let extension = break_string_fixed(text, max_len);
    self.data.extend(extension);
    self
  }
}

pub fn break_u16l(val: u16) -> [u8; 2] {
  let mut result = [0u8, 0u8];

  result[0] = ((val) & 0xFF) as u8;
  result[1] = ((val >> 8) & 0xFF) as u8;

  result
}

pub fn break_i16l(val: i16) -> [u8; 2] {
  let mut result = [0u8, 0u8];

  result[0] = (val & 0xFF) as u8;
  result[1] = ((val >> 8) & 0xFF) as u8;

  result
}

pub fn break_string(text: &str) -> Vec<u8> {
  let len_descriptor = break_u16l(text.len() as u16);

  let mut result = len_descriptor.to_vec();
  result.extend(text.as_bytes().to_vec());
  result
}

pub fn break_string_fixed(text: &str, max_len : u16) -> Vec<u8> {
  let data = text.as_bytes();

  let len = if data.len() > max_len as usize {
   max_len
  } else {
   data.len() as u16
  };

  let mut result : Vec<u8> = Vec::with_capacity(max_len as usize);

  for i in 0..max_len {
    if i < len {
      result.push(data[i as usize]);
    }
    else {
      result.push(0);
    }
  }

  result
}

#[cfg(test)]
mod tests {

  use super::*;

  #[test]
  fn test_break_u16l() {
    let val = 0x00_03_u16;

    let data = break_u16l(val);

    assert_eq!(data, [0x03, 0x00]);
  }

  #[test]
  fn test_break_i16l() {
    let val = 0x08_04;

    let data = break_i16l(val);

    assert_eq!(data, [0x04u8, 0x08u8]);
  }

  #[test]
  fn test_break_string() {
    let val = String::from("hello");

    let data = break_string(&val);

    assert_eq!(data , vec![0x05u8, 0x00u8, 'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8]);
  }

  #[test]
  fn test_break_string_fixed() {
    let val = String::from("hello");

    let data = break_string_fixed(&val, 16);

    assert_eq!(data, vec![
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8,
      'o' as u8, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ]);
  }

}
