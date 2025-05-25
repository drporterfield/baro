//! Reads the Barometric Pressure

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Delay, Timer};
use {defmt_rtt as _, panic_probe as _};
use bmp390::{Bmp390};

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

// Create a static Mutex to share the sensor
static SENSOR: Mutex<ThreadModeRawMutex, Option<Bmp390<Twim<'static, peripherals::TWISPI0>>>> =
    Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let twim_config = twim::Config::default();
    let twi = Twim::new(
        p.TWISPI0, 
        Irqs, 
        p.P0_27, // SDA
        p.P0_28, // SCL
        twim_config);

    info!("Initializing BMP...");
    let bmp390_config = bmp390::Configuration::default();
    let sensor = Bmp390::try_new(twi, bmp390::Address::Up, Delay, &bmp390_config)
        .await
        .unwrap();

    // Store the sensor in the static Mutex
    *SENSOR.lock().await = Some(sensor);

    unwrap!(spawner.spawn(measure()));
}

#[embassy_executor::task]
async fn measure() {
    loop {
       // Access the shared sensor
       if let Some(sensor) = SENSOR.lock().await.as_mut() {

        // Take one measurement and print it to info
        let measurement = sensor.measure().await.unwrap();
        defmt::info!("Measurement: {}", measurement);
        } else {
            defmt::warn!("Sensor not initialized!");
        }

        // Wait for one second before taking the next measurement
        Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
