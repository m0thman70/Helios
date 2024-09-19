use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
//use tree_sitter::{Parser, Language, Node};
//use tree_sitter_highlight::{Highlighter, HighlightConfiguration, HighlightEvent, Highlight};
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

/* extern "C" {
    fn tree_sitter_rust() -> Language;
    fn tree_sitter_c() -> Language;
    fn tree_sitter_haskell() -> Language;
} */
struct Atto {
    cursor_x: usize,
    cursor_y: usize,
    buffer: Vec<String>,
    terminal_height: usize,
    terminal_width: usize,
    filename: Option<String>,
    show_binds: bool,
    //language: Language,
}

impl Atto {
    fn new(filename: Option<String>) -> Self {
        let (width, height) = crossterm::terminal::size().unwrap();
        Self {
            cursor_y: 0,
            cursor_x: 0,
            buffer: vec![String::new()],
            terminal_height: height as usize,
            terminal_width: width as usize,
            filename,
            show_binds: false,
            //language,
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
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        loop {
            terminal.draw(|f| self.render(f))?;

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
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    fn render<B: Backend>(&self, f: &mut tui::Frame<B>) {
        let size = f.size();
        let block = Block::default().borders(Borders::ALL).title("Atto");

        let mut lines = self.buffer.clone();
        if self.cursor_y < lines.len() {
            if self.cursor_x < lines[self.cursor_y].len() {
                lines[self.cursor_y].insert(self.cursor_x, '|');
            } else {
                lines[self.cursor_y].push('|');
            }
        }

        let paragraph = Paragraph::new(
            lines.iter().map(|line| {
                let line = line.replace("\t", "    ");
                Spans::from(Span::raw(line))
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
