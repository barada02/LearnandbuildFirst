use crate::app::{ActiveTab, App};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Frame,
};

/// The main entry point for our TUI rendering.
/// Called on every tick/event to draw the current application state.
pub fn render(f: &mut Frame, app: &mut App) {
    // 1. Divide the terminal screen vertically into Header, Body, and Footer blocks.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Main workspace body
            Constraint::Length(3), // Footer key bindings
        ])
        .split(f.size());

    // 2. Render Header (Logo and Active Tabs)
    render_header(f, chunks[0], app);

    // 3. Render Body depending on active tab
    match app.active_tab {
        ActiveTab::Dashboard => render_dashboard(f, chunks[1], app),
        ActiveTab::Tasks => render_tasks(f, chunks[1], app),
        ActiveTab::SystemInfo => render_system_info(f, chunks[1], app),
    }

    // 4. Render Footer (Shortcut menu bar)
    render_footer(f, chunks[2], app);

    // 5. If show_add_popup is true, overlay a gorgeous centered popup modal!
    if app.show_add_popup {
        render_add_popup(f, app);
    }
}

/// Renders the logo brand and navigation tabs.
fn render_header(f: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(16), // Brand Logo
            Constraint::Min(20),    // Navigation Tabs
        ])
        .split(area);

    // Brand Paragraph
    let brand = Paragraph::new("⚡ FLOW CLI")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(brand, header_chunks[0]);

    // Active Navigation Tabs
    let tab_titles = vec!["[1] Dashboard", "[2] Task List", "[3] System Info"];
    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)))
        .select(app.active_tab.index())
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        );
    f.render_widget(tabs, header_chunks[1]);
}

/// Renders the Dashboard View (Splash page, motivational text, completion metrics)
fn render_dashboard(f: &mut Frame, area: Rect, app: &App) {
    let dashboard_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(55), // Left: Greeting & session metrics
            Constraint::Percentage(45), // Right: General stats & goal progress
        ])
        .split(area);

    // Left Panel: Dynamic Greeting Card
    let total_tasks = app.store.tasks.len();
    let completed_tasks = app.store.tasks.iter().filter(|t| t.completed).count();
    let pending_tasks = total_tasks - completed_tasks;

    let greeting_text = vec![
        Line::from(vec![
            Span::styled("Welcome back, ", Style::default().fg(Color::White)),
            Span::styled("Developer", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("!", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("“The secret of getting ahead is getting started.”", Style::default().fg(Color::LightYellow).add_modifier(Modifier::ITALIC)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("📊 SESSION HIGHLIGHTS:", Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED)),
        ]),
        Line::from(format!("  • Tasks toggled this session: {}", app.session_completions)),
        Line::from(format!("  • Total active tasks on disk: {}", total_tasks)),
        Line::from(format!("  • Pending tasks in buffer  : {}", pending_tasks)),
    ];

    let greeting_card = Paragraph::new(greeting_text)
        .block(
            Block::default()
                .title(" 👋 Dashboard Overview ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(greeting_card, dashboard_chunks[0]);

    // Right Panel: Progress Tracker Box
    let completion_rate = if total_tasks > 0 {
        (completed_tasks as f64 / total_tasks as f64) * 100.0
    } else {
        0.0
    };

    // Construct custom progress bar: e.g. [██████░░░░] 60%
    let progress_blocks = (completion_rate / 10.0).round() as usize;
    let filled_bar = "█".repeat(progress_blocks);
    let empty_bar = "░".repeat(10 - progress_blocks);

    let progress_color = if completion_rate > 80.0 {
        Color::Green
    } else if completion_rate > 40.0 {
        Color::Yellow
    } else {
        Color::LightRed
    };

    let stats_text = vec![
        Line::from("🏆 GOAL COMPLETION PROGRESS:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  [{}{}] ", filled_bar, empty_bar), Style::default().fg(progress_color)),
            Span::styled(format!("{:.1}% Done", completion_rate), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from("💡 Tips for CLI Productivity:"),
        Line::from("  • Press 'a' at any time on Tab [2] to add a task."),
        Line::from("  • Press 'Space' to check/uncheck a task."),
        Line::from("  • Edit 'tasks.json' directly for bulk edits!"),
    ];

    let stats_card = Paragraph::new(stats_text)
        .block(
            Block::default()
                .title(" 🎯 Target Tracker ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(stats_card, dashboard_chunks[1]);
}

/// Renders the Interactive Task Manager View
fn render_tasks(f: &mut Frame, area: Rect, app: &App) {
    let task_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(65), // Left: Task List
            Constraint::Percentage(35), // Right: Selected Task Details card
        ])
        .split(area);

    // Left Panel: Interactive Task List
    let tasks = &app.store.tasks;
    
    let list_items: Vec<ListItem> = if tasks.is_empty() {
        vec![ListItem::new("📝 Empty! Press 'a' to add your first task!").style(Style::default().fg(Color::DarkGray))]
    } else {
        tasks
            .iter()
            .enumerate()
            .map(|(idx, task)| {
                // Formatting depending on state
                let status_symbol = if task.completed { " [✅ Done]    " } else { " [⏳ Pending] " };
                let item_style = if task.completed {
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
                } else {
                    Style::default().fg(Color::White)
                };

                // Add active highlighting if this item is selected/hovered
                let prefix = if idx == app.selected_task_index {
                    " 👉 "
                } else {
                    "    "
                };

                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                    Span::styled(status_symbol, Style::default().fg(if task.completed { Color::Green } else { Color::Yellow })),
                    Span::styled(&task.title, item_style),
                ]);

                ListItem::new(line)
            })
            .collect()
    };

    let list_title = format!(" 📝 Tasks Database ({}) ", tasks.len());
    let list_block = List::new(list_items)
        .block(
            Block::default()
                .title(list_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(30, 40, 50)) // Deep slate highlight background
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(list_block, task_chunks[0]);

    // Right Panel: Task Details Card
    let details_content = if tasks.is_empty() || app.selected_task_index >= tasks.len() {
        vec![Line::from("No task selected.")]
    } else {
        let task = &tasks[app.selected_task_index];
        vec![
            Line::from(vec![
                Span::styled("🆔 Task ID    : ", Style::default().fg(Color::Cyan)),
                Span::styled(task.id.to_string(), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📌 Title      : ", Style::default().fg(Color::Cyan)),
                Span::styled(&task.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("⚡ Status     : ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    if task.completed { "Completed (Done)" } else { "Pending (In Progress)" },
                    Style::default().fg(if task.completed { Color::Green } else { Color::Yellow }),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("📅 Created At : ", Style::default().fg(Color::Cyan)),
                Span::styled(&task.created_at, Style::default().fg(Color::Gray)),
            ]),
            Line::from(""),
            Line::from("🔧 Controls:"),
            Line::from("  • [Space] Toggle State"),
            Line::from("  • [Delete] Remove Task"),
            Line::from("  • [a] Add New Task"),
        ]
    };

    let details_card = Paragraph::new(details_content)
        .block(
            Block::default()
                .title(" ℹ️ Task Details Card ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(details_card, task_chunks[1]);
}

/// Renders System Information View
fn render_system_info(f: &mut Frame, area: Rect, app: &App) {
    let elapsed = chrono::Local::now().signed_duration_since(app.start_time);
    let seconds = elapsed.num_seconds() % 60;
    let minutes = (elapsed.num_seconds() / 60) % 60;
    let hours = elapsed.num_seconds() / 3600;

    let info_text = vec![
        Line::from(vec![
            Span::styled("🚀 APPLICATION METRICS & ENVIRONMENT:", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  • App Uptime        : ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:02}:{:02}:{:02}", hours, minutes, seconds), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  • Database Engine   : ", Style::default().fg(Color::Gray)),
            Span::styled("Local File System Serialization (JSON)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  • Database Location : ", Style::default().fg(Color::Gray)),
            Span::styled("./tasks.json", Style::default().fg(Color::LightBlue)),
        ]),
        Line::from(vec![
            Span::styled("  • Framework         : ", Style::default().fg(Color::Gray)),
            Span::styled("Rust with Ratatui & Crossterm", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  • Edition           : ", Style::default().fg(Color::Gray)),
            Span::styled("Rust 2021 Standard Binary Edition", Style::default().fg(Color::White)),
        ]),
        Line::from(""),
        Line::from("💡 System status is operating normally. Tasks updates will autosave on disk automatically."),
    ];

    let info_card = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(" 🖥️ Environment & System Info ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(info_card, area);
}

/// Renders Keybindings footer.
fn render_footer(f: &mut Frame, area: Rect, _app: &App) {
    let menu_text = vec![
        Span::styled("Tab/1-3: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Switch Tabs  ", Style::default().fg(Color::White)),
        Span::styled("↑↓: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Navigate List  ", Style::default().fg(Color::White)),
        Span::styled("Space: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle Task  ", Style::default().fg(Color::White)),
        Span::styled("a: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Add Task  ", Style::default().fg(Color::White)),
        Span::styled("Del/d: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Delete Task  ", Style::default().fg(Color::White)),
        Span::styled("Esc/q: ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
        Span::styled("Exit App", Style::default().fg(Color::White)),
    ];

    let menu = Paragraph::new(Line::from(menu_text))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(menu, area);
}

/// Renders a beautiful centered pop-up textbox for task creation.
fn render_add_popup(f: &mut Frame, app: &App) {
    // 1. Define bounds centered in the middle of terminal
    let popup_area = centered_rect(60, 20, f.size());
    
    // 2. Clear out the underlying content (important!)
    f.render_widget(Clear, popup_area);

    // 3. Render layout box details
    let text_lines = vec![
        Line::from("Type the title of your new task below:"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" > ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&app.input_text, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("█", Style::default().fg(Color::Cyan)), // Simulated typing cursor!
        ]),
        Line::from(""),
        Line::from(Span::styled("Press [Enter] to Create, [Esc] to Cancel", Style::default().fg(Color::DarkGray))),
    ];

    let popup_block = Paragraph::new(text_lines)
        .block(
            Block::default()
                .title(" ➕ Add New Task ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)),
        )
        .wrap(Wrap { trim: true });

    f.render_widget(popup_block, popup_area);
}

/// Helper function to calculate standard coordinates for a centered rectangle.
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
