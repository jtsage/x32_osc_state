use tokio::net::UdpSocket;
use std::io;
use std::sync::Arc;
use x32_osc_state as x32;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut x32_state = x32::X32Console::default();
    let x32_all = x32::x32::ConsoleRequest::full_update();

    let x32 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 77)), 10023);
    let sock = UdpSocket::bind("0.0.0.0:10023".parse::<SocketAddr>().unwrap()).await?;
    let r = Arc::new(sock);
    let s = r.clone();
    let u = r.clone();

    // Ask for the full state of the X32 every 5 minutes.
    // Includes a pause of 50ms between each command sent to the
    // X32 to ensure we don't send data faster than it can handle
    tokio::spawn(async move {
        loop {
            println!("asking for data");
            for item in x32_all.clone() {
                u.send_to(item.as_slice(), x32).await.expect("broken socket");
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            tokio::time::sleep(Duration::from_secs(300)).await;
        }
    });

    // send the xremote command every 5 seconds
    // the x32 xremote timer expires after 8 minutes, this ensures we
    // always are receiving data
    tokio::spawn(async move {
        loop {
            println!("sending xremote");
            s.send_to(x32::enums::X32_XREMOTE.as_slice(), x32).await.expect("broken socket");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });

    // Main loop, message received from the X32
    let mut buf = [0; 1024];
    loop {
        let (len, addr) = r.recv_from(&mut buf).await?;
        let buffer = x32::osc::Buffer::from(buf.clone().to_vec());
        x32_state.process(buffer);
        println!("{:?} bytes received from {:?}", len, addr);
    }
}
