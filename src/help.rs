use crate::version::VERSION;

pub fn show_help() {
    println!("nodash {} - Node.js project dashboard", VERSION);
    println!();
    println!("USAGE:");
    println!("    nodash [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("    help      Show this help message");
    println!("    add       Add current directory as a project");
    println!("    update    Update nodash to the latest version");
    println!("    version   Show the current version of nodash");
    println!();
    println!("INTERACTIVE CONTROLS:");
    println!("    ↑/↓       Navigate projects");
    println!("    Enter     Open selected project");
    println!("    a         Add new project");
    println!("    /         Search projects");
    println!("    Esc       Clear search");
    println!("    q         Quit");
}