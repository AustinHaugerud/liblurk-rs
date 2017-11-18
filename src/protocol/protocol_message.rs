use super::primitive_parse::*;

pub struct LurkMessageFrame
{
  message_type: u8,
  message_data: Vec<u8>,
}

impl LurkMessageFrame
{
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

pub trait FromLurkMessageFrame<T>
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<T, String> where T: LurkMessageType;
}

pub trait ToLurkMessageFrame
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame;
}

pub trait LurkMessageType
{
  fn message_type() -> u8;
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Message
{
  message: String,
  sender: String,
  receiver: String,
}

impl FromLurkMessageFrame<Message> for Message
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Message, String>
  {
    let data = lurk_message_frame.message_data.as_slice();
    let mut cursor = ReadBufferCursor::new(&data);

    let message_len = cursor.parse_u16l();

    if message_len.is_err()
    {
      return Err(String::from("Failed to parse message."))
    }


    let receiver = cursor.parse_string(32);
    let sender = cursor.parse_string(32);
    let message = cursor.parse_string(message_len.unwrap());

    if receiver.is_err() || sender.is_err() || message.is_err()
    {
      return Err(String::from("Failed to parse message."))
    }

    Ok(Message
    {
      message : message.unwrap().to_string(),
      sender : sender.unwrap().to_string(),
      receiver : receiver.unwrap().to_string(),
    })
  }
}

impl ToLurkMessageFrame for Message
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(1, vec![])
  }
}

impl LurkMessageType for Message
{
  fn message_type() -> u8
  {
    1
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct ChangeRoom
{
  room_number: u16,
}

impl FromLurkMessageFrame<ChangeRoom> for ChangeRoom
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<ChangeRoom, String>
  {
    Ok(ChangeRoom { room_number: 0 })
  }
}

impl ToLurkMessageFrame for ChangeRoom
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(2, vec![])
  }
}

impl LurkMessageType for ChangeRoom
{
  fn message_type() -> u8
  {
    2
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Fight;

impl ToLurkMessageFrame for Fight
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(3, vec![])
  }
}

impl LurkMessageType for Fight
{
  fn message_type() -> u8
  {
    3
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct PvpFight
{
  target: String,
}

impl FromLurkMessageFrame<PvpFight> for PvpFight
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<PvpFight, String>
  {
    Ok(PvpFight { target: String::new() })
  }
}

impl ToLurkMessageFrame for PvpFight
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(4, vec![])
  }
}

impl LurkMessageType for PvpFight
{
  fn message_type() -> u8
  {
    4
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Loot
{
  target: String,
}

impl FromLurkMessageFrame<Loot> for Loot
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Loot, String>
  {
    Ok(Loot { target: String::new() })
  }
}

impl ToLurkMessageFrame for Loot
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(5, vec![])
  }
}

impl LurkMessageType for Loot
{
  fn message_type() -> u8
  {
    5
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Start;

impl ToLurkMessageFrame for Start
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(6, vec![])
  }
}

impl LurkMessageType for Start
{
  fn message_type() -> u8
  {
    6
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Error
{
  error_code: u8,
  error_message: String,
}

impl FromLurkMessageFrame<Error> for Error
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Error, String>
  {
    Ok(Error { error_code: 0, error_message: String::new() })
  }
}

impl ToLurkMessageFrame for Error
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(7, vec![])
  }
}

impl LurkMessageType for Error
{
  fn message_type() -> u8
  {
    7
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Accept
{
  action_type: u8
}

impl FromLurkMessageFrame<Accept> for Accept
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Accept, String>
  {
    Ok(Accept { action_type: 0 })
  }
}

impl ToLurkMessageFrame for Accept
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(8, vec![])
  }
}

impl LurkMessageType for Accept
{
  fn message_type() -> u8
  {
    8
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Room
{
  room_number: u16,
  room_name: String,
  room_description: String,
}

impl FromLurkMessageFrame<Room> for Room
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Room, String>
  {
    Ok(Room { room_number: 0, room_name: String::new(), room_description: String::new() })
  }
}

impl ToLurkMessageFrame for Room
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(9, vec![])
  }
}

impl LurkMessageType for Room
{
  fn message_type() -> u8
  {
    9
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Character
{
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

impl FromLurkMessageFrame<Character> for Character
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Character, String>
  {
    Ok(Character
      {
        player_name: String::new(),
        is_alive: false,
        join_battles: false,
        is_monster: false,
        is_started: false,
        is_ready: false,
        attack: 0,
        defense: 0,
        regeneration: 0,
        health: 0,
        gold: 0,
        current_room_number: 0,
        description: String::new()
      })
  }
}

impl ToLurkMessageFrame for Character
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(10, vec![])
  }
}

impl LurkMessageType for Character
{
  fn message_type() -> u8
  {
    10
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Game
{
  initial_points: u16,
  stat_limit: u16,
  description: String,
}

impl FromLurkMessageFrame<Game> for Game
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Game, String>
  {
    Ok(Game { initial_points: 0, stat_limit: 0, description: String::new() })
  }
}

impl ToLurkMessageFrame for Game
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(11, vec![])
  }
}

impl LurkMessageType for Game
{
  fn message_type() -> u8
  {
    11
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Leave;

impl ToLurkMessageFrame for Leave
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(12, vec![])
  }
}

impl LurkMessageType for Leave
{
  fn message_type() -> u8
  {
    12
  }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Connection
{
  room_number: u16,
  room_name: String,
  room_description: String,
}

impl FromLurkMessageFrame<Connection> for Connection
{
  fn from_lurk_message_frame(lurk_message_frame: &LurkMessageFrame) -> Result<Connection, String>
  {
    Ok(Connection { room_number: 0, room_name: String::new(), room_description: String::new() })
  }
}

impl ToLurkMessageFrame for Connection
{
  fn to_lurk_message_frame(&self) -> LurkMessageFrame
  {
    LurkMessageFrame::new(13, vec![])
  }
}

impl LurkMessageType for Connection
{
  fn message_type() -> u8
  {
    13
  }
}

