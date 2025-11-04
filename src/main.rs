//! Radio Shack Executive Decision Maker – Rust TUI
//! ------------------------------------------------
//! - Press Enter or Space (or click the "ASK" prompt) to get a random answer.
//! - The chosen answer lights up for 1.5 s.
//! - Quit with `q`, `Esc`, or Ctrl+C.

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
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

const ANIMATION_DURATION_MS: u64 = 2_000;
const ANIMATION_STEP_MS: u64 = 120;
const ANSWER_FLASH_MS: u64 = 1_500;
const TICK_RATE_MS: u64 = 50;

#[derive(Clone, Copy, PartialEq)]
enum State {
    Idle,
    Animating {
        final_index: usize,
        current_index: usize,
        end_at: Instant,
        next_switch: Instant,
    },
    Showing {
        index: usize,
        until: Instant,
    },
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
        let mut rng = rand::thread_rng();
        let final_idx = rng.gen_range(0..ANSWERS.len());
        let mut current_idx = final_idx;
        if ANSWERS.len() > 1 {
            while current_idx == final_idx {
                current_idx = rng.gen_range(0..ANSWERS.len());
            }
        }

        let now = Instant::now();
        self.last_answer = None;
        self.state = State::Animating {
            final_index: final_idx,
            current_index: current_idx,
            end_at: now + Duration::from_millis(ANIMATION_DURATION_MS),
            next_switch: now,
        };
    }

    fn tick(&mut self) {
        let now = Instant::now();
        match self.state {
            State::Idle => {}
            State::Animating {
                final_index,
                current_index,
                end_at,
                next_switch,
            } => {
                if now >= end_at {
                    self.last_answer = Some(final_index);
                    self.state = State::Showing {
                        index: final_index,
                        until: now + Duration::from_millis(ANSWER_FLASH_MS),
                    };
                } else if now >= next_switch {
                    let mut rng = rand::thread_rng();
                    let mut next_index = rng.gen_range(0..ANSWERS.len());
                    if ANSWERS.len() > 1 {
                        while next_index == current_index {
                            next_index = rng.gen_range(0..ANSWERS.len());
                        }
                    }
                    self.state = State::Animating {
                        final_index,
                        current_index: next_index,
                        end_at,
                        next_switch: now + Duration::from_millis(ANIMATION_STEP_MS),
                    };
                }
            }
            State::Showing { until, .. } => {
                if now >= until {
                    self.state = State::Idle;
                }
            }
        }
    }

    fn toggle_help(&mut self) {
        self.help_visible = !self.help_visible;
    }

    /// Returns true if the app should terminate.
    fn on_key(&mut self, key: KeyEvent) -> bool {
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
        .constraints([
            Constraint::Length(5),
            Constraint::Min(7),
            Constraint::Length(3),
        ])
        .margin(2)
        .split(f.area());

    render_header(f, chunks[0], app);
    render_buttons(f, chunks[1], app);
    render_footer(f, chunks[2], app);
    if app.help_visible {
        render_help_overlay(f);
    }
}

/// Draw the six answer “buttons”
fn render_buttons(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
        ])
        .split(area);

    let row_chunks = |rect| {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(rect)
    };

    let active_index = match app.state {
        State::Animating { current_index, .. } => Some(current_index),
        State::Showing { index, .. } => Some(index),
        State::Idle => None,
    };

    // Row 1
    let top_row = row_chunks(rows[0]);
    draw_button(f, top_row[0], ANSWERS[0], active_index == Some(0));
    draw_button(f, top_row[1], ANSWERS[1], active_index == Some(1));
    draw_button(f, top_row[2], ANSWERS[2], active_index == Some(2));

    // Row 2
    let bottom_row = row_chunks(rows[1]);
    draw_button(f, bottom_row[0], ANSWERS[3], active_index == Some(3));
    draw_button(f, bottom_row[1], ANSWERS[4], active_index == Some(4));
    draw_button(f, bottom_row[2], ANSWERS[5], active_index == Some(5));
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

    let widget = Paragraph::new(Span::styled(text, style))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(widget, area);
}

fn render_header(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    let title_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);
    let mut lines = vec![
        Line::from(Span::styled("EXECUTIVE DECISION MAKER", title_style)),
        Line::raw(""),
    ];
    lines.push(Line::raw(
        "Think of your question, then press Enter or Space to consult the oracle.",
    ));
    match app.state {
        State::Animating { .. } => {
            lines.push(Line::raw("Lights are shuffling... hold tight!"));
        }
        State::Showing { .. } => {
            lines.push(Line::raw("Final answer locked in. Ask again any time."));
        }
        State::Idle => {
            if app.last_answer.is_none() {
                lines.push(Line::raw("Need instructions? Press Ctrl+H for help."));
            } else {
                lines.push(Line::raw(
                    "Ready for another? Press Enter or Space to ask again.",
                ));
            }
        }
    }

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Radio Shack "),
    );
    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut ratatui::Frame, area: ratatui::layout::Rect, app: &App) {
    let (status_line, help_line) = match app.state {
        State::Animating { .. } => (
            "Consulting the oracle...".to_string(),
            "Lights flash in random order before the final answer appears.",
        ),
        State::Showing { index, .. } => (
            format!("Answer: {}", ANSWERS[index]),
            "Highlight stays on briefly so you can see the result.",
        ),
        State::Idle => match app.last_answer {
            Some(idx) => (
                format!("Final Answer: {}", ANSWERS[idx]),
                "Press Enter/Space to ask again · Ctrl+H for help · q/Esc to quit",
            ),
            None => (
                "Ready when you are.".to_string(),
                "Press Enter/Space to ask · Ctrl+H for help · q/Esc to quit",
            ),
        },
    };

    let content = vec![Line::from(status_line), Line::raw(""), Line::raw(help_line)];
    let paragraph = Paragraph::new(content)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title(" Status "));

    f.render_widget(paragraph, area);
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
