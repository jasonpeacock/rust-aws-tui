use crate::toml_parser::Profile;
use ratatui::widgets::ListState;

#[derive(Debug)]
pub struct ProfileSelection {
    pub list_state: ListState,
    pub profiles: Vec<Profile>,
}

impl ProfileSelection {
    pub fn new(profiles: Vec<Profile>) -> Self {
        let mut list_state = ListState::default();
        if !profiles.is_empty() {
            list_state.select(Some(0));
        }

        Self {
            list_state,
            profiles,
        }
    }

    pub fn next(&mut self) {
        if !self.profiles.is_empty() {
            let current = self.list_state.selected().unwrap_or(0);
            let next = (current + 1).min(self.profiles.len() - 1);
            self.list_state.select(Some(next));
        }
    }

    pub fn previous(&mut self) {
        if !self.profiles.is_empty() {
            let current = self.list_state.selected().unwrap_or(0);
            let next = current.saturating_sub(1);
            self.list_state.select(Some(next));
        }
    }

    pub fn selected_profile(&self) -> Option<Profile> {
        self.list_state.selected().map(|i| self.profiles[i].clone())
    }
}
