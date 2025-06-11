#![warn(clippy::pedantic)]

mod installer;

#[cfg(feature = "packager")]
pub use flint_packager::*;
pub use installer::*;
