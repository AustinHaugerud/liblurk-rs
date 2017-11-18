use std::str::*;

pub struct ReadBufferCursor<'a>
{
  index: usize,
  bytes: &'a [u8],
}

impl<'a> ReadBufferCursor<'a>
{
  pub fn new(buffer: &'a [u8]) -> ReadBufferCursor
  {
    ReadBufferCursor { index: 0, bytes: buffer }
  }

  pub fn parse_u16l(&mut self) -> Result<u16, String>
  {
    if self.bytes.len() - (self.index + 1) < 2
    {
      return Err(String::from("Not enough bytes remaining to parse u16."))
    }

    let result = parse_u16l(&self.bytes[self.index..(self.index + 2)]);
    self.index += 2;
    Ok(result)
  }

  pub fn parse_i16l(&mut self) -> Result<i16, String>
  {
    if self.bytes.len() - (self.index + 1) < 2
    {
      return Err(String::from("Not enough bytes remaining to parse i16."))
    }

    let result = parse_i16l(&self.bytes[self.index..(self.index + 2)]);
    self.index += 2;
    Ok(result)
  }

  pub fn parse_var_string(&mut self) -> Result<&str, String>
  {
    let size_result = self.parse_u16l();

    if size_result.is_err()
    {
      return Err(String::from("Failed to parse string size."))
    }

    let size = size_result.unwrap() as usize;

    let result = parse_var_string(&self.bytes[self.index..(self.index + size)]);
    self.index += size;

    match result
    {
      Ok(T) => Ok(T),
      Err(E) => Err(String::from("Failed to parse string."))
    }
  }

  pub fn parse_string(&mut self, capacity: u16) -> Result<&str, Utf8Error>
  {
    let result = parse_string(&self.bytes[self.index..(self.index + (capacity as usize))], capacity);
    self.index += capacity as usize;
    result
  }
}

fn parse_u16l(bytes: &[u8]) -> u16
{
  (bytes[0] as u16 | ((bytes[1] as u16) << 8))
}

fn parse_i16l(bytes: &[u8]) -> i16
{
  let val = parse_u16l(bytes);

  if val < 0x7fff { val as i16 } else { -1 - (0xffffu32 - (val as u32)) as i16 }
}

fn parse_var_string(bytes: &[u8]) -> Result<&str, Utf8Error>
{
  from_utf8(&bytes)
}

fn parse_string(bytes: &[u8], capacity: u16) -> Result<&str, Utf8Error>
{
  let mut index: usize = 0;

  for i in 0..capacity
    {
      if bytes[i as usize] == 0
        {
          index = i as usize;
          break;
        }
    }

  return from_utf8(&bytes[0..index]);
}

