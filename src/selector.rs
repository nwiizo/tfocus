use crate::error::Result;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{self, Stylize},
    terminal::{self, ClearType},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::io::{stdout, Write};

pub struct SelectItem {
    pub display: String,     // 表示用の文字列
    pub search_text: String, // 検索用の文字列
    pub data: String,        // 選択時に返すデータ
}

pub struct Selector {
    items: Vec<SelectItem>,
    query: String,
    selected: usize,
    filtered_items: Vec<usize>,
    matcher: SkimMatcherV2,
    window_size: usize,
}

impl Selector {
    pub fn new(items: Vec<SelectItem>) -> Self {
        let filtered_items: Vec<usize> = (0..items.len()).collect();
        Self {
            items,
            query: String::new(),
            selected: 0,
            filtered_items,
            matcher: SkimMatcherV2::default(),
            window_size: 15,
        }
    }

    fn filter_items(&mut self) {
        let query = self.query.to_lowercase();
        let mut matches: Vec<(usize, i64)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                self.matcher
                    .fuzzy_match(&item.search_text.to_lowercase(), &query)
                    .map(|score| (index, score))
            })
            .collect();

        matches.sort_by_key(|&(_, score)| -score);
        self.filtered_items = matches.into_iter().map(|(index, _)| index).collect();
        self.selected = self
            .selected
            .min(self.filtered_items.len().saturating_sub(1));
    }

    fn get_terminal_size() -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }

    fn render_screen(&mut self) -> Result<()> {
        let mut stdout = stdout();
        let (term_width, _) = Self::get_terminal_size();

        // 画面クリアとカーソル位置の初期化
        execute!(
            stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // ヘッダーの表示
        let query_line = format!("QUERY> {}", self.query);
        execute!(stdout, style::Print(&query_line), cursor::MoveToNextLine(1))?;

        // セパレータの表示
        let separator = "─".repeat(term_width as usize);
        execute!(stdout, style::Print(&separator), cursor::MoveToNextLine(1))?;

        let start = if self.filtered_items.len() > self.window_size {
            self.selected
                .saturating_sub(self.window_size / 2)
                .min(self.filtered_items.len() - self.window_size)
        } else {
            0
        };

        let end = (start + self.window_size).min(self.filtered_items.len());

        // アイテムリストの表示
        for i in start..end {
            let item_idx = self.filtered_items[i];
            let item = &self.items[item_idx];

            if i == self.selected {
                execute!(
                    stdout,
                    style::PrintStyledContent("▶ ".green()),
                    style::PrintStyledContent(item.display.clone().green()),
                    cursor::MoveToNextLine(1)
                )?;
            } else {
                execute!(
                    stdout,
                    style::Print("  "),
                    style::Print(&item.display),
                    cursor::MoveToNextLine(1)
                )?;
            }
        }

        // フッターの表示
        if self.filtered_items.len() > self.window_size {
            execute!(
                stdout,
                cursor::MoveToNextLine(1),
                style::Print(&separator),
                cursor::MoveToNextLine(1)
            )?;
        }

        // ステータスラインの表示
        let status = format!("{}/{} items", self.filtered_items.len(), self.items.len());
        let help = "[↑/k]Up [↓/j]Down [Enter]Select [Esc/Ctrl+C]Cancel";

        execute!(
            stdout,
            style::Print(&status),
            cursor::MoveToColumn(term_width - help.len() as u16),
            style::Print(help),
            cursor::MoveToNextLine(1)
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<Option<String>> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;

        let result = self.run_loop();

        execute!(stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;

        result
    }

    fn run_loop(&mut self) -> Result<Option<String>> {
        loop {
            self.render_screen()?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match (key.code, key.modifiers) {
                    (KeyCode::Enter, _) => {
                        if let Some(&idx) = self.filtered_items.get(self.selected) {
                            return Ok(Some(self.items[idx].data.clone()));
                        }
                    }
                    (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        return Ok(None);
                    }
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => {
                        if self.selected > 0 {
                            self.selected -= 1;
                        }
                    }
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => {
                        if self.selected + 1 < self.filtered_items.len() {
                            self.selected += 1;
                        }
                    }
                    (KeyCode::Backspace, _) => {
                        if !self.query.is_empty() {
                            self.query.pop();
                            self.filter_items();
                        }
                    }
                    (KeyCode::Char(c), m)
                        if m == KeyModifiers::NONE || m == KeyModifiers::SHIFT =>
                    {
                        self.query.push(c);
                        self.filter_items();
                    }
                    _ => {}
                }
            }
        }
    }
}
