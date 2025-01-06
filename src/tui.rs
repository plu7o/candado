use crate::Entry;
use anyhow::{anyhow, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Margin, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        Block, Cell, HighlightSpacing, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
        TableState,
    },
    DefaultTerminal, Frame,
};
use style::palette::tailwind;
use unicode_width::UnicodeWidthStr;

const ITEM_HEIGHT: usize = 2;

struct TableColors {
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_row_style_fg: Color,
    selected_cell_style_fg: Color,
}

impl TableColors {
    const fn new() -> Self {
        Self {
            header_bg: Color::Green,
            header_fg: tailwind::BLACK,
            row_fg: tailwind::SLATE.c200,
            selected_row_style_fg: Color::Green,
            selected_cell_style_fg: tailwind::PURPLE.c400,
        }
    }
}

pub struct TableApp {
    state: TableState,
    items: Vec<Entry>,
    colors: TableColors,
    longest_item_lens: (u16, u16, u16, u16, u16, u16), // order is (id, service, email, password, username, url)
    scroll_state: ScrollbarState,
    unlocked: bool,
    exit: bool,
}

impl TableApp {
    pub fn new(items: Vec<Entry>) -> Result<Self, anyhow::Error> {
        Ok(Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: constraint_len_calculator(&items),
            colors: TableColors::new(),
            scroll_state: ScrollbarState::new((items.len().saturating_add(1)) * ITEM_HEIGHT),
            items,
            unlocked: false,
            exit: false,
        })
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let title = Line::from(vec![
            format!(
                " Candado {} ",
                if self.unlocked {
                    "\u{f2fc}"
                } else {
                    "\u{f023}"
                }
            )
            .green()
            .bold(),
            format!(" | Total: {} ", self.items.len()).into(),
        ]);
        let instructions = Line::from(vec![
            " Exit ".into(),
            "<Esc / q> | ".bold(),
            "Move up ".into(),
            "<(↑) / k> | ".bold(),
            "Move down ".into(),
            "<(↓) / j> | ".bold(),
            "Show ".into(),
            "<u> ".bold(),
        ]);

        let block = Block::bordered()
            .title(title)
            .title_bottom(instructions.centered())
            .border_set(border::ROUNDED)
            .green();
        let inner = block.inner(frame.area());
        frame.render_widget(block, frame.area());

        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
        let rects = vertical.split(inner);
        self.render_table(frame, rects[0]);
        self.render_scrollbar(frame, rects[0]);
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                // let shift_pressed = key_event.modifiers.contains(KeyModifiers::SHIFT);
                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.exit(),
                    KeyCode::Char('j') | KeyCode::Down => self.next_row(),
                    KeyCode::Char('k') | KeyCode::Up => self.previous_row(),
                    KeyCode::Char('u') => self.unlocked = !self.unlocked,
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let header_style = Style::default()
            .fg(self.colors.header_fg)
            .bg(self.colors.header_bg);

        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_row_style_fg);

        let selected_col_style = Style::default().fg(self.colors.selected_cell_style_fg);

        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(self.colors.selected_cell_style_fg);

        let header = ["id", "Service", "Email", "Password", "username", "url"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);

        let rows = self.items.iter().enumerate().map(|(i, data)| {
            let item = data.ref_array();
            item.into_iter()
                .enumerate()
                .map(|(pos, content)| {
                    if pos == 3
                        && !(self.unlocked && self.state.selected().or(Some(0)).unwrap() == i)
                    {
                        return Cell::from(Text::from(format!("\n{}\n", "************")));
                    }
                    Cell::from(Text::from(format!("\n{content}\n")))
                })
                .collect::<Row>()
                .style(Style::new().fg(self.colors.row_fg))
                .height(3)
        });
        let bar = " \u{f111} ";
        let t = Table::new(
            rows,
            [
                Constraint::Length(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1),
                Constraint::Min(self.longest_item_lens.2),
                Constraint::Min(self.longest_item_lens.3 + 1),
                Constraint::Min(self.longest_item_lens.4),
                Constraint::Min(self.longest_item_lens.5),
            ],
        )
        .header(header)
        .row_highlight_style(selected_row_style)
        .column_highlight_style(selected_col_style)
        .cell_highlight_style(selected_cell_style)
        .highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    fn render_scrollbar(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }
}

fn constraint_len_calculator(items: &[Entry]) -> (u16, u16, u16, u16, u16, u16) {
    let id_len = items
        .iter()
        .map(Entry::id)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let service_len = items
        .iter()
        .map(Entry::service)
        .flat_map(str::lines)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let email_len = items
        .iter()
        .map(Entry::email)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let password_len = items
        .iter()
        .map(Entry::password)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let username_len = items
        .iter()
        .map(Entry::username)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);
    let url_len = items
        .iter()
        .map(Entry::url)
        .map(UnicodeWidthStr::width)
        .max()
        .unwrap_or(0);

    #[allow(clippy::cast_possible_truncation)]
    (
        id_len as u16,
        service_len as u16,
        email_len as u16,
        password_len as u16,
        username_len as u16,
        url_len as u16,
    )
}

pub enum App {
    Table(TableApp),
}

pub struct CandadoTui {}

impl CandadoTui {
    pub fn init(app: App) -> Result<()> {
        color_eyre::install().map_err(|e| anyhow!("{e}"))?;
        let terminal = ratatui::init();

        let app_result = match app {
            App::Table(app) => app.run(terminal),
        };

        ratatui::restore();
        app_result
    }
}
