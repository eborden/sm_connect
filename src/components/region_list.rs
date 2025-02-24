use super::{Action, HandleAction, Render, RenderHelp, View};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table},
    Frame,
};
use anyhow::Result;
#[derive(Default, Debug, Clone)]
pub struct RegionList {
    state: ListState,
    items: Vec<String>,
    favorites: Vec<String>,
}

impl RegionList {
    pub fn with_items(items: Vec<String>) -> RegionList {
        let mut state = ListState::default();
        state.select(Some(0));
        RegionList {
            state,
            items,
            favorites: Vec::new(),
        }
    }

    pub fn update_items(&mut self, items: Vec<String>) {
        self.items = items;
        if let Some(i) = self.state.selected_mut() {
            if *i >= self.items.len() {
                *i = self.items.len() - 1;
            }
        }
        self.sort_list();
    }

    pub fn set_favorites(&mut self, favorites: Vec<String>) {
        self.favorites = favorites;
        self.sort_list();
    }

    fn sort_list(&mut self) {
        self.items.sort_by(|a, b| {
            if self.favorites.contains(a) && !self.favorites.contains(b) {
                return std::cmp::Ordering::Less;
            }
            if !self.favorites.contains(a) && self.favorites.contains(b) {
                return std::cmp::Ordering::Greater;
            }
            a.cmp(b)
        });
    }

    fn next(&mut self) {
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
    }

    fn previous(&mut self) {
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
    }

    fn current(&self) -> Option<String> {
        self.state.selected().map(|i| self.items[i].clone())
    }
}

impl HandleAction for RegionList {
    fn handle_action(&mut self, action: Event) -> Result<Action> {
       let action =  match action {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Action::Exit,
                KeyCode::Char('h') => Action::Hide(self.current().unwrap()),
                KeyCode::Char('r') => Action::Reset,
                KeyCode::Char('c') => Action::OpenConfig,
                KeyCode::Char('*') => Action::ToggleFavorite(self.current().unwrap()),
                KeyCode::Down => {
                    self.next();
                    Action::Noop
                }
                KeyCode::Up => {
                    self.previous();
                    Action::Noop
                }
                KeyCode::Right | KeyCode::Enter => match self.current() {
                    Some(str) => Action::Return(str.to_owned()),
                    None => Action::Noop,
                },
                _ => Action::Noop,
            },
            _ => Action::Noop,
        };
        Ok(action)
    }
}

#[allow(refining_impl_trait)]
impl View for RegionList {
    fn get_widget(&self) -> List {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|i| {
                let prefix = if self.favorites.contains(i) {
                    "★"
                } else {
                    ""
                };
                ListItem::new(format!("{} {}", prefix, i))
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ")
    }
}

impl Render for RegionList {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(area);

        let widget = self.get_widget();
        frame.render_stateful_widget(widget, vertical_layout[0], &mut self.state.clone());
        self.render_help(frame, vertical_layout[1]);
    }
}

impl RenderHelp for RegionList {
    fn render_help(&mut self, frame: &mut Frame, area: Rect) {
        let rows = vec![
            Row::new(vec![
                Cell::from(Span::styled("'q' Exit", Style::default().fg(Color::White))),
                Cell::from(Span::styled("'h' Hide", Style::default().fg(Color::White))),
                Cell::from(Span::styled(
                    "'r' Reset regions",
                    Style::default().fg(Color::White),
                ))]),
            Row::new(vec![
                Cell::from(Span::styled(
                    "'*' Toggle Favorite",
                    Style::default().fg(Color::White),
                )),
                Cell::from(Span::styled(
                    "'c' to open configuration",
                    Style::default().fg(Color::White),
                )),
        ])];
        let table = Table::new(
            rows,
            vec![
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Min(10),
                Constraint::Min(10),
            ],
        );
        frame.render_widget(table, area);
    }
}
