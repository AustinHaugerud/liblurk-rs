pub mod client_session;

use self::client_session::ClientSession;

pub fn connect(hostname: String, port: u16) -> ClientSession
{
  ClientSession {}
}

