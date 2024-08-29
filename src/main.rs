use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

struct Atto {
    cursor_x: usize,
    cursor_y: usize,
    buffer: Vec<String>,
    terminal_height: usize,
    terminal_width: usize,
}

impl Atto {
    fn new() -> Self {
        let (width, height) = crossterm::terminal::size().unwrap();
        Self {
            cursor_y: 0,
            cursor_x: 0,
            buffer: vec![String::new()],
            terminal_height: height as usize,
            terminal_width: width as usize,
        }
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('e') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                    KeyCode::Char('j') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.move_up(),
                    KeyCode::Char('k') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.move_down(),
                    KeyCode::Char('h') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.move_left(),
                    KeyCode::Char('l') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.move_right(),
                    KeyCode::Enter => self.new_line(),
                    KeyCode::Char(v) => self.input_char(v),
                    KeyCode::Backspace => self.backspace(),
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    fn render<B: Backend>(&self, f: &mut tui::Frame<B>) {
        let size = f.size();
        let block = Block::default().borders(Borders::ALL).title("Atto");

        let mut lines = self.buffer.clone();
        if self.cursor_y < lines.len() {
            if self.cursor_x < lines[self.cursor_y].len() {
                lines[self.cursor_y].insert(self.cursor_x, '_');
            } else {
                lines[self.cursor_y].push('_');
            }
        }

        let paragraph = Paragraph::new(lines.iter().map(|line| Spans::from(Span::raw(line))).collect::<Vec<_>>())
            .block(block);
        f.render_widget(paragraph, size);
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y < self.buffer.len() - 1 {
            self.cursor_y += 1;
            self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_x < self.buffer[self.cursor_y].len() {
            self.cursor_x += 1;
        }
    }

    fn input_char(&mut self, c: char) {
        if self.cursor_x < self.terminal_width - 1 {
            self.buffer[self.cursor_y].insert(self.cursor_x, c);
            self.cursor_x += 1;
        }
    }

    fn new_line(&mut self) {
        if self.cursor_y < self.terminal_height - 1 {
            let new_line = self.buffer[self.cursor_y].split_off(self.cursor_x);
            self.buffer.insert(self.cursor_y + 1, new_line);
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn backspace(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            self.buffer[self.cursor_y].remove(self.cursor_x);
        } else if self.cursor_y > 0 {
            let current_line = self.buffer.remove(self.cursor_y);
            self.cursor_y -= 1;
            self.cursor_x = self.buffer[self.cursor_y].len();
            self.buffer[self.cursor_y].push_str(&current_line);
        }
    }
}

fn main() -> io::Result<()> {
    let mut atto = Atto::new();
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    atto.run(&mut terminal)
}


