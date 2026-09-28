#![no_std]
#![no_main]

mod configuration;
mod error;
mod serial;

use embassy_executor::Spawner;
use error::Error;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    peripherals::Peripherals,
    timer::timg::TimerGroup,
    uart::{Config, UartTx},
    usb::usb_serial_jtag::UsbSerialJtag,
};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> () {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 64 * 1024);
    esp_println::println!("Hello");
    if let Err(e) = run(spawner, peripherals).await {
        panic!("Zippy failed: {e:?}");
    }
    core::future::pending::<()>().await;
}

async fn run(spawner: Spawner, peripherals: Peripherals) -> Result<(), Error> {
    let _debug_uart = UartTx::new(peripherals.UART0, Config::default())?.with_tx(peripherals.GPIO6);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);
    // loop {
    //     esp_println::println!("UART works");
    //     embassy_time::Timer::after_secs(1).await;
    // }

    // spawn the interface listening for CliActions
    let (rx, tx) = UsbSerialJtag::new(peripherals.USB_DEVICE).into_async().split();

    spawner.spawn(serial::cli_driver(rx, tx).unwrap());
    Ok(())
}
