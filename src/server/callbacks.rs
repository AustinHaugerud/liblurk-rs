use protocol::protocol_message::{ChangeRoom, Character, Loot, Message, PvpFight};
use server::context::ServerEventContext;
use server::server_access::WriteContext;
use std::sync::{Arc, Mutex, MutexGuard};

pub trait ServerCallbacks {
    fn on_connect(&mut self, context: &ServerEventContext);

    fn on_disconnect(&mut self, context: &ServerEventContext);

    fn on_message(&mut self, context: &ServerEventContext, message: &Message);

    fn on_change_room(&mut self, context: &ServerEventContext, change_room: &ChangeRoom);

    fn on_fight(&mut self, context: &ServerEventContext);

    fn on_pvp_fight(&mut self, context: &ServerEventContext, pvp_fight: &PvpFight);

    fn on_loot(&mut self, context: &ServerEventContext, loot: &Loot);

    fn on_start(&mut self, context: &ServerEventContext);

    fn on_character(&mut self, context: &ServerEventContext, character: &Character);

    fn on_leave(&mut self, context: &ServerEventContext);

    fn update(&mut self, context: WriteContext);
}

pub struct CallbacksWrapper<T>
where
    T: ServerCallbacks + Send,
{
    callbacks_impl: Arc<Mutex<T>>,
}

impl<T> CallbacksWrapper<T>
where
    T: ServerCallbacks + Send,
{
    pub fn new(callback: T) -> Callbacks<T> {
        Arc::new(CallbacksWrapper {
            callbacks_impl: Arc::new(Mutex::new(callback)),
        })
    }

    pub fn on_connect(&self, context: &ServerEventContext) {
        self.acquire_lock().on_connect(context)
    }

    pub fn on_disconnect(&self, context: &ServerEventContext) {
        self.acquire_lock().on_disconnect(context);
    }

    pub fn on_message(&self, context: &ServerEventContext, message: &Message) {
        self.acquire_lock().on_message(context, message);
    }

    pub fn on_change_room(&self, context: &ServerEventContext, change_room: &ChangeRoom) {
        self.acquire_lock().on_change_room(context, change_room);
    }

    pub fn on_fight(&self, context: &ServerEventContext) {
        self.acquire_lock().on_fight(context);
    }

    pub fn on_pvp_fight(&self, context: &ServerEventContext, pvp_fight: &PvpFight) {
        self.acquire_lock().on_pvp_fight(context, pvp_fight);
    }

    pub fn on_loot(&self, context: &ServerEventContext, loot: &Loot) {
        self.acquire_lock().on_loot(context, loot);
    }

    pub fn on_start(&self, context: &ServerEventContext) {
        self.acquire_lock().on_start(context)
    }

    pub fn on_character(&self, context: &ServerEventContext, character: &Character) {
        self.acquire_lock().on_character(context, character)
    }

    pub fn on_leave(&self, context: &ServerEventContext) {
        self.acquire_lock().on_leave(context)
    }

    pub fn update(&self, context: WriteContext) {
        self.acquire_lock().update(context)
    }

    fn acquire_lock(&self) -> MutexGuard<T> {
        self.callbacks_impl
            .lock()
            .expect("ServerWrapper - Poisoned thread.")
    }
}

pub type Callbacks<T> = Arc<CallbacksWrapper<T>>;
