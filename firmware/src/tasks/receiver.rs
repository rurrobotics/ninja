use embassy_net::{Stack, tcp::TcpSocket};
use embedded_io_async::Write as _;

use crate::{
    COMMAND_CHANNEL,
    config::{RECEIVER_BUFFER_SIZE, RECEIVER_KEEP_ALIVE_INTERVAL},
    packet::{RequestPacket, ResponsePacket},
};

#[embassy_executor::task]
pub async fn task(stack: Stack<'static>) -> ! {
    let mut rx_buffer = [0; RECEIVER_BUFFER_SIZE];
    let mut tx_buffer = [0; RECEIVER_BUFFER_SIZE];
    let mut buf = [0; RECEIVER_BUFFER_SIZE];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_keep_alive(Some(RECEIVER_KEEP_ALIVE_INTERVAL));

        log::info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    log::warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    log::warn!("read error: {:?}", e);
                    break;
                }
            };

            let req: RequestPacket = match postcard::from_bytes(&buf[..n]) {
                Ok(rp) => rp,
                Err(e) => {
                    log::warn!("format is invalid: {:?}", e);
                    break;
                }
            };

            log::info!("{:?}", req);
            COMMAND_CHANNEL.signal(req);

            let resp = match postcard::to_vec::<_, 4>(&ResponsePacket { status: true }) {
                Ok(rp) => rp,
                Err(e) => {
                    log::warn!("response error: {:?}", e);
                    break;
                }
            };

            match socket.write_all(&resp).await {
                Ok(()) => {}
                Err(e) => {
                    log::warn!("write error: {:?}", e);
                    break;
                }
            };
        }
    }
}
