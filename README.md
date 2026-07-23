# Statepad

> [!CAUTION]
> **statepad** can automate keyboard and mouse input. Do not use it where automation is prohibited, including games, competitive environments, examinations, or services that forbid macros or scripted input. Users are responsible for ensuring that their configuration complies with all applicable rules.

**statepad** is a general-purpose programmable USB HID controller for building
custom keyboard input devices and automating repetitive computer tasks. It can
represent input behaviour as configurable states and transitions, allowing it
to produce keyboard and mouse actions in response to physical button presses.

## Parts used in the project
- [TENSTAR RP2040 Pro Micro Development Board 16MB](https://www.aliexpress.com/item/1005009890367599.html)
- [2pcs PBT Keycaps](https://www.aliexpress.com/item/1005009364428646.html)
	- **OR** any cherry compatible keycap works
- [2pc Mechanical Keyboard Switches](https://www.aliexpress.com/item/1005012157731141.html)
	- **OR** any cherry compatible switches
- [5 pairs JST 1.25mm](https://www.aliexpress.com/item/1005007617385129.html)
	- 1.25mm are possible but may cause clearance issues
	- Not mandatory but makes life easier
- [1 pcs 6x6x4.3MM 4PIN Push Button](https://www.aliexpress.com/item/32874867657.html)
- [SSD1306 128X32 0.91 inch OLED display](https://www.aliexpress.com/item/1005004622593515.html)
	- Has to be SSD1306
	- Has to support i2c

## Wiring diagram
<p align="center">
  <img src="https://raw.githubusercontent.com/iktrnch/statepad/refs/heads/main/assets/wiring.svg" alt="Wiring Diagram">
</p>

## Printing the enclosure
Ready-to-print enclosure files are available on the [Releases](../../releases) page. Print them at 100% scale using PLA or PLA+. The base should be printed flat with the internal mounts facing upward and does not require supports. Print the top shell in its normal orientation with the open bottom facing the build plate, using build-plate-only supports and a small brim.

| Setting | Recommended value |
|---|---|
| Nozzle | 0.4 mm |
| Layer height | 0.16–0.20 mm |
| Walls | 3 |
| Infill | 15% |
| Top shell supports | Normal/snug, build plate only |
| Top shell brim | 5–6 mm |
| Base supports | Off |

Remove all supports before fitting the components, and lightly file any openings that are too tight due to printer tolerances. Flash and test the RP2040 before installing it in the enclosure, since its onboard BOOT and RESET buttons are difficult to access after assembly.

## Flashing the firmware
Before compiling the project run the following in your terminal:
```bash
rustup target add thumbv6m-none-eabi
cargo install elf2uf2-rs
```
Afterwards, define the desired profiles in `src/profile.rs`, specifying the keyboard and mouse output for each state and any timed transitions between them.
It is recommended to flash the board before installing it in the enclosure, as the onboard **BOOT** and **RESET** buttons are difficult to access afterwards.
For the first flash, connect the board by USB, hold **BOOT**, briefly press **RESET**, then release **BOOT** and run:
```bash
cargo run --release
```
The firmware will be compiled, copied to the RP2040 bootloader drive, and started automatically. For subsequent flashes, leave the board connected and hold the **top preset button** for five seconds. The firmware will release all active HID inputs and restart the board in USB bootloader mode. Once the `RPI-RP2` device appears, run `cargo run --release` again.
