use cyw43::NetDriver;
use embassy_net::{Config, Runner, Stack, StackResources};
use embassy_rp::clocks::RoscRng;
use static_cell::StaticCell;

pub async fn wrapper(
    net_device: NetDriver<'static>,
) -> (Stack<'static>, Runner<'static, NetDriver<'static>>) {
    let config = Config::dhcpv4(Default::default());

    let seed = RoscRng.next_u64();

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        seed,
    );

    (stack, runner)
}
