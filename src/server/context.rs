use server::server_access::WriteContext;
use uuid::Uuid;
use protocol::protocol_message::LurkMessageBlobify;

pub struct ServerEventContext {
    write_context: WriteContext,
    client_id: Uuid,
}

impl ServerEventContext {
    pub fn new(write_context: WriteContext, client_id: Uuid) -> ServerEventContext {
        ServerEventContext {
            write_context: write_context.clone(),
            client_id,
        }
    }

    pub fn get_write_context(&self) -> WriteContext {
        self.write_context.clone()
    }

    pub fn get_client_id(&self) -> &Uuid {
        &self.client_id
    }

    pub fn enqueue_message<T: 'static>(&self, message : T, target: &Uuid) where T: LurkMessageBlobify + Send {
        self.write_context.enqueue_message(message, &target);
    }

    pub fn enqueue_message_many<T: 'static>(&self, message: T, targets: &[&Uuid]) where T: LurkMessageBlobify + Send + Clone {
        self.write_context.enqueue_message_many(message, targets);
    }

    pub fn enqueue_message_this<T: 'static>(&self, message: T) where T: LurkMessageBlobify + Send {
        self.write_context.enqueue_message(message, &self.client_id);
    }
}
