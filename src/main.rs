#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let mut led1 = Output::new(p.P0_13, Level::Low, OutputDrive::Standard);
    let mut led2 = Output::new(p.P0_14, Level::Low, OutputDrive::Standard);

    loop {
        led1.set_high();
        led2.set_low();
        Timer::after_millis(300).await;
        led1.set_low();
        led2.set_high();
        Timer::after_millis(300).await;
    }
}
