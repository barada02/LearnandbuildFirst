use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use chrono::Local;

/// Represents a single task in our system.
/// We derive Serialize and Deserialize traits from serde to allow easy JSON translation.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: usize,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
}

/// TaskStore handles reading, writing, and modifying our tasks in a local JSON file.
pub struct TaskStore {
    file_path: PathBuf,
    pub tasks: Vec<Task>,
}

impl TaskStore {
    /// Initializes the store by loading tasks from 'tasks.json' in the workspace.
    pub fn new() -> Self {
        let file_path = PathBuf::from("tasks.json");
        let mut store = TaskStore {
            file_path,
            tasks: Vec::new(),
        };
        let _ = store.load(); // Load tasks if file exists, ignore error if it doesn't (starts empty)
        store
    }

    /// Loads tasks from the file system.
    fn load(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.file_path.exists() {
            return Ok(());
        }

        let mut file = File::open(&self.file_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        // Deserialize the JSON string into our Vec<Task>
        self.tasks = serde_json::from_str(&contents)?;
        Ok(())
    }

    /// Saves the current list of tasks to the file system in a human-readable formatted JSON.
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = serde_json::to_string_pretty(&self.tasks)?;
        let mut file = File::create(&self.file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    /// Adds a new task with a unique, auto-incrementing ID.
    pub fn add_task(&mut self, title: String) -> Result<usize, Box<dyn std::error::Error>> {
        // Calculate the next available ID
        let next_id = self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1;
        
        let new_task = Task {
            id: next_id,
            title: title.trim().to_string(),
            completed: false,
            created_at: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        };

        self.tasks.push(new_task);
        self.save()?;
        Ok(next_id)
    }

    /// Toggles the completion status of a task by ID. Returns true if found and changed.
    pub fn toggle_task(&mut self, id: usize) -> Result<bool, Box<dyn std::error::Error>> {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            task.completed = !task.completed;
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Deletes a task by ID. Returns true if found and removed.
    pub fn delete_task(&mut self, id: usize) -> Result<bool, Box<dyn std::error::Error>> {
        let initial_len = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        
        if self.tasks.len() < initial_len {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
