#  STM32L452 serial example

Developed with [Rust Cortex M](https://docs.rs/cortex-m-quickstart) support library, this 
code implements the UART interface on the STM32L452 chip. Tested with the Nucleo STM32L452RE-P

Screen `/dev/ttyACM0` at 115200 baud rate, see "Hello, world!" appear.

It also blinks the LED using a timer, because of course there should be a blinkenlicht!

Build using ` cargo build --release --target thumbv7em-none-eabihf` using a Nightly rust, works with
2018-09-20.

# Notes & tips

- The Nucleo `-p` version with SMPS does not have an external HSE crystal. This would not be
worth knowing if the default clock source weren't the MSE clock, which is inaccurate, and thus
not suitable for driving the USART clock. Example using the MSI clock, with auto-calibration.
- The `write()` function in the peripherals lib operates on an internal reset state, not he 
current value in the target register. Use `modify()` if you want to base the modification
on the current register state
- If something does not work as expected, but also does not trigger any faults, it's the clock source.
**It's always the clock source**. Use MXCube for figuring out the clock configuration values.
- The `svd` command provided by the `gdb.py` script (check inside for install instructions) is a lifesafer.
- Use Atollic TrueStudio when in need of a decent GDB GUI

# TODO

cleanup


