use crate::git;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

fn friendly_error(e: &anyhow::Error) -> String {
    e.to_string()
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Filter,
    Confirm,
    Create,
    Rename,
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    Delete {
        branch: String,
        is_merged: bool,
        force: bool,
    },
}

pub struct App {
    pub branches: Vec<git::Branch>,
    pub selected: usize,
    pub input_mode: InputMode,
    pub filter: String,
    pub input_buf: String,
    pub confirm_action: Option<ConfirmAction>,
    pub preview_commits: Vec<String>,
    pub status_message: Option<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let branches = git::list_branches()?;
        let mut app = App {
            branches,
            selected: 0,
            input_mode: InputMode::Normal,
            filter: String::new(),
            input_buf: String::new(),
            confirm_action: None,
            preview_commits: Vec::new(),
            status_message: None,
            should_quit: false,
        };
        app.select_current_branch();
        app.update_preview();
        Ok(app)
    }

    fn select_current_branch(&mut self) {
        let filtered = self.filtered_indices();
        for (i, &idx) in filtered.iter().enumerate() {
            if self.branches[idx].is_current {
                self.selected = i;
                return;
            }
        }
    }

    pub fn filtered_indices(&self) -> Vec<usize> {
        self.branches
            .iter()
            .enumerate()
            .filter(|(_, b)| {
                self.filter.is_empty()
                    || b.name.to_lowercase().contains(&self.filter.to_lowercase())
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub fn selected_branch(&self) -> Option<&git::Branch> {
        let filtered = self.filtered_indices();
        filtered.get(self.selected).map(|&i| &self.branches[i])
    }

    fn current_branch_name(&self) -> String {
        self.branches
            .iter()
            .find(|b| b.is_current)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "HEAD".to_string())
    }

    fn update_preview(&mut self) {
        self.preview_commits = if let Some(branch) = self.selected_branch() {
            git::get_log(&branch.name, 20).unwrap_or_default()
        } else {
            Vec::new()
        };
    }

    fn refresh_branches(&mut self) {
        if let Ok(branches) = git::list_branches() {
            self.branches = branches;
            let filtered = self.filtered_indices();
            if self.selected >= filtered.len() {
                self.selected = filtered.len().saturating_sub(1);
            }
            self.update_preview();
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if self.input_mode == InputMode::Normal {
            self.status_message = None;
        }
        match self.input_mode {
            InputMode::Normal => self.handle_normal(key),
            InputMode::Filter => self.handle_filter(key),
            InputMode::Confirm => self.handle_confirm(key),
            InputMode::Create => self.handle_create(key),
            InputMode::Rename => self.handle_rename(key),
        }
    }

    fn handle_normal(&mut self, key: KeyEvent) {
        let filtered_len = self.filtered_indices().len();
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if filtered_len > 0 {
                    self.selected = (self.selected + 1) % filtered_len;
                    self.update_preview();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if filtered_len > 0 {
                    self.selected = (self.selected + filtered_len - 1) % filtered_len;
                    self.update_preview();
                }
            }
            KeyCode::Enter => self.action_switch(),
            KeyCode::Char('y') => self.action_copy(),
            KeyCode::Char('d') => self.action_delete(),
            KeyCode::Char('n') => {
                let current = self.current_branch_name();
                self.input_mode = InputMode::Create;
                self.input_buf.clear();
                self.status_message = Some(format!("Create from '{}':", current));
            }
            KeyCode::Char('r') => {
                if let Some(branch) = self.selected_branch() {
                    self.input_buf = branch.name.clone();
                    self.input_mode = InputMode::Rename;
                    self.status_message = Some("Rename to:".to_string());
                }
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Filter;
                self.filter.clear();
                self.status_message = Some("Filter:".to_string());
            }
            _ => {}
        }
    }

    fn handle_filter(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.filter.clear();
                self.selected = 0;
                self.select_current_branch();
                self.status_message = None;
                self.update_preview();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.status_message = None;
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected = 0;
                self.update_preview();
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.selected = 0;
                self.update_preview();
            }
            _ => {}
        }
    }

    fn handle_confirm(&mut self, key: KeyEvent) {
        if let Some(action) = self.confirm_action.clone() {
            match action {
                ConfirmAction::Delete {
                    ref branch,
                    is_merged,
                    force,
                } => {
                    if is_merged || force {
                        // Merged: y/n
                        match key.code {
                            KeyCode::Char('y') => {
                                match git::delete_branch(branch, force) {
                                    Ok(_) => {
                                        self.status_message =
                                            Some(format!("Deleted branch '{}'", branch))
                                    }
                                    Err(e) => self.status_message = Some(friendly_error(&e)),
                                }
                                self.confirm_action = None;
                                self.input_mode = InputMode::Normal;
                                self.refresh_branches();
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                self.confirm_action = None;
                                self.input_mode = InputMode::Normal;
                                self.status_message = None;
                            }
                            _ => {}
                        }
                    } else {
                        // Unmerged: d to force, n to cancel
                        match key.code {
                            KeyCode::Char('d') => {
                                self.confirm_action = Some(ConfirmAction::Delete {
                                    branch: branch.clone(),
                                    is_merged: false,
                                    force: true,
                                });
                                self.status_message = Some(format!(
                                    "Force delete '{}' ? [y] confirm  [n] cancel",
                                    branch
                                ));
                            }
                            KeyCode::Char('n') | KeyCode::Esc => {
                                self.confirm_action = None;
                                self.input_mode = InputMode::Normal;
                                self.status_message = None;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    fn handle_create(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buf.clear();
                self.status_message = None;
            }
            KeyCode::Enter => {
                let name = self.input_buf.trim().to_string();
                if !name.is_empty() {
                    match git::create_branch(&name) {
                        Ok(_) => {
                            self.status_message = Some(format!("Created branch '{}'", name));
                        }
                        Err(e) => {
                            self.status_message = Some(friendly_error(&e));
                        }
                    }
                    self.refresh_branches();
                }
                self.input_mode = InputMode::Normal;
                self.input_buf.clear();
            }
            KeyCode::Backspace => {
                self.input_buf.pop();
            }
            KeyCode::Char(c) => {
                self.input_buf.push(c);
            }
            _ => {}
        }
    }

    fn handle_rename(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.input_buf.clear();
                self.status_message = None;
            }
            KeyCode::Enter => {
                let new_name = self.input_buf.trim().to_string();
                if let Some(branch) = self.selected_branch() {
                    let old_name = branch.name.clone();
                    if !new_name.is_empty() && new_name != old_name {
                        match git::rename_branch(&old_name, &new_name) {
                            Ok(_) => {
                                self.status_message =
                                    Some(format!("Renamed '{}' → '{}'", old_name, new_name));
                            }
                            Err(e) => {
                                self.status_message = Some(friendly_error(&e));
                            }
                        }
                        self.refresh_branches();
                    }
                }
                self.input_mode = InputMode::Normal;
                self.input_buf.clear();
            }
            KeyCode::Backspace => {
                self.input_buf.pop();
            }
            KeyCode::Char(c) => {
                self.input_buf.push(c);
            }
            _ => {}
        }
    }

    fn action_switch(&mut self) {
        if let Some(branch) = self.selected_branch() {
            if branch.is_current {
                return;
            }
            let name = branch.name.clone();
            match git::switch_branch(&name) {
                Ok(_) => {
                    self.should_quit = true;
                }
                Err(e) => {
                    self.status_message = Some(friendly_error(&e));
                }
            }
        }
    }

    fn action_copy(&mut self) {
        if let Some(branch) = self.selected_branch() {
            let name = branch.name.clone();
            match git::copy_to_clipboard(&name) {
                Ok(_) => self.status_message = Some(format!("Copied '{}'", name)),
                Err(e) => self.status_message = Some(friendly_error(&e)),
            }
        }
    }

    fn action_delete(&mut self) {
        if let Some(branch) = self.selected_branch() {
            if branch.is_current {
                self.status_message = Some("Cannot delete current branch".to_string());
                return;
            }
            let name = branch.name.clone();
            let is_merged = branch.is_merged;
            if is_merged {
                self.status_message = Some(format!("Delete branch '{}'? [y/n]", name));
            } else {
                self.status_message = Some(format!(
                    "⚠ '{}' is not fully merged. [d] force delete  [n] cancel",
                    name
                ));
            }
            self.confirm_action = Some(ConfirmAction::Delete {
                branch: name,
                is_merged,
                force: false,
            });
            self.input_mode = InputMode::Confirm;
        }
    }
}
