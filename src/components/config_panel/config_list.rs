use crate::components::{Action, HandleAction, Render, RenderHelp, View};
use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, List, ListItem, ListState, Row, Table},
    Frame,
};

#[derive(Debug, Clone, Copy)]
pub enum ConfigOption {
    ResetRecent,
    SetRecentTimeout,
}

impl From<ConfigOption> for String {
    fn from(option: ConfigOption) -> String {
        match option {
            ConfigOption::ResetRecent => "Reset Recent Instances".to_string(),
            ConfigOption::SetRecentTimeout => "Set Recent Timeout".to_string(),
        }
    }
}

const CONFIG_OPTIONS: [ConfigOption; 2] =
    [ConfigOption::ResetRecent, ConfigOption::SetRecentTimeout];
#[derive(Debug)]
pub struct ConfigList {
    state: ListState,
}

impl ConfigList {
    pub fn new() -> ConfigList {
        let mut state = ListState::default();
        state.select(Some(0));
        ConfigList { state }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= CONFIG_OPTIONS.len() - 1 {
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
                    CONFIG_OPTIONS.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn current(&self) -> Option<ConfigOption> {
        self.state.selected().map(|i| CONFIG_OPTIONS[i])
    }
}

impl HandleAction for ConfigList {
    fn handle_action(&mut self, action: Event) -> Action {
        match action {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Action::Exit,
                KeyCode::Down => {
                    self.next();
                    Action::Noop
                }
                KeyCode::Up => {
                    self.previous();
                    Action::Noop
                }
                KeyCode::Right | KeyCode::Enter => match self.current() {
                    Some(option) => Action::ReturnConfig(option),
                    None => Action::Noop,
                },
                _ => Action::Noop,
            },
            _ => Action::Noop,
        }
    }
}

#[allow(refining_impl_trait)]
impl View for ConfigList {
    fn get_widget(&self) -> List {
        let items: Vec<ListItem> = CONFIG_OPTIONS
            .iter()
            .map(|i| {
                let name: String = (*i).into();
                ListItem::new(name)
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

impl Render for ConfigList {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(100)])
            .split(area);

        let widget = self.get_widget();
        frame.render_stateful_widget(widget, vertical_layout[0], &mut self.state.clone());
    }
}

impl RenderHelp for ConfigList {
    fn render_help(&mut self, frame: &mut Frame, area: Rect) {
        let rows = vec![Row::new(vec![Cell::from(Span::styled(
            "'q' Exit",
            Style::default().fg(Color::White),
        ))])];
        let table = Table::new(rows, vec![Constraint::Min(10)]);
        frame.render_widget(table, area);
    }
}
