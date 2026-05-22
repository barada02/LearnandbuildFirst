use crossterm::{
    cursor,
    event::{self, Event, KeyEvent},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, stdout, Stdout};
use std::time::Duration;

/// Defines a wrapper struct to safely initialize and restore the terminal state.
pub struct Tui {
    /// The Ratatui Terminal instance bound to Crossterm stdout
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Creates a new Tui instance, enabling raw mode and entering the alternate screen buffer.
    pub fn new() -> io::Result<Self> {
        // 1. Put terminal in raw mode (receives keyboard inputs immediately without pressing enter)
        terminal::enable_raw_mode()?;
        
        // 2. Hide cursor and enter Alternate Screen (keeps the user's shell terminal screen clean)
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        
        Ok(Tui { terminal })
    }

    /// Helper function to check if a terminal keyboard event has occurred.
    /// Polls for an event with a timeout to prevent blocking.
    pub fn poll_event(&self, timeout: Duration) -> io::Result<Option<KeyEvent>> {
        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                return Ok(Some(key_event));
            }
        }
        Ok(None)
    }
}

/// The Drop trait is one of Rust's most powerful features!
/// It acts as a destructor. When our `Tui` struct goes out of scope,
/// this code runs automatically—guaranteeing that the user's terminal is
/// restored to normal, even if the program panics or crashes!
impl Drop for Tui {
    fn drop(&mut self) {
        // 1. Disable raw mode
        let _ = terminal::disable_raw_mode();
        
        // 2. Leave Alternate Screen and restore the cursor
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            cursor::Show
        );
    }
}
