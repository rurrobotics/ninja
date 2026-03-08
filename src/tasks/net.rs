use cyw43::NetDriver;
use embassy_net::Runner;

#[embassy_executor::task]
pub async fn task(mut runner: Runner<'static, NetDriver<'static>>) -> ! {
    runner.run().await
}
