# atmega32u4-pwm-fan-controller-cli

Companion CLI for [atmega32u4-pwm-fan-controller](https://github.com/0x6b/atmega32u4-pwm-fan-controller).

## Tested Device

- [Adafruit Feather 32u4 Bluefruit LE](https://learn.adafruit.com/adafruit-feather-32u4-bluefruit-le) (`feather32u4`)
- macOS Ventura 13.3

## Install

```console
$ cargo install --git https://github.com/0x6b/atmega32u4-pwm-fan-controller-cli
```

## Usage

```console
$ fanctl -h
fanctl 0.1.0
Control the fan speed

USAGE:
    fanctl [speed]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <speed>    Fan speed in percentage [default: 10]
```

## LICENSE

MIT. See [LICENSE](LICENSE) for details.

