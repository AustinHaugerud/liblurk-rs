use protocol::read::LurkReadChannel;
use protocol::send::LurkSendChannel;
use protocol::protocol_message::*;
use uuid::Uuid;
use std::net::TcpStream;
use std::net::IpAddr;
use std::net::TcpListener;
use std::collections::HashMap;
use std::thread;
use std::time;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Client {
  stream : TcpStream,
  id : Uuid,
  active : bool,
}

impl Client {

  pub fn get_send_channel(&mut self) -> LurkSendChannel<TcpStream> {
    LurkSendChannel::new(&mut self.stream)
  }

  fn data_available(&self) -> bool {
    let mut buf = [0u8];
    let peek_result = self.stream.peek(&mut buf);
    if let Ok(num_read) = peek_result {
      return num_read > 0;
    }
    return false;
  }

  fn update(&mut self, callbacks : Arc<Mutex<Box<ServerCallbacks + Send>>>, server_access : &ServerAccess) -> Result<(), String> {

    if !self.data_available() {
      return Ok(());
    }

    println!("Data available");


    let mut callbacks_guard = callbacks.lock().map_err(|_| { String::from("Mutex poison error.") })?;

    let msg_result = self.pull_client_message();

    if msg_result.is_err() {
      return Ok(());
    }

    let (kind, data) = msg_result.unwrap();
    let send_channel = LurkSendChannel::new(&mut self.stream);
    //let mut context = ServerEventContext::new(server_access, send_channel, &self.id);
    let mut context = ServerEventContext {
      server: server_access,
      write_channel: send_channel,
      client_id: self.id.clone(),
    };

    match kind {
      LurkMessageKind::Message => {
        let (message, _) = Message::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_message(&mut context, &message);
      },
      LurkMessageKind::ChangeRoom => {
        let (message, _) = ChangeRoom::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_change_room(&mut context, &message);
      },
      LurkMessageKind::Fight => {
        let (message, _) = Fight::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_fight(&mut context, &message);
      },
      LurkMessageKind::PvPFight => {
        let (message, _) = PvpFight::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_pvp_fight(&mut context, &message);
      },
      LurkMessageKind::Loot => {
        let (message, _) = Loot::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_loot(&mut context, &message);
      },
      LurkMessageKind::Start => {
        let (message, _) = Start::parse_lurk_message(data.as_slice())?;
        callbacks_guard.on_start(&mut context, &message);
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
        callbacks_guard.on_character(&mut context, &message);
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

  fn pull_client_message(&mut self) -> Result<(LurkMessageKind, Vec<u8>), ()> {
    let mut read_channel = LurkReadChannel::new(&mut self.stream);

    fn read_fail(_ : ()) { }
    read_channel.read_next().map_err(read_fail)
  }
}

pub trait ServerCallbacks {
  fn on_connect(&mut self, context : &mut ServerEventContext);
  fn on_disconnect(&mut self, client_id : &Uuid);

  fn on_message(&mut self,     context : &mut ServerEventContext, message : &Message);
  fn on_change_room(&mut self, context : &mut ServerEventContext, change_room : &ChangeRoom);
  fn on_fight(&mut self,       context : &mut ServerEventContext, fight : &Fight);
  fn on_pvp_fight(&mut self,   context : &mut ServerEventContext, pvp_fight : &PvpFight);
  fn on_loot(&mut self,        context : &mut ServerEventContext, loot : &Loot);
  fn on_start(&mut self,       context : &mut ServerEventContext, start : &Start);
  fn on_character(&mut self,   context : &mut ServerEventContext, character : &Character);
  fn on_leave(&mut self, client_id : &Uuid);
}

pub struct ServerAccess {
  clients : Arc<Mutex<HashMap<Uuid, Arc<Mutex<Client>>>>>,
}

pub struct ServerEventContext<'a> {
  server : &'a ServerAccess,
  write_channel : LurkSendChannel<'a, TcpStream>,
  client_id : Uuid,
}

impl<'a> ServerEventContext<'a> {

  pub fn get_client(&self, id : &Uuid) -> Option<Arc<Mutex<Client>>> {
    let guard = self.server.clients.lock().expect("Client retrieval: poison error");

    if guard.contains_key(id) {
      return Some(guard[id].clone());
    }

    None
  }

  pub fn get_send_channel(&mut self) -> &mut LurkSendChannel<'a, TcpStream> {
    &mut self.write_channel
  }

  pub fn get_client_id(&self) -> Uuid {
    self.client_id.clone()
  }

}

pub struct Server {
  clients : Arc<Mutex<HashMap<Uuid, Arc<Mutex<Client>>>>>,
  callbacks : Arc<Mutex<Box<ServerCallbacks + Send>>>,
  running : bool,
  server_address : (IpAddr, u16)
}

impl Server {
  pub fn create((host, port) : (IpAddr, u16), behavior : Box<ServerCallbacks + Send>) -> Result<Server, String> {
    Ok(Server {
      clients : Arc::new(Mutex::new(HashMap::new())),
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
            active : true,
          };

          if client.stream.set_nonblocking(true).is_err() {
            println!("Failed to set client stream to non-blocking");
          }
          else {
            if self.add_client(client).is_err() {
              println!("Failed to add client.");
            }
          }
        },
        Err(_) => {},
      };
    }
  }

  fn add_client(&mut self, client : Client) -> Result<(), String> {

    let key = client.id.clone();
    {
      let mut clients_guard = self.clients.lock().expect("poison error!");
      clients_guard.insert(key, Arc::new(Mutex::new(client)));
      let mut client_ref = clients_guard.get(&key).unwrap().lock().unwrap();
      let mut guard = self.callbacks.lock().map_err(|_| { String::from("Failed to lock for client addition.") })?;
      let send_channel = LurkSendChannel::new(&mut client_ref.stream);
      let mut context = ServerEventContext {
        server: &ServerAccess { clients: self.clients.clone() },
        write_channel: send_channel,
        client_id: key.clone(),
      };

      guard.on_connect(&mut context);
    }

    let clients_guard = self.clients.lock().expect("poison error!");

    let client_ref = clients_guard[&key].clone();
    let callbacks = self.callbacks.clone();
    let server_access = ServerAccess { clients : self.clients.clone() };

    thread::spawn(move || {

      loop {

        // Leave this in the inner block, the mutex needs to unlock during the sleep time!
        {
          let mut guard = client_ref.lock().unwrap();

          if !guard.active {
            break;
          }

          let result = guard.update(callbacks.clone(), &server_access);
          if result.is_err() {
            println!("Error encountered.");
          }
        }

        thread::sleep(time::Duration::from_millis(10));
      }
    });

    Ok(())
  }
}
