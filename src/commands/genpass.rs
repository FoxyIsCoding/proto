use crate::style;
use owo_colors::OwoColorize;

pub fn run(length: usize, no_symbols: bool, no_numbers: bool, count: usize) {
    let upper = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let lower = "abcdefghijklmnopqrstuvwxyz";
    let digits = "0123456789";
    let symbols = "!@#$%^&*-_=+[]{};:,.<>?";

    let mut charset = String::from(upper) + lower;
    if !no_numbers {
        charset.push_str(digits);
    }
    if !no_symbols {
        charset.push_str(symbols);
    }

    let chars: Vec<u8> = charset.bytes().collect();

    fn rand_byte() -> u8 {
        let mut buf = [0u8; 1];
        if std::fs::File::open("/dev/urandom")
            .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut buf).map(|_| buf))
            .is_ok()
        {
            return buf[0];
        }
        // fallback
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        ((t ^ (t >> 32)) & 0xff) as u8
    }

    println!("{}", style::header("Generate Password"));
    println!("{}", style::divider());

    for _ in 0..count {
        let password: String = (0..length)
            .map(|_| {
                let b = rand_byte();
                chars[(b as usize) % chars.len()] as char
            })
            .collect();
        println!("  {}", password.style(style::Theme::VALUE));
    }

    println!(
        "\n  {} {} chars, charset: {}",
        style::muted(""),
        length.style(style::Theme::VALUE),
        charset.len().style(style::Theme::VALUE)
    );
    println!(
        "  {} {} password(s) generated",
        style::muted(""),
        count.style(style::Theme::VALUE)
    );
    println!();
}
