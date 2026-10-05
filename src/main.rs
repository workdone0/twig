use anyhow::{Context, Result};
use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use twig::{
    cli::{check, print, Cli},
    tui::app::App,
};

fn restore_terminal() {
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    );
}
struct TerminalGuard;
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn main() {
    if let Err(e) = run() {
        if e.chain().any(|e| {
            e.downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::BrokenPipe)
        }) {
            return;
        }
        eprintln!("twig: {e:#}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let cli = Cli::parse();
    if cli.clear_cache {
        println!(
            "Removed {} cache files",
            twig::adapters::loader::clear_cache()?
        );
        return Ok(());
    }
    if cli.fix || cli.print {
        return print::run(&cli);
    }
    let file = cli.file.as_deref().context("a file is required")?;
    let mut input_temp = None;
    let file = if file == std::path::Path::new("-") {
        anyhow::ensure!(cli.check, "stdin requires --print, --fix, or --check");
        let mut temp = tempfile::NamedTempFile::new()?;
        std::io::copy(&mut std::io::stdin().lock(), &mut temp)?;
        input_temp = Some(temp);
        input_temp.as_ref().unwrap().path()
    } else {
        file
    };
    let options = twig::adapters::loader::LoadOptions {
        no_cache: cli.no_cache || input_temp.is_some(),
        ..Default::default()
    };
    if cli.check {
        return check::run_with_options(file, cli.rebuild_db, options);
    }
    use std::io::IsTerminal;
    anyhow::ensure!(
        std::io::stdin().is_terminal() && std::io::stdout().is_terminal(),
        "the TUI requires a terminal; use --print or --check"
    );
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        previous(info);
    }));
    crossterm::terminal::enable_raw_mode().context("enabling raw mode")?;
    let _guard = TerminalGuard;
    crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;
    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let mut app = App::new(file, cli.rebuild_db);
    app.load_options = options;
    app.run(&mut terminal)
}
