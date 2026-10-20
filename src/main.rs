//! Reads the Barometric Pressure, Temperature, and Altitude
#![no_std]
#![no_main]

use defmt::{info};
use embassy_embedded_hal::shared_bus::asynch::i2c::{I2cDevice};
use embassy_executor::Spawner;
use embassy_nrf::{ bind_interrupts, peripherals };
use embassy_nrf::peripherals::TWISPI0;
use embassy_nrf::twim::{ self, Twim };
use embassy_nrf::gpio::{Input, Pull};
use embassy_sync::mutex::Mutex; 
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex, CriticalSectionRawMutex};
use embedded_graphics::{mono_font::ascii::FONT_7X13, text::Text};
use embedded_graphics::{mono_font::MonoTextStyle, pixelcolor::BinaryColor, prelude::*};
use embassy_time::{ Delay, Duration, Ticker, block_for };
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};
use bmp581::{Bmp581, I2cAddr};
use bmp581::types::{Odr, Osr, OsrConfig, PowerMode};
use ssd1306::{prelude::*, rotation, I2CDisplayInterface, Ssd1306Async};
use embassy_sync::signal::Signal;
use uom::si::length::foot;
use heapless::String;
use core::fmt::Write;

static SHARED: Signal<ThreadModeRawMutex, f32> = Signal::new();

type I2c1Bus = Mutex<NoopRawMutex, Twim<'static, peripherals::TWISPI0>>;

const BMP_ADDR: I2cAddr = I2cAddr::Alternative;

bind_interrupts!(struct Irqs {
    TWISPI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[derive(Clone, Copy)]
pub struct Measurement {
    pub pressure_pa: f32,
    pub temp_c: f32,
}

pub static MEASUREMENT: Signal<CriticalSectionRawMutex, Measurement> = Signal::new();


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());

    info!("Initializing TWI...");
    
    // shared i2c bus
    static I2C_BUS: StaticCell<I2c1Bus> = StaticCell::new();
    
    let twim_config = twim::Config::default();
    let i2c = twim::Twim::new(
        p.TWISPI0, 
        Irqs, 
        p.P0_27, // SDA P0_27 nrfdk, P0_04 xiao
        p.P0_28, // SCL P0_28 nrfdk, P0_05 xiao
        twim_config);
    let i2c_bus = Mutex::new(i2c);
    let i2c_bus = I2C_BUS.init(i2c_bus);

    // note: must_spawn will panic if anything goes wrong here.
    spawner.must_spawn(measure(i2c_bus));   
    // Timer::after_millis(100).await;
    // spawner.must_spawn(display(i2c_bus));
    loop {
        let r = MEASUREMENT.wait().await;
        defmt::info!("{} Pa, {} C", r.pressure_pa, r.temp_c);
    }
}

#[embassy_executor::task]
async fn measure(bus: &'static I2c1Bus) {
    // One-time setup
    {
        let mut twim = bus.lock().await;
        let mut sensor = Bmp581::new_i2c(&mut *twim, BMP_ADDR);
        
        // BMP581
        // sensor.init(&mut Delay).unwrap();

        //BMP585
        sensor.soft_reset().unwrap();
        block_for(Duration::from_millis(2));
        let id = sensor.get_chip_id().unwrap();
        defmt::info!("BMP58x chip id: {=u8:#x}", id);
        if id != 0x50 && id != 0x51 {
            defmt::panic!("unexpected chip id {=u8:#x}", id);
        }

        sensor.set_osr_config(OsrConfig {
            pressure_enable: true,
            osr_pressure: Osr::Osr128,
            osr_temperature: Osr::Osr8,
        }).unwrap();
        sensor.set_odr(Odr::Hz10_000).unwrap();
        sensor.set_power_mode(PowerMode::Normal).unwrap();
    } // lock released here, so the screen can use the bus

    let mut ticker = Ticker::every(Duration::from_millis(100));
    loop {
        ticker.next().await;
        let result = {
            let mut twim = bus.lock().await;
            let mut sensor = Bmp581::new_i2c(&mut *twim, BMP_ADDR);
            sensor.read_pressure().and_then(|p| Ok((p, sensor.read_temperature()?)))
        };
        match result {
            Ok((p, t)) => MEASUREMENT.signal(Measurement { pressure_pa: p, temp_c: t }),
            Err(e) => defmt::warn!("bmp58x: {}", defmt::Debug2Format(&e)),
        }
    }
}

#[embassy_executor::task]
async fn display(i2c_bus: &'static I2c1Bus) {
    // Intended to be a hello world with the display. 
    info!("Initializing Display...");
    let i2c_display = I2cDevice::new(i2c_bus);
    let interface = I2CDisplayInterface::new_alternate_address(i2c_display); 
    let mut disp = Ssd1306Async::new(interface, DisplaySize128x64, rotation::DisplayRotation::Rotate0)
    .into_buffered_graphics_mode();
    
    disp.init().await.expect("Display initialization");
    disp.flush().await.expect("Cleans the display");
    let mut heapless_string: String<32> = String::new();

    loop {
        disp.clear(BinaryColor::Off).expect("Clearing the display");
        let style = MonoTextStyle::new(&FONT_7X13, BinaryColor::On);

        let val = SHARED.wait().await;
        write!(heapless_string, "{:.1}", val).expect("format failure");
        Text::new(&heapless_string, Point::new(0, 12), style) // (10, 24) halved these for 64x32
            .draw(&mut disp)
            .expect("Drawing text");
    
        disp.flush().await.expect("Render display");
        heapless_string.clear(); // if we don't clear the string, it will write to the next available position.
    }
}