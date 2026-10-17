#![no_std]
#![no_main]

mod actions;
mod configuration;
mod db;
mod error;
mod net;
mod serial;

use anyhow::Result;
use crypto_common::rand_core::Rng;
use embassy_executor::Spawner;
use embassy_net::Config;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    peripherals::{FROM_CPU_INTR0, HMAC, TIMG0, WIFI},
    rng::{Trng, TrngSource},
    timer::timg::TimerGroup,
};
use esp_storage::FlashStorage;
use static_cell::StaticCell;

use crate::db::{Database, EncryptionKey};

extern crate alloc;

esp_bootloader_esp_idf::esp_app_desc!();

// NOTE: DHCP and DNS need one socket slot if enabled. This is why we're provisioning space for 3
// sockets here: one for DHCP, one for DNS, and one for your code (e.g. TCP). If you use more
// sockets you must increase this. If you don't enable DHCP or DNS you can decrease it. https://docs.rs/embassy-net/latest/embassy_net/struct.Stack.html#example
static NET_RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();

#[esp_rtos::main]
async fn main(spawner: Spawner) -> () {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(
        #[esp_hal::ram(reclaimed)]
        size: 64 * 1024
    );
    esp_alloc::heap_allocator!(size: 36 * 1024);
    esp_println::println!("Hello");

    let rw = esp_hal::uart::Uart::new(peripherals.UART0, Default::default())
        .expect("UART init failed")
        .with_rx(peripherals.GPIO5)
        .with_tx(peripherals.GPIO6)
        .into_async();

    let db = FlashStorage::new(peripherals.FLASH);
    // need to register a TrngSource and keep it alive during execution.
    let _entropy = TrngSource::new(peripherals.RNG, peripherals.ADC1);

    if let Err(e) = run()
        .spawner(spawner)
        .timer(peripherals.TIMG0)
        .cpu(peripherals.FROM_CPU_INTR0)
        .rw(rw)
        .db(db)
        .hmac(peripherals.HMAC)
        .wifi(peripherals.WIFI)
        .call()
        .await
    {
        panic!("Zippy failed: {e:?}");
    }
    core::future::pending::<()>().await;
}

#[bon::builder]
async fn run(
    spawner: Spawner,
    timer: TIMG0<'static>,
    cpu: FROM_CPU_INTR0<'static>,
    rw: esp_hal::uart::Uart<'static, esp_hal::Async>,
    db: FlashStorage<'static>,
    hmac: HMAC<'static>,
    wifi: WIFI<'static>,
) -> Result<()> {
    let timg0 = TimerGroup::new(timer);
    esp_rtos::start(timg0.timer0, cpu);

    // init serial connection
    esp_println::println!("spawning tasks\r");
    static COMMANDS: Channel<CriticalSectionRawMutex, roasted_types::zippy::CliAction, 8> =
        Channel::new();
    let (uart_rx, uart_tx) = rw.split();
    let uart = serial::UartCli::builder().uart_rx(uart_rx).tx(COMMANDS.dyn_sender()).build();
    spawner.spawn(serial::cli_driver(uart)?);

    // init the actions runner
    let key = EncryptionKey::hmac(hmac)?;
    let db = Database::new(db, key)?;
    let cmds = actions::CommandExecutor::builder()
        .db(db)
        .uart_tx(uart_tx)
        .rx(COMMANDS.dyn_receiver())
        .build()?;
    spawner.spawn(actions::run(cmds)?);

    // init wifi
    let controller = esp_radio::wifi::WifiController::new(wifi, Default::default())?;
    let driver = esp_radio::wifi::Interface::station();
    let mut dhcp = embassy_net::DhcpConfig::default();
    let hostname = heapless::String::try_from(crate::configuration::net::HOSTNAME).expect("static");
    dhcp.hostname = Some(hostname);
    let config = Config::dhcpv4(dhcp);
    let mut rng = Trng::try_new().expect("enabled source in main()");
    let seed = rng.next_u64();
    let (stack, runner) = embassy_net::new(
        driver,
        config,
        NET_RESOURCES.init(embassy_net::StackResources::new()),
        seed,
    );
    spawner.spawn(net::bg_runner(runner)?);
    spawner.spawn(net::ws(stack)?);
    Ok(())
}
