mod aws;
use aws::InstanceInfo;
mod ui;
use ui::{restore_terminal, setup_terminal};
mod app;
use app::App;
use std::process::Command;
mod components;
mod history;
use history::{History, HistoryEntry};

use anyhow::{Context, Result};
use signal_hook::{consts::signal::*, iterator::Signals};

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = setup_terminal().context("setup failed")?;
    let Ok(mut app) = App::new() else {
        restore_terminal(&mut terminal).context("restore terminal failed")?;
        return Ok(());
    };
    let selected = app.run(&mut terminal).await;

    restore_terminal(&mut terminal).context("restore terminal failed")?;
    match selected {
        Err(e) => match e.downcast_ref() {
            Some(app::RuntimeError::UserExit) => {}
            _ => {
                println!("{:?}", e);
            }
        },
        Ok(instance) => connect(instance)?,
    }
    Ok(())
}

fn connect(instance: InstanceInfo) -> Result<()> {
    // Run the AWS command
    let entry = HistoryEntry::new(instance.get_instance_id());
    History::save(entry)?;
    let mut child = Command::new("aws")
        .args([
            "--region",
            instance.get_region().as_ref(),
            "ssm",
            "start-session",
            "--target",
            &instance.get_instance_id(),
        ])
        .spawn()?;

    // Catch SIGINT, SIGSTP signal and do nothing
    // So that actually ctrl+c / ctrl+z works on the aws ssm session instead of killing / stopping us
    let mut _signals = Signals::new([SIGINT,SIGTSTP])?;

    child.wait()?;
    Ok(())
}
