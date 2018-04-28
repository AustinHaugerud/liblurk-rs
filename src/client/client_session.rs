use std::net::TcpStream;
use std::net::IpAddr;
use protocol::protocol_message::*;
use protocol::read::LurkReadChannel;
use protocol::send::LurkSendChannel;

pub trait ClientSessionCallbacks {
    fn on_message(&mut self, channel: &mut LurkSendChannel<TcpStream>, message: &Message);
    fn on_error(&mut self, channel: &mut LurkSendChannel<TcpStream>, error: &Error);
    fn on_accept(&mut self, channel: &mut LurkSendChannel<TcpStream>, accept: &Accept);
    fn on_room(&mut self, channel: &mut LurkSendChannel<TcpStream>, room: &Room);
    fn on_character(&mut self, channel: &mut LurkSendChannel<TcpStream>, character: &Character);
    fn on_game(&mut self, channel: &mut LurkSendChannel<TcpStream>, game: &Game);
    fn on_connection(&mut self, channel: &mut LurkSendChannel<TcpStream>, connection: &Connection);
}

pub struct ClientSession {
    stream: TcpStream,
    callbacks: Box<ClientSessionCallbacks>,
}

impl ClientSession {
    pub fn create(
        (host, port): (IpAddr, u16),
        behavior: Box<ClientSessionCallbacks>,
    ) -> Result<ClientSession, String> {
        match TcpStream::connect((host, port)) {
            Ok(t) => {
                let client = ClientSession {
                    stream: t,
                    callbacks: behavior,
                };

                Ok(client)
            }
            Err(_) => Err(String::from("Failed to connect.")),
        }
    }

    pub fn pull_message(&mut self) -> Result<(LurkMessageKind, Vec<u8>), String> {
        let mut read_channel = LurkReadChannel::new(&mut self.stream);

        fn read_fail(_: ()) -> String {
            String::from("Read channel failure")
        }
        read_channel.read_next().map_err(read_fail)
    }

    pub fn update(&mut self) -> Result<(), String> {
        let (kind, data) = self.pull_message()?;

        match kind {
            LurkMessageKind::Message => {
                let (message, _) = Message::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_message(&mut send_channel, &message);
            }
            LurkMessageKind::ChangeRoom => {
                ChangeRoom::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Fight => {
                Fight::parse_lurk_message((data.as_slice()))?;
            }
            LurkMessageKind::PvPFight => {
                PvpFight::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Loot => {
                Loot::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Start => {
                Start::parse_lurk_message(data.as_slice())?;
            }
            LurkMessageKind::Error => {
                let (message, _) = Error::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_error(&mut send_channel, &message);
            }
            LurkMessageKind::Accept => {
                let (message, _) = Accept::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_accept(&mut send_channel, &message);
            }
            LurkMessageKind::Room => {
                let (message, _) = Room::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_room(&mut send_channel, &message);
            }
            LurkMessageKind::Character => {
                let (message, _) = Character::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_character(&mut send_channel, &message);
            }
            LurkMessageKind::Game => {
                let (message, _) = Game::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_game(&mut send_channel, &message);
            }
            LurkMessageKind::Leave => {}
            LurkMessageKind::Connection => {
                let (message, _) = Connection::parse_lurk_message(data.as_slice())?;
                let mut send_channel = LurkSendChannel::new(&mut self.stream);
                self.callbacks.on_connection(&mut send_channel, &message);
            }
        };

        Ok(())
    }

    pub fn get_send_channel(&mut self) -> LurkSendChannel<TcpStream> {
        LurkSendChannel::new(&mut self.stream)
    }
}
