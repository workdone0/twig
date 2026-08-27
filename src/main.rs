use anyhow::Result;
use clap::Parser;

use twig::cli::{fix, print as print_mode, Cli};

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.fix {
        return fix::run(&cli);
    }
    if cli.print {
        return print_mode::run(&cli);
    }

    // Interactive TUI lands in a later commit.
    eprintln!(
        "twig {} — interactive TUI coming up next commit; for now use --fix or --print.",
        env!("CARGO_PKG_VERSION")
    );
    Ok(())
}