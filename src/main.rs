use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    cursor::{MoveTo, Show, Hide},
};
use std::io::{self, Read, Write};
use std::env;
use std::fs::{File, OpenOptions};
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
    cursor_offset_x: u16,
    cursor_offset_y: u16,
    buffer: Vec<String>,
    terminal_height: usize,
    terminal_width: usize,
    filename: Option<String>,
    show_binds: bool,
    scroll_offset: usize,
}

impl Atto {
    fn new(filename: Option<String>) -> Self {
        let (width, height) = crossterm::terminal::size().unwrap();
        Self {
            cursor_y: 0,
            cursor_x: 0,
            cursor_offset_x: 6,
            cursor_offset_y: 1,
            buffer: vec![String::new()],
            terminal_height: height as usize,
            terminal_width: width as usize,
            filename,
            show_binds: false,
            scroll_offset: 0,
        }
    }

    fn read_file(&mut self) -> io::Result<()> {
        if let Some(ref filename) = self.filename {
            let mut file = File::open(filename)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            self.buffer = if contents.is_empty() {
                vec![String::new()]
            } else {
                contents.lines().map(|line| line.to_string()).collect()
            };
            self.cursor_x = 0;
            self.cursor_y = 0;
        }
        Ok(())
    }

    fn write_file(&self) -> io::Result<()> {
        if let Some(ref filename) = self.filename {
            let mut file = OpenOptions::new().write(true).truncate(true).open(filename)?;
            for line in &self.buffer {
                writeln!(file, "{}", line)?;
            }
        }
        Ok(())
    }

    fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture, Show)?;

        loop {
            terminal.draw(|f| self.render(f))?;

            // Move the cursor to the correct position
            execute!(io::stdout(), MoveTo(self.cursor_x as u16 + self.cursor_offset_x, self.cursor_y as u16 - self.scroll_offset as u16 + self.cursor_offset_y), Show)?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                    KeyCode::Up => self.move_up(),
                    KeyCode::Down => self.move_down(),
                    KeyCode::Left => self.move_left(),
                    KeyCode::Right => self.move_right(),
                    KeyCode::Char('w') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.write_file()?,
                    KeyCode::Char('r') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => self.read_file()?,
                    KeyCode::Esc => self.show_binds = !self.show_binds,
                    KeyCode::Enter => self.new_line(),
                    KeyCode::Char(v) => self.input_char(v),
                    KeyCode::Backspace => self.backspace(),
                    KeyCode::Tab => self.input_tab(),
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture, Show)?;
        Ok(())
    }

    fn render<B: Backend>(&self, f: &mut tui::Frame<B>) {
        let size = f.size();
        let block = Block::default().borders(Borders::ALL).title("Atto");

        let paragraph = Paragraph::new(
            self.buffer.iter().enumerate().skip(self.scroll_offset).take(self.terminal_height).map(|(i, line)| {
                let line_number = format!("{:<4} ", i + 1);
                let line_with_number = format!("{}{}", line_number, line.replace("\\t", "    "));
                Spans::from(Span::raw(line_with_number))
            }).collect::<Vec<_>>()
        ).block(block);
        f.render_widget(paragraph, size);

        if self.show_binds {
            let popup_block = Block::default().borders(Borders::ALL).title("Keybindings");
            let popup_area = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(size)[1];

            let hints = vec![
                KeyBindingHint::new("Ctrl-Q", "Quit"),
                KeyBindingHint::new("Ctrl-K", "Move Up"),
                KeyBindingHint::new("Ctrl-J", "Move Down"),
                KeyBindingHint::new("Ctrl-H", "Move Left"),
                KeyBindingHint::new("Ctrl-L", "Move Right"),
                KeyBindingHint::new("Ctrl-W", "Save"),
                KeyBindingHint::new("Ctrl-R", "Reload"),
            ];

            let hint_lines: Vec<Spans> = hints.iter().map(|hint| {
                Spans::from(vec![
                    Span::styled(&hint.key, Style::default().fg(Color::Yellow)),
                    Span::raw(" - "),
                    Span::raw(&hint.description),
                ])
            }).collect();

            let hint_paragraph = Paragraph::new(hint_lines).block(popup_block);
            f.render_widget(hint_paragraph, popup_area);
        }
    }

    fn input_tab(&mut self) {
        if self.cursor_y < self.buffer.len() && self.cursor_x < self.terminal_width {
            self.buffer[self.cursor_y].insert_str(self.cursor_x, "    ");
            self.cursor_x += 4;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y > 0 {
            self.cursor_y -= 1;
            if self.cursor_y < self.scroll_offset {
                self.scroll_offset -= 1;
            }
            self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y < self.buffer.len() - 1 {
            self.cursor_y += 1;
            if self.cursor_y >= self.scroll_offset + (self.terminal_height - 1) {
                self.scroll_offset += 1;
            }
            self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        }
    }

    fn move_right(&mut self) {
        if self.cursor_y < self.buffer.len() && self.cursor_x < self.buffer[self.cursor_y].len() {
            self.cursor_x += 1;
        }
    }

    fn input_char(&mut self, c: char) {
        if self.cursor_y < self.buffer.len() && self.cursor_x < self.terminal_width - 1 {
            self.buffer[self.cursor_y].insert(self.cursor_x, c);
            self.cursor_x += 1;
        }
    }

    fn new_line(&mut self) {
        let new_line = self.buffer[self.cursor_y].split_off(self.cursor_x);
        self.buffer.insert(self.cursor_y + 1, new_line);
        self.cursor_y += 1;
        self.cursor_x = 0;
        if self.cursor_y >= self.scroll_offset + self.terminal_height {
            self.scroll_offset += 1;
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
            if self.cursor_y < self.scroll_offset {
                self.scroll_offset -= 1;
            }
        }
    }
}

struct KeyBindingHint {
    key: String,
    description: String,
}

impl KeyBindingHint {
    fn new(key: &str, description: &str) -> Self {
        Self {
            key: key.to_string(),
            description: description.to_string(),
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let filename = if args.len() < 2 {
        None
    } else {
        Some(args[1].clone())
    };
    let mut atto = Atto::new(filename);
    atto.read_file()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    atto.run(&mut terminal)?;
    atto.write_file()?;
    Ok(())
}
