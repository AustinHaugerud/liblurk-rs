use server::write_queue::WriteQueue;
use std::sync::Arc;

pub struct ServerAccess {
    pub write_queue: Arc<WriteQueue>,
}

impl ServerAccess {
    pub fn new() -> WriteContext {
        Arc::new(ServerAccess {
            write_queue: Arc::new(WriteQueue::new()),
        })
    }
}

pub type WriteContext = Arc<ServerAccess>;
