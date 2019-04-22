use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use server::callbacks::{ServerCallbacks, CallbacksWrapper, Callbacks};
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::mpsc::Receiver;
use tokio::codec::{Framed, FramedRead, FramedWrite};
use protocol::codec::LurkMessageCodec;
use server::context::ServerEventContext;
use server::server_access::{WriteContext, ServerAccess};
use protocol::protocol_message::LurkMessage;
use std::net::SocketAddr;
use tokio::timer::Interval;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use tokio::io::WriteHalf;

pub fn execute_server<T>(addr: &SocketAddr, update_freq : Duration, behaviour: T) -> Result<(), ()> where T: 'static + ServerCallbacks + Send {
    let listener = TcpListener::bind(&addr).map_err(|_| ())?;

    let callbacks = CallbacksWrapper::new(behaviour);
    let write_context = ServerAccess::new();

    let write_halves = Arc::new(Mutex::new(HashMap::new()));

    let server = {
        let write_halves = write_halves.clone();
        let callbacks = callbacks.clone();
        let write_context = write_context.clone();

        listener.incoming().for_each(move |socket| {
            let writes = write_halves.clone();
            let id = Uuid::new_v4();
            let callbacks = callbacks.clone();
            let write_context = write_context.clone();

            let (read, write) = socket.split();
            let (fread, fwrite) = (FramedRead::new(read, LurkMessageCodec), FramedWrite::new(write, LurkMessageCodec));

            writes.lock().expect("Failed to store writer.").insert(id, fwrite);

            callbacks.on_connect(&ServerEventContext::new(write_context.clone(), id));

            let read_proc = fread.for_each(move |msg| {
                let cbs = callbacks.clone();
                let write_context = write_context.clone();

                let event_context = ServerEventContext::new(write_context.clone(), id);

                match msg {
                    LurkMessage::Message(m) => cbs.on_message(&event_context, &m),
                    LurkMessage::ChangeRoom(m) => cbs.on_change_room(&event_context, &m),
                    LurkMessage::Fight(_) => cbs.on_fight(&event_context),
                    LurkMessage::PvpFight(m) => cbs.on_pvp_fight(&event_context, &m),
                    LurkMessage::Loot(m) => cbs.on_loot(&event_context, &m),
                    LurkMessage::Start(_) => cbs.on_start(&event_context),
                    LurkMessage::Character(m) => cbs.on_character(&event_context, &m),
                    LurkMessage::Leave(_) => cbs.on_leave(&event_context),
                    _ => {},
                }

                Ok(())
            }).map_err(|e| eprintln!("Failed lurk read on client"));

            tokio::spawn(read_proc);

            Ok(())
        }).map_err(|_| ())
    };

    let update_proc = {
        let write_halves = write_halves.clone();
        let callbacks = callbacks.clone();
        let write_context = write_context.clone();

        Interval::new_interval(update_freq).for_each(move |i| {
            let write_peers = write_halves.clone();
            let callbacks = callbacks.clone();
            let write_context = write_context.clone();

            callbacks.update(write_context.clone());

            while let Some(item) = write_context.write_queue.pop_message() {
                let mut write_peers_l = write_peers.lock().expect("Failed to lock write peers.");
                if let Some(mut peer) = write_peers_l.remove(&item.target) {
                    let future = peer.send(item.packet);
                    peer = future.wait().expect("the horror");
                    write_peers_l.insert(item.target, peer);
                }
            }

            Ok(())
        }).map_err(|e| eprintln!("oh no"))
    };

    let proc = server.join(update_proc).map(|_| ());

    tokio::run(proc);

    Ok(())
}

fn run_client(socket: TcpStream) -> Result<(), ()> {
    unimplemented!()
}
