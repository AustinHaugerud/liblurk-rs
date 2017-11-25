use ::protocol::protocol_message::*;
use ::protocol::primitive_parse::parse_u16l;

use std::net::TcpStream;
use std::io::Read;

use std::mem::size_of;

fn read_bytes(source: &mut Read, amount: usize) -> Result<Vec<u8>, ()> {
  let mut buf = vec![0u8; amount];

  let result = source.read_exact(&mut buf);

  if result.is_err() {
    return Err(());
  }

  Ok(buf)
}

fn read_var_bytes(source: &mut Read) -> Result<Vec<u8>, ()> {
  let len_desc_bytes = read_bytes(source, size_of::<u16>())?;

  let len = parse_u16l(&len_desc_bytes.as_slice());

  let mut block = read_bytes(source, len as usize)?;

  let mut result = Vec::with_capacity(len_desc_bytes.len() + block.len());
  result.extend(len_desc_bytes);
  result.extend(block);

  Ok(result)
}

pub fn extract_message(mut source: &mut Read) -> Result<LurkMessageFrame, ()> {
  let mut type_byte = read_bytes(&mut source, 1)?;

  match type_byte[0] {
    MESSAGE_TYPE     => Message::pull_message_data(&mut source),
    CHANGE_ROOM_TYPE => ChangeRoom::pull_message_data(&mut source),
    FIGHT_TYPE       => Fight::pull_message_data(&mut source),
    PVP_FIGHT_TYPE   => PvpFight::pull_message_data(&mut source),
    LOOT_TYPE        => Loot::pull_message_data(&mut source),
    START_TYPE       => Start::pull_message_data(&mut source),
    ERROR_TYPE       => Error::pull_message_data(&mut source),
    ACCEPT_TYPE      => Accept::pull_message_data(&mut source),
    ROOM_TYPE        => Room::pull_message_data(&mut source),
    CHARACTER_TYPE   => Character::pull_message_data(&mut source),
    GAME_TYPE        => Game::pull_message_data(&mut source),
    LEAVE_TYPE       => Leave::pull_message_data(&mut source),
    CONNECTION_TYPE  => Connection::pull_message_data(&mut source),
    _ => Err(())
  }
}

pub trait PullMessageData {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()>;
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Message {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let message_len_bytes = read_bytes(source, size_of::<u16>())?;
    let message_len = parse_u16l(&message_len_bytes);

    let mid_block_bytes = read_bytes(source, NAME_LENGTH as usize * 2)?;

    let message_bytes = read_bytes(source, message_len as usize)?;

    let mut result = Vec::with_capacity(message_len_bytes.len() + mid_block_bytes.len() + message_bytes.len());

    result.extend(message_len_bytes);
    result.extend(mid_block_bytes);
    result.extend(message_bytes);

    Ok(LurkMessageFrame::new(MESSAGE_TYPE, result))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for ChangeRoom {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let bytes = read_bytes(source, size_of::<u16>())?;
    Ok(LurkMessageFrame::new(CHANGE_ROOM_TYPE, bytes))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Fight {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    Ok(LurkMessageFrame::new(FIGHT_TYPE, vec![]))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for PvpFight {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let bytes = read_bytes(source, NAME_LENGTH as usize)?;
    Ok(LurkMessageFrame::new(PVP_FIGHT_TYPE, bytes))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Loot {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let bytes = read_bytes(source, NAME_LENGTH as usize)?;
    Ok(LurkMessageFrame::new(LOOT_TYPE, bytes))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Start {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    Ok(LurkMessageFrame::new(START_TYPE, vec![]))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Error {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let error_code_byte = read_bytes(source, 1)?;
    let error_bytes = read_var_bytes(source)?;
    let mut result = Vec::with_capacity(error_code_byte.len() + error_bytes.len());
    result.extend(error_code_byte);
    result.extend(error_bytes);
    Ok(LurkMessageFrame::new(ERROR_TYPE, result))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Accept {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let action_byte = read_bytes(source, 1)?;
    Ok(LurkMessageFrame::new(ACCEPT_TYPE, action_byte))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Room {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let pre_block = read_bytes(source, NAME_LENGTH as usize + size_of::<u16>())?;
    let description_block = read_var_bytes(source)?;
    let mut result = Vec::with_capacity(pre_block.len() + description_block.len());
    result.extend(pre_block);
    result.extend(description_block);
    Ok(LurkMessageFrame::new(ROOM_TYPE, result))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Character {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let pre_block = read_bytes(source, 45)?;
    let description_block = read_var_bytes(source)?;
    let mut result = Vec::with_capacity(pre_block.len() + description_block.len());
    result.extend(pre_block);
    result.extend(description_block);
    Ok(LurkMessageFrame::new(CHARACTER_TYPE, result))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Game {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let pre_block = read_bytes(source, 4)?;
    let description_block = read_var_bytes(source)?;
    let mut result = Vec::with_capacity(pre_block.len() + description_block.len());
    result.extend(pre_block);
    result.extend(description_block);
    Ok(LurkMessageFrame::new(GAME_TYPE, result))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Leave {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    Ok(LurkMessageFrame::new(LEAVE_TYPE, vec![]))
  }
}

////////////////////////////////////////////////////////////////////////////////////////////
impl PullMessageData for Connection {
  fn pull_message_data(source: &mut Read) -> Result<LurkMessageFrame, ()> {
    let pre_block = read_bytes(source, 34)?;
    let description_block = read_var_bytes(source)?;
    let mut result = Vec::with_capacity(pre_block.len() + description_block.len());
    result.extend(pre_block);
    result.extend(description_block);
    Ok(LurkMessageFrame::new(CONNECTION_TYPE, result))
  }
}

#[cfg(test)]
mod tests {

  use ::protocol::protocol_message::*;
  use super::*;
  use std::io::BufReader;

  #[test]
  fn test_message_extraction() {
    let data = vec![
      MESSAGE_TYPE,
      0x05, 0x00, // MESSAGE len

      'r' as u8, 'e' as u8, 'c' as u8, 'i' as u8, // Recipient
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,

      's' as u8, 'e' as u8, 'n' as u8, 'd' as u8, // Sender
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,
      0x00,0x00,0x00,0x00,

      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8 // MESSAGE
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(MESSAGE_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_change_room_extraction() {
    let data = vec![CHANGE_ROOM_TYPE, 0x08_u8, 0x00_u8];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(CHANGE_ROOM_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_fight_extraction() {

    let data = vec![FIGHT_TYPE];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(FIGHT_TYPE, vec![]);

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_pvp_fight_extraction() {
    let data = vec![
      PVP_FIGHT_TYPE,
      't' as u8, 'a' as u8, 'r' as u8, 'g' as u8,

      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(PVP_FIGHT_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_loot_extraction() {
    let data = vec![
      LOOT_TYPE,
      'l' as u8, 'o' as u8, 'o' as u8, 't' as u8,

      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(LOOT_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_start_extraction() {
    let data = vec![START_TYPE];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(START_TYPE, vec![]);

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_error_extraction() {
    let data = vec![
      ERROR_TYPE,
      0x06,
      0x03, 0x00,
      'c' as u8, 'a' as u8, 't' as u8
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(ERROR_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_accept_extraction() {
    let data = vec![ACCEPT_TYPE, 0x05u8];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(ACCEPT_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_room_extraction() {
    let data = vec![
      ROOM_TYPE,
      0x08, 0x00,

      'r' as u8, 'o' as u8, 'o' as u8, 'm' as u8,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,

      0x04, 0x00,
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(ROOM_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_character_extraction() {
    let data = vec![
      CHARACTER_TYPE,
      'p' as u8, 'l' as u8, 'a' as u8, 'y' as u8, 0x00, 0x00, 0x00, 0x00, // name
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

      0b10101010, // flags

      0xF0, 0x00, // attack
      0x0F, 0x00, // def
      0xAA, 0x00, // regen
      0xFF, 0x00, // health
      0xFF, 0x00, // gold
      0x03, 0x00, // room number

      0x04, 0x00, // description
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(CHARACTER_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_game_extraction() {
    let data = vec![
      GAME_TYPE,
      0x00, 0xFF, // init points
      0xFF, 0x00, // stat limit

      0x04, 0x00,
      'g' as u8, 'a' as u8, 'm' as u8, 'e' as u8
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(GAME_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_leave_extraction() {
    let data = vec![LEAVE_TYPE];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(LEAVE_TYPE, vec![]);

    assert_eq!(expected_frame, frame);
  }

  #[test]
  fn test_connection_extraction() {
    let data = vec![
      CONNECTION_TYPE,
      0x03, 0x00, // room number

      'r' as u8, 'o' as u8, 'o' as u8, 'm' as u8, 0x00, 0x00, 0x00, 0x00, // name
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

      0x04, 0x00, // description
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8
    ];

    let mut reader = BufReader::new(data.as_slice());

    let frame = extract_message(&mut reader).unwrap();

    let expected_frame = LurkMessageFrame::new(CONNECTION_TYPE, data[1..data.len()].to_vec());

    assert_eq!(expected_frame, frame);
  }
}
