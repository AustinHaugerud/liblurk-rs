use std::net::TcpListener;

use std::net::SocketAddr;
use std::time::Duration;

use server::callbacks::{Callbacks, CallbacksWrapper, ServerCallbacks};
use server::client_session::ClientSession;
use server::client_store::{ClientStore, ServerClientStore};
use server::context::ServerEventContext;
use server::server_access::{ServerAccess, WriteContext};
use server::thread_pool::ClientThreadPool;
use server::timing::Clock;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::channel;

pub struct Server<T>
where
    T: 'static + ServerCallbacks + Send,
{
    clients: ClientStore,
    callbacks: Callbacks<T>,
    write_context: WriteContext,
    running: AtomicBool,
    thread_pool: ClientThreadPool<T>,
    listener: TcpListener,
    frame_time: Duration,
    read_timeout: Duration,
}

impl<T> Server<T>
where
    T: 'static + ServerCallbacks + Send,
{
    pub fn create(
        addr: SocketAddr,
        timeout: Duration,
        frame_time: Duration,
        max_connections: usize,
        behavior: T,
    ) -> Result<Server<T>, String> {
        let listener =
            TcpListener::bind(addr).map_err(|_| format!("Failed to bind to address {}.", addr))?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "Failed to create non-blocking listener.".to_string())?;

        let clients = ServerClientStore::new();
        let callbacks = CallbacksWrapper::new(behavior);
        let write_context = ServerAccess::new();

        let thread_pool = ClientThreadPool::new(
            clients.clone(),
            write_context.clone(),
            callbacks.clone(),
            max_connections,
        )
        .map_err(|_| "Failed to create thread pool.".to_string())?;

        Ok(Server {
            frame_time,
            read_timeout: timeout,
            listener,
            clients,
            callbacks,
            write_context,
            running: AtomicBool::new(false),
            thread_pool,
        })
    }

    pub fn start(&mut self) {
        self.running.store(true, Relaxed);
        self.main();
    }

    pub fn stop(&mut self) {
        self.running.store(false, Relaxed);
    }

    fn main(&mut self) {
        use std::thread;

        let mut clock = Clock::new();

        loop {
            let running = self.running.load(Relaxed);
            if !running {
                break;
            }

            println!("Accepting connections.");
            self.accept_connections();

            println!("Update.");
            self.callbacks.update(self.write_context.clone());

            println!("Process write queue.");
            self.process_write_queue();

            println!("Clean client store.");
            self.clean_client_store();

            let time = clock.get_elapsed();

            if let Some(sleep_time) = self.frame_time.checked_sub(time) {
                thread::sleep(sleep_time);
            }
        }
    }

    fn accept_connections(&mut self) {
        use std::io;
        loop {
            if !self.thread_pool.is_full() {
                match self.listener.accept() {
                    Ok((socket, _)) => {
                        let success = socket.set_read_timeout(Some(self.read_timeout)).is_ok();

                        if success {
                            println!("Setting up client.");
                            let (sender, receiver) = channel();
                            let client = ClientSession::new(socket, sender);
                            let id = *client.get_id();
                            println!("Adding to store.");
                            self.clients.add_client(client);
                            println!("Added.");
                            let write_context = self.write_context.clone();
                            let context = ServerEventContext::new(write_context, id);
                            println!("Installing to thread pool.");
                            self.thread_pool
                                .start_client(id, receiver)
                                .expect("Bug: Cannot add as client thread pool full.");
                            println!("Installed.");
                            self.callbacks.on_connect(&context);
                        }
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::WouldBlock {
                            break;
                        } else {
                            self.running.store(false, Relaxed);
                        }
                    }
                }
            } else {
                println!("Thread pool is full.");
                break;
            }
        }
    }

    fn process_write_queue(&mut self) {
        while let Some(write_item) = self.write_context.write_queue.pop_message() {
            let packet = write_item.packet.as_ref();
            let target = write_item.target;

            if self.clients.write_client(packet, &target).is_err() {
                self.clients.flag_close_client(&write_item.target);
            }
        }
    }

    fn clean_client_store(&mut self) {
        let to_close = self.clients.collect_close_flagged_ids();

        for id in to_close {
            self.clients.shutdown_client(&id);
            self.clients.remove_client(&id);
            let context = ServerEventContext::new(self.write_context.clone(), id);
            self.callbacks.on_disconnect(&context);
        }
    }
}
