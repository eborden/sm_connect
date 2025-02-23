use crate::aws::fetch_instances;
use crate::aws::InstanceInfo;
use crate::components::instance_details::InstanceDetails;
use crate::components::region_list::RegionList;
use crate::components::{Action, HandleAction, Render};

use crate::components::instance_selection::InstanceSelection;

use aws_config::Region;
use crossterm::event::{self};

use ratatui::style::Style;
use ratatui::{prelude::*, widgets::*};

use std::io::Stdout;

use anyhow::Result;
use thiserror::Error;

mod config;

#[derive(Debug, Clone)]
pub enum AppStatus {
    RegionSelectState,
    MainScreen,
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("User exited the application")]
    UserExit,
}

#[derive(Debug, Clone)]
pub struct App {
    config: config::Config,
    region_select_component: RegionList,
    status: AppStatus,
    info_panel_component: InstanceDetails,
    instance_selection_component: InstanceSelection,
}

impl App {
    pub fn new() -> Self {
        let config = config::Config::new();
        let mut region_select = RegionList::with_items(config.get_visible_regions());
        region_select.set_favorites(config.get_favorite_regions());
        App {
            config: config.clone(),
            region_select_component: region_select,
            status: AppStatus::RegionSelectState,
            info_panel_component: InstanceDetails::default(),
            instance_selection_component: InstanceSelection::default(),
        }
    }

    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<InstanceInfo> {
        let mut should_exit = false;
        let mut return_value: Option<InstanceInfo> = None;
        loop {
            // render
            terminal.draw(|frame| {
                    // Set global layout
                    let render_area = self.get_component_render_area(frame);

                    match self.status {
                        AppStatus::RegionSelectState => {
                            self.region_select_component.render(frame, render_area);
                        }
                        AppStatus::MainScreen => {
                            self.instance_selection_component.render(frame, render_area);
                        }
                    }
                }
            ).unwrap();

            // handle events
            let event = event::read().unwrap();
            match self.status {
                AppStatus::RegionSelectState => {
                    let action = self.region_select_component.handle_action(event);
                    // TODO: Move config management to be owned by the component
                    match action {
                        Action::Exit => {
                            should_exit = true;
                        }
                        Action::Return(region) => {
                            self.status = AppStatus::MainScreen;
                            let instances = fetch_instances(Region::new(region)).await?;
                            self.instance_selection_component.update_instances(instances);
                            
                        }
                        Action::Hide(region) => {
                            self.config.hide_region(region);
                            self.region_select_component
                                .update_items(self.config.get_visible_regions());
                        }
                        Action::Reset => {
                            self.config.reset_hidden_regions();
                            self.region_select_component
                                .update_items(self.config.get_visible_regions());
                        }
                        Action::ToggleFavorite(region) => {
                            self.config.toggle_favorite_region(region);
                            self.region_select_component
                                .set_favorites(self.config.get_favorite_regions());
                        }
                        _ => {}
                    }
                }
                AppStatus::MainScreen => {
                    let action = self.instance_selection_component.handle_action(event);
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
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(90),
            ]
            .as_ref(),
        )
        .split(frame.area());

        let tabs = Tabs::new(vec!["Region", "Instances", "Connection"])
            .block(Block::bordered())
            .style(Style::default().white())
            .highlight_style(Style::default().yellow())
            .select(match self.status {
                AppStatus::RegionSelectState => 0,
                AppStatus::MainScreen => 1,
            });
        frame.render_widget(tabs, outer[0]);
        outer[1]
    }
}
