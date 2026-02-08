//! Backend-agnostic event types for terminal input.
//!
//! These types abstract over the differences between termion and crossterm event systems,
//! allowing the rest of the application to work with a unified event type regardless of
//! which backend is compiled in.

/// A backend-agnostic terminal event.
///
/// # Example
///
/// ```
/// use yamlquill::input::backend::{BackendEvent, BackendKey};
///
/// let event = BackendEvent::Key(BackendKey::Char('j'));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendEvent {
    /// A key was pressed
    Key(BackendKey),
    /// A mouse event occurred
    Mouse(BackendMouse),
}

/// A backend-agnostic key event.
///
/// Note: Enter is represented as `Char('\n')` and Tab as `Char('\t')`
/// to match termion's convention and minimize changes to the input handler.
///
/// # Example
///
/// ```
/// use yamlquill::input::backend::BackendKey;
///
/// let key = BackendKey::Char('q');
/// let ctrl = BackendKey::Ctrl('c');
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendKey {
    /// A regular character (including '\n' for Enter and '\t' for Tab)
    Char(char),
    /// Ctrl + character
    Ctrl(char),
    /// Escape key
    Esc,
    /// Backspace key
    Backspace,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up key
    PageUp,
    /// Page Down key
    PageDown,
    /// Function key (F1, F2, etc.)
    F(u8),
}

/// A backend-agnostic mouse event.
///
/// Only wheel events are supported since that's all the editor uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendMouse {
    /// Mouse wheel scrolled up
    WheelUp,
    /// Mouse wheel scrolled down
    WheelDown,
}
