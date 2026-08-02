use crate::style;
use owo_colors::OwoColorize;

pub fn run() {
    println!("{}", style::header("Color Palette"));
    println!("{}", style::divider());

    // Standard 8 ANSI colors
    println!("  {} Standard:", style::muted(""));
    let std = [
        ("BLACK  ", "\x1b[40m  \x1b[0m"),
        ("RED    ", "\x1b[41m  \x1b[0m"),
        ("GREEN  ", "\x1b[42m  \x1b[0m"),
        ("YELLOW ", "\x1b[43m  \x1b[0m"),
        ("BLUE   ", "\x1b[44m  \x1b[0m"),
        ("MAGENTA", "\x1b[45m  \x1b[0m"),
        ("CYAN   ", "\x1b[46m  \x1b[0m"),
        ("WHITE  ", "\x1b[47m  \x1b[0m"),
    ];
    for (name, swatch) in &std {
        print!("    {} {}", swatch, name);
    }
    println!("\n");

    // Bright 8 ANSI colors
    println!("  {} Bright:", style::muted(""));
    let bright = [
        ("BLACK  ", "\x1b[100m  \x1b[0m"),
        ("RED    ", "\x1b[101m  \x1b[0m"),
        ("GREEN  ", "\x1b[102m  \x1b[0m"),
        ("YELLOW ", "\x1b[103m  \x1b[0m"),
        ("BLUE   ", "\x1b[104m  \x1b[0m"),
        ("MAGENTA", "\x1b[105m  \x1b[0m"),
        ("CYAN   ", "\x1b[106m  \x1b[0m"),
        ("WHITE  ", "\x1b[107m  \x1b[0m"),
    ];
    for (name, swatch) in &bright {
        print!("    {} {}", swatch, name);
    }
    println!("\n");

    // 256-color cube (6x6x6 = 216 colors) — show a compact grid
    println!("  {} 256-color cube (6x6x6):", style::muted(""));
    for g in 0..6 {
        print!("  ");
        for r in 0..6 {
            for b in 0..6 {
                let code = 16 + 36 * r + 6 * g + b;
                print!("\x1b[48;5;{}m  \x1b[0m", code);
            }
            print!(" ");
        }
        println!();
    }
    println!();

    // Grayscale ramp
    println!("  {} Grayscale:", style::muted(""));
    print!("  ");
    for i in 232..=255 {
        print!("\x1b[48;5;{}m \x1b[0m", i);
    }
    println!("\n");

    // Theme sample
    println!(
        "  {} Proto theme sample:",
        style::muted("")
    );
    println!(
        "    {}  HEADER / {}  LABEL / {}  VALUE / {}  SUCCESS / {}  WARN / {}  ERROR",
        "HEADER".style(style::Theme::HEADER),
        "LABEL".style(style::Theme::LABEL),
        "VALUE".style(style::Theme::VALUE),
        "SUCCESS".style(style::Theme::SUCCESS),
        "WARN".style(style::Theme::WARN),
        "ERROR".style(style::Theme::ERROR),
    );
    println!();
}
