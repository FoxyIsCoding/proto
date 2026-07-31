use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum ConvertAction {
    #[command(about = "Convert between units (time, length, weight, digital, temp, speed, volume, currency)")]
    Run {
        #[arg(required = true, value_name = "VALUE", help = "Value with unit (e.g. 6m, 10.5km, 500ms, 2GB)")]
        input: String,
        #[arg(value_name = "TO", help = "Target unit (e.g. cm, min, MB, F)")]
        to: Option<String>,
    },
}

struct Unit {
    short: &'static str,
    long: &'static str,
    category: &'static str,
    factor: f64,
    offset: f64,
}

impl Unit {
    fn new_base(short: &'static str, long: &'static str, category: &'static str) -> Self {
        Self { short, long, category, factor: 1.0, offset: 0.0 }
    }
    fn new(short: &'static str, long: &'static str, category: &'static str, factor: f64) -> Self {
        Self { short, long, category, factor, offset: 0.0 }
    }
}

fn all_units() -> Vec<Unit> {
    vec![
        Unit::new("ns", "nanosecond", "time", 1e-9), Unit::new("\u{b5}s", "microsecond", "time", 1e-6),
        Unit::new("ms", "millisecond", "time", 0.001), Unit::new("s", "second", "time", 1.0).to_base(),
        Unit::new("min", "minute", "time", 60.0), Unit::new("h", "hour", "time", 3600.0),
        Unit::new("d", "day", "time", 86400.0), Unit::new("w", "week", "time", 604800.0),
        Unit::new("mo", "month", "time", 2.628e6), Unit::new("y", "year", "time", 3.154e7),

        Unit::new("nm", "nanometer", "length", 1e-9), Unit::new("μm", "micrometer", "length", 1e-6),
        Unit::new("mm", "millimeter", "length", 0.001), Unit::new("cm", "centimeter", "length", 0.01),
        Unit::new("m", "meter", "length", 1.0).to_base(), Unit::new("km", "kilometer", "length", 1000.0),
        Unit::new("in", "inch", "length", 0.0254), Unit::new("ft", "foot", "length", 0.3048),
        Unit::new("yd", "yard", "length", 0.9144), Unit::new("mi", "mile", "length", 1609.344),

        Unit::new("mg", "milligram", "weight", 0.000001), Unit::new("g", "gram", "weight", 0.001),
        Unit::new("kg", "kilogram", "weight", 1.0).to_base(), Unit::new("t", "tonne", "weight", 1000.0),
        Unit::new("oz", "ounce", "weight", 0.0283495), Unit::new("lb", "pound", "weight", 0.453592),

        Unit::new("b", "bit", "digital", 1.0).to_base(), Unit::new("B", "byte", "digital", 8.0),
        Unit::new("KB", "kilobyte", "digital", 8192.0), Unit::new("MB", "megabyte", "digital", 8.389e6),
        Unit::new("GB", "gigabyte", "digital", 8.59e9), Unit::new("TB", "terabyte", "digital", 8.796e12),
        Unit::new("PB", "petabyte", "digital", 9.007e15),

        Unit { short: "°C", long: "celsius", category: "temperature", factor: 1.0, offset: 0.0 },
        Unit { short: "°F", long: "fahrenheit", category: "temperature", factor: 1.0, offset: 0.0 },
        Unit { short: "K", long: "kelvin", category: "temperature", factor: 1.0, offset: 0.0 },

        Unit::new("m/s", "m/s", "speed", 1.0).to_base(), Unit::new("km/h", "km/h", "speed", 0.277778),
        Unit::new("mph", "mph", "speed", 0.44704), Unit::new("kn", "knot", "speed", 0.514444),

        Unit::new("mL", "milliliter", "volume", 0.001), Unit::new("L", "liter", "volume", 1.0).to_base(),
        Unit::new("gal", "gallon", "volume", 3.78541), Unit::new("qt", "quart", "volume", 0.946353),
        Unit::new("pt", "pint", "volume", 0.473176), Unit::new("cup", "cup", "volume", 0.236588),
    ]
}

impl Unit {
    fn to_base(self) -> Self { Self { factor: 1.0, ..self } }
}

pub fn run(action: &ConvertAction) {
    match action {
        ConvertAction::Run { input, to } => convert(input, to.as_deref()),
    }
}

fn convert(input: &str, to: Option<&str>) {
    let (value, mut unit_str) = parse_input(input);

    if unit_str.eq_ignore_ascii_case("c") { unit_str = "°C".into(); }
    if unit_str.eq_ignore_ascii_case("f") { unit_str = "°F".into(); }

    let units = all_units();

    let matches: Vec<&Unit> = units.iter().filter(|u|
        u.short == unit_str || u.long == unit_str
    ).collect();

    if matches.is_empty() {
        eprintln!("{} Unknown unit: '{}'", style::error(""), unit_str);
        show_all_units();
        return;
    }

    if matches.len() > 1 {
        let categories: Vec<String> = matches.iter().map(|u| u.category.to_string()).collect();
        println!("{} Ambiguous unit '{}'. It could be:", style::warn(""), unit_str);
        for m in &matches {
            println!("  {} {} ({})", "▸".style(style::Theme::ACCENT), m.long, m.category);
        }
        println!("\n{} Try specifying more precisely, e.g.:", style::Theme::MUTED.style(" ".to_string()));
        for m in &matches {
            println!("    proto convert {}{} {}", value, unit_str, m.short);
        }
        return;
    }

    let from = matches[0];

    let to_final = to.map(|t| {
        if t.eq_ignore_ascii_case("c") { "°C" } else if t.eq_ignore_ascii_case("f") { "°F" } else { t }
    });
    let to = to_final.as_deref();

    if to.is_none() {
        println!("\n{} {} {} ({})", "◆".style(style::Theme::ACCENT), value, from.long.style(style::Theme::ACCENT), from.category);
        println!("{}", style::divider());
        for u in &units {
            if u.category == from.category && u.short != from.short {
                let converted = convert_value(value, from, u);
                println!("{}", style::label_value(u.long, &format_value(converted, u)));
            }
        }
        println!("{}", style::divider());
        return;
    }

    let to_str = to.unwrap();
    let to_units: Vec<&Unit> = units.iter().filter(|u|
        (u.short == to_str || u.long == to_str) && u.category == from.category
    ).collect();

    if to_units.is_empty() {
        eprintln!("{} Unknown or incompatible target: '{}'", style::error(""), to_str);
        eprintln!("{} Category: {}", style::warn(""), from.category);
        eprintln!("{} Available:", style::warn(""));
        for u in &units { if u.category == from.category { println!("  {} ({})", u.short, u.long); } }
        return;
    }

    let to_unit = to_units[0];
    let result = convert_value(value, from, to_unit);
    println!("\n{} {} {} {} {}",
        "◆".style(style::Theme::ACCENT),
        format!("{} {}", value, from.short).style(style::Theme::ACCENT),
        "=".dimmed(),
        format_value(result, to_unit).style(style::Theme::ACCENT).bold(),
        to_unit.short.style(style::Theme::ACCENT),
    );
}

fn parse_input(s: &str) -> (f64, String) {
    let s = s.trim();
    if let Some(pos) = s.find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != '°') {
        let val_str = if s.starts_with('°') { &s[..2] } else { &s[..pos] };
        if let Ok(val) = val_str.parse::<f64>() {
            let unit = s[pos..].trim().to_string();
            if !unit.is_empty() { return (val, unit); }
        }
    }
    if let Ok(val) = s.parse::<f64>() {
        (val, String::new())
    } else {
        eprintln!("{} Cannot parse: '{}'", style::error(""), s);
        std::process::exit(1);
    }
}

fn convert_value(value: f64, from: &Unit, to: &Unit) -> f64 {
    if from.category == "temperature" {
        let celsius = match from.short {
            "°C" => value,
            "°F" => (value - 32.0) * 5.0 / 9.0,
            "K" => value - 273.15,
            _ => value,
        };
        return match to.short {
            "°C" => celsius,
            "°F" => celsius * 9.0 / 5.0 + 32.0,
            "K" => celsius + 273.15,
            _ => celsius,
        };
    }
    let base = value * from.factor;
    base / to.factor
}

fn format_value(v: f64, unit: &Unit) -> String {
    if v.abs() < 0.01 || v.abs() > 1_000_000_000.0 { format!("{:.6e}", v) }
    else if v.abs() < 1.0 { format!("{:.6}", v).trim_end_matches('0').trim_end_matches('.').to_string() }
    else if v.fract() == 0.0 { format!("{:.0}", v) }
    else { format!("{:.4}", v).trim_end_matches('0').trim_end_matches('.').to_string() }
}

fn show_all_units() {
    println!("\n{}", "Available units:".style(style::Theme::HEADER));
    let units = all_units();
    let cats = ["time", "length", "weight", "digital", "temperature", "speed", "volume"];
    for cat in cats {
        print!("  {}  ", cat.dimmed());
        let list: Vec<String> = units.iter().filter(|u| u.category == cat).map(|u| u.short.to_string()).collect();
        println!("{}", list.join(", ").dimmed());
    }
}
