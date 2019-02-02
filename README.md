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

Currently, the test suite only tests opcode size and timings. In the future, more
complete tests will be developed. You can run the test suite with:

```shell
cargo +nightly test
```

## Features

The emulator is still a long way from being complete. The current status and roadmap
are shown below.

### Progress

| Peripheral | Progress | Notes                               |
| ---------- | -------- | ----------------------------------- |
| CPU        | 95%      | DAA missing, more testing required  |
| Video      | 10%      | BG support and screen blanking only |
| Sound      | 0%       | Not implemented yet                 |
| Joypad     | 0%       | Not implemented yet                 |
| Link cable | 0%       | Not implemented yet                 |
| Timers     | 80%      | INT support missing                 |
| Interrupts | 10%      | Basic IME support                   |
| MBC        | 0%       | Not implemented yet                 |

### Blargg's Test ROMs

[Blargg's Gameboy hardware test ROMs](https://github.com/retrio/gb-test-roms) results.

**NOTE**: soon there will be an automated test suite to run and check these.

| Test ROM       | Progress | Notes                  |
| -------------- | -------- | ---------------------- |
| cpu_instrs     | 30%      | 06, 07, 08, 10 passing |
| instr_timing   | -        | Not tested yet         |
| interrupt_time | -        | Not tested yet         |
| mem_timing-2   | -        | Not tested yet         |
| mem_timing     | -        | Not tested yet         |
| halt_bug       | -        | Not tested yet         |
| oam_bug        | -        | Not tested yet         |
| dmg_sound      | -        | Not tested yet         |
