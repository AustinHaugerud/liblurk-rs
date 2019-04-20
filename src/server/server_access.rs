use server::write_queue::{WriteQueue, WriteQueueItem, Sender};
use std::sync::Arc;
use uuid::Uuid;
use protocol::protocol_message::LurkMessageBlobify;

pub struct ServerAccess {
    pub write_queue: Arc<WriteQueue>,
}

impl ServerAccess {
    pub fn new() -> WriteContext {
        Arc::new(ServerAccess {
            write_queue: Arc::new(WriteQueue::new()),
        })
    }

    pub fn enqueue_message<T: 'static>(&self, message : T, target: &Uuid) where T: LurkMessageBlobify + Send {
        let item = WriteQueueItem::new(message, Sender::Server, *target);
        self.write_queue.enqueue_message(item);
    }

    pub fn enqueue_message_many<T: 'static>(&self, message: T, targets: &[&Uuid]) where T: LurkMessageBlobify + Send + Clone {
        for target in targets {
            let item = WriteQueueItem::new(message.clone(), Sender::Server, **target);
            self.write_queue.enqueue_message(item);
        }
    }
}

pub type WriteContext = Arc<ServerAccess>;
