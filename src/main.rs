//! Reads the Barometric Pressure, Temperature, and Altitude

#![no_std]
#![no_main]

use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::i2c::{self, I2cDevice};
use embassy_executor::Spawner;
use embassy_nrf::{ bind_interrupts, peripherals, twim };
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embedded_graphics::{mono_font::ascii::FONT_5X8, text::Text};
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::BinaryColor, prelude::*};
use embassy_time::{ Timer, Delay };
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};
use bmp390::{Bmp390};
use ssd1306::{prelude::*, rotation, I2CDisplayInterface, Ssd1306Async};

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("Initializing TWI...");
    static I2C_BUS: StaticCell<Mutex<NoopRawMutex, twim::Twim<peripherals::TWISPI0>>> = StaticCell::new();
    
    let twim_config = twim::Config::default();
    let i2c = twim::Twim::new(
        p.TWISPI0, 
        Irqs, 
        p.P0_27, // SDA
        p.P0_28, // SCL
        twim_config);
    let i2c_bus = Mutex::new(i2c);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    // This is one measurement using the BMP
    info!("Initializing BMP...");

    let i2c_sensor = I2cDevice::new(i2c_bus);

    let bmp390_config = bmp390::Configuration::default();
    let mut sensor = Bmp390::try_new(i2c_sensor, bmp390::Address::Up, Delay, &bmp390_config)
        .await
        .unwrap();

    // take one measurement and print it to info
    let measurement = sensor.measure().await.unwrap();
    defmt::info!("Measurement: {}", measurement);
    

    // Intended to be a hello world with the display. 
    info!("Initializing Display...");
    let i2c_display = I2cDevice::new(i2c_bus);
    let interface = I2CDisplayInterface::new(i2c_display); 
    let mut disp = Ssd1306Async::new(interface, DisplaySize64x32, rotation::DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();
    
    disp.init().await.expect("Display initialization");
    disp.flush().await.expect("Cleans the display");

    let style = MonoTextStyle::new(&FONT_5X8, BinaryColor::On);

    Text::new("Hello Violet!", Point::new(0, 12), style) // (10, 24) halved these for 64x32
        .draw(&mut disp)
        .expect("Drawing text");

    disp.flush().await.expect("Render display");
    
    // unwrap!(spawner.spawn(measure(i2c_sensor)));
    // unwrap!(spawner.spawn(display()));

}

#[embassy_executor::task]
async fn measure() {

}

#[embassy_executor::task]
async fn display() {
}
