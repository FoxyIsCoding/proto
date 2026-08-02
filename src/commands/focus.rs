use crate::style;
use owo_colors::OwoColorize;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

pub fn run(work: u64, short_break: u64, long_break: u64, cycles: u64) {
    println!("{}", style::header("Focus Timer"));
    println!(
        "  {} {} min work / {} min break / {} min long break / {} cycles",
        style::muted(""),
        work.style(style::Theme::VALUE),
        short_break.style(style::Theme::VALUE),
        long_break.style(style::Theme::VALUE),
        cycles.style(style::Theme::VALUE),
    );
    println!("{}", style::divider());
    println!("  Press Ctrl+C to stop\n");

    for cycle in 1..=cycles {
        run_phase("WORK", work * 60, style::Theme::LABEL);
        if cycle == cycles {
            break;
        }
        if cycle % 4 == 0 {
            run_phase("LONG BREAK", long_break * 60, style::Theme::SUCCESS);
        } else {
            run_phase("BREAK", short_break * 60, style::Theme::SUCCESS);
        }
    }

    println!("\n  {} All cycles complete!", style::success(""));
}

fn run_phase(label: &str, total_secs: u64, color: owo_colors::Style) {
    let start = Instant::now();
    let end = start + Duration::from_secs(total_secs);

    print!(
        "  {} {} | remaining: ",
        label.style(color),
        "█".repeat(20).style(style::Theme::MUTED),
    );
    io::stdout().flush().unwrap();

    loop {
        let now = Instant::now();
        if now >= end {
            break;
        }
        let remaining = end.duration_since(now).as_secs();
        let elapsed = now.duration_since(start).as_secs();
        let mins = remaining / 60;
        let secs = remaining % 60;
        let pct = elapsed as f64 / total_secs as f64;
        let filled = (pct * 20.0) as usize;
        let bar: String = (0..20)
            .map(|i| if i < filled { '█' } else { '░' })
            .collect();

        print!(
            "\r  {} {} | {:02}:{:02}",
            label.style(color),
            bar.style(if filled > 15 {
                style::Theme::ERROR
            } else {
                style::Theme::VALUE
            }),
            mins,
            secs,
        );
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(250));
    }
    println!("\r  {} {} | done!{}\n", label.style(color), "████████████████████".style(style::Theme::SUCCESS), " ".repeat(10));
    print!("\x07"); // bell
    io::stdout().flush().unwrap();
}
