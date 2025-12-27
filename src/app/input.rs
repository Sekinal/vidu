//! Input handling and key mappings

use crossterm::event::{KeyCode, KeyModifiers};

/// Actions that can be triggered by user input
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    // Application
    Quit,

    // Navigation
    MoveUp,
    MoveDown,
    GoToTop,
    GoToBottom,
    PageUp,
    PageDown,
    Enter,
    GoBack,
    GoToRoot,

    // Actions
    Delete,
    ToggleMark,
    Refresh,
    FullRescan,
    TogglePreview,

    // View
    CycleSort,
    ToggleSortOrder,
    ToggleHidden,
    StartSearch,
    ShowHelp,

    // Cleaning utility views
    ShowJunkAnalysis,
    ShowDuplicates,
    ShowFileTypes,
    ShowOldFiles,
    ShowLargeFiles,
    ShowCaches,
    CleanSelected,
    CleanAllJunk,
    ToggleDeletionMode,

    // No action
    None,
}

/// Key binding configuration
pub struct KeyBindings;

impl KeyBindings {
    /// Map a key press in browsing mode to an action
    pub fn browsing_action(code: KeyCode, modifiers: KeyModifiers) -> Action {
        match code {
            // Quit
            KeyCode::Char('q') => Action::Quit,
            KeyCode::Esc => Action::GoBack, // Will quit if at root

            // Navigation
            KeyCode::Char('j') | KeyCode::Down => Action::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => Action::MoveUp,
            KeyCode::Char('g') if modifiers == KeyModifiers::NONE => Action::GoToTop,
            KeyCode::Char('G') => Action::GoToBottom,
            KeyCode::Home => Action::GoToTop,
            KeyCode::End => Action::GoToBottom,
            KeyCode::PageDown => Action::PageDown,
            KeyCode::PageUp => Action::PageUp,

            // Enter directory
            KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => Action::Enter,

            // Go back
            KeyCode::Backspace | KeyCode::Char('h') | KeyCode::Left => Action::GoBack,

            // Delete
            KeyCode::Char('d') | KeyCode::Delete => Action::Delete,

            // Mark/unmark
            KeyCode::Char(' ') => Action::ToggleMark,

            // Sort
            KeyCode::Char('s') => Action::CycleSort,
            KeyCode::Char('S') => Action::ToggleSortOrder,

            // Refresh
            KeyCode::Char('r') => Action::Refresh,
            KeyCode::Char('R') => Action::FullRescan,

            // Preview
            KeyCode::Char('p') => Action::TogglePreview,

            // Help
            KeyCode::Char('?') => Action::ShowHelp,

            // Search
            KeyCode::Char('/') => Action::StartSearch,

            // Toggle hidden
            KeyCode::Char('.') => Action::ToggleHidden,

            // Go to root
            KeyCode::Char('~') => Action::GoToRoot,

            // Cleaning utility views (Shift keys for major views)
            KeyCode::Char('J') => Action::ShowJunkAnalysis,
            KeyCode::Char('u') => Action::ShowDuplicates,
            KeyCode::Char('T') => Action::ShowFileTypes,
            KeyCode::Char('o') => Action::ShowOldFiles,
            KeyCode::Char('L') => Action::ShowLargeFiles,
            KeyCode::Char('K') => Action::ShowCaches,

            // Cleaning actions
            KeyCode::Char('c') => Action::CleanSelected,
            KeyCode::Char('C') => Action::CleanAllJunk,
            KeyCode::Char('!') => Action::ToggleDeletionMode,

            _ => Action::None,
        }
    }

    /// Map a key press in delete confirmation mode
    pub fn delete_confirm_action(code: KeyCode) -> DeleteConfirmAction {
        match code {
            KeyCode::Char('y') | KeyCode::Char('Y') => DeleteConfirmAction::Confirm,
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                DeleteConfirmAction::Cancel
            }
            _ => DeleteConfirmAction::None,
        }
    }

    /// Map a key press in preview mode
    pub fn preview_action(code: KeyCode) -> PreviewAction {
        match code {
            KeyCode::Char('p') | KeyCode::Esc | KeyCode::Char('q') => PreviewAction::Close,
            KeyCode::Char('j') | KeyCode::Down => PreviewAction::ScrollDown,
            KeyCode::Char('k') | KeyCode::Up => PreviewAction::ScrollUp,
            KeyCode::PageDown => PreviewAction::PageDown,
            KeyCode::PageUp => PreviewAction::PageUp,
            KeyCode::Home | KeyCode::Char('g') => PreviewAction::GoToTop,
            KeyCode::End | KeyCode::Char('G') => PreviewAction::GoToBottom,
            _ => PreviewAction::None,
        }
    }

    /// Map a key press in help mode
    pub fn help_action(code: KeyCode) -> HelpAction {
        match code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Enter => {
                HelpAction::Close
            }
            _ => HelpAction::None,
        }
    }

    /// Map a key press in search mode
    pub fn search_action(code: KeyCode) -> SearchAction {
        match code {
            KeyCode::Esc => SearchAction::Cancel,
            KeyCode::Enter => SearchAction::Execute,
            KeyCode::Backspace => SearchAction::Backspace,
            KeyCode::Char(c) => SearchAction::AddChar(c),
            _ => SearchAction::None,
        }
    }
}

/// Actions for delete confirmation dialog
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteConfirmAction {
    Confirm,
    Cancel,
    None,
}

/// Actions for preview mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewAction {
    Close,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    GoToTop,
    GoToBottom,
    None,
}

/// Actions for help screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpAction {
    Close,
    None,
}

/// Actions for search mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchAction {
    Cancel,
    Execute,
    AddChar(char),
    Backspace,
    None,
}

/// Actions for analysis views (junk, duplicates, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisAction {
    Close,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    GoToTop,
    GoToBottom,
    Select,
    ToggleMark,
    CleanSelected,
    CleanAll,
    ToggleDeletionMode,
    None,
}

impl KeyBindings {
    /// Map a key press in analysis view mode
    pub fn analysis_action(code: KeyCode) -> AnalysisAction {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => AnalysisAction::Close,
            KeyCode::Char('j') | KeyCode::Down => AnalysisAction::ScrollDown,
            KeyCode::Char('k') | KeyCode::Up => AnalysisAction::ScrollUp,
            KeyCode::PageDown => AnalysisAction::PageDown,
            KeyCode::PageUp => AnalysisAction::PageUp,
            KeyCode::Home | KeyCode::Char('g') => AnalysisAction::GoToTop,
            KeyCode::End | KeyCode::Char('G') => AnalysisAction::GoToBottom,
            KeyCode::Enter => AnalysisAction::Select,
            KeyCode::Char(' ') => AnalysisAction::ToggleMark,
            KeyCode::Char('c') => AnalysisAction::CleanSelected,
            KeyCode::Char('C') => AnalysisAction::CleanAll,
            KeyCode::Char('!') => AnalysisAction::ToggleDeletionMode,
            _ => AnalysisAction::None,
        }
    }
}
