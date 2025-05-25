# baro

This respository is a work in progress, to make an altimeter for a stomp rocket (and eventually, rockets with engines).

## Setup baro
### Flash firmware using probe-rs/jlink connected to an nrf52840 devkit:
`cd smartneb-firmware && cargo run --release`

### Just test that it builds:
`cd smartneb-firmware && cargo build --release`

Then use the terminal to inspect the output. 

## Why
As a learning exercise, it will be programmed mostly in Rust, with Embassy on the nRF52840 (specifically, the tiny seeed studios supported Xiao microcontroller). This project is designed to get my daughter interested in rocket science. Her name is Violet, hence the crazy coloring in VSCode (courtesy of the "Peacock" plugin).

Embassy is added exclusively through the use of `cargo add` because why not. We're here to learn, and if stuff breaks because we're on the bleeding edge, then it will only be another learning opportunity.