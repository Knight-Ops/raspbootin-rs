//! Trait definitions for coupling `kernel` and `BSP` code.
//!
//! ```
//!         +-------------------+
//!         | Interface (Trait) |
//!         |                   |
//!         +--+-------------+--+
//!            ^             ^
//!            |             |
//!            |             |
//! +----------+--+       +--+----------+
//! | Kernel code |       |  BSP Code   |
//! |             |       |             |
//! +-------------+       +-------------+
//! ```

/// System console operations.
pub mod console {
    /// Console write functions.
    ///
    /// `core::fmt::Write` is exactly what we need. Re-export it here because
    /// implementing `console::Write` gives a better hint to the reader about
    /// the intention.
    pub use core::fmt::Write;

    /// Console read functions.
    pub trait Read {
        fn read_char(&mut self) -> char;
    }
}
