use uuid::Uuid;
use tokio::net::TcpStream;
use tokio::codec::Framed;
use protocol::codec::{LurkMessageCodec, LurkMessageReadError};
use futures::stream::Stream;
use protocol::protocol_message::LurkMessage;
use futures::Poll;
use futures::sink::Sink;
use tokio::prelude::future::Future;
use tokio::prelude::AsyncSink;
use std::collections::VecDeque;

pub struct ClientSession {
    id: Uuid,
    frame_stream: Framed<TcpStream, LurkMessageCodec>,
    pending: VecDeque<LurkMessage>,
}

impl ClientSession {
    pub fn new(frame_stream: Framed<TcpStream, LurkMessageCodec>) -> ClientSession {
        ClientSession {
            id: Uuid::new_v4(),
            frame_stream,
            pending: VecDeque::new(),
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn poll_message(&mut self) -> Poll<Option<LurkMessage>, LurkMessageReadError> {
        self.frame_stream.poll()
    }

    pub fn enqueue_message(&mut self, lurk_message: LurkMessage) {
        self.pending.push_back(lurk_message);
    }

    pub fn poll_writing(&mut self) -> bool {
        if let Ok(s) = self.frame_stream.poll_complete() {
            if s.is_ready() {
                if let Some(item) = self.pending.pop_front() {
                    self.frame_stream.start_send(item).unwrap();
                }
            }
            true
        }
        else {
            false
        }
    }
}
