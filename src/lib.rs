#![feature(try_from)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;

mod gb;

pub use gb::dbg;
pub use gb::GameBoy;
