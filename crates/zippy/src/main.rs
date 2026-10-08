#![no_std]
#![no_main]

mod configuration;
mod error;
mod serial;
mod db;

use embassy_executor::Spawner;
use error::Error;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    peripherals::{FROM_CPU_INTR0, TIMG0},
    timer::timg::TimerGroup,
};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> () {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 64 * 1024);
    esp_println::println!("Hello");

    let rw = esp_hal::uart::Uart::new(peripherals.UART0, Default::default())
        .expect("UART init failed")
        .with_rx(peripherals.GPIO5)
        .with_tx(peripherals.GPIO6)
        .into_async();

    if let Err(e) = run(spawner, peripherals.TIMG0, peripherals.FROM_CPU_INTR0, rw).await {
        panic!("Zippy failed: {e:?}");
    }
    core::future::pending::<()>().await;
}

async fn run(
    spawner: Spawner,
    timer: TIMG0<'static>,
    cpu: FROM_CPU_INTR0<'static>,
    rw: serial::CommsPeripheral,
) -> Result<(), Error> {
    let timg0 = TimerGroup::new(timer);
    esp_rtos::start(timg0.timer0, cpu);

    esp_println::println!("spawning\r");
    spawner.spawn(serial::cli_driver(rw).unwrap());
    Ok(())
}
