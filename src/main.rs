mod app;
mod git;
mod ui;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--version" || a == "-V") {
        println!("githop {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if std::env::args().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    // Verify we're in a git repo
    std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .ok_or_else(|| {
            anyhow::anyhow!("Not a git repository. Run githop from inside a git project directory.")
        })?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn print_help() {
    println!("githop {} - Interactive git branch switcher", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE: githop [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("  -h, --help       Print help");
    println!("  -V, --version    Print version");
    println!();
    println!("KEYBINDINGS:");
    println!("  j/k, ↑/↓         Navigate branches");
    println!("  Enter            Switch to selected branch");
    println!("  y                Copy branch name to clipboard");
    println!("  d                Delete branch");
    println!("  n                Create new branch");
    println!("  r                Rename branch");
    println!("  /                Filter branches");
    println!("  Esc              Cancel / clear filter");
    println!("  q                Quit");
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = app::App::new()?;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
