//! Backend abstraction layer for terminal input.
//!
//! This module provides a unified interface for reading terminal events,
//! abstracting over the differences between termion (macOS/Linux) and
//! crossterm (Windows) backends.
//!
//! The backend is selected at compile time via feature flags:
//! - `backend-termion` (default): Uses termion for macOS/Linux
//! - `backend-crossterm`: Uses crossterm for Windows

mod event;

#[cfg(feature = "backend-termion")]
mod termion_backend;

#[cfg(feature = "backend-crossterm")]
mod crossterm_backend;

pub use event::{BackendEvent, BackendKey, BackendMouse};

#[cfg(feature = "backend-termion")]
pub use termion_backend::EventReader;

#[cfg(feature = "backend-crossterm")]
pub use crossterm_backend::EventReader;
