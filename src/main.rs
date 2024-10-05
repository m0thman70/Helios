use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEvent, MouseEventKind},
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
use rlua::{Lua, RluaCompat, Table};
use std::path::Path;
use crossterm::event::{KeyEvent, KeyModifiers};
use crossterm::event::Event::Key;

struct KeyBindings {
    save: (KeyCode, KeyModifiers),
    quit: (KeyCode, KeyModifiers),
    move_up: (KeyCode, KeyModifiers),
    move_down: (KeyCode, KeyModifiers),
    move_left: (KeyCode, KeyModifiers),
    move_right: (KeyCode, KeyModifiers),
}

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
    horizontal_scroll_offset: usize,
    key_bindings: KeyBindings,

}

impl Atto {
    fn new(filename: Option<String>, preset: &str) -> Self {
        let (width, height) = crossterm::terminal::size().unwrap();
        let key_bindings = match preset {
            "atto" => KeyBindings {
                save: (KeyCode::Char('w'), KeyModifiers::CONTROL),
                quit: (KeyCode::Char('q'), KeyModifiers::CONTROL),
                move_up: (KeyCode::Char('k'), KeyModifiers::CONTROL),
                move_down: (KeyCode::Char('j'), KeyModifiers::CONTROL),
                move_left: (KeyCode::Char('h'), KeyModifiers::CONTROL),
                move_right: (KeyCode::Char('l'), KeyModifiers::CONTROL),
            },
            "nano" => KeyBindings {
                save: (KeyCode::Char('o'), KeyModifiers::CONTROL),
                quit: (KeyCode::Char('x'), KeyModifiers::CONTROL),
                move_up: (KeyCode::Char('k'), KeyModifiers::CONTROL),
                move_down: (KeyCode::Char('j'), KeyModifiers::CONTROL),
                move_left: (KeyCode::Char('h'), KeyModifiers::CONTROL),
                move_right: (KeyCode::Char('l'), KeyModifiers::CONTROL),
            },
            "micro" => KeyBindings {
                save: (KeyCode::Char('s'), KeyModifiers::CONTROL),
                quit: (KeyCode::Char('q'), KeyModifiers::CONTROL),
                move_up: (KeyCode::Char('k'), KeyModifiers::CONTROL),
                move_down: (KeyCode::Char('j'), KeyModifiers::CONTROL),
                move_left: (KeyCode::Char('h'), KeyModifiers::CONTROL),
                move_right: (KeyCode::Char('l'), KeyModifiers::CONTROL),
            },
            "emacs" => KeyBindings {
                save: (KeyCode::Char('x'), KeyModifiers::CONTROL),
                quit: (KeyCode::Char('c'), KeyModifiers::CONTROL),
                move_up: (KeyCode::Char('p'), KeyModifiers::CONTROL),
                move_down: (KeyCode::Char('n'), KeyModifiers::CONTROL),
                move_left: (KeyCode::Char('b'), KeyModifiers::CONTROL),
                move_right: (KeyCode::Char('f'), KeyModifiers::CONTROL),
            },
            _ => KeyBindings {
                save: (KeyCode::Char('t'), KeyModifiers::CONTROL),
                quit: (KeyCode::Char('w'), KeyModifiers::CONTROL),
                move_up: (KeyCode::Char('k'), KeyModifiers::CONTROL),
                move_down: (KeyCode::Char('j'), KeyModifiers::CONTROL),
                move_left: (KeyCode::Char('h'), KeyModifiers::CONTROL),
                move_right: (KeyCode::Char('l'), KeyModifiers::CONTROL),
            }
        };
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
            horizontal_scroll_offset: 0,
            key_bindings,
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

            execute!(io::stdout(), MoveTo(self.cursor_x as u16 + self.cursor_offset_x, self.cursor_y as u16 - self.scroll_offset as u16 + self.cursor_offset_y), Show)?;

            if let Event::Key(key) = event::read()? {
                match (key.code, key.modifiers) {
                    (code, modifiers) if (code, modifiers) == self.key_bindings.quit => break,
                    (code, modifiers) if (code, modifiers) == self.key_bindings.save => self.write_file()?,
                    (code, modifiers) if (code, modifiers) == self.key_bindings.move_up => self.move_up(),
                    (code, modifiers) if (code, modifiers) == self.key_bindings.move_down => self.move_down(),
                    (code, modifiers) if (code, modifiers) == self.key_bindings.move_left => self.move_left(),
                    (code, modifiers) if (code, modifiers) == self.key_bindings.move_right => self.move_right(),
                    (KeyCode::Up, _) => self.move_up(),
                    (KeyCode::Down, _) => self.move_down(),
                    (KeyCode::Left, _) => self.move_left(),
                    (KeyCode::Right, _) => self.move_right(),
                    (KeyCode::Char('r'), KeyModifiers::CONTROL) => self.read_file()?,
                    (KeyCode::Esc, _) => self.show_binds = !self.show_binds,
                    (KeyCode::Enter, _) => self.new_line(),
                    (KeyCode::Char(v), _) => self.input_char(v),
                    (KeyCode::Backspace, _) => self.backspace(),
                    (KeyCode::Tab, _) => self.input_tab(),
                    (KeyCode::PageUp, _) => self.page_up(),
                    (KeyCode::PageDown, _) => self.page_down(),
                    _ => {}
                }
            } else if let Event::Mouse(mouse_event) = event::read()? {
                match mouse_event.kind {
                    MouseEventKind::ScrollUp => self.scroll_up(),
                    MouseEventKind::ScrollDown => self.scroll_down(),
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture, Show)?;
        Ok(())
    }

    fn page_up(&mut self) {
        if self.scroll_offset > 0 {
            let scroll_amount = std::cmp::min(self.scroll_offset, self.terminal_height);
            self.scroll_offset -= scroll_amount;
            self.cursor_y = self.scroll_offset;
        } else {
            self.cursor_y = 0;
        }
        self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
    }

    fn page_down(&mut self) {
        if self.scroll_offset + self.terminal_height < self.buffer.len() {
            let scroll_amount = std::cmp::min(self.terminal_height, self.buffer.len() - self.scroll_offset - self.terminal_height);
            self.scroll_offset += scroll_amount;
            self.cursor_y = self.scroll_offset + self.terminal_height - 3;
        } else {
            self.cursor_y = self.buffer.len() - 3;
        }
        self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
            if self.cursor_y > 0 {
                self.cursor_y -= 1;
            }
        }
    }

    fn scroll_down(&mut self) {
        if self.scroll_offset + self.terminal_height < self.buffer.len() {
            self.scroll_offset += 1;
            if self.cursor_y < self.buffer.len() - 1 {
                self.cursor_y += 1;
            }
        }
    }


    fn render<B: Backend>(&self, f: &mut tui::Frame<B>) {
        let size = f.size();
        let block = Block::default().borders(Borders::ALL).title("Atto");

        let paragraph = Paragraph::new(
            self.buffer.iter().enumerate().skip(self.scroll_offset).take(self.terminal_height).map(|(i, line)| {
                let line_number = format!("{:>4} ", i + 1);
                let line_with_number = format!("{}{}", line_number, line.replace("\\t", "    "));
                let visible_line = if line_with_number.len() > self.horizontal_scroll_offset {
                    line_with_number[self.horizontal_scroll_offset..].to_string() // Clone the string slice
                } else {
                    String::new()
                };
                Spans::from(Span::raw(visible_line))
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
            if self.cursor_y >= self.scroll_offset + (self.terminal_height - 2) {
                self.scroll_offset += 1;
            }
            self.cursor_x = std::cmp::min(self.cursor_x, self.buffer[self.cursor_y].len());
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
            if self.cursor_x < self.horizontal_scroll_offset {
                self.horizontal_scroll_offset -= 1;
            }
        }
    }

    fn move_right(&mut self) {
        if self.cursor_y < self.buffer.len() && self.cursor_x < self.buffer[self.cursor_y].len() {
            self.cursor_x += 1;
            if self.cursor_x >= self.horizontal_scroll_offset + self.terminal_width {
                self.horizontal_scroll_offset += 7;
            }
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

    let lua = Lua::new();

    let paths: Vec<&str> = vec![
        r"C:\Program Files\atto",
        "/etc/atto",
        "/usr/local/etc/atto",
        "/usr/share/config/atto",
        ".config/atto",
        "/Library/Application Support/atto",
        "/Users/YourUsername/Library/Application Support/atto",
    ];

    // Search for config file in specified paths and current/parent directories
    let config_path = find_config_file(&paths)?;

    let preset: String = lua.context(|lua_ctx| {
        let config: Table = lua_ctx.load(&std::fs::read_to_string(config_path).unwrap()).eval().unwrap();
        config.get("key_binding_preset").unwrap()
    });

    let mut atto = Atto::new(filename, &preset);
    atto.read_file()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    atto.run(&mut terminal)?;
    atto.write_file()?;
    Ok(())
}

fn find_config_file(paths: &[&str]) -> io::Result<String> {
    let config_file_names = ["config.lua", "atto.lua"];

    for config_file_name in config_file_names {
        // Check in specified paths
        for path in paths {
            let config_path = Path::new(path).join(config_file_name);
            if config_path.exists() {
                return Ok(config_path.to_str().unwrap().to_string());
            }
        }

        // Check in current and parent directories
        let config_path = find_config_in_current_and_parent(config_file_name)?;
        if !config_path.is_empty() {
            return Ok(config_path);
        }
    }

    Err(io::Error::new(io::ErrorKind::NotFound, "No configuration file found!"))
}

fn find_config_in_current_and_parent(filename: &str) -> io::Result<String> {
    let current_dir_path = Path::new(filename);
    if current_dir_path.exists() {
        return Ok(current_dir_path.to_str().unwrap().to_string());
    }

    let mut current_dir = env::current_dir().unwrap();
    while current_dir.exists() {
        let config_path = current_dir.join(filename);
        if config_path.exists() {
            return Ok(config_path.to_str().unwrap().to_string());
        }
        current_dir.pop();
    }

    Ok(String::new())
}
