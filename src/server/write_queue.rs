use protocol::protocol_message::{LurkMessageBlobify, LurkMessage};
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

pub enum Sender {
    Server,
    Client(Uuid),
}

pub struct WriteQueueItem {
    pub packet: LurkMessage,
    pub target: Uuid,
    pub sender: Sender,
}

impl WriteQueueItem {
    pub fn new(packet: LurkMessage, sender: Sender, target: Uuid) -> WriteQueueItem {
        WriteQueueItem {
            packet,
            target,
            sender,
        }
    }
}

pub struct WriteQueue {
    items: Arc<Mutex<Vec<WriteQueueItem>>>,
}

impl WriteQueue {
    pub fn new() -> WriteQueue {
        WriteQueue::default()
    }

    pub fn enqueue_message(&self, item: WriteQueueItem) {
        let mut items = self.acquire_items_lock();
        items.push(item);
    }

    pub fn pop_message(&self) -> Option<WriteQueueItem> {
        let mut items = self.acquire_items_lock();
        if items.len() > 0 {
            Some(items.remove(0))
        } else {
            None
        }
    }

    fn acquire_items_lock(&self) -> MutexGuard<Vec<WriteQueueItem>> {
        self.items
            .lock()
            .expect("WriteQueueItem - Poisoned thread.")
    }
}

impl Default for WriteQueue {
    fn default() -> Self {
        WriteQueue {
            items: Arc::new(Mutex::new(vec![])),
        }
    }
}
