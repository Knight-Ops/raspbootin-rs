#[cfg(feature = "bsp_rpi3")]
pub mod rpi3;
#[cfg(feature = "bsp_rpi3")]
pub use rpi3::*;

#[cfg(feature = "bsp_rpi4")]
pub mod rpi4;
#[cfg(feature = "bsp_rpi4")]
pub use rpi4::*;
