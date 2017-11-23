use super::primitive_parse::*;
use super::primitive_break::OutputBuffer;
use ::util::ResultLinkChecker;
use ::util::ResultChainChecker;
use ::util::BitField;

pub const NAME_LENGTH : u16 = 32;

pub const MESSAGE_TYPE     : u8 = 1;
pub const CHANGE_ROOM_TYPE : u8 = 2;
pub const FIGHT_TYPE       : u8 = 3;
pub const PVP_FIGHT_TYPE   : u8 = 4;
pub const LOOT_TYPE        : u8 = 5;
pub const START_TYPE       : u8 = 6;
pub const ERROR_TYPE       : u8 = 7;
pub const ACCEPT_TYPE      : u8 = 8;
pub const ROOM_TYPE        : u8 = 9;
pub const CHARACTER_TYPE   : u8 = 10;
pub const GAME_TYPE        : u8 = 11;
pub const LEAVE_TYPE       : u8 = 12;
pub const CONNECTION_TYPE  : u8 = 13;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LurkMessageFrame
{
  message_type: u8,
  message_data: Vec<u8>,
}

impl LurkMessageFrame {
  pub fn parse<F>(&self) -> Result<F, String> where F: FromLurkMessageFrame<F> + LurkMessageType
  {
    F::from_lurk_message_frame(self)
  }

  pub fn from_message<F>(message: &F) -> LurkMessageFrame where F: ToLurkMessageFrame + LurkMessageType
  {
    message.to_lurk_message_frame()
  }

  pub fn new(type_code: u8, data: Vec<u8>) -> LurkMessageFrame
  {
    LurkMessageFrame { message_type: type_code, message_data: data }
  }
}

pub trait FromLurkMessageFrame<T> {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<T, String> where T: LurkMessageType;
}

pub trait ToLurkMessageFrame {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame;
}

pub trait LurkMessageType {
  fn message_type() -> u8;
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Message {
  message: String,
  sender: String,
  receiver: String,
}

impl FromLurkMessageFrame<Message> for Message {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Message, String> {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let message_len = cursor.parse_u16l();

    if message_len.is_err() {
      return Err(String::from("Failed to parse message."));
    }


    let receiver = cursor.parse_string(NAME_LENGTH);
    let sender = cursor.parse_string(NAME_LENGTH);

    let len = message_len.unwrap();

    if len as usize > cursor.bytes_remaining() {
      return Err(String::from("Not enough bytes remaining for message."))
    }

    let message = cursor.parse_string(len);

    if receiver.is_err() || sender.is_err() || message.is_err() {
      return Err(String::from("Failed to parse message."));
    }

    Ok(Message {
      message: message.unwrap().to_string(),
      sender: sender.unwrap().to_string(),
      receiver: receiver.unwrap().to_string(),
    })
  }
}

impl ToLurkMessageFrame for Message {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder
      .write_u16l(self.message.len() as u16)
      .write_string_fixed(&self.receiver, NAME_LENGTH)
      .write_string_fixed(&self.sender, NAME_LENGTH)
      .write_string_fixed(&self.message, self.message.len() as u16);

    LurkMessageFrame::new(Message::message_type(), builder.data)
  }
}

impl LurkMessageType for Message {
  fn message_type() -> u8
  {
    MESSAGE_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct ChangeRoom {
  room_number: u16,
}

impl FromLurkMessageFrame<ChangeRoom> for ChangeRoom {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<ChangeRoom, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    match cursor.parse_u16l() {
      Ok(T) => Ok(ChangeRoom{ room_number : T }),
      Err(E) => Err(E)
    }
  }
}

impl ToLurkMessageFrame for ChangeRoom {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder.write_u16l(self.room_number);

    LurkMessageFrame::new(ChangeRoom::message_type(), builder.data)
  }
}

impl LurkMessageType for ChangeRoom {
  fn message_type() -> u8
  {
    CHANGE_ROOM_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Fight;

impl ToLurkMessageFrame for Fight {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    return LurkMessageFrame::new(Fight::message_type(), vec![]);
  }
}

impl LurkMessageType for Fight {
  fn message_type() -> u8
  {
    FIGHT_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct PvpFight {
  target: String,
}

impl FromLurkMessageFrame<PvpFight> for PvpFight {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<PvpFight, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    match cursor.parse_string(NAME_LENGTH) {
      Ok(T) => Ok(PvpFight { target: T }),
      Err(E) => Err(E)
    }
  }
}

impl ToLurkMessageFrame for PvpFight {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder.write_string_fixed(&self.target, NAME_LENGTH);

    LurkMessageFrame::new(PvpFight::message_type(), builder.data)
  }
}

impl LurkMessageType for PvpFight {
  fn message_type() -> u8
  {
    PVP_FIGHT_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Loot {
  target: String,
}

impl FromLurkMessageFrame<Loot> for Loot {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Loot, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    match cursor.parse_string(NAME_LENGTH) {
      Ok(T) => Ok(Loot { target : T }),
      Err(E) => Err(E)
    }
  }
}

impl ToLurkMessageFrame for Loot {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder.write_string_fixed(&self.target, NAME_LENGTH);

    LurkMessageFrame::new(Loot::message_type(), builder.data)
  }
}

impl LurkMessageType for Loot {
  fn message_type() -> u8
  {
    LOOT_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Start;

impl ToLurkMessageFrame for Start {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(Start::message_type(), vec![])
  }
}

impl LurkMessageType for Start {
  fn message_type() -> u8
  {
    START_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Error {
  error_code: u8,
  error_message: String,
}

impl FromLurkMessageFrame<Error> for Error {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Error, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let error_code = cursor.get_byte();

    if error_code.is_err() {
      return Err(error_code.unwrap_err())
    }

    match cursor.parse_var_string() {
      Ok(T) => Ok(Error { error_code : error_code.unwrap(), error_message : T }),
      Err(E) => Err(E)
    }
  }
}

impl ToLurkMessageFrame for Error {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder.write_byte(self.error_code).write_string(&self.error_message);

    LurkMessageFrame::new(Error::message_type(), builder.data)
  }
}

impl LurkMessageType for Error {
  fn message_type() -> u8
  {
    ERROR_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Accept {
  action_type: u8
}

impl FromLurkMessageFrame<Accept> for Accept {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Accept, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    match cursor.get_byte() {
      Ok(T) => Ok(Accept { action_type : T }),
      Err(E) => Err(E)
    }
  }
}

impl ToLurkMessageFrame for Accept {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder.write_byte(self.action_type);

    LurkMessageFrame::new(Accept::message_type(), builder.data)
  }
}

impl LurkMessageType for Accept {
  fn message_type() -> u8
  {
    ACCEPT_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Room {
  room_number: u16,
  room_name: String,
  room_description: String,
}

impl FromLurkMessageFrame<Room> for Room {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Room, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let room_number = cursor.parse_u16l();
    let room_name = cursor.parse_string(NAME_LENGTH);
    let description = cursor.parse_var_string();

    if room_number.is_err() || room_name.is_err() || description.is_err() {
      return Err(String::from("Failed to parse room message."))
    }

    Ok(Room {
      room_number : room_number.unwrap(),
      room_name : room_name.unwrap(),
      room_description : description.unwrap()
    })
  }
}

impl ToLurkMessageFrame for Room {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder
      .write_u16l(self.room_number)
      .write_string_fixed(&self.room_name, NAME_LENGTH)
      .write_string(&self.room_description);

    LurkMessageFrame::new(Room::message_type(), builder.data)
  }
}

impl LurkMessageType for Room {
  fn message_type() -> u8
  {
    ROOM_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Character {
  player_name: String,
  is_alive: bool,
  join_battles: bool,
  is_monster: bool,
  is_started: bool,
  is_ready: bool,
  attack: u16,
  defense: u16,
  regeneration: u16,
  health: i16,
  gold: u16,
  current_room_number: u16,
  description: String,
}

impl FromLurkMessageFrame<Character> for Character {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Character, String> {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let player_name = cursor.parse_string(NAME_LENGTH);
    let flags = cursor.get_byte();
    let attack = cursor.parse_u16l();
    let defense = cursor.parse_u16l();
    let regen = cursor.parse_u16l();
    let health = cursor.parse_i16l();
    let gold = cursor.parse_u16l();
    let room_number = cursor.parse_u16l();
    let description = cursor.parse_var_string();

    let mut checker = ResultChainChecker::new();

    checker
      .check(&player_name)
      .check(&flags)
      .check(&attack)
      .check(&defense)
      .check(&regen)
      .check(&health)
      .check(&gold)
      .check(&room_number)
      .check(&description);

    match checker.success() {
      true => {
        let bit_field = BitField { field : flags.unwrap() };

        Ok(Character {
          player_name: player_name.unwrap(),
          is_alive : bit_field.get(7),
          join_battles : bit_field.get(6),
          is_monster : bit_field.get(5),
          is_started : bit_field.get(4),
          is_ready : bit_field.get(3),
          attack : attack.unwrap(),
          defense : defense.unwrap(),
          regeneration : regen.unwrap(),
          health : health.unwrap(),
          gold : gold.unwrap(),
          current_room_number : room_number.unwrap(),
          description : description.unwrap(),
        })
      },
      false => Err(String::from("Failed to parse character."))
    }
  }
}

impl ToLurkMessageFrame for Character {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut bit_field = BitField { field : 0 };

    bit_field.configure(7, self.is_alive);
    bit_field.configure(6, self.join_battles);
    bit_field.configure(5, self.is_monster);
    bit_field.configure(4, self.is_started);
    bit_field.configure(3, self.is_ready);

    let mut builder = OutputBuffer::new();

    builder
      .write_string_fixed(&self.player_name, NAME_LENGTH)
      .write_byte(bit_field.field)
      .write_u16l(self.attack)
      .write_u16l(self.defense)
      .write_u16l(self.regeneration)
      .write_i16l(self.health)
      .write_u16l(self.gold)
      .write_u16l(self.current_room_number)
      .write_string(&self.description);

    LurkMessageFrame::new(Character::message_type(), builder.data)
  }
}

impl LurkMessageType for Character {
  fn message_type() -> u8
  {
    CHARACTER_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Game {
  initial_points: u16,
  stat_limit: u16,
  description: String,
}

impl FromLurkMessageFrame<Game> for Game {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Game, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let initial_points = cursor.parse_u16l();
    let stat_limit = cursor.parse_u16l();
    let description = cursor.parse_var_string();

    let mut checker = ResultChainChecker::new();

    checker
      .check(&initial_points)
      .check(&stat_limit)
      .check(&description);

    match checker.success() {
      true => Ok(Game {
        initial_points : initial_points.unwrap(),
        stat_limit : stat_limit.unwrap(),
        description : description.unwrap(),
      }),
      false => Err(String::from("Failed to parse game."))
    }
  }
}

impl ToLurkMessageFrame for Game {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder
      .write_u16l(self.initial_points)
      .write_u16l(self.stat_limit)
      .write_string(&self.description);

    LurkMessageFrame::new(Game::message_type(), builder.data)
  }
}

impl LurkMessageType for Game {
  fn message_type() -> u8
  {
    GAME_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Leave;

impl ToLurkMessageFrame for Leave {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(12, vec![])
  }
}

impl LurkMessageType for Leave {
  fn message_type() -> u8
  {
    LEAVE_TYPE
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Connection {
  room_number: u16,
  room_name: String,
  room_description: String,
}

impl FromLurkMessageFrame<Connection> for Connection {
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Connection, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let room_number = cursor.parse_u16l();
    let room_name = cursor.parse_string(NAME_LENGTH);
    let description = cursor.parse_var_string();

    let mut checker = ResultChainChecker::new();

    checker
      .check(&room_number)
      .check(&room_name)
      .check(&description);

    match checker.success() {
      true => Ok(Connection {
        room_number : room_number.unwrap(),
        room_name : room_name.unwrap(),
        room_description : description.unwrap(),
      }),
      false => Err(String::from("Failed to parse connection."))
    }
  }
}

impl ToLurkMessageFrame for Connection {
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    let mut builder = OutputBuffer::new();

    builder
      .write_u16l(self.room_number)
      .write_string_fixed(&self.room_name, NAME_LENGTH)
      .write_string(&self.room_description);

    LurkMessageFrame::new(Connection::message_type(), builder.data)
  }
}

impl LurkMessageType for Connection {
  fn message_type() -> u8
  {
    CONNECTION_TYPE
  }
}
/////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {

  use super::*;

  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_message_read() {
    let data = vec![
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

    let message_frame = LurkMessageFrame::new(Message::message_type(), data);

    let message = message_frame.parse::<Message>().unwrap();

    assert_eq!(message.message, String::from("hello"));
    assert_eq!(message.receiver, String::from("reci"));
    assert_eq!(message.sender, String::from("send"));
  }

  #[test]
  fn test_message_write() {
    let message = Message {
      message: String::from("mess"),
      sender: String::from("send"),
      receiver: String::from("reci"),
    };

    let data = message.to_lurk_message_frame();

    let expectation = [
      0x04, 0x00,

      'r' as u8, 'e' as u8, 'c' as u8, 'i' as u8,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,

      's' as u8, 'e' as u8, 'n' as u8, 'd' as u8,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,

      'm' as u8, 'e' as u8, 's' as u8, 's' as u8
    ];

    assert_eq!(data.message_type, Message::message_type());
    assert_eq!(data.message_data, expectation.to_vec());
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_changeroom_read() {
    let data = vec![0x08_u8, 0x00_u8];

    let message_frame = LurkMessageFrame::new(ChangeRoom::message_type(), data);

    let change_room = message_frame.parse::<ChangeRoom>().unwrap();

    assert_eq!(change_room.room_number, 8);
  }

  #[test]
  fn test_changeroom_write() {
    let change_room = ChangeRoom {
      room_number: 8,
    };

    let data = change_room.to_lurk_message_frame();

    let expectation = [
      0x08_u8, 0x00_u8
    ];

    assert_eq!(data.message_type, ChangeRoom::message_type());
    assert_eq!(data.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_fight_write() {
    let fight = Fight {};

    let message_frame = fight.to_lurk_message_frame();

    assert_eq!(message_frame.message_type, Fight::message_type());
    assert_eq!(message_frame.message_data, vec![]);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_pvpfight_read() {
    let data = vec![
      't' as u8, 'a' as u8, 'r' as u8, 'g' as u8,

      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ];

    let message_frame = LurkMessageFrame::new(PvpFight::message_type(), data);

    let pvp_fight = message_frame.parse::<PvpFight>().unwrap();

    assert_eq!(pvp_fight.target, String::from("targ"));
  }

  #[test]
  fn test_pvpfight_write() {
    let pvpfight = PvpFight {
      target: String::from("targ"),
    };

    let message_frame = pvpfight.to_lurk_message_frame();

    let expectation = vec![
      't' as u8, 'a' as u8, 'r' as u8, 'g' as u8,

      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ];

    assert_eq!(message_frame.message_type, PvpFight::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  ////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_loot_read() {
    let data = vec![
      'l' as u8, 'o' as u8, 'o' as u8, 't' as u8,

      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ];

    let message_frame = LurkMessageFrame::new(4, data);

    let loot = message_frame.parse::<Loot>().unwrap();

    assert_eq!(loot.target, String::from("loot"));
  }

  #[test]
  fn test_loot_write() {

    let loot = Loot {
      target: String::from("loot"),
    };

    let message_frame = loot.to_lurk_message_frame();

    let expectation = vec![
      'l' as u8, 'o' as u8, 'o' as u8, 't' as u8,

      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
    ];

    assert_eq!(message_frame.message_type, Loot::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_start_write() {
    let start = Start {};

    let message_frame = start.to_lurk_message_frame();

    assert_eq!(message_frame.message_type, Start::message_type());
    assert_eq!(message_frame.message_data, vec![]);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_error_read() {
    let data = vec![
      0x06,
      0x03, 0x00,
      'c' as u8, 'a' as u8, 't' as u8
    ];

    let message_frame = LurkMessageFrame::new(Error::message_type(), data);

    let error = message_frame.parse::<Error>().unwrap();

    assert_eq!(error.error_code, 6);
    assert_eq!(error.error_message, String::from("cat"))
  }

  #[test]
  fn test_error_write() {
    let error = Error {
      error_code: 6,
      error_message: String::from("cat"),
    };

    let message_frame = error.to_lurk_message_frame();

    let expectation = vec![
      0x06,
      0x03, 0x00,
      'c' as u8, 'a' as u8, 't' as u8
    ];

    assert_eq!(message_frame.message_type, Error::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_accept_read() {
    let data = vec![0x05 as u8];

    let message_frame = LurkMessageFrame::new(Error::message_type(), data);

    let accept = message_frame.parse::<Accept>().unwrap();

    assert_eq!(accept.action_type, 0x05_u8);
  }

  #[test]
  fn test_accept_write() {
    let accept = Accept {
      action_type: 5,
    };

    let message_frame = accept.to_lurk_message_frame();

    let expectation = vec![0x05 as u8];

    assert_eq!(message_frame.message_type, Accept::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_room_read() {
    let data = vec![
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

    let message_frame = LurkMessageFrame::new(Room::message_type(), data);

    let room = message_frame.parse::<Room>().unwrap();

    assert_eq!(room.room_number, 0x08_u16);
    assert_eq!(room.room_name, String::from("room"));
    assert_eq!(room.room_description, String::from("hell"));
  }

  #[test]
  fn test_room_write() {
    let room = Room {
      room_number: 8,
      room_name: String::from("room"),
      room_description: String::from("hell"),
    };

    let message_frame = room.to_lurk_message_frame();

    let expectation = vec![
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

    assert_eq!(message_frame.message_type, Room::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_character_read() {
    let data = vec![

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

    let message_frame = LurkMessageFrame::new(Character::message_type(), data);

    let character = message_frame.parse::<Character>().unwrap();

    assert_eq!(character.player_name, String::from("play"));
    assert_eq!(character.is_alive, true);
    assert_eq!(character.join_battles, false);
    assert_eq!(character.is_monster, true);
    assert_eq!(character.is_started, false);
    assert_eq!(character.is_ready, true);
    assert_eq!(character.attack, 0x00_F0);
    assert_eq!(character.defense, 0x00_0F);
    assert_eq!(character.regeneration, 0x00_AA);
    assert_eq!(character.health, 0x00_FF);
    assert_eq!(character.gold, 0x00_FF);
    assert_eq!(character.current_room_number, 0x00_03);
    assert_eq!(character.description, String::from("hell"));
  }

  #[test]
  fn test_character_write() {
    let character = Character {
      player_name: String::from("play"),
      is_alive: true,
      join_battles: false,
      is_monster: true,
      is_started: false,
      is_ready: true,
      attack: 0x00_F0,
      defense: 0x00_0F,
      regeneration: 0x00_AA,
      health: 0x00_FF,
      gold: 0x00_FF,
      current_room_number: 0x00_03,
      description: String::from("hell"),
    };

    let message_frame = character.to_lurk_message_frame();

    let expectation = vec![

      'p' as u8, 'l' as u8, 'a' as u8, 'y' as u8, 0x00, 0x00, 0x00, 0x00, // name
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

      0b10101000, // flags

      0xF0, 0x00, // attack
      0x0F, 0x00, // def
      0xAA, 0x00, // regen
      0xFF, 0x00, // health
      0xFF, 0x00, // gold
      0x03, 0x00, // room number

      0x04, 0x00, // description
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8
    ];

    assert_eq!(message_frame.message_type, Character::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_game_read() {
    let data = vec![
      0x00, 0xFF, // init points
      0xFF, 0x00, // stat limit

      0x04, 0x00,
      'g' as u8, 'a' as u8, 'm' as u8, 'e' as u8
    ];

    let message_frame = LurkMessageFrame::new(Game::message_type(), data);

    let game = message_frame.parse::<Game>().unwrap();

    assert_eq!(game.initial_points, 0xFF_00);
    assert_eq!(game.stat_limit, 0x00_FF);
    assert_eq!(game.description, String::from("game"));
  }

  #[test]
  fn test_game_write() {
    let game = Game {
      initial_points: 0xFF_00,
      stat_limit: 0x00_FF,
      description: String::from("game"),
    };

    let message_frame = game.to_lurk_message_frame();

    let expectation = vec![
      0x00, 0xFF, // init points
      0xFF, 0x00, // stat limit

      0x04, 0x00,
      'g' as u8, 'a' as u8, 'm' as u8, 'e' as u8
    ];

    assert_eq!(message_frame.message_type, Game::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_leave_write() {
    let leave = Leave {};

    let message_frame = leave.to_lurk_message_frame();

    let expectation = vec![];

    assert_eq!(message_frame.message_type, Leave::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
  #[test]
  fn test_connection_read() {
    let data = vec![
      0x03, 0x00, // room number

      'r' as u8, 'o' as u8, 'o' as u8, 'm' as u8, 0x00, 0x00, 0x00, 0x00, // name
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

      0x04, 0x00, // description
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8
    ];

    let message_frame = LurkMessageFrame::new(Connection::message_type(), data);

    let connection = message_frame.parse::<Connection>().unwrap();

    assert_eq!(connection.room_number, 0x00_03);
    assert_eq!(connection.room_name, String::from("room"));
    assert_eq!(connection.room_description, String::from("hell"));
  }

  #[test]
  fn test_connection_write() {
    let connection = Connection {
      room_number: 3,
      room_name: String::from("room"),
      room_description: String::from("hell"),
    };

    let message_frame = connection.to_lurk_message_frame();

    let expectation = vec![
      0x03, 0x00, // room number

      'r' as u8, 'o' as u8, 'o' as u8, 'm' as u8, 0x00, 0x00, 0x00, 0x00, // name
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,

      0x04, 0x00, // description
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8
    ];

    assert_eq!(message_frame.message_type, Connection::message_type());
    assert_eq!(message_frame.message_data, expectation);
  }
  /////////////////////////////////////////////////////////////////////////////////////////////////
}
