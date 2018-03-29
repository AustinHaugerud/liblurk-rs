use protocol::read::LurkReadChannel;
use protocol::send::LurkSendChannel;
use protocol::protocol_message::*;
use uuid::Uuid;
use std::net::TcpStream;
use std::net::IpAddr;
use std::net::TcpListener;
use std::collections::HashMap;
use std::thread;
use std::sync::Arc;
use std::sync::Mutex;

struct Client {
  stream : TcpStream,
  id : Uuid,
  active : bool,
}

impl Client {

  pub fn main(&mut self, callbacks : Arc<Mutex<Box<ServerCallbacks + Send>>>) {
    while self.active {
      let result = self.update(callbacks.clone());
      if result.is_err() {
        println!("Error: {}", result.unwrap_err());
      }
    }
  }

  fn update(&mut self, callbacks : Arc<Mutex<Box<ServerCallbacks + Send>>>) -> Result<(), String> {

    let mut callbacks_guard = callbacks.lock().map_err(|_| { String::from("Mutex poison error.") })?;

    let (kind, data) = self.pull_client_message()?;
    let mut send_channel = LurkSendChannel::new(&mut self.stream);

    match kind {
      LurkMessageKind::Message => {
        let (message, _) = Message::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_message(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::ChangeRoom => {
        let (message, _) = ChangeRoom::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_change_room(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::Fight => {
        let (message, _) = Fight::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_fight(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::PvPFight => {
        let (message, _) = PvpFight::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_pvp_fight(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::Loot => {
        let (message, _) = Loot::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_loot(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::Start => {
        let (message, _) = Start::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_start(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::Error => {
        Error::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Accept => {
        Accept::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Room => {
        Room::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Character => {
        let (message, _) = Character::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_character(&mut send_channel, &self.id, &message);
      },
      LurkMessageKind::Game => {
        Game::parse_lurk_message(data.as_slice())?;
      },
      LurkMessageKind::Leave => {
        callbacks_guard.on_leave(&self.id);
      },
      LurkMessageKind::Connection => {
        Connection::parse_lurk_message(data.as_slice())?;
      },
    };

    Ok(())
  }

  fn pull_client_message(&mut self) -> Result<(LurkMessageKind, Vec<u8>), String> {
    let mut read_channel = LurkReadChannel::new(&mut self.stream);

    fn read_fail(_ : ()) -> String { String::from("Read channel failure.") }
    read_channel.read_next().map_err(read_fail)
  }
}

pub trait ServerCallbacks {
  fn on_connect(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid);
  fn on_disconnect(&mut self, channel : &Uuid);

  fn on_message(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, message : &Message);
  fn on_change_room(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, change_room : &ChangeRoom);
  fn on_fight(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, fight : &Fight);
  fn on_pvp_fight(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, fight : &PvpFight);
  fn on_loot(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, loot : &Loot);
  fn on_start(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, fight : &Start);
  fn on_character(&mut self, channel : &mut LurkSendChannel<TcpStream>, id : &Uuid, character : &Character);
  fn on_leave(&mut self, id : &Uuid);
}

pub struct Server {
  clients : HashMap<Uuid, Arc<Mutex<Client>>>,
  callbacks : Arc<Mutex<Box<ServerCallbacks + Send>>>,
  running : bool,
  server_address : (IpAddr, u16)
}

impl Server {
  pub fn create((host, port) : (IpAddr, u16), behavior : Box<ServerCallbacks + Send>) -> Result<Server, String> {
    Ok(Server {
      clients : HashMap::new(),
      callbacks : Arc::new(Mutex::new(behavior)),
      running : false,
      server_address : (host, port)
    })
  }

  pub fn start(&mut self) -> bool {

    let listener_result = TcpListener::bind(self.server_address);

    if listener_result.is_err() {
      return false;
    }

    self.running = true;
    self.main(listener_result.unwrap());

    return true;
  }

  pub fn stop(&mut self) {
    self.running = false;
  }

  fn main(&mut self, listener : TcpListener) {
    for client_request in listener.incoming() {

      if !self.running {
        break;
      }


      match client_request {
        Ok(t) => {
          let client = Client {
            stream : t,
            id : Uuid::new_v4(),
            active : false,
          };

          if self.add_client(client).is_err() {
            println!("Failed to add client.");
          }
        },
        Err(_) => {},
      };
    }
  }

  fn add_client(&mut self, client : Client) -> Result<(), String> {

    let key = client.id.clone();
    {
      self.clients.insert(key, Arc::new(Mutex::new(client)));
      let mut client_ref = self.clients.get(&key).unwrap().lock().unwrap();
      let mut guard = self.callbacks.lock().map_err(|_| { String::from("Failed to lock for client addition.") })?;
      let mut send_channel = LurkSendChannel::new(&mut client_ref.stream);
      guard.on_connect(&mut send_channel, &key);
    }

    let client_ref = self.clients[&key].clone();
    let callbacks = self.callbacks.clone();

    thread::spawn(move || {
      let mut guard = client_ref.lock().unwrap();
      guard.main(callbacks);
    });

    Ok(())
  }
}
