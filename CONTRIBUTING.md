# Contributing guide

Thank you for investing your time in contributing to calendar Rust libraries!

## Development

The development environment is managed by [Nix](https://nixos.org/download.html).
Running `nix-shell` will spawn a shell with everything you need to get started with the lib.

If you do not want to use Nix, you can either use [rustup](https://rust-lang.github.io/rustup/index.html):

```text
rustup update
```

or install manually the following dependencies:

- [cargo](https://doc.rust-lang.org/cargo/)
- [rustc](https://doc.rust-lang.org/stable/rustc/platform-support.html)

## Build

```text
cargo build
```

You can disable default [features](https://doc.rust-lang.org/cargo/reference/features.html) with `--no-default-features` and enable features with `--features feat1,feat2,feat3`.

Finally, you can build a release with `--release`:

```text
cargo build --no-default-features --release
```

## Structure

The core library [`calendar`](https://github.com/pimalaya/calendar/tree/master/calendar-lib) follows the [Sans I/O](https://sans-io.readthedocs.io/) pattern. It does not rely on any standard environment or async runtime. These are provided by external libraries called I/O connectors.

![sans-io](./sans-io.svg)

- A flow is a state machine defined as a Rust `Iterator` producing I/O request(s).
- The I/O connector is responsible for executing the requested I/O.
- The I/O connector communicates with the flow via its inner I/O state (take input, set output).
- A flow that does not produce any(more) I/O requests is considered terminated.
- A terminated flow exposes an `output()` function that takes the final output away from itself.

This repository comes with few I/O connectors:

- [`calendar-caldav`](https://github.com/pimalaya/calendar/tree/master/calendar-caldav), a standard (blocking) Caldav I/O connector
- [`calendar-caldav-rustls`](https://github.com/pimalaya/calendar/tree/master/calendar-caldav-rustl), a standard (blocking) Caldav I/O connector over TLS using [`rustls`](https://docs.rs/rustls/latest/rustls/)
- [`calendar-caldav-native-tls`](https://github.com/pimalaya/calendar/tree/master/calendar-caldav-native-tls), a standard (blocking) Caldav I/O connector over TLS using [`native-tls`](https://docs.rs/native-tls/latest/native_tls/)
- [`calendar-vdir`](https://github.com/pimalaya/calendar/tree/master/calendar-vdir), a standard (blocking) [vdir](https://vdirsyncer.pimutils.org/en/stable/vdir.html) I/O connector

## Commit style

Calendar libraries follow the [conventional commits specification](https://www.conventionalcommits.org/en/v1.0.0/#summary).
