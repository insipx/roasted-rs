use edge_nal_embassy::{Tcp, TcpBuffers};
use embassy_net::Stack;
use embassy_time::{Duration, Timer};

mod wifi;
mod ws_session;
mod ws_stream;
pub use wifi::*;
pub use ws_session::*;
pub use ws_stream::*;

#[embassy_executor::task]
pub async fn bg_runner(mut runner: embassy_net::Runner<esp_radio::wifi::Interface, 'static>) {
    runner.run().await;
}

#[embassy_executor::task]
pub async fn ws(stack: Stack<'static>) {
    let mut buf = [0u8; 8192]; // 8kb  scratch (necessary?)
    let tcp_buffers = TcpBuffers::<1, 1024, 1024>::new();
    let tcp = Tcp::new(stack, &tcp_buffers);
    loop {
        stack.wait_config_up().await;

        let temp_url = "ws://www.example.com/ws".parse().unwrap();
        let session = WebSocketSession::builder()
            .stack(stack)
            .url(temp_url)
            .tcp(&tcp)
            .buf(&mut buf)
            .connect()
            .await;
        if let Err(error) = session {
            esp_println::println!("WebSocket disconnected: {error:?}");
        }

        Timer::after(Duration::from_secs(3)).await;
    }
}
