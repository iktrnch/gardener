#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_rp::i2c::{Config as I2cConfig, I2c};
use embassy_rp::peripherals::USB;
use embassy_rp::rom_data::reset_to_usb_boot;
use embassy_rp::usb::{Driver as UsbDriver, InterruptHandler as UsbInterruptHandler};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_time::{Duration, Instant, Timer};
use embassy_usb::class::hid::{
    Config as HidConfig, HidBootProtocol, HidSubclass, HidWriter, State as HidState,
};
use embassy_usb::{Builder as UsbBuilder, Config as UsbConfig};
use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_7X14},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use ssd1306::{I2CDisplayInterface, Ssd1306, prelude::*};
use usbd_hid::descriptor::{KeyboardReport, MouseReport, SerializedDescriptor};

use panic_halt as _;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let usb_driver = UsbDriver::new(p.USB, Irqs);
    let mut usb_config = UsbConfig::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("Macropad");
    usb_config.product = Some("Macropad HID");
    usb_config.serial_number = Some("00000001");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;
    usb_config.composite_with_iads = false;
    usb_config.device_class = 0;
    usb_config.device_sub_class = 0;
    usb_config.device_protocol = 0;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buffer = [0; 64];
    let mut keyboard_hid_state = HidState::new();
    let mut mouse_hid_state = HidState::new();

    let mut usb_builder = UsbBuilder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buffer,
    );

    let keyboard_hid_config = HidConfig {
        report_descriptor: KeyboardReport::desc(),
        request_handler: None,
        poll_ms: 10,
        max_packet_size: 8,
        hid_subclass: HidSubclass::No,
        hid_boot_protocol: HidBootProtocol::None,
    };
    let mouse_hid_config = HidConfig {
        report_descriptor: MouseReport::desc(),
        request_handler: None,
        poll_ms: 10,
        max_packet_size: 5,
        hid_subclass: HidSubclass::No,
        hid_boot_protocol: HidBootProtocol::None,
    };

    let mut keyboard_writer = HidWriter::<_, 8>::new(
        &mut usb_builder,
        &mut keyboard_hid_state,
        keyboard_hid_config,
    );
    let mut mouse_writer =
        HidWriter::<_, 5>::new(&mut usb_builder, &mut mouse_hid_state, mouse_hid_config);
    let mut usb = usb_builder.build();

    let i2c = I2c::new_blocking(p.I2C0, p.PIN_1, p.PIN_0, I2cConfig::default());
    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    let _ = display.init();

    let mut led = Output::new(p.PIN_17, Level::Low);
    let button_2 = Input::new(p.PIN_2, Pull::Up);
    let button_3 = Input::new(p.PIN_3, Pull::Up);
    let button_4 = Input::new(p.PIN_4, Pull::Up);

    let text_style = MonoTextStyle::new(&FONT_7X14, BinaryColor::On);
    let mut previous_pressed = [false; 3];
    let mut press_order = [0_u64; 3];
    let mut next_press_order = 0_u64;
    let mut last_press_at: [Option<Instant>; 3] = [None; 3];
    let mut displayed_button: Option<Option<u8>> = None;
    let mut button_2_held_since: Option<Instant> = None;
    let mut keyboard_a_active = false;
    let mut mouse_left_active = false;

    let keyboard_signal = Signal::<NoopRawMutex, bool>::new();
    let mouse_signal = Signal::<NoopRawMutex, bool>::new();
    keyboard_signal.signal(false);
    mouse_signal.signal(false);

    let app_fut = async {
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
                    let debounce_complete = match last_press_at[index] {
                        Some(last_press) => last_press.elapsed() >= Duration::from_millis(30),
                        None => true,
                    };

                    if debounce_complete {
                        last_press_at[index] = Some(Instant::now());
                        next_press_order += 1;
                        press_order[index] = next_press_order;

                        match index {
                            0 => {
                                mouse_left_active = !mouse_left_active;
                                mouse_signal.signal(mouse_left_active);
                            }
                            1 => {
                                keyboard_a_active = !keyboard_a_active;
                                keyboard_signal.signal(keyboard_a_active);
                            }
                            _ => {}
                        }
                    }
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

                let button_label = match selected_button {
                    Some(2) => "Button 2",
                    Some(3) => "Button 3",
                    Some(4) => "Button 4",
                    _ => "No button",
                };

                let _ = Text::with_baseline(
                    "Click any button",
                    Point::new(0, 2),
                    text_style,
                    Baseline::Top,
                )
                .draw(&mut display);
                let _ = Text::with_baseline(
                    button_label,
                    Point::new(0, 32),
                    text_style,
                    Baseline::Bottom,
                )
                .draw(&mut display);
                let _ = display.flush();
                displayed_button = Some(selected_button);
            }

            Timer::after(Duration::from_millis(1)).await;
        }
    };

    let keyboard_fut = async {
        loop {
            let active = keyboard_signal.wait().await;
            let report = if active {
                [0, 0, 4, 0, 0, 0, 0, 0]
            } else {
                [0; 8]
            };
            let _ = keyboard_writer.write(&report).await;
        }
    };

    let mouse_fut = async {
        loop {
            let active = mouse_signal.wait().await;
            let report = [u8::from(active), 0, 0, 0, 0];
            let _ = mouse_writer.write(&report).await;
        }
    };

    join(usb.run(), join(app_fut, join(keyboard_fut, mouse_fut))).await;
}
