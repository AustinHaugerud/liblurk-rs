use protocol::primitive_parse::parse_u16l;

use protocol::protocol_message::{LurkMessageParse, LurkMessageKind, LurkMessage, Fight, Start, Leave, Message, ChangeRoom, PvpFight, Loot, Error, Accept, Room, Character, Game, Connection, LurkMessageBlobify};
use tokio::codec::{Decoder, Encoder};
use bytes::BytesMut;
use std::mem::size_of;

const LURK_FIXED_STRING_SIZE: u16 = 32;

pub struct LurkMessageReadError;

impl From<std::io::Error> for LurkMessageReadError {
    fn from(_ : std::io::Error) -> Self {
        LurkMessageReadError
    }
}

pub struct LurkMessageCodec;

pub type LurkMessageReadResult = Result<Option<LurkMessage>, LurkMessageReadError>;

impl Encoder for LurkMessageCodec {
    type Item = LurkMessage;
    type Error = std::io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        use bytes::BufMut;
        let bytes = match item {
            LurkMessage::Message(m) => m.produce_lurk_message_blob(),
            LurkMessage::ChangeRoom(m) => m.produce_lurk_message_blob(),
            LurkMessage::Fight(m) => m.produce_lurk_message_blob(),
            LurkMessage::PvpFight(m) => m.produce_lurk_message_blob(),
            LurkMessage::Loot(m) => m.produce_lurk_message_blob(),
            LurkMessage::Start(m) => m.produce_lurk_message_blob(),
            LurkMessage::Error(m) => m.produce_lurk_message_blob(),
            LurkMessage::Accept(m) => m.produce_lurk_message_blob(),
            LurkMessage::Room(m) => m.produce_lurk_message_blob(),
            LurkMessage::Character(m) => m.produce_lurk_message_blob(),
            LurkMessage::Game(m) => m.produce_lurk_message_blob(),
            LurkMessage::Leave(m) => m.produce_lurk_message_blob(),
            LurkMessage::Connection(m) => m.produce_lurk_message_blob()
        };

        dst.reserve(bytes.len());
        dst.put(bytes);
        Ok(())
    }
}

impl Decoder for LurkMessageCodec {
    type Item = LurkMessage;
    type Error = LurkMessageReadError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {

        if let Some(type_byte) = src.get(0).cloned() {
            let kind = LurkMessageKind::from_code(type_byte).map_err(|_| LurkMessageReadError)?;

            match kind {
                LurkMessageKind::Message => decode_message(src),
                LurkMessageKind::ChangeRoom => decode_change_room(src),
                LurkMessageKind::Fight => {
                    src.split_to(1);
                    Ok(Some(LurkMessage::Fight(Fight)))
                },
                LurkMessageKind::PvpFight => decode_pvp_fight(src),
                LurkMessageKind::Loot => decode_loot(src),
                LurkMessageKind::Start => {
                    src.split_to(1);
                    Ok(Some(LurkMessage::Start(Start)))
                },
                LurkMessageKind::Error => decode_error(src),
                LurkMessageKind::Accept => decode_accept(src),
                LurkMessageKind::Room => decode_room(src),
                LurkMessageKind::Character => decode_character(src),
                LurkMessageKind::Game => decode_game(src),
                LurkMessageKind::Leave => {
                    src.split_to(1);
                    Ok(Some(LurkMessage::Leave(Leave)))
                },
                LurkMessageKind::Connection => decode_connection(src),
            }
        }
        else {
            Ok(None)
        }
    }
}

fn decode_message(src: &mut BytesMut) -> LurkMessageReadResult {
    if let Some(len) = decode_u16(src, 1) {
        let total_len_needed =
            1 + 2 + (2 * LURK_FIXED_STRING_SIZE as usize) + len as usize;

        if src.len() >= total_len_needed {
            let bytes = src.split_to(total_len_needed).to_vec();
            let message = Message::parse_lurk_message(&bytes.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
            Ok(Some(LurkMessage::Message(message.0)))
        }
        else {
            Ok(None)
        }
    }
    else {
        Ok(None)
    }
}

fn decode_change_room(src: &mut BytesMut) -> LurkMessageReadResult {
    const PACKET_SIZE: usize = 3;
    if src.len() >= PACKET_SIZE {
        let data = src.split_to(PACKET_SIZE).to_vec();
        let change_room = ChangeRoom::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
        Ok(Some(LurkMessage::ChangeRoom(change_room.0)))
    }
    else {
        Ok(None)
    }
}

fn decode_pvp_fight(src: &mut BytesMut) -> LurkMessageReadResult {
    const PACKET_SIZE: usize = LURK_FIXED_STRING_SIZE as usize + 1;
    if src.len() >= PACKET_SIZE {
        let data = src.split_to(PACKET_SIZE).to_vec();
        let pvp_fight = PvpFight::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
        Ok(Some(LurkMessage::PvpFight(pvp_fight.0)))
    }
    else {
        Ok(None)
    }
}

fn decode_loot(src: &mut BytesMut) -> LurkMessageReadResult {
    const PACKET_SIZE: usize = LURK_FIXED_STRING_SIZE as usize + 1;
    if src.len() >= PACKET_SIZE {
        let data = src.split_to(PACKET_SIZE).to_vec();
        let loot = Loot::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
        Ok(Some(LurkMessage::Loot(loot.0)))
    }
    else {
        Ok(None)
    }
}

fn decode_error(src: &mut BytesMut) -> LurkMessageReadResult {
    if let Some(err_msg_len) = decode_u16(src, 2) {
        let total_bytes_needed = 4 + err_msg_len as usize;
        if src.len() >= total_bytes_needed {
            let data = src.split_to(total_bytes_needed).to_vec();
            let error = Error::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
            Ok(Some(LurkMessage::Error(error.0)))
        }
        else {
            Ok(None)
        }
    }
    else {
        Ok(None)
    }
}

fn decode_accept(src: &mut BytesMut) -> LurkMessageReadResult {
    const PACKET_SIZE: usize = 2;
    if src.len() >= PACKET_SIZE {
        let data = src.split_to(PACKET_SIZE).to_vec();
        let accept = Accept::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
        Ok(Some(LurkMessage::Accept(accept.0)))
    }
    else {
        Ok(None)
    }
}

fn decode_room(src: &mut BytesMut) -> LurkMessageReadResult {
    if let Some(desc_len) = decode_u16(src, 35) {
        let total_needed = 1 + 2 + 32 + 2 + desc_len as usize;
        if src.len() >= total_needed {
            let data = src.split_to(total_needed).to_vec();
            let room = Room::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
            Ok(Some(LurkMessage::Room(room.0)))
        }
        else {
            Ok(None)
        }
    }
    else {
        Ok(None)
    }
}

fn decode_character(src: &mut BytesMut) -> LurkMessageReadResult {
    if let Some(desc_len) = decode_u16(src, 46) {
        let total_bytes_needed =
            1 + 32 + 1 + 2 + 2 + 2 + 2 + 2 + 2 + 2 + desc_len as usize;
        if src.len() >= total_bytes_needed {
            let data = src.split_to(total_bytes_needed).to_vec();
            let character = Character::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
            Ok(Some(LurkMessage::Character(character.0)))
        }
        else {
            Ok(None)
        }
    }
    else {
        Ok(None)
    }
}

fn decode_game(src: &mut BytesMut) -> LurkMessageReadResult {
    if let Some(desc_len) = decode_u16(src, 5) {
        let total_bytes_needed = 1 + 2 + 2 + 2 + desc_len as usize;
        if src.len() >= total_bytes_needed {
            let data = src.split_to(total_bytes_needed).to_vec();
            let game = Game::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
            Ok(Some(LurkMessage::Game(game.0)))
        }
        else {
            Ok(None)
        }
    }
    else {
        Ok(None)
    }
}

fn decode_connection(src: &mut BytesMut) -> LurkMessageReadResult {
    if let Some(desc_len) = decode_u16(src, 35) {
        let total_needed = 1 + 2 + 32 + 2 + desc_len as usize;
        if src.len() >= total_needed {
            let data = src.split_to(total_needed).to_vec();
            let connection = Connection::parse_lurk_message(&data.as_slice()[1..]).map_err(|_| LurkMessageReadError)?;
            Ok(Some(LurkMessage::Connection(connection.0)))
        }
        else {
            Ok(None)
        }
    }
    else {
        Ok(None)
    }
}

fn decode_u16(src: &mut BytesMut, index: usize) -> Option<u16> {
    if src.len() >= index + size_of::<u16>() {
        let b1 = *src.get(index).unwrap();
        let b2 = *src.get(index + 1).unwrap();
        Some(parse_u16l(&[b1, b2]))
    }
    else {
        None
    }
}
