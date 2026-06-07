use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use deep_world_tui::charts::load::load_charts;
use deep_world_tui::ui::App;

#[derive(Parser)]
#[command(
    name = "deep-world-tui",
    about = "A procedural life-RPG in the Deep World"
)]
struct Cli {
    /// World seed (deterministic generation)
    #[arg(long)]
    seed: Option<u64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let charts = load_charts("data/charts.ron")?;
    let mut app = App::new(seed, charts);

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    while app.running {
        terminal.draw(|f| deep_world_tui::ui::render::draw(f, app))?;

        let timeout = Duration::from_millis(app.tick_interval);
        if let Some(event) = deep_world_tui::ui::event::poll(timeout) {
            app.handle_event(event);
        }
    }
    Ok(())
}
