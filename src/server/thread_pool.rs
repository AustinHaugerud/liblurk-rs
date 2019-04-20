use server::callbacks::{Callbacks, ServerCallbacks};
use server::client_store::ClientStore;
use server::server_access::WriteContext;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::TryRecvError;
use std::sync::{atomic::AtomicUsize, mpsc::Receiver, Arc};
use uuid::Uuid;

pub struct ClientThreadPool<T>
where
    T: 'static + ServerCallbacks + Send,
{
    client_store: ClientStore,
    write_context: WriteContext,
    callbacks: Callbacks<T>,
    max_threads: usize,
    num_active: Arc<AtomicUsize>,
}

impl<T> ClientThreadPool<T>
where
    T: 'static + ServerCallbacks + Send,
{
    pub fn new(
        client_store: ClientStore,
        write_context: WriteContext,
        callbacks: Callbacks<T>,
        size: usize,
    ) -> ClientThreadPool<T> {
        ClientThreadPool {
            max_threads: size,
            client_store,
            write_context,
            callbacks,
            num_active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn start_client(&mut self, id: Uuid, close_channel_rx: Receiver<()>) -> Result<(), ()> {
        use std::thread;

        println!("Starting client.");
        let client = {
            self.client_store
                .get_client(&id)
                .unwrap_or_else(|| panic!("Bug: start_client client {:?} does not exist.", id))
        };
        if !self.is_full() {
            let write_context = self.write_context.clone();
            let callbacks = self.callbacks.clone();
            let num_active = self.num_active.clone();
            let client_store = self.client_store.clone();

            thread::spawn(move || {
                num_active.fetch_add(1, Relaxed);

                loop {
                    match close_channel_rx.try_recv() {
                        Ok(_) | Err(TryRecvError::Disconnected) => break,

                        Err(TryRecvError::Empty) => {
                            let keep_open = client
                                .lock()
                                .expect("")
                                .update(callbacks.clone(), write_context.clone());

                            if !keep_open {
                                client_store.alert_close_client(&id);
                            }
                        }
                    }
                }

                num_active.fetch_sub(1, Relaxed);
            });

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn is_full(&self) -> bool {
        self.num_active.load(Relaxed) == self.max_threads
    }
}
