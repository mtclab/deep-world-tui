use std::env;
use std::process::Command;

fn run(cmd: &mut Command, label: &str) -> bool {
    eprintln!("==> {}", label);
    let status = cmd.status().expect("failed to execute command");
    if !status.success() {
        eprintln!("==> FAILED: {}", label);
        return false;
    }
    true
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("check");

    let ok = match subcmd {
        "check" => {
            let mut ok = true;
            ok = ok
                && run(
                    Command::new("cargo").args(["fmt", "--check"]),
                    "cargo fmt --check",
                );
            ok = ok
                && run(
                    Command::new("cargo").args(["clippy", "--all-targets", "--", "-D", "warnings"]),
                    "cargo clippy --all-targets -- -D warnings",
                );
            ok = ok && run(Command::new("cargo").args(["build"]), "cargo build");
            ok = ok && run(Command::new("cargo").args(["test"]), "cargo test");
            ok
        }
        "lint" => {
            let mut ok = true;
            ok = ok
                && run(
                    Command::new("cargo").args(["fmt", "--check"]),
                    "cargo fmt --check",
                );
            ok = ok
                && run(
                    Command::new("cargo").args(["clippy", "--all-targets", "--", "-D", "warnings"]),
                    "cargo clippy --all-targets -- -D warnings",
                );
            ok
        }
        "format" | "fmt" => run(Command::new("cargo").args(["fmt"]), "cargo fmt"),
        "test" => run(Command::new("cargo").args(["test"]), "cargo test"),
        other => {
            eprintln!("unknown subcommand: {}", other);
            eprintln!("usage: cargo xtask <check|lint|format|test>");
            std::process::exit(1);
        }
    };

    if !ok {
        std::process::exit(1);
    }
}
