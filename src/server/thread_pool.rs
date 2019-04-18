use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use server::callbacks::{Callbacks, ServerCallbacks};
use server::client_store::ClientStore;
use server::server_access::WriteContext;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{atomic::AtomicUsize, Arc};
use uuid::Uuid;

pub struct ClientThreadPool<T>
where
    T: 'static + ServerCallbacks + Send,
{
    pool: ThreadPool,
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
    ) -> Result<ClientThreadPool<T>, ThreadPoolBuildError> {
        let pool = ThreadPoolBuilder::new().num_threads(size).build()?;
        Ok(ClientThreadPool {
            max_threads: size,
            pool,
            client_store,
            write_context,
            callbacks,
            num_active: Arc::new(AtomicUsize::new(0)),
        })
    }

    pub fn start_client(&mut self, id: Uuid) -> Result<(), ()> {
        if !self.is_full() {
            let client_store = self.client_store.clone();
            let write_context = self.write_context.clone();
            let callbacks = self.callbacks.clone();
            let num_active = self.num_active.clone();
            let monitor = self
                .client_store
                .get_client_running_monitor(&id)
                .unwrap_or_else(|| panic!("start_client bug, client {:?} does not exist.", id));

            self.pool.spawn(move || {
                num_active.fetch_add(1, Relaxed);
                while monitor.is_running() {
                    client_store.update_client(&id, callbacks.clone(), write_context.clone());
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
