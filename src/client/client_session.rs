use std::net::TcpStream;
use std::net::IpAddr;
use protocol::protocol_message::*;
use protocol::read::LurkReadChannel;
use protocol::send::LurkSendChannel;

pub struct ClientSession {
  stream : TcpStream,
  callbacks : ClientSessionCallbacks,
}

impl ClientSession {

  pub fn connect((host, port) : (IpAddr, u16)) -> Result<ClientSession, ()> {
    match TcpStream::connect((host, port)) {
      Ok(t) => {
        let client = ClientSession {
          stream: t,
          callbacks: ClientSessionCallbacks::default()
        };

        Ok(client)
      },
      Err(_) => Err(())
    }
  }

  pub fn set_on_message(&mut self, func : Box<Fn(&mut LurkSendChannel<TcpStream>, &Message)>) {
    self.callbacks.message_callback = func;
  }

  pub fn pull_message(&mut self) -> Result<(LurkMessageKind, Vec<u8>), String> {
    let mut read_channel = LurkReadChannel::new(&mut self.stream);

    fn read_fail(_ : ()) -> String { String::from("Read channel failure") }
    read_channel.read_next().map_err(read_fail)
  }

  pub fn update(&mut self) -> Result<(), String> {

    let (kind, data) = self.pull_message()?;

    match kind {
      LurkMessageKind::Message => {
        let (message, _) = Message::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.message_callback.as_ref()(&mut send_channel, &message);
      },
      LurkMessageKind::ChangeRoom => {
        ChangeRoom::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Fight => {
        Fight::parse_lurk_message((data.as_slice()))?;
      },
      LurkMessageKind::PvPFight => {
        PvpFight::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Loot => {
        Loot::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Start => {
        Start::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Error => {
        let (message, _) = Error::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.error_callback.as_ref()(&mut send_channel, &message);
      },
      LurkMessageKind::Accept => {
        let (message, _) = Accept::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.accept_callback.as_ref()(&mut send_channel, &message);
      },
      LurkMessageKind::Room => {
        let (message, _) = Room::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.room_callback.as_ref()(&mut send_channel, &message);
      },
      LurkMessageKind::Character => {
        let (message, _) = Character::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.character_callback.as_ref()(&mut send_channel, &message);
      },
      LurkMessageKind::Game => {
        let (message, _) = Game::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.game_callback.as_ref()(&mut send_channel, &message);
      },
      LurkMessageKind::Leave => {
      },
      LurkMessageKind::Connection => {
        let (message, _) = Connection::parse_lurk_message(data.as_slice())?;
        let mut send_channel = LurkSendChannel::new(&mut self.stream);
        self.callbacks.connection_callback.as_ref()(&mut send_channel, &message);
      },
    };

    Ok(())
  }
}

struct ClientSessionCallbacks {
  message_callback    : Box<Fn(&mut LurkSendChannel<TcpStream>, &Message)>,
  error_callback      : Box<Fn(&mut LurkSendChannel<TcpStream>, &Error)>,
  accept_callback     : Box<Fn(&mut LurkSendChannel<TcpStream>, &Accept)>,
  room_callback       : Box<Fn(&mut LurkSendChannel<TcpStream>, &Room)>,
  character_callback  : Box<Fn(&mut LurkSendChannel<TcpStream>, &Character)>,
  game_callback       : Box<Fn(&mut LurkSendChannel<TcpStream>, &Game)>,
  connection_callback : Box<Fn(&mut LurkSendChannel<TcpStream>, &Connection)>,
}

impl ClientSessionCallbacks {
  fn default() -> ClientSessionCallbacks {

    ClientSessionCallbacks {
      message_callback    : Box::new(|_, _| {}),
      error_callback      : Box::new(|_, _| {}),
      accept_callback     : Box::new(|_, _| {}),
      room_callback       : Box::new(|_, _| {}),
      character_callback  : Box::new(|_, _| {}),
      game_callback       : Box::new(|_, _| {}),
      connection_callback : Box::new(|_, _| {}),
    }
  }
}
