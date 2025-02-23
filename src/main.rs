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
        Ok(instance) => connect(instance),
    }
    Ok(())
}

fn connect(instance: InstanceInfo) {
    // Run the AWS command
    // If fails, run SSH
    let entry = HistoryEntry::new(instance.get_instance_id());
    History::save(entry).unwrap();
    let mut child = Command::new("aws")
        .args([
            "--region",
            instance.get_region().as_ref(),
            "ssm",
            "start-session",
            "--target",
            &instance.get_instance_id(),
        ])
        .spawn()
        .unwrap_or_else(|_| {
            println!("failed to run aws ssm start-session. Falling back to SSH");
            Command::new("ssh")
                .args([instance.get_public_ip()])
                .spawn()
                .expect("Failed even with SSH")
        });

    // Catch SIGINT signal and do nothing
    // So that actually ctrl+c works on the aws ssm session
    // TODO: Handle ctrl+z too
    let mut _signals = Signals::new([SIGINT]).unwrap();

    child.wait().expect("failed to aws sm connect");
}
