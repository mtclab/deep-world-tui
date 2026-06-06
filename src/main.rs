use clap::Parser;

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let seed = cli.seed.unwrap_or_else(|| {
        // Use a simple entropy source for default seed
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    println!("Deep World TUI — seed: {seed}");
    println!("(TUI scaffold: issue #6)");
    Ok(())
}
