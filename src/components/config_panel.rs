use std::sync::{Arc, Mutex};

use crate::{
    app::config::Config,
    components::text_input::TextInput,
    history::History,
};
use config_list::{ConfigList, ConfigOption};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    Frame,
};

use super::{Action, HandleAction, Render, RenderHelp};
pub mod config_list;
use anyhow::Result;
#[derive(Debug)]
pub struct ConfigPanel {
    config: Arc<Mutex<Config>>,
    config_list: ConfigList,
    input_component: TextInput,
    input_active: bool,
    modifying_action: Option<ConfigOption>,
    last_operation_success: Option<bool>,
}

impl ConfigPanel {
    pub fn new(config: Arc<Mutex<Config>>) -> ConfigPanel {
        let config_list = ConfigList::new();
        let input_component = TextInput::new("Value: ".to_string());
        ConfigPanel {
            config,
            config_list,
            input_component,
            input_active: false,
            modifying_action: None,
            last_operation_success: None,
        }
    }
}

impl HandleAction for ConfigPanel {
    fn handle_action(&mut self, action: crossterm::event::Event) -> Result<Action> {
        if self.input_active {
            let action = self.input_component.handle_action(action)?;
            match action {
                Action::Exit => {
                    self.input_active = false;
                }
                Action::Return(search) => {
                    if let Some(ConfigOption::SetRecentTimeout) = self.modifying_action {
                        if let Ok(timeout) = search.parse::<u64>() {
                            self.config
                                .lock()
                                .unwrap()
                                .set_recent_timeout(timeout)?;
                            self.last_operation_success = Some(true);
                        } else {
                            self.last_operation_success = Some(false);
                        }
                    }
                    self.input_active = false;
                }
                _ => {}
            }
            Ok(Action::Noop)
        } else {
            let action = self.config_list.handle_action(action)?;
            match action {
                Action::ReturnConfig(config) => {
                    match config {
                        ConfigOption::ResetRecent => match History::reset() {
                            Ok(_) => {
                                self.last_operation_success = Some(true);
                            }
                            Err(e) => {
                                eprintln!("Error resetting history: {:?}", e);
                                self.last_operation_success = Some(false);
                            }
                        },
                        ConfigOption::SetRecentTimeout => {
                            self.modifying_action = Some(ConfigOption::SetRecentTimeout);
                            self.input_active = true;
                            let current_value = self.config.lock().unwrap().get_recent_timeout();
                            self.input_component.set_value(current_value.to_string());
                        }
                    }
                    Ok(Action::Noop)
                }
                other => Ok(other),
            }
        }
    }
}

impl Render for ConfigPanel {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(area);
        self.config_list.render(frame, vertical_layout[0]);
        if self.input_active {
            self.input_component.render(frame, vertical_layout[1]);
            frame.set_cursor_position((
                vertical_layout[1].x + self.input_component.get_cursor_position() as u16,
                vertical_layout[1].y,
            ));
        } else {
            self.render_help(frame, vertical_layout[1]);
            if self.last_operation_success.is_some() {
                // Show success message
                self.last_operation_success = None;
            }
        }
    }
}

impl RenderHelp for ConfigPanel {
    fn render_help(&mut self, frame: &mut Frame, area: Rect) {
        match self.last_operation_success {
            Some(true) => {
                let line = Line::from("Operation successful")
                    .centered()
                    .bg(Color::Green);
                frame.render_widget(line, area);
            }
            Some(false) => {
                let line = Line::from("Operation failed").centered().bg(Color::Red);
                frame.render_widget(line, area);
            }
            None => {
                self.config_list.render_help(frame, area);
            }
        }
    }
}
