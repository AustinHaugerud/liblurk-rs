use protocol::protocol_message::{
    ChangeRoom, Character, Loot, LurkMessageKind, LurkMessageParse, Message, PvpFight,
};
use protocol::read::LurkReadChannel;
use protocol::send::LurkSendChannel;
use server::callbacks::{Callbacks, ServerCallbacks};
use server::context::ServerEventContext;
use server::server_access::WriteContext;
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use uuid::Uuid;

pub struct ClientSession {
    id: Uuid,
    stream: TcpStream,
    keep_open: Arc<AtomicBool>,
    close_transmitter: Sender<()>,
}

impl ClientSession {
    pub fn new(stream: TcpStream, close_tx: Sender<()>) -> ClientSession {
        // Read timeout needs to be set so that clients can eventually
        // timeout and be closed if inactive too long.
        debug_assert!(stream.read_timeout().unwrap().is_some());
        ClientSession {
            id: Uuid::new_v4(),
            stream,
            keep_open: Arc::new(AtomicBool::new(true)),
            close_transmitter: close_tx,
        }
    }

    pub fn get_id(&self) -> &Uuid {
        &self.id
    }

    pub fn shutdown(&mut self) {
        self.close_transmitter.send(()).ok();

        self.flag_close();

        if self.stream.shutdown(Shutdown::Both).is_err() {
            println!("Failed to shutdown TCP Stream.");
        }
    }

    pub fn get_send_channel(&mut self) -> LurkSendChannel<TcpStream> {
        LurkSendChannel::new(&mut self.stream)
    }

    pub fn flag_close(&mut self) {
        self.close_transmitter.send(()).ok();
        self.keep_open.store(false, Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.keep_open.load(Relaxed)
    }

    pub fn update<T>(&mut self, callbacks: Callbacks<T>, write_context: WriteContext) -> bool
    where
        T: ServerCallbacks + Send,
    {
        if self.is_running() {
            match self.pull_client_message() {
                Ok(op_data) => {
                    if let Some(data) = op_data {
                        let id = self.id;
                        let is_ok = self
                            .handle_lurk_message(
                                ServerEventContext::new(write_context, id),
                                callbacks,
                                data,
                            )
                            .is_ok();

                        if !is_ok {
                            self.flag_close();
                        }
                    } else {
                        self.flag_close();
                    }
                }
                Err(_) => {
                    self.flag_close();
                }
            }
        }

        self.keep_open.load(Relaxed)
    }

    pub fn pull_client_message(&mut self) -> Result<Option<(LurkMessageKind, Vec<u8>)>, ()> {
        let data = LurkReadChannel::new(&mut self.stream).read_next()?;
        let is_not_server_valid = {
            if let Some((kind, _)) = &data {
                !kind.is_server_recipient()
            } else {
                false
            }
        };

        if is_not_server_valid {
            Err(())
        } else {
            Ok(data)
        }
    }

    fn handle_lurk_message<T>(
        &mut self,
        context: ServerEventContext,
        callbacks: Callbacks<T>,
        (kind, data): (LurkMessageKind, Vec<u8>),
    ) -> Result<(), ()>
    where
        T: ServerCallbacks + Send,
    {
        println!("Handling lurk message.");
        match kind {
            LurkMessageKind::Message => {
                let (message, _) = Message::parse_lurk_message(data.as_slice())?;
                callbacks.on_message(&context, &message);
            }
            LurkMessageKind::ChangeRoom => {
                let (change_room, _) = ChangeRoom::parse_lurk_message(data.as_slice())?;
                callbacks.on_change_room(&context, &change_room);
            }
            LurkMessageKind::Fight => {
                callbacks.on_fight(&context);
            }
            LurkMessageKind::PvPFight => {
                let (pvp_fight, _) = PvpFight::parse_lurk_message(data.as_slice())?;
                callbacks.on_pvp_fight(&context, &pvp_fight);
            }
            LurkMessageKind::Loot => {
                let (loot, _) = Loot::parse_lurk_message(data.as_slice())?;
                callbacks.on_loot(&context, &loot);
            }
            LurkMessageKind::Start => {
                callbacks.on_start(&context);
            }
            LurkMessageKind::Character => {
                let (character, _) = Character::parse_lurk_message(data.as_slice())?;
                callbacks.on_character(&context, &character);
            }
            LurkMessageKind::Leave => {
                self.flag_close();
                callbacks.on_leave(&context);
            }
            _ => panic!(
                "Bug: Invalid package for sever recipient in ClientSession::handle_lurk_message."
            ),
        }

        Ok(())
    }
}
