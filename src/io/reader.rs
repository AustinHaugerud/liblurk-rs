use ::protocol::protocol_message::FromLurkMessageFrame;

pub trait LurkReader
{
  fn read_message<F>() -> super::Result<F> where F: FromLurkMessageFrame<F>;
}

