use crossterm::event;
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Frame};

use crate::{aws::InstanceInfo, components::{instance_table::InstanceTable, text_input::TextInput}};

use super::{instance_details::InstanceDetails, Action, HandleAction, Render, RenderHelp};


#[derive(Debug, Clone)]
pub struct InstanceSelection {
    instances_table_component: InstanceTable,
    search_component: TextInput,
    instance_details: InstanceDetails,
    search_active: bool,
    info_panel_enabled: bool,
}

impl Default for InstanceSelection {
    fn default() -> Self {
        let instances = vec![];
        InstanceSelection::new(instances)
    }
}

impl InstanceSelection {
    pub fn new(instances: Vec<InstanceInfo>) -> InstanceSelection {
        let instance_table = InstanceTable::with_items(instances);
        let search_component = TextInput::default();
        let instance_details = InstanceDetails::default();
        InstanceSelection {
            instances_table_component: instance_table,
            search_component,
            instance_details,
            search_active: false,
            info_panel_enabled: false,
        }
    }

    pub fn update_instances(&mut self, instances: Vec<InstanceInfo>) {
        self.instances_table_component = InstanceTable::with_items_and_filter(instances, self.search_component.get_value());
    }
}

impl HandleAction for InstanceSelection {
    fn handle_action(&mut self, action: crossterm::event::Event) -> Action {
        if self.search_active{
            let action = self.search_component.handle_action(action);
            match action {
                Action::Exit => {
                    self.search_active = false;
                }
                Action::Return(search) => {
                    self.instances_table_component.apply_filter(search);
                    self.search_active = false;
                }
                Action::PartialReturn(search) => {
                    self.instances_table_component.apply_filter(search);
                }
                Action::ReturnWithKey(key) => {
                    match key {
                        event::KeyCode::Up => {
                            self.instances_table_component.previous();
                        }
                        event::KeyCode::Down => {
                            self.instances_table_component.next();
                        }
                        _ => {}
                    }
                    self.search_active = false;
                }
                _ => {}
            }
            Action::Noop
        }else {
            let action = self.instances_table_component.handle_action(action);
            match action {
                Action::Search => {
                    self.search_active = true;
                    Action::Noop
                }
                Action::ToggleInfoPanel => {
                    self.info_panel_enabled = !self.info_panel_enabled;
                    Action::Noop
                }
                other => other
            }
        }
    }
}

impl Render for InstanceSelection {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(90), Constraint::Percentage(10)])
            .split(area);
        if self.search_active {
            self.search_component.render(frame, vertical_layout[1]);
            frame.set_cursor(
                vertical_layout[1].x
                    + self.search_component.get_cursor_position() as u16,
                    vertical_layout[1].y,
            );
        }else {
            self.instances_table_component.render_help(frame, vertical_layout[1]);
        }
        
        if self.info_panel_enabled {
            let inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![Constraint::Percentage(75), Constraint::Percentage(25)])
                .split(area);
            self.instances_table_component.render(frame, inner_layout[0]);
            self.instance_details.render(frame, inner_layout[1]);
        } else {
            self.instances_table_component.render(frame, vertical_layout[0]);
        }
        
    }
}