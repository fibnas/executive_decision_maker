//! Radio Shack Executive Decision Maker – Rust TUI
//! ------------------------------------------------
//! - Press Enter or Space (or click the "ASK" prompt) to get a random answer.
//! - The chosen answer lights up for 1.5 s.
//! - Quit with `q`, `Esc`, or Ctrl+C.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};
use std::{
    io,
    time::{Duration, Instant},
};

/// The six possible answers (exactly as on the original device)
const ANSWERS: [&str; 6] = [
    "DEFINITELY",
    "FORGET IT",
    "ASK AGAIN",
    "NEVER",
    "POSSIBLY",
    "WHY NOT",
];

const ANSWER_FLASH_MS: u64 = 1_500;
const TICK_RATE_MS: u64 = 50;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Showing { index: usize, until: Instant },
}

struct App {
    state: State,
    help_visible: bool,
    last_answer: Option<usize>,
}

impl App {
    fn new() -> Self {
        Self {
            state: State::Idle,
            help_visible: false,
            last_answer: None,
        }
    }

    fn ask(&mut self) {
        let idx = rand::thread_rng().gen_range(0..ANSWERS.len());
        self.last_answer = Some(idx);
        self.state = State::Showing {
            index: idx,
            until: Instant::now() + Duration::from_millis(ANSWER_FLASH_MS),
        };
    }

    fn tick(&mut self) {
        if let State::Showing { until, .. } = self.state {
            if Instant::now() >= until {
                self.state = State::Idle;
            }
        }
    }

    fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    /// Returns true if the app should terminate.
    fn on_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }

        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if ctrl {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('C') => return true,
                KeyCode::Char('h') | KeyCode::Char('H') => {
                    self.toggle_help();
                    return false;
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => {
                if self.help_visible {
                    self.help_visible = false;
                    false
                } else {
                    true
                }
            }
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                if self.help_visible {
                    self.help_visible = false;
                    false
                } else {
                    true
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if self.help_visible {
                    self.help_visible = false;
                } else {
                    self.ask();
                }
                false
            }
            _ => false,
        }
    }
}

type TerminalBackend = CrosstermBackend<io::Stdout>;
type AppTerminal = Terminal<TerminalBackend>;

fn setup_terminal() -> io::Result<AppTerminal> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    match Terminal::new(backend) {
        Ok(mut terminal) => {
            terminal.hide_cursor()?;
            terminal.clear()?;
            Ok(terminal)
        }
        Err(err) => {
            let _ = disable_raw_mode();
            let _ = io::stdout().execute(LeaveAlternateScreen);
            Err(err)
        }
    }
}

fn cleanup_terminal(terminal: &mut AppTerminal) -> io::Result<()> {
    terminal.show_cursor()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    disable_raw_mode()
}

fn run_app(terminal: &mut AppTerminal) -> io::Result<()> {
    let mut app = App::new();

    loop {
        app.tick();
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(Duration::from_millis(TICK_RATE_MS))? {
            if let Event::Key(key) = event::read()? {
                if app.on_key(key) {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal);
    cleanup_terminal(&mut terminal)?;
    result
}

/// Render the whole UI
fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .margin(2)
        .split(f.area());

    // Title block
    let title = Paragraph::new("EXECUTIVE DECISION MAKER")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Radio Shack "),
        );
    f.render_widget(title, chunks[0]);

    // Buttons area
    let btn_area = chunks[1];
    render_buttons(f, btn_area, app);

    if app.help_visible {
        render_help_overlay(f);
    }
}

/// Draw the six answer “buttons”
fn render_buttons(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    let btn_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(area);

    let left = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(btn_chunks[0]);

    let mid = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(btn_chunks[1]);

    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(btn_chunks[2]);

    // Helper to decide if a button should be highlighted
    let is_active = |i: usize| matches!(app.state, State::Showing { index, .. } if index == i);

    // Row 1
    draw_button(f, left[0], ANSWERS[0], is_active(0));
    draw_button(f, mid[0], ANSWERS[1], is_active(1));
    draw_button(f, right[0], ANSWERS[2], is_active(2));

    // Row 2
    draw_button(f, left[1], ANSWERS[3], is_active(3));
    draw_button(f, mid[1], ANSWERS[4], is_active(4));
    draw_button(f, right[1], ANSWERS[5], is_active(5));

    // ASK button (centered below the grid)
    let ask_area = area.inner(Margin {
        vertical: 2,
        horizontal: 0,
    });
    let ask_text = match app.state {
        State::Idle => app
            .last_answer
            .map(|idx| {
                format!(
                    "Last answer: {}  ·  Enter/Space: Ask · Ctrl+H: Help · q/Esc: Quit",
                    ANSWERS[idx]
                )
            })
            .unwrap_or_else(|| "Enter/Space: Ask · Ctrl+H: Help · q/Esc: Quit".to_string()),
        State::Showing { .. } => "Consulting the oracle...".to_string(),
    };
    let ask = Paragraph::new(ask_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(ask, ask_area);
}

/// Render a single answer button
fn draw_button(f: &mut ratatui::Frame, area: ratatui::layout::Rect, text: &str, active: bool) {
    let style = if active {
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightGreen)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    };

    let widget = Paragraph::new(Span::styled(format!(" {text} "), style))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(widget, area);
}

fn render_help_overlay(f: &mut ratatui::Frame) {
    let area = centered_rect(60, 50, f.area());

    let help = [
        "EXECUTIVE DECISION MAKER",
        "",
        "How to play:",
        "  - Press Enter or Space to light up a random answer.",
        "  - The highlighted answer stays on for about 1.5 s.",
        "",
        "Controls:",
        "  Enter / Space    Ask (or close this help)",
        "  Ctrl+H           Toggle help",
        "  q / Esc          Quit (Esc closes help first)",
        "  Ctrl+C           Quit immediately",
    ]
    .join("\n");

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));

    let paragraph = Paragraph::new(help)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left)
        .block(block);

    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    area: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
