use rayon::{ThreadPool, ThreadPoolBuildError, ThreadPoolBuilder};
use server::callbacks::{Callbacks, ServerCallbacks};
use server::client_store::ClientStore;
use server::server_access::WriteContext;
use uuid::Uuid;

pub struct ClientThreadPool<T>
where
    T: ServerCallbacks + Send,
{
    pool: ThreadPool,
    client_store: ClientStore,
    write_context: WriteContext,
    callbacks: Callbacks<T>,
    max_threads: usize,
}

impl<T> ClientThreadPool<T>
where
    T: ServerCallbacks + Send,
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
        })
    }

    pub fn start_client(&self, id: &Uuid) -> Result<(), ()> {
        if self.is_full() {
            let client_store = self.client_store.clone();
            let write_context = self.write_context.clone();
            let callbacks = self.callbacks.clone();

            self.pool.install(move || {
                while let Some(running) = client_store.check_client_running(id) {
                    if running {
                        client_store.update_client(id, callbacks.clone(), write_context.clone());
                    } else {
                        break;
                    }
                }
            });

            Ok(())
        } else {
            Err(())
        }
    }

    pub fn is_full(&self) -> bool {
        self.pool.current_num_threads() == self.max_threads
    }
}
