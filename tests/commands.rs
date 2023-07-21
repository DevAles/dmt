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

async fn prepare_to_test_installation(packages: &Vec<String>) -> Result<(), String> {
    for package in packages {
        if let Ok(_) = run_command(&format!("which {}", package)).await {
            run_command(&format!("yay -R {} --noconfirm", package))
                .await
                .unwrap();
        }
    }

    Ok(())
}

async fn prepare_to_test_removal(packages: &Vec<String>) -> Result<(), String> {
    for package in packages {
        if let Err(_) = run_command(&format!("which {}", package)).await {
            run_command(&format!("yay -S {} --noconfirm", package))
                .await
                .unwrap();
        }
    }

    Ok(())
}

#[serial_test::serial]
#[tokio::test]
async fn install() {
    let package_list = vec!["cmatrix".to_string()];
    prepare_to_test_installation(&package_list).await.unwrap();

    let command = CommandHelper {
        option: CommandOptions::Install,
        packages: package_list,
    };

    command.process();

    assert!(run_command("which cmatrix").await.is_ok());
}

#[serial_test::serial]
#[tokio::test]
async fn remove() {
    let packages = vec!["cmatrix".to_string()];
    prepare_to_test_removal(&packages).await.unwrap();

    let command = CommandHelper {
        option: CommandOptions::Remove,
        packages,
    };

    command.process();

    assert!(run_command("which cmatrix").await.is_err());
}

#[serial_test::serial]
#[tokio::test]
async fn multiple_install() {
    let package_list = vec!["cmatrix".to_string(), "cowsay".to_string()];
    prepare_to_test_installation(&package_list).await.unwrap();

    let command = CommandHelper {
        option: CommandOptions::Install,
        packages: package_list,
    };

    command.process();

    assert!(run_command("which cmatrix").await.is_ok());
    assert!(run_command("which cowsay").await.is_ok());
}
