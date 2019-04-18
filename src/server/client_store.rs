use protocol::protocol_message::LurkMessageBlobify;
use server::client_session::ClientSession;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

pub struct ServerClientStore {
    clients: Mutex<HashMap<Uuid, Arc<Mutex<ClientSession>>>>,
    to_close : Mutex<Vec<Uuid>>,
}

impl ServerClientStore {
    pub fn new() -> ClientStore {
        Arc::new(ServerClientStore {
            clients: Mutex::new(HashMap::new()),
            to_close: Mutex::new(vec![])
        })
    }

    pub fn write_client<F>(&self, packet: &F, id: &Uuid) -> Result<(), ()>
    where
        F: LurkMessageBlobify + Send + ?Sized,
    {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            let mut lock = client
                .lock()
                .expect("write_client client lock thread poisoned");
            let mut send_channel = lock.get_send_channel();
            send_channel.write_message(packet)
        } else {
            Err(())
        }
    }

    pub fn get_client(&self, id: &Uuid) -> Option<Arc<Mutex<ClientSession>>> {
        self.acquire_lock().get(id).cloned()
    }

    pub fn flag_close_client(&self, id: &Uuid) {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            self.to_close.lock().expect("flag_close_client poisoned thread").push(*id);
            client
                .lock()
                .expect("flag_close_client poisoned thread")
                .flag_close();
        }
    }

    pub fn shutdown_client(&self, id: &Uuid) {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            client
                .lock()
                .expect("shutdown_client poisoned thread")
                .shutdown();
        }
    }

    pub fn remove_client(&self, id: &Uuid) {
        self.acquire_lock().remove(&id);
    }

    pub fn add_client(&self, client: ClientSession) {
        let id = *client.get_id();
        self.acquire_lock().insert(id, Arc::new(Mutex::new(client)));
    }

    pub fn collect_close_ids(&self) -> Vec<Uuid> {
        let result = self.to_close.lock().expect("").clone();
        *self.to_close.lock().expect("") = vec![];
        result
    }

    fn acquire_lock(&self) -> MutexGuard<HashMap<Uuid, Arc<Mutex<ClientSession>>>> {
        self.clients
            .lock()
            .expect("ServerClientStore - Poisoned thread.")
    }
}

pub type ClientStore = Arc<ServerClientStore>;
