use anyhow::{Context, Result};
use clap::Parser;
use owo_colors::{OwoColorize, Stream::Stdout};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use twig::cli::{fix, print as print_mode, Cli};
use twig::tui::app::App;

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.fix {
        return fix::run(&cli);
    }
    if cli.print {
        return print_mode::run(&cli);
    }

    // Interactive TUI.
    let mut terminal =
        Terminal::new(CrosstermBackend::new(std::io::stdout())).context("opening terminal")?;
    crossterm::terminal::enable_raw_mode().context("enabling raw mode")?;
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    );

    let mut app = App::new(&cli.file, cli.rebuild_db);
    let res = app.run(&mut terminal);

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    );

    if let Err(err) = res {
        eprintln!();
        let error_label = "Error"
            .if_supports_color(Stdout, |t| t.bright_red())
            .to_string();
        let detail = err
            .to_string()
            .if_supports_color(Stdout, |t| t.white())
            .to_string();
        eprintln!(" {error_label}: {detail}");

        // Suggest --fix for parse errors, since that's the most likely
        // failure mode when a user runs `twig some-bad.json`.
        let lower = format!("{err:#}").to_lowercase();
        let hint_label = "Tip"
            .if_supports_color(Stdout, |t| t.bright_yellow())
            .to_string();
        if lower.contains("json")
            || lower.contains("parse")
            || lower.contains("expected")
            || lower.contains("trailing")
        {
            let cmd = format!(
                "twig --fix {} -o {}",
                cli.file.display(),
                cli.file.display()
            );
            let cmd_colored = cmd.if_supports_color(Stdout, |t| t.green()).to_string();
            eprintln!(" {hint_label}: this looks like a parse error — try:");
            eprintln!("   {cmd_colored}");
        } else if !lower.contains("permission") && !lower.contains("not found") {
            eprintln!(" {hint_label}: re-run with --fix if the file is malformed:");
            eprintln!(
                "   twig --fix {} -o {}",
                cli.file.display(),
                cli.file.display()
            );
        }
        eprintln!();
        std::process::exit(1);
    }
    Ok(())
}
