use protocol::protocol_message::LurkMessageKind;
use std::io::Read;
use protocol::extraction::Extractor;

pub struct LurkReadChannel<'a, T> where &'a mut T: 'a + Read {
  read_source : &'a mut T,
}

impl<'a, T> LurkReadChannel<'a, T> where &'a mut T: 'a + Read {
  pub fn new(source : &'a mut T) -> LurkReadChannel<'a, T> {
    LurkReadChannel { read_source : source }
  }

  pub fn read_next(&mut self) -> Result<(LurkMessageKind, Vec<u8>), ()> {
    let type_byte = self.read_type_byte()?;
    let message_kind = LurkMessageKind::from_code(type_byte)?;

    let extractor = match message_kind {
      LurkMessageKind::Message    => Extractor::message(),
      LurkMessageKind::ChangeRoom => Extractor::change_room(),
      LurkMessageKind::Fight      => Extractor::fight(),
      LurkMessageKind::PvPFight   => Extractor::pvp_fight(),
      LurkMessageKind::Loot       => Extractor::loot(),
      LurkMessageKind::Start      => Extractor::start(),
      LurkMessageKind::Error      => Extractor::error(),
      LurkMessageKind::Accept     => Extractor::accept(),
      LurkMessageKind::Room       => Extractor::room(),
      LurkMessageKind::Character  => Extractor::character(),
      LurkMessageKind::Game       => Extractor::game(),
      LurkMessageKind::Leave      => Extractor::leave(),
      LurkMessageKind::Connection => Extractor::connection(),
    };

    let data_result = extractor.extract(&mut self.read_source);

    if data_result.is_err() {
      return Err(());
    }

    Ok((message_kind, data_result.unwrap()))
  }

  fn read_type_byte(&mut self) -> Result<u8, ()> {
    let mut buf = vec![0u8];
    if self.read_source.read_exact(&mut buf).is_err() {
      return Err(());
    }
    Ok(buf[0])
  }
}

#[cfg(test)]
mod test {

  use super::*;
  use protocol::protocol_message::*;
  use std::io::BufReader;

  #[test]
  fn test_message_pull() {
    let data = vec![
      Message::message_type(),

      4u8, 0u8,

      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,

      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,

      0u8, 0u8, 0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Message);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_change_room_pull() {
    let data = vec![
      ChangeRoom::message_type(),
      0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::ChangeRoom);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_fight_pull() {
    let data = vec![Fight::message_type()];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Fight);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_pvp_fight_pull() {
    let data = vec![
      PvpFight::message_type(),
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::PvPFight);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_loot_pull() {
    let data = vec![
      Loot::message_type(),
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Loot);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_start_pull() {
    let data = vec![Start::message_type()];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Start);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_error_pull() {

    let data = vec![
      Error::message_type(),

      0u8,

      4u8, 0u8,
      0u8, 0u8, 0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Error);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_accept_pull() {
    let data = vec![Accept::message_type(), 0u8];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Accept);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_room_pull() {
    let data = vec![
      Room::message_type(),

      0u8, 0u8,

      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,

      4u8, 0u8,
      0u8, 0u8, 0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Room);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_character_pull() {
    let data = vec![
      Character::message_type(),

      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,

      0u8,

      0u8, 0u8,
      0u8, 0u8,
      0u8, 0u8,
      0u8, 0u8,
      0u8, 0u8,
      0u8, 0u8,

      4u8, 0u8,
      0u8, 0u8, 0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let (kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Character);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_game_pull() {
    let data = vec![
      Game::message_type(),

      0u8, 0u8,
      0u8, 0u8,

      4u8, 0u8,
      0u8, 0u8, 0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let(kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Game);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_leave_pull() {
    let data = vec![Leave::message_type()];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let(kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Leave);
    assert_eq!(extract_data.len(), data.len() - 1);
  }

  #[test]
  fn test_connection_pull() {
    let data = vec![
      Connection::message_type(),

      0u8, 0u8,

      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
      0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,

      4u8, 0u8,
      0u8, 0u8, 0u8, 0u8
    ];

    let mut readable = BufReader::new(data.as_slice());

    let mut channel = LurkReadChannel::new(&mut readable);

    let result = channel.read_next();

    assert!(result.is_ok());

    let(kind, extract_data) = result.unwrap();

    assert_eq!(kind, LurkMessageKind::Connection);
    assert_eq!(extract_data.len(), data.len() - 1);
  }
}
