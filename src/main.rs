//! Reads the Barometric Pressure

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::{bind_interrupts, peripherals};
use embassy_time::Delay;
use {defmt_rtt as _, panic_probe as _};
use bmp390::{Bmp390};

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
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
    let mut sensor = Bmp390::try_new(twi, bmp390::Address::Up, Delay, &bmp390_config)
        .await
        .unwrap();

    // take one measurement and print it to info
    let measurement = sensor.measure().await.unwrap();
    defmt::info!("Measurement: {}", measurement);
}
