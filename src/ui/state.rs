use crate::github::PrInfo;

#[derive(Debug, Clone)]
pub enum Screen {
    MainMenu,
    PrList,
    Progress,
    Error,
}

#[derive(Debug)]
pub struct AppState {
    pub current_screen: Screen,
    pub prs: Vec<PrInfo>,
    pub pr_list_state: ListState,
    // Inline prompt/input mode (minimal, no boxes)
    pub input_active: bool,
    pub input_title: String,
    pub input_placeholder: String,
    pub input_buffer: String,
    pub filter_query: Option<String>,
    pub display_indices: Vec<usize>,
    pub error_message: Option<String>,
    pub loading_message: Option<String>,
    pub success_message: Option<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ListState {
    selected: Option<usize>,
    items_count: usize,
}

impl ListState {
    pub fn new() -> Self {
        Self {
            selected: None,
            items_count: 0,
        }
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.selected = index;
    }

    pub fn selected(&self) -> Option<usize> {
        self.selected
    }

    pub fn select_next(&mut self) {
        if self.items_count == 0 {
            return;
        }

        let i = match self.selected {
            Some(i) => {
                if i >= self.items_count - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    pub fn select_previous(&mut self) {
        if self.items_count == 0 {
            return;
        }

        let i = match self.selected {
            Some(i) => {
                if i == 0 {
                    self.items_count - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.select(Some(i));
    }

    pub fn set_items_count(&mut self, count: usize) {
        self.items_count = count;
        if count == 0 {
            self.selected = None;
        } else if self.selected.is_none() {
            self.selected = Some(0);
        } else if let Some(selected) = self.selected {
            if selected >= count {
                self.selected = Some(count - 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ListState;

    #[test]
    fn selection_wraps_and_initializes() {
        let mut ls = ListState::new();
        ls.set_items_count(3);
        assert_eq!(ls.selected(), Some(0));

        ls.select_next();
        assert_eq!(ls.selected(), Some(1));

        ls.select_next();
        ls.select_next(); // wrap to 0
        assert_eq!(ls.selected(), Some(0));

        ls.select_previous(); // wrap to last
        assert_eq!(ls.selected(), Some(2));
    }

    #[test]
    fn selection_resets_when_items_change() {
        let mut ls = ListState::new();
        ls.set_items_count(5);
        ls.select(Some(4));
        assert_eq!(ls.selected(), Some(4));
        ls.set_items_count(3);
        assert_eq!(ls.selected(), Some(2));
        ls.set_items_count(0);
        assert_eq!(ls.selected(), None);
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::MainMenu,
            prs: Vec::new(),
            pr_list_state: ListState::new(),
            input_active: false,
            input_title: String::new(),
            input_placeholder: String::new(),
            input_buffer: String::new(),
            filter_query: None,
            display_indices: Vec::new(),
            error_message: None,
            loading_message: None,
            success_message: None,
        }
    }

    pub fn set_prs(&mut self, prs: Vec<PrInfo>) {
        self.prs = prs;
        self.recompute_display_indices();
        self.loading_message = None;
        self.error_message = None;
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
        self.loading_message = None;
        self.success_message = None;
    }

    pub fn set_loading(&mut self, message: &str) {
        self.loading_message = Some(message.to_string());
        self.error_message = None;
        self.success_message = None;
    }

    pub fn set_success(&mut self, message: &str) {
        self.success_message = Some(message.to_string());
        self.loading_message = None;
        self.error_message = None;
    }

    #[allow(dead_code)] // Useful utility method for future use
    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.loading_message = None;
        self.success_message = None;
    }

    // Prompt helpers
    pub fn start_prompt(&mut self, title: &str, placeholder: &str, initial: &str) {
        self.input_active = true;
        self.input_title = title.to_string();
        self.input_placeholder = placeholder.to_string();
        self.input_buffer = initial.to_string();
    }

    pub fn cancel_prompt(&mut self) {
        self.input_active = false;
        self.input_title.clear();
        self.input_placeholder.clear();
        self.input_buffer.clear();
    }

    pub fn confirm_prompt(&mut self) -> String {
        let res = self.input_buffer.trim().to_string();
        self.cancel_prompt();
        res
    }

    pub fn set_filter_query(&mut self, q: Option<String>) {
        self.filter_query = q.filter(|s| !s.trim().is_empty());
        self.recompute_display_indices();
    }

    pub fn recompute_display_indices(&mut self) {
        self.display_indices.clear();
        if let Some(q) = &self.filter_query {
            let ql = q.to_lowercase();
            for (i, pr) in self.prs.iter().enumerate() {
                let n = pr.number.to_string();
                if pr.title.to_lowercase().contains(&ql)
                    || pr.author.to_lowercase().contains(&ql)
                    || n.contains(&ql)
                {
                    self.display_indices.push(i);
                }
            }
        } else {
            self.display_indices.extend(0..self.prs.len());
        }
        self.pr_list_state
            .set_items_count(self.display_indices.len());
    }
}
