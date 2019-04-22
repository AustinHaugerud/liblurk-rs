use tokio::codec::FramedWrite;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use protocol::codec::LurkMessageCodec;

pub struct WritePeer {
    write: FramedWrite<WriteHalf<TcpStream>, LurkMessageCodec>
}
