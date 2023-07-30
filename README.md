# atmega32u4-pwm-fan-controller-cli

Companion CLI for [atmega32u4-pwm-fan-controller](https://github.com/0x6b/atmega32u4-pwm-fan-controller).

## Tested Device

- [Adafruit Feather 32u4 Bluefruit LE](https://learn.adafruit.com/adafruit-feather-32u4-bluefruit-le) (`feather32u4`)
- macOS Ventura 13.3

## Install

```console
$ cargo install --git https://github.com/0x6b/atmega32u4-pwm-fan-controller-cli
```

For macOS, you have to permit your terminal bluetooth access.
 
## Usage

```console
$ fanctl -h
Control the fan speed of a PWM fan connected to an ATmega32U4 microcontroller via Bluetooth LE

Usage: fanctl [SPEED]

Arguments:
  [SPEED]  Fan speed in percentage [default: 10]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## LICENSE

MIT. See [LICENSE](LICENSE) for details.
