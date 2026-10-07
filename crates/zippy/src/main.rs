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
    peripherals::{FROM_CPU_INTR0, TIMG0},
    timer::timg::TimerGroup,
    uart::{Config, UartTx},
};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> () {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(size: 64 * 1024);
    esp_println::println!("Hello");

    #[cfg(bare_metal)]
    let rw = esp_hal::usb::usb_serial_jtag::UsbSerialJtag::new(peripherals.USB_DEVICE).into_async();
    #[cfg(emulated)]
    let rw = esp_hal::uart::Uart::new(peripherals.UART1, Default::default())
        .expect("UART1 initialization failed")
        .into_async();
    let _debug_uart = UartTx::new(peripherals.UART0, Config::default())
        .expect("UART failed to init")
        .with_tx(peripherals.GPIO6);
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
