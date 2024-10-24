pub mod function_selection;
pub mod profile_selection;

#[derive(Debug)]
pub enum AppState {
    ProfileSelection,
    FunctionList,
}
