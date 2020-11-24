#![no_std]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(bool_to_option)]
#![feature(step_trait)]
#![feature(step_trait_ext)]

#[macro_use]
extern crate register;
#[macro_use]
extern crate bitflags;

pub mod addr;
pub mod asm;
pub mod paging;

pub use cortex_a::*;

pub extern crate ux;
