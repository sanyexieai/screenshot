#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
pub mod impl_platform;

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
pub mod impl_platform;

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
pub mod impl_platform;