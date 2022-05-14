#![warn(clippy::all)]

pub mod notify;

#[cfg(feature = "renderer")]
mod texture;

#[cfg(any(feature = "framework", feature = "renderer"))]
pub mod gfx;
#[cfg(feature = "renderer")]
mod renderer;

#[cfg(feature = "framework")]
mod simple;
#[cfg(feature = "framework")]
pub use simple::Framework;
