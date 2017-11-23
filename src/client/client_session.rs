use std::net::TcpStream;
use std::net::IpAddr;

use ::protocol::protocol_message::LurkMessageFrame;

pub struct ClientSession {
  stream : TcpStream,
}

impl ClientSession {
  pub fn connect((host, port) : (IpAddr, u16)) -> Result<ClientSession, ()> {
    match TcpStream::connect((host, port)) {
      Ok(T) => Ok(ClientSession { stream : T }),
      Err(_) => Err(())
    }
  }

}
