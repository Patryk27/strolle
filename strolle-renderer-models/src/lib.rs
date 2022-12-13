#![no_std]

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Params {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
