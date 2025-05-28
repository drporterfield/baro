//! Reads the Barometric Pressure, Temperature, and Altitude

#![no_std]
#![no_main]

use defmt::*;
// use embassy_embedded_hal::shared_bus::asynch::i2c::{self, I2cDevice};
use embassy_executor::Spawner;
use embassy_nrf::{ bind_interrupts, peripherals, twim };
// use embassy_sync::blocking_mutex::raw::{ ThreadModeRawMutex };
// use embassy_sync::mutex::Mutex;
use embedded_graphics::{mono_font::ascii::FONT_5X8, text::Text};
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::BinaryColor, prelude::*};
use embassy_time::{ Timer };
use {defmt_rtt as _, panic_probe as _};
use bmp390::{Bmp390};
use ssd1306::{prelude::*, rotation, I2CDisplayInterface, Ssd1306Async};


bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

// // Create a static Mutex to share the sensor
// static SENSOR: Mutex<ThreadModeRawMutex, Option<Bmp390<twim::Twim<'static, peripherals::TWISPI0>>>> =
//     Mutex::new(None);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("Initializing TWI...");
    let twim_config = twim::Config::default();
    let twi = twim::Twim::new(
        p.TWISPI0, 
        Irqs, 
        p.P0_27, // SDA
        p.P0_28, // SCL
        twim_config);

    // // This code for I2CBus sharing.
    // static I2CBUS: StaticCell<I2c1Bus> = StaticCell::new();
    // let i2c_bus = I2CBUS.init(Mutex::new(twi));

    // TODO: spawn two tasks here



    info!("Initializing Display...");
    let interface = I2CDisplayInterface::new(twi); 
    let mut disp = Ssd1306Async::new(interface, DisplaySize64x32, rotation::DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();
    
    disp.init().await.expect("Display initialization");
    disp.flush().await.expect("Cleans the display");

    let style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);

    Text::new("Hello Violet!", Point::new(0, 12), style) // (10, 24) halved these for 64x32
        .draw(&mut disp)
        .expect("Drawing text");

    disp.flush().await.expect("Render display");
    
    // info!("Initializing BMP...");
    // let bmp390_config = bmp390::Configuration::default();
    // let sensor = Bmp390::try_new(twi, bmp390::Address::Up, Delay, &bmp390_config)
    // .await
    // .unwrap();

    // unwrap!(spawner.spawn(measure()));
    unwrap!(spawner.spawn(display()));
}

#[embassy_executor::task]
async fn measure() {
    loop {
    //    //   Access the shared sensor
    //    if let Some(sensor) = SENSOR.lock().await.as_mut() {

    //     // Take one measurement and print it to info
    //     let measurement = sensor.measure().await.unwrap();
    //     defmt::info!("Measurement: {}", measurement);
    //     } else {
    //         defmt::warn!("Sensor not initialized!");
    //     }

        // Wait for one second before taking the next measurement
        Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}

#[embassy_executor::task]
async fn display() {
}
