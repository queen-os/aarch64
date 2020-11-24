#![no_std]
#![feature(asm)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(bool_to_option)]
#![feature(step_trait)]
#![feature(step_trait_ext)]

pub mod addr;
pub mod asm;
pub mod paging;
pub mod cache;
pub mod registers;
pub mod barrier;


pub extern crate ux;
