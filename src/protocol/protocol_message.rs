use super::primitive_parse::*;
use super::primitive_break::OutputBuffer;
use util::ResultLinkChecker;
use util::ResultChainChecker;
use util::BitField;

pub const NAME_LENGTH: u16 = 32;

pub const MESSAGE_TYPE: u8 = 1;
pub const CHANGE_ROOM_TYPE: u8 = 2;
pub const FIGHT_TYPE: u8 = 3;
pub const PVP_FIGHT_TYPE: u8 = 4;
pub const LOOT_TYPE: u8 = 5;
pub const START_TYPE: u8 = 6;
pub const ERROR_TYPE: u8 = 7;
pub const ACCEPT_TYPE: u8 = 8;
pub const ROOM_TYPE: u8 = 9;
pub const CHARACTER_TYPE: u8 = 10;
pub const GAME_TYPE: u8 = 11;
pub const LEAVE_TYPE: u8 = 12;
pub const CONNECTION_TYPE: u8 = 13;

#[derive(PartialEq, Debug)]
pub enum LurkMessageKind {
    Message,
    ChangeRoom,
    Fight,
    PvPFight,
    Loot,
    Start,
    Error,
    Accept,
    Room,
    Character,
    Game,
    Leave,
    Connection,
}

impl LurkMessageKind {
    pub fn from_code(code: u8) -> Result<LurkMessageKind, ()> {
        match code {
            MESSAGE_TYPE => Ok(LurkMessageKind::Message),
            CHANGE_ROOM_TYPE => Ok(LurkMessageKind::ChangeRoom),
            FIGHT_TYPE => Ok(LurkMessageKind::Fight),
            PVP_FIGHT_TYPE => Ok(LurkMessageKind::PvPFight),
            LOOT_TYPE => Ok(LurkMessageKind::Loot),
            START_TYPE => Ok(LurkMessageKind::Start),
            ERROR_TYPE => Ok(LurkMessageKind::Error),
            ACCEPT_TYPE => Ok(LurkMessageKind::Accept),
            ROOM_TYPE => Ok(LurkMessageKind::Room),
            CHARACTER_TYPE => Ok(LurkMessageKind::Character),
            GAME_TYPE => Ok(LurkMessageKind::Game),
            LEAVE_TYPE => Ok(LurkMessageKind::Leave),
            CONNECTION_TYPE => Ok(LurkMessageKind::Connection),
            _ => Err(()),
        }
    }
}

pub trait LurkMessageParse<T> {
    fn parse_lurk_message(message_data: &[u8]) -> Result<(T, usize), String>
    where
        T: LurkMessageType;
}

pub trait LurkMessageBlobify {
    fn produce_lurk_message_blob(&self) -> Vec<u8>;
}

pub trait LurkMessageType {
    fn message_type() -> u8;
}

/////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct Message {
    pub message: String,
    pub sender: String,
    pub receiver: String,
}

impl Message {
    pub fn new(message_content: String, sender: String, receiver: String) -> Result<Message, ()> {
        if message_content.len() < u16::max_value() as usize && sender.len() <= 32
            && receiver.len() <= 32
        {
            return Ok(Message {
                message: message_content,
                sender,
                receiver,
            });
        }

        Err(())
    }
}

impl LurkMessageParse<Message> for Message {
    fn parse_lurk_message(data: &[u8]) -> Result<(Message, usize), String> {
        let mut bytes_read: usize = 0;
        let mut cursor = ReadBufferCursor::new(&data);

        let message_len = cursor.parse_u16l();

        if message_len.is_err() {
            return Err(String::from("Failed to parse message."));
        }

        let receiver = cursor.parse_string(NAME_LENGTH);
        let sender = cursor.parse_string(NAME_LENGTH);

        bytes_read += NAME_LENGTH as usize * 2 + 2;

        let len = message_len.unwrap();

        if len as usize > cursor.bytes_remaining() {
            return Err(String::from("Not enough bytes remaining for message."));
        }

        let message = cursor.parse_string(len);

        if receiver.is_err() || sender.is_err() || message.is_err() {
            return Err(String::from("Failed to parse message."));
        }

        bytes_read += len as usize;

        Ok((
            Message {
                message: message.unwrap().to_string(),
                sender: sender.unwrap().to_string(),
                receiver: receiver.unwrap().to_string(),
            },
            bytes_read,
        ))
    }
}

impl LurkMessageBlobify for Message {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Message::message_type())
            .write_u16l(self.message.len() as u16)
            .write_string_fixed(&self.receiver, NAME_LENGTH)
            .write_string_fixed(&self.sender, NAME_LENGTH)
            .write_string_fixed(&self.message, self.message.len() as u16);

        builder.data
    }
}

impl LurkMessageType for Message {
    fn message_type() -> u8 {
        MESSAGE_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct ChangeRoom {
    pub room_number: u16,
}

impl ChangeRoom {
    pub fn new(room_number: u16) -> ChangeRoom {
        ChangeRoom { room_number }
    }
}

impl LurkMessageParse<ChangeRoom> for ChangeRoom {
    fn parse_lurk_message(data: &[u8]) -> Result<(ChangeRoom, usize), String> {
        let mut cursor = ReadBufferCursor::new(&data);

        match cursor.parse_u16l() {
            Ok(t) => Ok((ChangeRoom { room_number: t }, 2)),
            Err(e) => Err(e),
        }
    }
}

impl LurkMessageBlobify for ChangeRoom {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(ChangeRoom::message_type())
            .write_u16l(self.room_number);
        builder.data
    }
}

impl LurkMessageType for ChangeRoom {
    fn message_type() -> u8 {
        CHANGE_ROOM_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Fight;

impl Fight {
    pub fn new() -> Fight {
        Fight {}
    }
}

impl LurkMessageParse<Fight> for Fight {
    fn parse_lurk_message(_: &[u8]) -> Result<(Fight, usize), String> {
        Ok((Fight {}, 0))
    }
}

impl LurkMessageBlobify for Fight {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();
        builder.write_byte(Fight::message_type());
        builder.data
    }
}

impl LurkMessageType for Fight {
    fn message_type() -> u8 {
        FIGHT_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct PvpFight {
    pub target: String,
}

impl PvpFight {
    pub fn new(target: String) -> Result<PvpFight, ()> {
        if target.len() <= 32 {
            return Ok(PvpFight { target });
        }

        Err(())
    }
}

impl LurkMessageParse<PvpFight> for PvpFight {
    fn parse_lurk_message(data: &[u8]) -> Result<(PvpFight, usize), String> {
        let mut cursor = ReadBufferCursor::new(&data);

        match cursor.parse_string(NAME_LENGTH) {
            Ok(t) => Ok((PvpFight { target: t }, NAME_LENGTH as usize)),
            Err(e) => Err(e),
        }
    }
}

impl LurkMessageBlobify for PvpFight {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(PvpFight::message_type())
            .write_string_fixed(&self.target, NAME_LENGTH);

        builder.data
    }
}

impl LurkMessageType for PvpFight {
    fn message_type() -> u8 {
        PVP_FIGHT_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Loot {
    pub target: String,
}

impl Loot {
    pub fn new(target: String) -> Result<Loot, ()> {
        if target.len() <= 32 {
            return Ok(Loot { target });
        }

        Err(())
    }
}

impl LurkMessageParse<Loot> for Loot {
    fn parse_lurk_message(data: &[u8]) -> Result<(Loot, usize), String> {
        let mut cursor = ReadBufferCursor::new(&data);

        match cursor.parse_string(NAME_LENGTH) {
            Ok(t) => Ok((Loot { target: t }, NAME_LENGTH as usize)),
            Err(e) => Err(e),
        }
    }
}

impl LurkMessageBlobify for Loot {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Loot::message_type())
            .write_string_fixed(&self.target, NAME_LENGTH);

        builder.data
    }
}

impl LurkMessageType for Loot {
    fn message_type() -> u8 {
        LOOT_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Start;

impl Start {
    pub fn new() -> Start {
        Start {}
    }
}

impl LurkMessageParse<Start> for Start {
    fn parse_lurk_message(_: &[u8]) -> Result<(Start, usize), String> {
        Ok((Start {}, 0))
    }
}

impl LurkMessageBlobify for Start {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();
        builder.write_byte(Start::message_type());
        builder.data
    }
}

impl LurkMessageType for Start {
    fn message_type() -> u8 {
        START_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////

pub const ERROR_TYPE_OTHER: u8 = 0;
pub const ERROR_TYPE_BAD_ROOM: u8 = 1;
pub const ERROR_TYPE_PLAYER_EXISTS: u8 = 2;
pub const ERROR_TYPE_BAD_MONSTER: u8 = 3;
pub const ERROR_TYPE_STAT_ERROR: u8 = 4;
pub const ERROR_TYPE_NOT_READY: u8 = 5;
pub const ERROR_TYPE_NO_TARGET: u8 = 6;
pub const ERROR_TYPE_NO_FIGHT: u8 = 7;
pub const ERROR_TYPE_NO_PVP: u8 = 8;

pub struct Error {
    pub error_code: u8,
    pub error_message: String,
}

impl Error {
    pub fn new(error_code: u8, error_message: String) -> Result<Error, ()> {
        if error_message.len() < u16::max_value() as usize {
            return Ok(Error {
                error_code,
                error_message,
            });
        }

        Err(())
    }

    pub fn other(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_OTHER, error_message)
    }

    pub fn bad_room(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_BAD_ROOM, error_message)
    }

    pub fn player_exists(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_PLAYER_EXISTS, error_message)
    }

    pub fn bad_monster(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_BAD_MONSTER, error_message)
    }

    pub fn stat_error(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_STAT_ERROR, error_message)
    }

    pub fn not_ready(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_NOT_READY, error_message)
    }

    pub fn no_target(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_NO_TARGET, error_message)
    }

    pub fn no_fight(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_NO_FIGHT, error_message)
    }

    pub fn no_pvp(error_message: String) -> Result<Error, ()> {
        Error::new(ERROR_TYPE_NO_PVP, error_message)
    }
}

impl LurkMessageParse<Error> for Error {
    fn parse_lurk_message(data: &[u8]) -> Result<(Error, usize), String> {
        let mut cursor = ReadBufferCursor::new(&data);

        let error_code = cursor.get_byte();

        if error_code.is_err() {
            return Err(error_code.unwrap_err());
        }

        match cursor.parse_var_string() {
            Ok(t) => {
                let bytes_read = t.len() + 1 + 2;
                Ok((
                    Error {
                        error_code: error_code.unwrap(),
                        error_message: t,
                    },
                    bytes_read,
                ))
            }
            Err(e) => Err(e),
        }
    }
}

impl LurkMessageBlobify for Error {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Error::message_type())
            .write_byte(self.error_code)
            .write_string(&self.error_message);

        builder.data
    }
}

impl LurkMessageType for Error {
    fn message_type() -> u8 {
        ERROR_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Accept {
    pub action_type: u8,
}

impl Accept {
    pub fn new(action_type: u8) -> Accept {
        Accept { action_type }
    }
}

impl LurkMessageParse<Accept> for Accept {
    fn parse_lurk_message(data: &[u8]) -> Result<(Accept, usize), String> {
        let mut cursor = ReadBufferCursor::new(&data);

        match cursor.get_byte() {
            Ok(t) => Ok((Accept { action_type: t }, 1)),
            Err(e) => Err(e),
        }
    }
}

impl LurkMessageBlobify for Accept {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Accept::message_type())
            .write_byte(self.action_type);

        builder.data
    }
}

impl LurkMessageType for Accept {
    fn message_type() -> u8 {
        ACCEPT_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Room {
    pub room_number: u16,
    pub room_name: String,
    pub room_description: String,
}

impl Room {
    pub fn new(room_number: u16, room_name: String, room_description: String) -> Result<Room, ()> {
        if room_name.len() <= 32 && room_description.len() < u16::max_value() as usize {
            return Ok(Room {
                room_number,
                room_name,
                room_description,
            });
        }

        Err(())
    }
}

impl LurkMessageParse<Room> for Room {
    fn parse_lurk_message(data: &[u8]) -> Result<(Room, usize), String> {
        let mut bytes_read = 0;
        let mut cursor = ReadBufferCursor::new(&data);

        let room_number = cursor.parse_u16l();
        bytes_read += 2;
        let room_name = cursor.parse_string(NAME_LENGTH);
        bytes_read += NAME_LENGTH as usize;
        let description = cursor.parse_var_string();

        if room_number.is_err() || room_name.is_err() || description.is_err() {
            return Err(String::from("Failed to parse room message."));
        }

        let desc = description.unwrap();
        bytes_read += desc.len() + 2;

        Ok((
            Room {
                room_number: room_number.unwrap(),
                room_name: room_name.unwrap(),
                room_description: desc,
            },
            bytes_read,
        ))
    }
}

impl LurkMessageBlobify for Room {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Room::message_type())
            .write_u16l(self.room_number)
            .write_string_fixed(&self.room_name, NAME_LENGTH)
            .write_string(&self.room_description);

        builder.data
    }
}

impl LurkMessageType for Room {
    fn message_type() -> u8 {
        ROOM_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
pub struct Character {
    pub player_name: String,
    pub is_alive: bool,
    pub join_battles: bool,
    pub is_monster: bool,
    pub is_started: bool,
    pub is_ready: bool,
    pub attack: u16,
    pub defense: u16,
    pub regeneration: u16,
    pub health: i16,
    pub gold: u16,
    pub current_room_number: u16,
    pub description: String,
}

impl Character {
    pub fn new(
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
    ) -> Result<Character, ()> {
        if player_name.len() <= 32 && description.len() < u16::max_value() as usize {
            return Ok(Character {
                player_name,
                is_alive,
                join_battles,
                is_monster,
                is_started,
                is_ready,
                attack,
                defense,
                regeneration,
                health,
                gold,
                current_room_number,
                description,
            });
        }

        Err(())
    }
}

impl LurkMessageParse<Character> for Character {
    fn parse_lurk_message(data: &[u8]) -> Result<(Character, usize), String> {
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
                let bit_field = BitField {
                    field: flags.unwrap(),
                };
                let desc = description.unwrap();
                let bytes_read: usize = NAME_LENGTH as usize + 1 + (2 * 7) + desc.len();

                Ok((
                    Character {
                        player_name: player_name.unwrap(),
                        is_alive: bit_field.get(7),
                        join_battles: bit_field.get(6),
                        is_monster: bit_field.get(5),
                        is_started: bit_field.get(4),
                        is_ready: bit_field.get(3),
                        attack: attack.unwrap(),
                        defense: defense.unwrap(),
                        regeneration: regen.unwrap(),
                        health: health.unwrap(),
                        gold: gold.unwrap(),
                        current_room_number: room_number.unwrap(),
                        description: desc,
                    },
                    bytes_read,
                ))
            }
            false => Err(String::from("Failed to parse character.")),
        }
    }
}

impl LurkMessageBlobify for Character {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut bit_field = BitField { field: 0 };

        bit_field.configure(7, self.is_alive);
        bit_field.configure(6, self.join_battles);
        bit_field.configure(5, self.is_monster);
        bit_field.configure(4, self.is_started);
        bit_field.configure(3, self.is_ready);

        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Character::message_type())
            .write_string_fixed(&self.player_name, NAME_LENGTH)
            .write_byte(bit_field.field)
            .write_u16l(self.attack)
            .write_u16l(self.defense)
            .write_u16l(self.regeneration)
            .write_i16l(self.health)
            .write_u16l(self.gold)
            .write_u16l(self.current_room_number)
            .write_string(&self.description);

        builder.data
    }
}

impl LurkMessageType for Character {
    fn message_type() -> u8 {
        CHARACTER_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Game {
    pub initial_points: u16,
    pub stat_limit: u16,
    pub description: String,
}

impl Game {
    pub fn new(initial_points: u16, stat_limit: u16, description: String) -> Result<Game, ()> {
        if description.len() < u16::max_value() as usize {
            return Ok(Game {
                initial_points,
                stat_limit,
                description,
            });
        }

        Err(())
    }
}

impl LurkMessageParse<Game> for Game {
    fn parse_lurk_message(data: &[u8]) -> Result<(Game, usize), String> {
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
            true => {
                let desc = description.unwrap();
                let bytes_read = desc.len() + (2 * 3);

                Ok((
                    Game {
                        initial_points: initial_points.unwrap(),
                        stat_limit: stat_limit.unwrap(),
                        description: desc,
                    },
                    bytes_read,
                ))
            }
            false => Err(String::from("Failed to parse game.")),
        }
    }
}

impl LurkMessageBlobify for Game {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Game::message_type())
            .write_u16l(self.initial_points)
            .write_u16l(self.stat_limit)
            .write_string(&self.description);

        builder.data
    }
}

impl LurkMessageType for Game {
    fn message_type() -> u8 {
        GAME_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Leave;

impl Leave {
    pub fn new() -> Leave {
        Leave {}
    }
}

impl LurkMessageBlobify for Leave {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();
        builder.write_byte(Leave::message_type());
        builder.data
    }
}

impl LurkMessageType for Leave {
    fn message_type() -> u8 {
        LEAVE_TYPE
    }
}

/////////////////////////////////////////////////////////////////////////////////////////////////
pub struct Connection {
    pub room_number: u16,
    pub room_name: String,
    pub room_description: String,
}

impl Connection {
    pub fn new(
        room_number: u16,
        room_name: String,
        room_description: String,
    ) -> Result<Connection, ()> {
        if room_name.len() <= 32 && room_description.len() < u16::max_value() as usize {
            return Ok(Connection {
                room_number,
                room_name,
                room_description,
            });
        }

        Err(())
    }
}

impl LurkMessageParse<Connection> for Connection {
    fn parse_lurk_message(data: &[u8]) -> Result<(Connection, usize), String> {
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
            true => {
                let desc = description.unwrap();
                let bytes_read = desc.len() + NAME_LENGTH as usize + 2 + 2;
                Ok((
                    Connection {
                        room_number: room_number.unwrap(),
                        room_name: room_name.unwrap(),
                        room_description: desc,
                    },
                    bytes_read,
                ))
            }
            false => Err(String::from("Failed to parse connection.")),
        }
    }
}

impl LurkMessageBlobify for Connection {
    fn produce_lurk_message_blob(&self) -> Vec<u8> {
        let mut builder = OutputBuffer::new();

        builder
            .write_byte(Connection::message_type())
            .write_u16l(self.room_number)
            .write_string_fixed(&self.room_name, NAME_LENGTH)
            .write_string(&self.room_description);

        builder.data
    }
}

impl LurkMessageType for Connection {
    fn message_type() -> u8 {
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
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      's' as u8, 'e' as u8, 'n' as u8, 'd' as u8, // Sender
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00,
      'h' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8 // MESSAGE
    ];

        let (message, bytes_read) = Message::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
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

        let data = message.produce_lurk_message_blob();

        let expectation = [
            Message::message_type(),
            0x04,
            0x00,
            'r' as u8,
            'e' as u8,
            'c' as u8,
            'i' as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            's' as u8,
            'e' as u8,
            'n' as u8,
            'd' as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            'm' as u8,
            'e' as u8,
            's' as u8,
            's' as u8,
        ];

        assert_eq!(data, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_changeroom_read() {
        let data = vec![0x08_u8, 0x00_u8];

        let (change_room, bytes_read) = ChangeRoom::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
        assert_eq!(change_room.room_number, 8);
    }

    #[test]
    fn test_changeroom_write() {
        let change_room = ChangeRoom { room_number: 8 };

        let data = change_room.produce_lurk_message_blob();

        let expectation = [ChangeRoom::message_type(), 0x08_u8, 0x00_u8];

        assert_eq!(data, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_fight_write() {
        let fight = Fight {};

        let expectation = [Fight::message_type()];

        let blob = fight.produce_lurk_message_blob();

        assert_eq!(blob, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_pvpfight_read() {
        let data = vec![
            't' as u8, 'a' as u8, 'r' as u8, 'g' as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let (message, _) = PvpFight::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(message.target, String::from("targ"));
    }

    #[test]
    fn test_pvpfight_write() {
        let pvpfight = PvpFight {
            target: String::from("targ"),
        };

        let data = pvpfight.produce_lurk_message_blob();

        let expectation = vec![
            PvpFight::message_type(),
            't' as u8,
            'a' as u8,
            'r' as u8,
            'g' as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        assert_eq!(data, expectation.to_vec());
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_loot_read() {
        let data = vec![
            'l' as u8, 'o' as u8, 'o' as u8, 't' as u8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let (message, bytes_read) = Loot::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
        assert_eq!(message.target, String::from("loot"));
    }

    #[test]
    fn test_loot_write() {
        let loot = Loot {
            target: String::from("loot"),
        };

        let data = loot.produce_lurk_message_blob();

        let expectation = vec![
            Loot::message_type(),
            'l' as u8,
            'o' as u8,
            'o' as u8,
            't' as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];

        assert_eq!(data, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_start_write() {
        let start = Start {};

        let blob = start.produce_lurk_message_blob();

        assert_eq!(blob, vec![Start::message_type()]);
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_error_read() {
        let data = vec![0x06, 0x03, 0x00, 'c' as u8, 'a' as u8, 't' as u8];

        let (error, bytes_read) = Error::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(bytes_read, data.len());
        assert_eq!(error.error_code, 6);
        assert_eq!(error.error_message, String::from("cat"))
    }

    #[test]
    fn test_error_write() {
        let error = Error {
            error_code: 6,
            error_message: String::from("cat"),
        };

        let blob = error.produce_lurk_message_blob();

        let expectation = vec![
            Error::message_type(),
            0x06,
            0x03,
            0x00,
            'c' as u8,
            'a' as u8,
            't' as u8,
        ];

        assert_eq!(blob, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_accept_read() {
        let data = vec![0x05 as u8];

        let (accept, bytes_read) = Accept::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
        assert_eq!(accept.action_type, 0x05_u8);
    }

    #[test]
    fn test_accept_write() {
        let accept = Accept { action_type: 5 };

        let blob = accept.produce_lurk_message_blob();

        let expectation = vec![Accept::message_type(), 0x05 as u8];

        assert_eq!(blob, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_room_read() {
        let data = vec![
            0x08, 0x00, 'r' as u8, 'o' as u8, 'o' as u8, 'm' as u8, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00, 'h' as u8, 'e' as u8,
            'l' as u8, 'l' as u8,
        ];

        let (room, bytes_read) = Room::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
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

        let blob = room.produce_lurk_message_blob();

        let expectation = vec![
            Room::message_type(),
            0x08,
            0x00,
            'r' as u8,
            'o' as u8,
            'o' as u8,
            'm' as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x04,
            0x00,
            'h' as u8,
            'e' as u8,
            'l' as u8,
            'l' as u8,
        ];

        assert_eq!(blob, expectation.to_vec());
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

        let (character, bytes_read) = Character::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
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

        let blob = character.produce_lurk_message_blob();

        let expectation = vec![
            Character::message_type(),
            'p' as u8,
            'l' as u8,
            'a' as u8,
            'y' as u8,
            0x00,
            0x00,
            0x00,
            0x00, // name
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0b10101000, // flags
            0xF0,
            0x00, // attack
            0x0F,
            0x00, // def
            0xAA,
            0x00, // regen
            0xFF,
            0x00, // health
            0xFF,
            0x00, // gold
            0x03,
            0x00, // room number
            0x04,
            0x00, // description
            'h' as u8,
            'e' as u8,
            'l' as u8,
            'l' as u8,
        ];

        assert_eq!(blob, expectation.to_vec());
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

        let (game, bytes_read) = Game::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
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

        let blob = game.produce_lurk_message_blob();

        let expectation = vec![
            Game::message_type(),
            0x00,
            0xFF, // init points
            0xFF,
            0x00, // stat limit
            0x04,
            0x00,
            'g' as u8,
            'a' as u8,
            'm' as u8,
            'e' as u8,
        ];

        assert_eq!(blob, expectation.to_vec());
    }

    /////////////////////////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test_leave_write() {
        let leave = Leave {};

        let blob = leave.produce_lurk_message_blob();

        let expectation = vec![Leave::message_type()];

        assert_eq!(blob, expectation.to_vec());
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

        let (connection, bytes_read) = Connection::parse_lurk_message(data.as_slice()).unwrap();

        assert_eq!(data.len(), bytes_read);
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

        let blob = connection.produce_lurk_message_blob();

        let expectation = vec![
            Connection::message_type(),
            0x03,
            0x00, // room number
            'r' as u8,
            'o' as u8,
            'o' as u8,
            'm' as u8,
            0x00,
            0x00,
            0x00,
            0x00, // name
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x04,
            0x00, // description
            'h' as u8,
            'e' as u8,
            'l' as u8,
            'l' as u8,
        ];

        assert_eq!(blob, expectation.to_vec());
    }
    /////////////////////////////////////////////////////////////////////////////////////////////////
}
