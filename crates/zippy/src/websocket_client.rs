use anyhow::Result;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use url::Url;

#[embassy_executor::task]
pub async fn net_task(mut runner: embassy_net::Runner<esp_radio::wifi::Interface, 'static>) {
    runner.run();
}

#[embassy_executor::task]
pub async fn ws_task(stack: Stack<'static>) {
    loop {
        stack.wait_config_up().await;

        if let Err(error) = websocket_session(stack, "www.example.com:80/ws".parse().unwrap()).await
        {
            esp_println::println!("WebSocket disconnected: {error:?}");
        }

        Timer::after(Duration::from_secs(3)).await;
    }
}

async fn websocket_session(stack: Stack<'static>, url: Url) -> Result<()> {
    todo!()
}
