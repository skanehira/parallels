use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Run multiple commands in parallel with TUI", long_about = None)]
struct Args {
    /// Commands to run in parallel
    #[arg(required = true)]
    commands: Vec<String>,
}

fn main() {
    let args = Args::parse();
    println!("Commands: {:?}", args.commands);
    // To be implemented in phase 10
}
