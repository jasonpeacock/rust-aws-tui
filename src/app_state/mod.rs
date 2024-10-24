pub mod date_selection;
pub mod function_selection;
pub mod profile_selection;

#[derive(Debug, PartialEq)]
pub enum AppState {
    ProfileSelection,
    FunctionList,
    DateSelection,
}
