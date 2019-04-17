use protocol::protocol_message::LurkMessageBlobify;
use server::callbacks::{Callbacks, ServerCallbacks};
use server::client_session::ClientSession;
use server::server_access::WriteContext;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

pub struct ServerClientStore {
    clients: Mutex<HashMap<Uuid, ClientSession>>,
}

impl ServerClientStore {
    pub fn new() -> ClientStore {
        Arc::new(ServerClientStore {
            clients: Mutex::new(HashMap::new()),
        })
    }

    pub fn write_client<F>(&self, packet: &F, id: &Uuid) -> Result<(), ()>
    where
        F: LurkMessageBlobify + Send + ?Sized,
    {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            let mut send_channel = client.get_send_channel();
            send_channel.write_message(packet)
        } else {
            Err(())
        }
    }

    pub fn flag_close_client(&self, id: &Uuid) {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            client.flag_close();
        }
    }

    pub fn shutdown_client(&self, id: &Uuid) {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            client.shutdown();
        }
    }

    pub fn update_client<T>(&self, id: &Uuid, callbacks: Callbacks<T>, write_context: WriteContext)
    where
        T: ServerCallbacks + Send,
    {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            client.update(callbacks, write_context);
        }
    }

    pub fn check_client_running(&self, id: &Uuid) -> Option<bool> {
        if let Some(client) = self.acquire_lock().get(id) {
            Some(client.is_running())
        } else {
            None
        }
    }

    pub fn remove_client(&self, id: &Uuid) {
        self.acquire_lock().remove(&id);
    }

    pub fn add_client(&self, client: ClientSession) {
        let id = *client.get_id();
        self.acquire_lock().insert(id, client);
    }

    pub fn collect_close_flagged_ids(&self) -> Vec<Uuid> {
        let mut ids = vec![];
        for (id, client) in self.acquire_lock().iter() {
            if !client.is_running() {
                ids.push(id.clone());
            }
        }
        ids
    }

    fn acquire_lock(&self) -> MutexGuard<HashMap<Uuid, ClientSession>> {
        self.clients
            .lock()
            .expect("ServerClientStore - Poisoned thread.")
    }
}

pub type ClientStore = Arc<ServerClientStore>;
