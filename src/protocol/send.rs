use std::io::Write;
use ::protocol::protocol_message::LurkMessageFrame;

use std::io::Error;

pub fn send(writer : &mut Write, message : LurkMessageFrame) -> Result<usize, Error> {
  writer.write(message.message_data.as_slice())
}
