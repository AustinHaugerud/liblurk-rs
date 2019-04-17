use server::server_access::WriteContext;
use uuid::Uuid;

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
}
