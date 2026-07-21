#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_time::{Duration, Timer};

use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_17, Level::Low);
    let button_0 = Input::new(p.PIN_2, Pull::Up);
    let button_1 = Input::new(p.PIN_3, Pull::Up);
    let button_2 = Input::new(p.PIN_4, Pull::Up);

    loop {
        // Each button pulls its input low while it is held.
        if button_0.is_low() || (button_1.is_low() && button_2.is_low()) {
            led.set_high();
        } else {
            led.set_low();
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
