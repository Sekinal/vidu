//! Application module
//!
//! This module contains the main application state and event handling logic.

mod actions;
mod input;
mod state;

pub use input::{
    Action, DeleteConfirmAction, HelpAction, KeyBindings, PreviewAction, SearchAction,
};
pub use state::{App, AppState, SortMode, SortOrder, ViewMode};