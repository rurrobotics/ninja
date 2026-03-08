use cyw43::Control;
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_time::Duration;
use embedded_io_async::Write as _;

use crate::{
    COMMAND_CHANNEL,
    packet::{RequestPacket, ResponsePacket},
};

#[embassy_executor::task]
pub async fn task(mut control: Control<'static>, stack: Stack<'static>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_keep_alive(Some(Duration::from_secs(10)));

        control.gpio_set(0, false).await;
        log::info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            log::warn!("accept error: {:?}", e);
            continue;
        }

        log::info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

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
            COMMAND_CHANNEL.send(req).await;

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
