#![no_std]
#![no_main]

mod actions;
mod configuration;
mod db;
mod error;
mod serial;

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use error::Error;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    peripherals::{FROM_CPU_INTR0, TIMG0},
    rng::TrngSource,
    timer::timg::TimerGroup,
};
use esp_storage::FlashStorage;

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

    let db = FlashStorage::new(peripherals.FLASH);
    let entropy = TrngSource::new(peripherals.RNG, peripherals.ADC1);

    if let Err(e) = run(spawner, peripherals.TIMG0, peripherals.FROM_CPU_INTR0, rw, db, entropy).await {
        panic!("Zippy failed: {e:?}");
    }
    core::future::pending::<()>().await;
}

async fn run(
    spawner: Spawner,
    timer: TIMG0<'static>,
    cpu: FROM_CPU_INTR0<'static>,
    rw: esp_hal::uart::Uart<'static, esp_hal::Async>,
    db: FlashStorage<'static>,
    entropy: TrngSource<'static>,
) -> Result<(), Error> {
    let timg0 = TimerGroup::new(timer);
    esp_rtos::start(timg0.timer0, cpu);

    esp_println::println!("spawning\r");
    static COMMANDS: Channel<CriticalSectionRawMutex, roasted_types::zippy::CliAction, 8> =
        Channel::new();
    let (uart_rx, uart_tx) = rw.split();
    let uart = serial::UartCli::builder().uart_rx(uart_rx).tx(COMMANDS.dyn_sender()).build();
    spawner.spawn(serial::cli_driver(uart).unwrap());
    let cmds = actions::CommandExecutor::builder()
        .db(db)
        .entropy(entropy)
        .uart_tx(uart_tx)
        .rx(COMMANDS.dyn_receiver())
        .build();
    spawner.spawn(actions::run(cmds).unwrap());
    Ok(())
}
