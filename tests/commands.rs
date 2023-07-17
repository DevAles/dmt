use std::process::Command;

use dmt::commands::{CommandHelper, CommandOptions};

async fn run_command(command: &str) -> Result<(), String> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| e.to_string())
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(String::from_utf8(output.stderr).unwrap())
            }
        })
}

#[serial_test::serial]
#[tokio::test]
async fn install() {
    if let Ok(_) = run_command("which cmatrix").await {
        run_command("yay -R cmatrix --noconfirm").await.unwrap();
    }

    let command = CommandHelper {
        option: CommandOptions::Install,
        packages: vec!["cmatrix".to_string()],
    };

    command.install(0);

    assert!(run_command("which cmatrix").await.is_ok());
}

#[serial_test::serial]
#[tokio::test]
async fn remove() {
    if let Err(_) = run_command("which cmatrix").await {
        run_command("yay -S cmatrix --noconfirm").await.unwrap();
    }

    let command = CommandHelper {
        option: CommandOptions::Remove,
        packages: vec!["cmatrix".to_string()],
    };

    command.remove(0);

    assert!(run_command("which cmatrix").await.is_err());
}
