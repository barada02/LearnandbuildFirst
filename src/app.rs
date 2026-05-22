use crate::task::TaskStore;

/// Enumerates the primary sections (tabs) in our dashboard app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard = 0,
    Tasks = 1,
    SystemInfo = 2,
}

impl ActiveTab {
    /// Helper to convert tab enum into a numeric index for Ratatui's Tab widget.
    pub fn index(self) -> usize {
        self as usize
    }

    /// Converts numeric index back to enum.
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => ActiveTab::Dashboard,
            1 => ActiveTab::Tasks,
            _ => ActiveTab::SystemInfo,
        }
    }
}

/// The main application state container.
pub struct App {
    /// The database connection
    pub store: TaskStore,
    /// Currently highlighted tab
    pub active_tab: ActiveTab,
    /// Has the user requested to exit the program?
    pub should_quit: bool,
    
    // Task-list visual states
    /// Index of the currently highlighted task in the Task List view
    pub selected_task_index: usize,
    
    // Popup visual textbox states (for adding tasks in TUI)
    /// Whether we are currently showing the "Add Task" popup modal
    pub show_add_popup: bool,
    /// The text the user has typed into the textbox
    pub input_text: String,
    
    // Productivity metrics
    /// Count of tasks completed during this specific run
    pub session_completions: u32,
    /// Uptime start timestamp
    pub start_time: chrono::DateTime<chrono::Local>,
}

impl App {
    /// Creates a new App instance, loading tasks from our database.
    pub fn new() -> Self {
        App {
            store: TaskStore::new(),
            active_tab: ActiveTab::Dashboard,
            should_quit: false,
            selected_task_index: 0,
            show_add_popup: false,
            input_text: String::new(),
            session_completions: 0,
            start_time: chrono::Local::now(),
        }
    }

    /// Toggles should_quit to exit the main loop.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Switches tab selection to the right (wrapping around).
    pub fn next_tab(&mut self) {
        let current = self.active_tab.index();
        let next = (current + 1) % 3;
        self.active_tab = ActiveTab::from_index(next);
        self.selected_task_index = 0; // Reset task list selection focus on switch
    }

    /// Switches tab selection to the left (wrapping around).
    pub fn prev_tab(&mut self) {
        let current = self.active_tab.index();
        let prev = if current == 0 { 2 } else { current - 1 };
        self.active_tab = ActiveTab::from_index(prev);
        self.selected_task_index = 0;
    }

    /// Increments list highlighted index.
    pub fn list_next(&mut self) {
        if self.store.tasks.is_empty() {
            return;
        }
        self.selected_task_index = (self.selected_task_index + 1) % self.store.tasks.len();
    }

    /// Decrements list highlighted index.
    pub fn list_prev(&mut self) {
        if self.store.tasks.is_empty() {
            return;
        }
        self.selected_task_index = if self.selected_task_index == 0 {
            self.store.tasks.len() - 1
        } else {
            self.selected_task_index - 1
        };
    }

    /// Toggles the currently selected task's status.
    pub fn toggle_selected_task(&mut self) {
        if self.store.tasks.is_empty() {
            return;
        }
        let task_id = self.store.tasks[self.selected_task_index].id;
        if let Ok(true) = self.store.toggle_task(task_id) {
            // If we completed it, increment session completion metric
            let is_completed = self.store.tasks.iter().find(|t| t.id == task_id).unwrap().completed;
            if is_completed {
                self.session_completions += 1;
            }
        }
    }

    /// Deletes the currently selected task.
    pub fn delete_selected_task(&mut self) {
        if self.store.tasks.is_empty() {
            return;
        }
        let task_id = self.store.tasks[self.selected_task_index].id;
        if let Ok(true) = self.store.delete_task(task_id) {
            // Adjust selection pointer if we deleted the last item
            if self.selected_task_index >= self.store.tasks.len() && !self.store.tasks.is_empty() {
                self.selected_task_index = self.store.tasks.len() - 1;
            } else if self.store.tasks.is_empty() {
                self.selected_task_index = 0;
            }
        }
    }

    /// Finalizes adding a task from the text popup box.
    pub fn add_task_from_input(&mut self) {
        if self.input_text.trim().is_empty() {
            self.show_add_popup = false;
            return;
        }
        let title = self.input_text.clone();
        if let Ok(_) = self.store.add_task(title) {
            self.input_text.clear();
            self.show_add_popup = false;
            // Focus selection on the newly added task (which is last)
            self.selected_task_index = self.store.tasks.len() - 1;
        }
    }
}
