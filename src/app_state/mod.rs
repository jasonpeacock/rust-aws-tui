pub mod date_selection;
pub mod function_selection;
pub mod log_viewer;
pub mod profile_selection;

#[derive(Debug, PartialEq)]
pub enum AppState {
    ProfileSelection,
    FunctionList,
    DateSelection,
    LogViewer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusedPanel {
    Left,
    Right,
}

impl Default for FocusedPanel {
    fn default() -> Self {
        Self::Left
    }
}
