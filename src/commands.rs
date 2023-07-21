use std::env;
use std::process::{Command, Stdio};

const DISTROTUNNEL: &str = env!("DISTROTUNNEL");

pub enum CommandOptions {
    Install,
    Remove,
    Query,
    Update,
    Help,
}

pub struct CommandHelper {
    pub option: CommandOptions,
    pub packages: Vec<String>,
}

impl CommandHelper {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();

        let mut option = CommandOptions::Help;
        let mut packages = Vec::new();

        if args.len() > 1 {
            option = CommandHelper::attribute_option(&args[1]);
            if args.len() > 2 {
                packages = args.iter().skip(2).map(|s| s.to_owned()).collect();
            }
        }

        Self { option, packages }
    }

    pub fn attribute_option(command: &str) -> CommandOptions {
        match command {
            "install" | "-i" => CommandOptions::Install,
            "remove" | "-r" => CommandOptions::Remove,
            "query" | "-q" => CommandOptions::Query,
            "update" | "-u" => CommandOptions::Update,
            "help" | "-h" => CommandOptions::Help,
            _ => CommandOptions::Help,
        }
    }

    pub fn install(&self, package_number: usize) {
        println!("> Adding package");

        let package = &self.packages[package_number];

        if let Err(e) = install(&package) {
            println!("> Error adding package: {}", e);
            return;
        }
    }

    pub fn query(&self, package_number: usize) -> Result<Vec<String>, ()> {
        let package = &self.packages[package_number];

        let binaries = query(&package);

        if let Err(_) = binaries {
            return Err(());
        }

        let binaries = binaries.unwrap();
        Ok(binaries)
    }

    pub fn export_app(&self, binaries: &Vec<String>) {
        if let Err(e) = export_app(&binaries) {
            if e.contains("cannot find any desktop files") {
                println!("> Not have a .desktop file");
            } else {
                println!("> Error exporting package: {}", e);
            }
        }
    }

    pub fn export_bin(&self, binaries: &Vec<String>) {
        export_bin(&binaries).unwrap();
    }

    pub fn unexport_app(&self, binaries: &Vec<String>) {
        if let Err(e) = unexport_app(&binaries) {
            if e.contains("cannot find any desktop files") {
                println!("> Not have a .desktop file");
            } else {
                println!("> Error unexporting package: {}", e);
            }
        }
    }

    pub fn unexport_bin(&self, binaries: &Vec<String>, package_number: usize) {
        let package = &self.packages[package_number];

        if let Err(e) = unexport_bin(&binaries) {
            if e.is_empty() {
                println!("> {} does not have binaries", package);
            } else {
                println!("> Package not exported, can't unexport")
            }
        }
    }

    pub fn remove(&self, package_number: usize) {
        let package = &self.packages[package_number];

        if let Err(e) = remove(&package) {
            if e.contains("breaks dependency") {
                let mut error = e.lines().collect::<Vec<&str>>();
                error.remove(0);
                error.remove(error.len() - 1);

                println!("{}", error.join(""));
            } else {
                println!("> Error removing package: {}", e);
            }
        } else {
            println!("> Bye bye {} :)", package);
            println!();
        }
    }

    pub fn update() {
        if let Err(e) = update() {
            println!("Error updating: {}", e);
            return;
        }
    }

    pub fn process(&self) {
        match self.option {
            CommandOptions::Install => {
                if self.packages.len() == 0 {
                    println!("> You need to specify a package to install");
                    return;
                }

                for i in 0..self.packages.len() {
                    self.install(i);

                    let binaries = self.query(i);

                    if let Err(_) = binaries {
                        return;
                    }

                    let binaries = binaries.unwrap();

                    self.export_app(&binaries);
                    self.export_bin(&binaries);
                }
            }

            CommandOptions::Remove => {
                if self.packages.len() == 0 {
                    println!("> You need to specify a package to remove");
                    return;
                }

                for i in 0..self.packages.len() {
                    let binaries = self.query(i);

                    if let Err(_) = binaries {
                        println!("> Package not added, can't remove");
                        return;
                    }

                    let binaries = binaries.unwrap();

                    self.unexport_app(&binaries);
                    self.unexport_bin(&binaries, i);
                    self.remove(i);
                }
            }

            CommandOptions::Update => {
                CommandHelper::update();
            }

            CommandOptions::Query => {
                if self.packages.len() == 0 {
                    println!("> You need to specify a package to query");
                    return;
                }

                for i in 0..self.packages.len() {
                    let binaries = self.query(i);

                    if let Err(_) = binaries {
                        println!("> Package not added, can't query");
                        return;
                    }

                    let binaries = binaries.unwrap();

                    println!("{}", binaries.join("\n"));
                }
            }

            CommandOptions::Help => {
                let help = "
                Usage: dmt [OPTION] [PACKAGE]

                Options:
                install, -i   Add package to distrotunnel
                remove, -r    Remove package from distrotunnel
                update, -u    Update distrotunnel
                help, -h      Show this help
            ";

                let help_trimmed = help
                    .lines()
                    .map(|line| line.trim_start())
                    .collect::<Vec<&str>>()
                    .join("\n");

                println!("{}", help_trimmed);
                return;
            }
        }
    }
}

pub fn query(package: &str) -> Result<Vec<String>, String> {
    let command = Command::new("yay")
        .args(&["-Ql", package])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(command.stdout).unwrap();
    let stderr = String::from_utf8(command.stderr).unwrap();

    if !stderr.is_empty() {
        return Err(stderr);
    }

    let start_of_line = format!("{} ", &package);
    let mut output: Vec<String> = stdout
        .lines()
        .filter(|line| line.contains("/usr/bin/"))
        .map(|line| line.trim_start_matches(&start_of_line).to_string())
        .collect();

    if output.len() > 0 {
        output.remove(0);
    }

    Ok(output)
}

pub fn install(package: &str) -> Result<String, String> {
    println!("> Trying to install {}...", &package);

    let output = Command::new("yay")
        .args(&["-S", "--noconfirm", package])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        return Err(format!("> {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::new())
}

pub fn remove(package: &str) -> Result<String, String> {
    println!("> Trying to remove {}...", &package);
    let command = Command::new("yay")
        .args(&["-R", "--noconfirm", package])
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(command.stdout).unwrap();
    let stderr = String::from_utf8(command.stderr).unwrap();

    if stdout.contains("breaks dependency") {
        return Err(stdout);
    }

    if !stderr.is_empty() {
        if stderr.contains("warning") {
            return Ok(stdout);
        }

        return Err(format!("> {}", &stderr));
    }

    Ok(stdout)
}

pub fn update() -> Result<String, String> {
    println!("> Trying to update...");

    let output = Command::new("yay")
        .args(&["-Syu", "--noconfirm"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("failed to execute process");

    if !output.status.success() {
        return Err(format!("> {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(String::new())
}

pub fn unexport_bin(binaries: &Vec<String>) -> Result<String, String> {
    println!("> Trying to unexport binaries...");
    for binary in binaries {
        let bin_path = format!("{}/bin", DISTROTUNNEL);
        match std::fs::metadata(&bin_path) {
            Ok(_) => {
                let unexport_bin = Command::new("distrobox-export")
                    .args(&["-b", &binary, "-ep", &bin_path, "--delete"])
                    .output()
                    .expect("failed to execute process");

                let stderr = String::from_utf8(unexport_bin.stderr).unwrap();

                if !stderr.is_empty() {
                    return Err(stderr);
                }

                println!(
                    "> Unexporting {}...",
                    binary.trim_start_matches("/usr/bin/")
                );
            }

            Err(_) => {
                println!(
                    "> {} not exists to be unexported",
                    &binary.trim_start_matches("/usr/bin/")
                );
            }
        }
    }

    Ok("".to_string())
}

pub fn unexport_app(binaries: &Vec<String>) -> Result<String, String> {
    for binary in binaries {
        let binary_name = binary.trim_start_matches("/usr/bin/");
        let unexport_app = Command::new("distrobox-export")
            .args(&["--app", binary_name, "--delete"])
            .output()
            .expect("failed to execute process");

        let stdout = String::from_utf8(unexport_app.stdout).unwrap();
        let stderr = String::from_utf8(unexport_app.stderr).unwrap();

        if !stderr.is_empty() {
            return Err(stderr);
        }

        return Ok(stdout);
    }

    Ok("".to_string())
}

pub fn export_bin(binaries: &Vec<String>) -> Result<String, String> {
    let bin_path = format!("{}/bin", DISTROTUNNEL);
    for binary in binaries {
        let export_bin = Command::new("distrobox-export")
            .args(&["-b", &binary, "-ep", &bin_path])
            .output()
            .expect("failed to execute process");

        let stderr = String::from_utf8(export_bin.stderr).unwrap();

        if !stderr.is_empty() {
            return Err(stderr);
        }

        println!("> Exporting {}...", &binary.trim_start_matches("/usr/bin/"));
        println!();
    }

    Ok("".to_string())
}

pub fn export_app(binaries: &Vec<String>) -> Result<String, String> {
    for binary in binaries {
        let binary_name = binary.trim_start_matches("/usr/bin/");
        let export_app = Command::new("distrobox-export")
            .args(&["--app", &binary_name])
            .output()
            .expect("failed to execute process");

        let stdout = String::from_utf8(export_app.stdout).unwrap();
        let stderr = String::from_utf8(export_app.stderr).unwrap();

        if !stderr.is_empty() {
            return Err(stderr);
        }

        return Ok(stdout);
    }

    Ok("".to_string())
}
