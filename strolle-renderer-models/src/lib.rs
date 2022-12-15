#![no_std]

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Params {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}
