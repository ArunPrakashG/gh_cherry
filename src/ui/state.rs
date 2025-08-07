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
    pub error_message: Option<String>,
    pub loading_message: Option<String>,
    pub success_message: Option<String>,
}

#[derive(Debug)]
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

impl AppState {
    pub fn new() -> Self {
        Self {
            current_screen: Screen::MainMenu,
            prs: Vec::new(),
            pr_list_state: ListState::new(),
            error_message: None,
            loading_message: None,
            success_message: None,
        }
    }

    pub fn set_prs(&mut self, prs: Vec<PrInfo>) {
        self.prs = prs;
        self.pr_list_state.set_items_count(self.prs.len());
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

    pub fn clear_messages(&mut self) {
        self.error_message = None;
        self.loading_message = None;
        self.success_message = None;
    }
}
