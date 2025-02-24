pub mod config_panel;
pub mod instance_details;
pub mod instance_selection;
pub mod instance_table;
pub mod region_list;
pub mod text_input;
use config_panel::config_list::ConfigOption;
use crossterm::event::{Event, KeyCode};
use anyhow::Result;
use ratatui::{layout::Rect, widgets::Widget, Frame};

use crate::aws::InstanceInfo;

pub enum Action {
    Noop,
    Exit,
    Return(String),
    ReturnWithKey(KeyCode),
    ReturnInstance(InstanceInfo),
    ReturnConfig(ConfigOption),
    OpenConfig,
    PartialReturn(String),
    Search,
    ToggleInfoPanel,
    Select(InstanceInfo),
    Hide(String),
    Reset,
    ToggleFavorite(String),
}

pub trait HandleAction {
    fn handle_action(&mut self, action: Event) -> Result<Action>;
}

trait View {
    fn get_widget(&self) -> impl Widget;
}

pub trait Render {
    fn render(&mut self, frame: &mut Frame, area: Rect);
}

pub trait RenderHelp {
    fn render_help(&mut self, frame: &mut Frame, area: Rect);
}
