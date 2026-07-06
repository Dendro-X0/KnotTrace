#[cfg(windows)]
mod windows;
#[cfg(not(windows))]
mod stub;

#[cfg(windows)]
pub use windows::*;
#[cfg(not(windows))]
pub use stub::*;
