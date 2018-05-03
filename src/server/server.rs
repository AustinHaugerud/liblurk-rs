use protocol::read::LurkReadChannel;
use protocol::send::LurkSendChannel;
use protocol::protocol_message::*;
use uuid::Uuid;
use std::io::Read;
use std::net::TcpStream;
use std::net::IpAddr;
use std::net::TcpListener;
use std::collections::HashMap;
use std::thread;
use std::time;
use std::sync::Arc;
use std::sync::Mutex;
use std::net::Shutdown;

#[derive(Eq, PartialEq)]
enum ClientHealthState {
    Good,
    Bad,
    Close,
}

struct WriteQueueItem {
    packet: Box<LurkMessageBlobify + Send>,
    target: Uuid,
}

impl WriteQueueItem {
    pub fn new<T>(packet_impl: T, target: Uuid) -> WriteQueueItem
    where
        T: 'static + LurkMessageBlobify + Send,
    {
        WriteQueueItem {
            packet: Box::new(packet_impl),
            target,
        }
    }
}

pub struct Client {
    stream: TcpStream,
    id: Uuid,
    active: bool,
    health_state: ClientHealthState,
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

    fn update(
        &mut self,
        callbacks: Arc<Mutex<Box<ServerCallbacks + Send>>>,
        server_access: &ServerAccess,
    ) -> Result<bool, String> {
        if !self.data_available() {
            return Ok(true);
        }

        let mut callbacks_guard = callbacks
            .lock()
            .map_err(|_| String::from("Mutex poison error."))?;

        let (kind, data) = self.pull_client_message()
            .map_err(|_| String::from("Failed to pull client message"))?;

        let mut context = ServerEventContext {
            server: server_access,
            client_id: self.id.clone(),
        };

        match kind {
            LurkMessageKind::Message => {
                let (message, _) = Message::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_message(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::ChangeRoom => {
                let (message, _) = ChangeRoom::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_change_room(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::Fight => {
                let (message, _) = Fight::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_fight(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::PvPFight => {
                let (message, _) = PvpFight::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_pvp_fight(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::Loot => {
                let (message, _) = Loot::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_loot(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::Start => {
                let (message, _) = Start::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_start(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::Error => {
                Error::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Accept => {
                Accept::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Room => {
                Room::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Character => {
                let (message, _) = Character::parse_lurk_message(data.as_slice())?;
                match callbacks_guard.on_character(&mut context, &message) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                }
            }
            LurkMessageKind::Game => {
                Game::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Leave => {
                match callbacks_guard.on_leave(&self.id) {
                    Err(_) => self.health_state = ClientHealthState::Bad,
                    _ => {}
                };
                self.stream.shutdown(Shutdown::Both).map_err(|_| "Failed to shutdown.".to_string())?;
            },
            LurkMessageKind::Connection => {
                Connection::parse_lurk_message(data.as_slice())?;
            }
        };

        Ok(true)
    }

    fn pull_client_message(&mut self) -> Result<(LurkMessageKind, Vec<u8>), ()> {
        let mut read_channel = LurkReadChannel::new(&mut self.stream);

        fn read_fail(_: ()) {}
        read_channel.read_next().map_err(read_fail)
    }
}

pub type LurkServerError = Result<(), ()>;

pub trait ServerCallbacks {
    fn on_connect(&mut self, context: &mut ServerEventContext) -> LurkServerError;
    fn on_disconnect(&mut self, client_id: &Uuid);

    fn on_message(
        &mut self,
        context: &mut ServerEventContext,
        message: &Message,
    ) -> LurkServerError;
    fn on_change_room(
        &mut self,
        context: &mut ServerEventContext,
        change_room: &ChangeRoom,
    ) -> LurkServerError;
    fn on_fight(&mut self, context: &mut ServerEventContext, fight: &Fight) -> LurkServerError;
    fn on_pvp_fight(
        &mut self,
        context: &mut ServerEventContext,
        pvp_fight: &PvpFight,
    ) -> LurkServerError;
    fn on_loot(&mut self, context: &mut ServerEventContext, loot: &Loot) -> LurkServerError;
    fn on_start(&mut self, context: &mut ServerEventContext, start: &Start) -> LurkServerError;
    fn on_character(
        &mut self,
        context: &mut ServerEventContext,
        character: &Character,
    ) -> LurkServerError;
    fn on_leave(&mut self, client_id: &Uuid) -> LurkServerError;

    fn update(&mut self);
}

pub struct ServerAccess {
    write_items_queue: Arc<Mutex<Vec<WriteQueueItem>>>,
}

pub struct ServerEventContext<'a> {
    server: &'a ServerAccess,
    client_id: Uuid,
}

impl<'a> ServerEventContext<'a> {
    pub fn get_client_id(&self) -> Uuid {
        self.client_id.clone()
    }

    pub fn enqueue_message<T>(&mut self, message: T, target: Uuid)
    where
        T: 'static + LurkMessageBlobify + Send,
    {
        match self.server.write_items_queue.lock() {
            Ok(mut queue) => {
                queue.push(WriteQueueItem::new(message, target));
            }
            Err(_) => {
                println!("Could not enqueue message.");
            }
        };
    }

    pub fn enqueue_message_this<T>(&mut self, message: T)
    where
        T: 'static + LurkMessageBlobify + Send,
    {
        let id = self.client_id.clone();
        self.enqueue_message(message, id)
    }
}

pub struct Server {
    clients: Arc<Mutex<HashMap<Uuid, Arc<Mutex<Client>>>>>,
    callbacks: Arc<Mutex<Box<ServerCallbacks + Send>>>,
    running: Arc<Mutex<bool>>,
    server_address: (IpAddr, u16),
    write_items_queue: Arc<Mutex<Vec<WriteQueueItem>>>,
}

impl Server {
    pub fn create(
        (host, port): (IpAddr, u16),
        behavior: Box<ServerCallbacks + Send>,
    ) -> Result<Server, String> {
        Ok(Server {
            clients: Arc::new(Mutex::new(HashMap::new())),
            callbacks: Arc::new(Mutex::new(behavior)),
            running: Arc::new(Mutex::new(false)),
            server_address: (host, port),
            write_items_queue: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub fn start(&mut self) -> Result<(), ()> {
        let listener = TcpListener::bind(self.server_address).map_err(|_| ())?;

        match self.running.lock() {
            Ok(mut running) => {
                *running = true;
            }
            Err(_) => {
                println!("Failed to set server to running.");
                return Err(());
            }
        };

        self.main(listener);

        Ok(())
    }

    pub fn stop(&mut self) {
        match self.running.lock() {
            Ok(mut running) => {
                *running = false;
            }
            Err(_) => {
                println!("Failed to stop server.");
            }
        }
    }

    fn main(&mut self, listener: TcpListener) {
        let clients = self.clients.clone();
        let write_items_queue = self.write_items_queue.clone();
        let running = self.running.clone();
        let callbacks = self.callbacks.clone();

        // Process clients write queue
        thread::spawn(move || loop {

            match running.lock() {
                Ok(status) => {
                    if *status == false {
                        println!("Processing thread ending.");
                        break;
                    }
                }
                Err(_) => {
                    println!(
                        "Critical: Failed to check server running status during queue processing."
                    );
                    break;
                }
            };

            match callbacks.lock() {
                Ok(mut c) => c.update(),
                Err(_) => {},
            };


            // We move items out of the queue so that the lock for it isn't needed at the same time
            // as a lock for a client. Clients have to write to the queue also, so we're
            // avoiding a deadlock with this. If the client tries to write during this lock,
            // the queue will have been emptied first, then the new item enqueued, then it will be processed
            // the next round which is fine.
            let mut queue = vec![];

            match write_items_queue.lock() {
                Ok(mut q) => {
                    for item in q.drain(..) {
                        queue.push(item);
                    }
                }
                Err(_) => {
                    println!("Critical: Write queue processing poison.");
                }
            };


            match clients.lock() {
                Ok(mut clients) => {
                    for item in queue.iter() {
                        if let Some(clientg) = clients.get_mut(&item.target) {
                            match clientg.lock() {
                                Ok(mut client) => {
                                    match client.get_send_channel().write_message_uptr(&item.packet)
                                    {
                                        Ok(_) => {}
                                        Err(_) => {
                                            println!("Failed to write to client.");
                                            client.health_state = ClientHealthState::Bad;
                                        }
                                    }
                                }
                                Err(_) => {
                                    println!("Failed to lock onto client for queue processing..")
                                }
                            }
                        } else {
                            println!("Invalid target in queue processing.");
                        }
                    }
                }
                Err(_) => {
                    println!("Failed lock clients for write queue processing.");
                }
            }
        });

        for client_request in listener.incoming() {
            match self.running.lock() {
                Ok(status) => {
                    if *status == false {
                        break;
                    }
                }
                Err(_) => {
                    println!("Critical: Failed to get server running status during listening.");
                    break;
                }
            }

            match client_request {
                Ok(t) => {
                    let mut client = Client {
                        stream: t,
                        id: Uuid::new_v4(),
                        active: true,
                        health_state: ClientHealthState::Good,
                    };

                    // Non-blocking disabled currently, instead we just peek to see if data is available per loop iteration
                    if client.stream.set_nonblocking(false).is_err() {
                        println!("Failed to set client stream to blocking");
                    } else {
                        let id = client.id.clone();
                        match self.add_client(client) {
                            Ok(_) => {}
                            Err(e) => {
                                client.health_state = ClientHealthState::Bad;
                                println!("Failed to add client: {}", e);
                                match self.remove_client(&id) {
                                    Ok(_) => {}
                                    Err(_) => {
                                        println!("Failed to drop bad client addition.");
                                    }
                                }
                            }
                        };
                    }
                }
                Err(e) => {
                    println!("Error accepting incoming connection: {}", e);
                }
            };
        }
    }

    fn remove_client(&mut self, id: &Uuid) -> Result<(), String> {
        let mut clients_guard = self.clients
            .lock()
            .map_err(|_| String::from("Poison error!"))?;
        clients_guard.remove(&id);
        Ok(())
    }

    fn add_client(&mut self, client: Client) -> Result<(), String> {
        let key = client.id.clone();
        {
            let mut clients_guard = self.clients
                .lock()
                .map_err(|_| String::from("Poison error!"))?;

            clients_guard.insert(key, Arc::new(Mutex::new(client)));


            let mut guard = self.callbacks
                .lock()
                .map_err(|_| String::from("Failed to lock for client addition."))?;

            let mut context = ServerEventContext {
                server: &ServerAccess {
                    write_items_queue: self.write_items_queue.clone(),
                },
                client_id: key.clone(),
            };

            guard
                .on_connect(&mut context)
                .map_err(|_| String::from("On connect callback error"))?;
        }

        let clients_guard = self.clients
            .lock()
            .map_err(|_| String::from("Poison error!"))?;

        let client_ref = clients_guard[&key].clone();
        let callbacks = self.callbacks.clone();
        let server_access = ServerAccess {
            write_items_queue: self.write_items_queue.clone(),
        };

        thread::spawn(move || {
            loop {
                // Leave this in the inner block, the mutex needs to unlock during the sleep time!
                {
                    match client_ref.lock() {
                        Ok(mut guard) => {
                            if guard.health_state == ClientHealthState::Bad
                                || guard.health_state == ClientHealthState::Close
                            {
                                guard.active = false;
                            }

                            if !guard.active {
                                break;
                            }

                            match guard.update(callbacks.clone(), &server_access) {
                                Ok(continue_status) => {
                                    if !continue_status {
                                        guard.health_state = ClientHealthState::Close;
                                        println!("Client {} disconnected.", guard.id);
                                    }
                                }
                                Err(e) => {
                                    println!("Error encountered: {}", e);
                                    match guard.get_send_channel().write_message(
                                        Error::other("Malformed communication.".to_string())
                                            .unwrap(),
                                    ) {
                                        Ok(_) => {}
                                        Err(_) => guard.health_state = ClientHealthState::Bad,
                                    }

                                    let mut buf = vec![];
                                    // Try to clear the buffer to reset communications
                                    match guard.stream.read_to_end(&mut buf) {
                                        Ok(_) => {}
                                        Err(_) => {
                                            guard.health_state = ClientHealthState::Bad;
                                        }
                                    }
                                }
                            };
                        }

                        Err(_) => {}
                    };
                }

                thread::sleep(time::Duration::from_millis(10));
            }
        });

        Ok(())
    }
}
