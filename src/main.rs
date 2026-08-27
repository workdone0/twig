use anyhow::{Context, Result};
use clap::Parser;
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

    res?;
    Ok(())
}
