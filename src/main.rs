#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::i2c::{Config as I2cConfig, I2c};
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_6X13},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};

use panic_halt as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let i2c = I2c::new_blocking(p.I2C0, p.PIN_1, p.PIN_0, I2cConfig::default());
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    let _ = display.init();

    let mut led = Output::new(p.PIN_17, Level::Low);
    let button_2 = Input::new(p.PIN_2, Pull::Up);
    let button_3 = Input::new(p.PIN_3, Pull::Up);
    let button_4 = Input::new(p.PIN_4, Pull::Up);

    let text_style = MonoTextStyle::new(&FONT_6X13, BinaryColor::On);
    let mut previous_pressed = [false; 3];
    let mut press_order = [0_u64; 3];
    let mut next_press_order = 0_u64;
    let mut displayed_button: Option<Option<u8>> = None;
    let mut button_2_held_since: Option<Instant> = None;

    loop {
        // Each button pulls its input low while it is held.
        let pressed = [button_2.is_low(), button_3.is_low(), button_4.is_low()];

        if pressed[0] || (pressed[1] && pressed[2]) {
            led.set_high();
        } else {
            led.set_low();
        }

        if pressed[0] {
            match button_2_held_since {
                Some(held_since) if held_since.elapsed() >= Duration::from_secs(5) => {
                    reset_to_usb_boot(0, 0);
                }
                Some(_) => {}
                None => button_2_held_since = Some(Instant::now()),
            }
        } else {
            button_2_held_since = None;
        }

        for index in 0..pressed.len() {
            if pressed[index] && !previous_pressed[index] {
                next_press_order += 1;
                press_order[index] = next_press_order;
            }
        }
        previous_pressed = pressed;

        let mut selected_button = None;
        let mut selected_order = 0;

        for index in 0..pressed.len() {
            if pressed[index] && press_order[index] >= selected_order {
                selected_button = Some(index as u8 + 2);
                selected_order = press_order[index];
            }
        }

        if displayed_button != Some(selected_button) {
            display.clear_buffer();

            let label = match selected_button {
                Some(2) => "Button 2",
                Some(3) => "Button 3",
                Some(4) => "Button 4",
                _ => "No button",
            };

            let _ = Text::with_baseline(label, Point::new(0, 4), text_style, Baseline::Top)
                .draw(&mut display);
            let _ = display.flush();
            displayed_button = Some(selected_button);
        }

        Timer::after(Duration::from_millis(1)).await;
    }
}
