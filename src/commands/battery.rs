use crate::panel::{self, PanelHistory, PanelMetric, PanelPayload};
use crate::style;
use owo_colors::OwoColorize;
use std::path::PathBuf;

pub fn run(serve: bool, interval: u64, port: u16) {
    let battery = match Battery::detect() {
        Some(b) => b,
        None => {
            eprintln!("{} No battery found on this system.", style::error(""));
            return;
        }
    };

    if serve {
        serve_mode(battery, interval, port);
        return;
    }

    let info = battery.snapshot();
    print_report(&info);
}

fn serve_mode(battery: Battery, interval: u64, port: u16) {
    let kind = "battery";
    if let Err(e) = panel::start(port) {
        eprintln!("{} {}", style::error(""), e);
        eprintln!("  {} Falling back to terminal-only output.", style::warn(""));
    } else {
        println!(
            "  {}",
            style::label_value("Panel", &panel::panel_url(port, kind)),
        );
        panel::open(port, kind);
    }

    let mut history: Vec<(String, f64)> = Vec::new();
    println!();
    println!(
        "  {} Monitoring battery every {}s. Ctrl+C to stop.",
        "◉".style(style::Theme::ACCENT),
        interval
    );

    loop {
        let info = battery.snapshot();
        print_snapshot_line(&info);
        history.push((
            chrono_like_timestamp(),
            info.capacity as f64,
        ));
        if history.len() > 300 {
            history.remove(0);
        }

        let mut p = PanelPayload::new("Battery Health", kind);
        p.updated = Some(format!("{} · {} ({})", chrono_like_timestamp(), info.status, info.model));
        p.metrics = vec![
            PanelMetric::new("Health", &format!("{:.1}", info.health_percent))
                .unit("%")
                .status(health_status(info.health_percent)),
            PanelMetric::new("Charge", &info.capacity.to_string())
                .unit("%")
                .status(if info.capacity > 20 { "ok" } else { "warn" }),
            PanelMetric::new("Status", &info.status),
            PanelMetric::new("Wattage", &format!("{:.1}", info.watts))
                .unit("W"),
        ];
        p.history = Some(PanelHistory {
            label: "Charge level over time".to_string(),
            points: history.clone(),
        });
        if let Some(cycles) = info.cycles {
            p.rows.push(
                crate::panel::PanelRow::new("Cycle count")
                    .cell("cycles", &cycles.to_string()),
            );
        }
        let _ = panel::ingest(port, &p);

        std::thread::sleep(std::time::Duration::from_secs(interval));
    }
}

fn print_snapshot_line(info: &BatteryInfo) {
    println!(
        "  {}  {}%  {}  {:.1} W  health {:.1}%  {}",
        info.status.style(style::Theme::ACCENT).bold(),
        info.capacity.to_string().style(style::Theme::VALUE),
        format!("{}/{}", info.full_short(), info.design_short()).dimmed(),
        info.watts,
        info.health_percent,
        format!("{} cycles", info.cycles.unwrap_or(0)).dimmed(),
    );
}

fn print_report(info: &BatteryInfo) {
    println!("{}", style::header("Battery Health"));
    println!("{}", style::divider());

    println!("  {}", style::label_value("Model", &info.model));
    println!("  {}", style::label_value("Status", &info.status));
    println!(
        "  {}",
        style::label_value(
            "Health",
            &format!("{:.1}%", info.health_percent)
        )
    );
    println!("  {}", style::label_value("Charge", &format!("{}%", info.capacity)));
    println!(
        "  {}",
        style::label_value("Capacity (current)", &info.current_capacity_str().to_string())
    );
    println!(
        "  {}",
        style::label_value("Capacity (design)", &info.design_capacity_str().to_string())
    );
    println!("  {}", style::label_value("Cycles", &info.cycles.map(|c| c.to_string()).unwrap_or_else(|| "n/a".into())));
    println!(
        "  {}",
        style::label_value(
            "Power",
            &format!("{:.2} W {}", info.watts, info.status.to_lowercase())
        )
    );

    println!();
    if info.health_percent < 80.0 {
        println!("  {} Battery health is degraded — consider a replacement.", style::warn(""));
    } else if info.health_percent < 60.0 {
        println!("  {} Battery health is critical.", style::error(""));
    } else {
        println!("  {} Battery health looks good.", style::success(""));
    }
}

fn health_status(health: f64) -> &'static str {
    if health < 60.0 {
        "bad"
    } else if health < 80.0 {
        "warn"
    } else {
        "ok"
    }
}

fn chrono_like_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = now.as_secs();
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

pub struct Battery {
    path: PathBuf,
    name: String,
}

struct BatteryInfo {
    model: String,
    status: String,
    capacity: u32,
    full: Option<u64>,
    full_design: Option<u64>,
    cycles: Option<u64>,
    watts: f64,
    health_percent: f64,
}

impl Battery {
    fn detect() -> Option<Battery> {
        let dir = PathBuf::from("/sys/class/power_supply");
        let entries = std::fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                return Some(Battery {
                    path: entry.path(),
                    name,
                });
            }
        }
        None
    }

    fn read_int(&self, field: &str) -> Option<u64> {
        let content = std::fs::read_to_string(self.path.join(field)).ok()?;
        content.trim().parse().ok()
    }

    fn read_str(&self, field: &str) -> Option<String> {
        std::fs::read_to_string(self.path.join(field))
            .ok()
            .map(|s| s.trim().to_string())
    }

    fn snapshot(&self) -> BatteryInfo {
        let model = self.read_str("model_name").unwrap_or_else(|| self.name.clone());
        let status = self.read_str("status").unwrap_or_else(|| "Unknown".into());
        let capacity = self.read_int("capacity").unwrap_or(0) as u32;

        let full = self
            .read_int("energy_full")
            .or_else(|| self.read_int("charge_full"));
        let full_design = self
            .read_int("energy_full_design")
            .or_else(|| self.read_int("charge_full_design"));
        let cycles = self.read_int("cycle_count");

        let watts = if let Some(p) = self.read_int("power_now") {
            p as f64 / 1_000_000.0
        } else if let (Some(v), Some(c)) = (self.read_int("voltage_now"), self.read_int("current_now")) {
            v as f64 * c as f64 / 1_000_000_000_000.0
        } else {
            0.0
        };

        let health_percent = match (full, full_design) {
            (Some(f), Some(fd)) if fd > 0 => f as f64 / fd as f64 * 100.0,
            _ => 100.0,
        };

        BatteryInfo {
            model,
            status,
            capacity,
            full,
            full_design,
            cycles,
            watts,
            health_percent,
        }
    }
}

impl BatteryInfo {
    fn full_short(&self) -> String {
        format_wh(self.full)
    }
    fn design_short(&self) -> String {
        format_wh(self.full_design)
    }
    fn current_capacity_str(&self) -> String {
        format_wh(self.full)
    }
    fn design_capacity_str(&self) -> String {
        format_wh(self.full_design)
    }
}

fn format_wh(val: Option<u64>) -> String {
    match val {
        Some(v) => {
            if v > 1_000_000 {
                format!("{:.1} Wh", v as f64 / 1_000_000.0)
            } else {
                format!("{:.1} Wh", v as f64 / 1_000.0)
            }
        }
        None => "n/a".to_string(),
    }
}
