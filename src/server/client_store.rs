use protocol::protocol_message::LurkMessageBlobify;
use server::client_session::ClientSession;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard};
use uuid::Uuid;

pub struct ServerClientStore {
    clients: Mutex<HashMap<Uuid, Arc<Mutex<ClientSession>>>>,
    close_transmitters: Mutex<HashMap<Uuid, Sender<()>>>,
    to_close: Mutex<Vec<Uuid>>,
}

impl ServerClientStore {
    pub fn new() -> ClientStore {
        Arc::new(ServerClientStore {
            clients: Mutex::new(HashMap::new()),
            to_close: Mutex::new(vec![]),
            close_transmitters: Mutex::new(HashMap::new()),
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
            self.to_close
                .lock()
                .expect("flag_close_client poisoned thread")
                .push(*id);
            self.transmit_client_close(&id);
            client
                .lock()
                .expect("flag_close_client poisoned thread")
                .flag_close();
        }
    }

    pub fn alert_close_client(&self, id: &Uuid) {
        if self.acquire_lock().contains_key(id) {
            self.to_close
                .lock()
                .expect("alert_close_client poisoned thread")
                .push(*id);
            self.transmit_client_close(id);
        }
    }

    pub fn shutdown_client(&self, id: &Uuid) {
        if let Some(client) = self.acquire_lock().get_mut(id) {
            self.transmit_client_close(&id);
            client
                .lock()
                .expect("shutdown_client poisoned thread")
                .shutdown();
        }
    }

    pub fn remove_client(&self, id: &Uuid) {
        self.acquire_lock().remove(&id);
    }

    pub fn add_client(&self, client: ClientSession, tx: Sender<()>) {
        let id = *client.get_id();
        self.close_transmitters.lock().expect("").insert(id, tx);
        self.acquire_lock().insert(id, Arc::new(Mutex::new(client)));
    }

    pub fn collect_close_ids(&self) -> Vec<Uuid> {
        let result = self.to_close.lock().expect("").clone();
        *self.to_close.lock().expect("") = vec![];
        result
    }

    fn transmit_client_close(&self, id: &Uuid) {
        if let Some(tx) = self.close_transmitters.lock().unwrap().get(&id) {
            tx.send(()).ok();
        }
    }

    fn acquire_lock(&self) -> MutexGuard<HashMap<Uuid, Arc<Mutex<ClientSession>>>> {
        self.clients
            .lock()
            .expect("ServerClientStore - Poisoned thread.")
    }
}

pub type ClientStore = Arc<ServerClientStore>;
