use std::net::TcpStream;
use std::net::IpAddr;
use std::io::Write;
use std::io::Error;

use ::protocol::protocol_message::LurkMessageFrame;
use ::protocol::receive::*;
use ::protocol::send::*;
use ::protocol::extractor::*;

pub struct ClientSession {
  stream  : TcpStream,
  receive : EventReceiver,
  host    : IpAddr,
  port    : u16,
}

impl ClientSession {
  pub fn connect((host, port) : (IpAddr, u16)) -> Result<ClientSession, ()> {
    match TcpStream::connect((host, port)) {
      Ok(T) => Ok(ClientSession {
        stream : T, receive : EventReceiver::new() ,
        host,
        port,
      }),
      Err(_) => Err(())
    }
  }

  pub fn send_message(&mut self, message : LurkMessageFrame) -> Result<usize, Error> {
    send(&mut self.stream as &mut Write, message)
  }

  pub fn bind_event_handler(&mut self, handler : fn(&LurkEvent)) {
    self.receive.bind_handler(handler);
  }

  pub fn get_host(&self) -> IpAddr {
    self.host.clone()
  }

  pub fn get_port(&self) -> u16 {
    self.port
  }

  fn update(&mut self) {
    match extract_message(&mut self.stream as &mut Read) {
      Ok(T) => self.receive.distribute_event(LurkEvent { frame : T }),
      Err(E) => (),
    };
  }
}
