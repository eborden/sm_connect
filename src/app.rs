use crate::aws::fetch_instances;
use crate::aws::InstanceInfo;
use crate::components::config_panel::ConfigPanel;
use crate::components::instance_details::InstanceDetails;
use crate::components::region_list::RegionList;
use crate::components::{Action, HandleAction, Render};

use crate::components::instance_selection::InstanceSelection;

use aws_config::Region;
use crossterm::event::{self};

use ratatui::style::Style;
use ratatui::{prelude::*, widgets::*};

use std::io::Stdout;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Result;
use thiserror::Error;

pub mod config;

#[derive(Debug, Clone)]
pub enum AppStatus {
    RegionSelectState,
    MainScreen,
    ConfigPanelState,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("User exited the application")]
    UserExit,
}

#[derive(Debug)]
pub struct App {
    config: Arc<Mutex<config::Config>>,
    config_panel: ConfigPanel,
    region_select_component: RegionList,
    status: AppStatus,
    info_panel_component: InstanceDetails,
    instance_selection_component: InstanceSelection,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = config::Config::new()?;
        let config = Arc::new(Mutex::new(config));
        let config_panel = ConfigPanel::new(config.clone());
        let unlocked = config.lock().unwrap();
        let mut region_select = RegionList::with_items(unlocked.get_visible_regions());
        region_select.set_favorites(unlocked.get_favorite_regions());
        drop(unlocked);
        Ok(App {
            config,
            config_panel,
            region_select_component: region_select,
            status: AppStatus::RegionSelectState,
            info_panel_component: InstanceDetails::default(),
            instance_selection_component: InstanceSelection::default(),
        })
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<InstanceInfo> {
        let mut should_exit = false;
        let mut return_value: Option<InstanceInfo> = None;
        loop {
            // render
            terminal
                .draw(|frame| {
                    // Set global layout
                    let render_area = self.get_component_render_area(frame);

                    match self.status {
                        AppStatus::RegionSelectState => {
                            self.region_select_component.render(frame, render_area);
                        }
                        AppStatus::MainScreen => {
                            self.instance_selection_component.render(frame, render_area);
                        }
                        AppStatus::ConfigPanelState => {
                            self.config_panel.render(frame, render_area);
                        }
                    }
                })?;

            // handle events
            let event = event::read()?;
            match self.status {
                AppStatus::RegionSelectState => {
                    let action = self.region_select_component.handle_action(event)?;
                    match action {
                        Action::Exit => {
                            should_exit = true;
                        }
                        Action::Return(region) => {
                            self.status = AppStatus::MainScreen;
                            let instances = fetch_instances(Region::new(region)).await?;
                            self.instance_selection_component
                                .update_instances(instances);
                        }
                        Action::Hide(region) => {
                            let mut config = self.config.lock().unwrap();
                            config.hide_region(region)?;
                            self.region_select_component
                                .update_items(config.get_visible_regions());
                        }
                        Action::Reset => {
                            let mut config = self.config.lock().unwrap();
                            config.reset_hidden_regions()?;
                            self.region_select_component
                                .update_items(config.get_visible_regions());
                        }
                        Action::ToggleFavorite(region) => {
                            let mut config = self.config.lock().unwrap();
                            config.toggle_favorite_region(region)?;
                            self.region_select_component
                                .set_favorites(config.get_favorite_regions());
                        }
                        Action::OpenConfig => {
                            self.status = AppStatus::ConfigPanelState;
                        }
                        _ => {}
                    }
                }
                AppStatus::MainScreen => {
                    let action = self.instance_selection_component.handle_action(event)?;
                    match action {
                        Action::Exit => {
                            self.status = AppStatus::RegionSelectState;
                        }
                        Action::ReturnInstance(instance) => {
                            should_exit = true;
                            return_value = Some(instance);
                        }
                        Action::Select(instance) => {
                            self.info_panel_component.set_instance(instance);
                        }
                        _ => {}
                    }
                }
                AppStatus::ConfigPanelState => {
                    let action = self.config_panel.handle_action(event)?;
                    if let Action::Exit = action {
                        self.status = AppStatus::RegionSelectState;
                    }
                }
            }

            if should_exit {
                break;
            }
        }
        match return_value {
            Some(instance) => Ok(instance),
            None => Err(RuntimeError::UserExit.into()),
        }
    }

    /**
     * Creates the app layout and returns the area for components to render themselves
     */
    fn get_component_render_area(&self, frame: &mut Frame) -> Rect {
        let outer = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Max(3), Constraint::Fill(1)].as_ref())
            .split(frame.area());

        let tabs = Tabs::new(vec!["Region", "Instances", "Connection"])
            .block(Block::bordered())
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(match self.status {
                AppStatus::RegionSelectState => Some(0),
                AppStatus::MainScreen => Some(1),
                _ => None,
            });
        frame.render_widget(tabs, outer[0]);
        outer[1]
    }
}
