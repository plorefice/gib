# gb-rs

[![CircleCI](https://circleci.com/gh/plorefice/gb-rs.svg?style=shield)](https://circleci.com/gh/plorefice/gb-rs)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-GPLv3-blue.svg)](LICENSE)

Original Gameboy (DMG) emulator written in Rust, also featuring several tools
for ROM debugging and development.

**NOTE**: This is still a WIP, no ROM playable yet.

## Building the project

After installing Rust (you can use [rustup](https://rustup.rs) for that), run:

```shell
git clone git@github.com:plorefice/gb-rs
cd gb-rs/
cargo +nightly build --release
```

Only `nightly` versions of Rust are supported right now.

## Running the emulator

Once you have a ROM file, you can use:

```shell
cargo +nightly run --release [rom-file]
```

where the optional `[rom-file]` argument can be used to load a ROM directly from
the command line. Alternatively, you can use the in-app menus.

## Running tests

Currently, unit tests exist for opcode size and timings, along with some peripherals.
In the future, more complete tests will be developed. Some golden tests are also
included to test against known working test ROMs (eg. blargg's).

You can run the test suite with:

```shell
cargo +nightly test --release
```

## Features

The emulator is still a long way from being complete. The current status and roadmap
are shown below.

### Progress

| Peripheral | Progress | Notes                                  |
| ---------- | -------- | -------------------------------------- |
| CPU        | 100%     | More testing required                  |
| Video      | 10%      | BG support and screen blanking only    |
| Sound      | 0%       | Not implemented yet                    |
| Joypad     | 0%       | Not implemented yet                    |
| Link cable | 0%       | Not implemented yet                    |
| Timers     | 95%      | More testing required                  |
| Interrupts | 70%      | Interrupt handling mechanism supported |
| MBC        | 20%      | Support for some functions of MBC1     |

### Blargg's Test ROMs

[Blargg's Gameboy hardware test ROMs](https://github.com/retrio/gb-test-roms) results.

The passing tests are also integrated in the emulator's test suite.

| Test ROM       | Progress | Notes          |
| -------------- | -------- | -------------- |
| cpu_instrs     | 100%     | Full pass!     |
| instr_timing   | 100%     | Full pass!     |
| interrupt_time | -        | Not tested yet |
| mem_timing-2   | -        | Not tested yet |
| mem_timing     | -        | Not tested yet |
| halt_bug       | -        | Not tested yet |
| oam_bug        | -        | Not tested yet |
| dmg_sound      | -        | Not tested yet |

### Gekkio's test suite

[Gekkio's mooneye-gb test ROMs](https://gekkio.fi/files/mooneye-gb/latest/) results.

The passing tests are also integrated in the emulator's test suite.

| Test Suite       | Progress | Notes      |
| ---------------- | -------- | ---------- |
| acceptance/timer | 100%     | Full pass! |

## Resources

- [GBDev Wiki](http://gbdev.gg8.se/wiki/articles/Main_Page)
- [The PanDocs](http://bgb.bircd.org/pandocs.htm)
- [Pastraiser's Gameboy CPU (LR35902) instruction set](http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [GameBoy Opcode Summary](http://www.devrs.com/gb/files/opcodes.html)
- [gbz80](https://rednex.github.io/rgbds/gbz80.7.html)
- [Gekkio's mooneye-gb test ROM sources](https://github.com/Gekkio/mooneye-gb/tree/master/tests)

... and any other brave soul posting any kind of GB info on the Internet :pray:
