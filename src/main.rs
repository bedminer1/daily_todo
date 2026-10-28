mod history;

use std::env;

// Edit these tasks whenever you choose what to work on.
const NOW: &str = "finish 4243 take home";
const NEXT: [&str; 2] = ["finish 2100 assg 3", "get defect map working"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match env::args().nth(1).as_deref() {
        Some("--done") => history::done(NOW, &NEXT),
        Some("--undo") => history::undo(NOW, &NEXT),
        Some("--skip") => history::skip(NOW, &NEXT),
        Some(arg) => {
            Err(format!("Unknown option: {arg}. Use --done, --undo, --skip, or no option.").into())
        }
        None => {
            let width = terminal_size::terminal_size()
                .map(|(terminal_size::Width(columns), _)| usize::from(columns))
                .unwrap_or(36);
            print_task("NOW", NOW, width);
            println!();
            if let Some(next) = NEXT.first() {
                print_task("NEXT", next, width);
            }

            Ok(())
        }
    }
}

fn print_task(label: &str, task: &str, width: usize) {
    println!("{label}:");
    let mut line = String::new();
    for word in task.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            println!("{line}");
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    println!("{line}");
}
