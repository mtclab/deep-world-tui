use std::time::Duration;
use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

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

    /// Path to charts.ron file (overrides embedded charts)
    #[arg(long)]
    charts: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });

    let charts = if let Some(ref path) = cli.charts {
        deep_world_tui::charts::load::load_charts_from_path(std::path::Path::new(path))?
    } else {
        deep_world_tui::charts::load::load_charts()?
    };
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
        let render_start = Instant::now();
        app.pre_draw();
        terminal.draw(|f| deep_world_tui::ui::render::draw(f, app))?;
        let render_us = render_start.elapsed().as_micros() as u64;
        app.perf_last_render_us = render_us;
        if render_us > 33_000 {
            app.perf_slow_frames = app.perf_slow_frames.saturating_add(1);
        }

        let timeout = Duration::from_millis(app.tick_interval);
        if let Some(event) = deep_world_tui::ui::event::poll(timeout) {
            app.handle_event(event);
        }
    }
    Ok(())
}
