#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{get_foreground_process_name, list_running_process_names, show_distraction_popup};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{get_foreground_process_name, list_running_process_names, show_distraction_popup};
