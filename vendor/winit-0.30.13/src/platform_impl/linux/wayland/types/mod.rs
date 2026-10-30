//! Wayland protocol implementation boilerplate.

pub mod cursor;
pub mod dialog;
pub mod alpha_modifier;
#[cfg(not(feature = "sctk-adwaita"))]
pub mod opacity_frame;
pub mod kwin_blur;
pub mod wp_fractional_scaling;
pub mod wp_viewporter;
pub mod xdg_activation;
