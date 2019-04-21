use tokio::net::TcpListener;
use tokio::prelude::*;
use server::callbacks::{ServerCallbacks, CallbacksWrapper, Callbacks};
use std::collections::HashMap;
use uuid::Uuid;
use server::client_session::ClientSession;
use std::sync::mpsc::Receiver;
use tokio::codec::Framed;
use protocol::codec::LurkMessageCodec;
use server::context::ServerEventContext;
use server::server_access::{WriteContext, ServerAccess};
use protocol::protocol_message::LurkMessage;
use std::net::SocketAddr;

pub struct Server<T> where T: 'static + ServerCallbacks + Send {
    listener: TcpListener,
    clients: HashMap<Uuid, ClientSession>,
    callbacks: Callbacks<T>,
    write_context: WriteContext,
}

impl<T> Server<T> where T: 'static + ServerCallbacks + Send {

    pub fn new(addr: SocketAddr, behaviour: T) -> Result<Server<T>, ()> {
        let listener = TcpListener::bind(&addr).map_err(|_| ())?;
        Ok(Server {
            listener,
            clients: HashMap::new(),
            callbacks: CallbacksWrapper::new(behaviour),
            write_context: ServerAccess::new(),
        })
    }

    pub fn start(&mut self, close_receiver: Receiver<()>) -> Result<(), ()> {
        loop {
            if let Ok(_) = close_receiver.try_recv() {
                return Ok(());
            }
            self.main()?;
        }
    }

    pub fn main(&mut self) -> Result<(), ()> {
        self.async_accept_connections()?;
        self.async_poll_clients()?;
        self.async_poll_clients_writing();
        self.process_write_queue()
    }

    fn async_accept_connections(&mut self) -> Result<(), ()> {
        loop {
            let poll_result = self.listener.poll_accept()
                .map_err(|_| ())?;

            if let Async::Ready((stream, _)) = poll_result {
                let mut framed = Framed::new(stream, LurkMessageCodec);
                let client = ClientSession::new(framed);
                self.clients.insert(client.get_id(), client);
            }
            else {
                break;
            }
        }
        Ok(())
    }

    fn async_poll_clients(&mut self) -> Result<(), ()> {
        for (id, client) in self.clients.iter_mut() {
            if let Ok(poll) = client.poll_message() {
                match poll {
                    Async::Ready(m) => {
                        if let Some(message) = m {
                            let context = ServerEventContext::new(self.write_context.clone(), *id);
                            match message {
                                LurkMessage::Message(m) => self.callbacks.on_message(&context, &m),
                                LurkMessage::ChangeRoom(m) => self.callbacks.on_change_room(&context, &m),
                                LurkMessage::Fight(_) => self.callbacks.on_fight(&context),
                                LurkMessage::PvpFight(m) => self.callbacks.on_pvp_fight(&context, &m),
                                LurkMessage::Loot(m) => self.callbacks.on_loot(&context, &m),
                                LurkMessage::Start(_) => self.callbacks.on_start(&context),
                                LurkMessage::Character(m) => self.callbacks.on_character(&context, &m),
                                LurkMessage::Leave(_) => self.callbacks.on_leave(&context),
                                _ => {},
                            };
                        }
                    },
                    _ => {},
                }
            }
            else {
                // TODO Handle client poll error
            }
        }

        Ok(())
    }

    fn process_write_queue(&mut self) -> Result<(), ()> {
        while let Some(item) = self.write_context.write_queue.pop_message() {
            if let Some(client) = self.clients.get_mut(&item.target) {
                client.enqueue_message(item.packet);
            }
        }

        Ok(())
    }

    fn async_poll_clients_writing(&mut self) {
        for (id, client) in self.clients.iter_mut() {
            // TODO Handle client write error
            if !client.poll_writing() {
                println!("God no why!");
            }
        }
    }
}
