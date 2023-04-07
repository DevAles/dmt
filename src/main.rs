mod commands;

use crate::commands::CommandHelper;

fn main() {
    let command = CommandHelper::new();
    command.process();
}
