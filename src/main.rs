#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{self, AnyPin, Pin};
use embassy_rp::i2c::{self, Config, InterruptHandler};
use embassy_rp::peripherals::I2C1;
use embassy_time::{Duration, Timer};
use gpio::{Level, Output};
use tmp117::*;
use {defmt_rtt as _, panic_probe as _};

// bind I2c IRQ
bind_interrupts!(struct Irqs {
    I2C1_IRQ => InterruptHandler<I2C1>;
});

// Declare async tasks
#[embassy_executor::task]
async fn blink(pin: AnyPin) {
    let mut led = Output::new(pin, Level::Low);

    loop {
        info!("led on!");
        led.set_high();
        Timer::after(Duration::from_secs(1)).await;

        info!("led off!");
        led.set_low();
        Timer::after(Duration::from_secs(1)).await;
    }
}

// Main is itself an async task as well.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Spawn a blinky task running in the background, concurrently with the main task.
    spawner.spawn(blink(p.PIN_25.degrade())).unwrap();

    // Communicate with tmp117
    info!("Setting up I2C");
    let instance = p.I2C1;
    let sda = p.PIN_14;
    let scl = p.PIN_15;
    let i2c = i2c::I2c::new_async(instance, scl, sda, Irqs, Config::default());
    let mut tmp117 = Tmp117::identify(i2c).unwrap();

    info!("Soft Reset tmp117");
    tmp117.reset().unwrap();
    // After a soft-reset, wait 200ms before starting conversions
    Timer::after(Duration::from_millis(200)).await;

    info!("Configure conversion mode");
    tmp117.enable().unwrap();

    info!("Read tmp117 TEMP_RESULT register");
    loop {
        let temperature = tmp117.read_temperature().unwrap();
        let (high, low) = tmp117.read_temperature_limits().unwrap();

        info!("Read value {} -> H: {} L: {}", temperature, high, low);
        Timer::after(Duration::from_secs(1)).await;
    }
}
