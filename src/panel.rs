use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

const PANEL_DIR: &str = "~/.proto/panel";
const DEFAULT_PORT: u16 = 4321;

#[derive(Serialize, Debug, Clone)]
pub struct PanelPayload {
    pub title: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metrics: Vec<PanelMetric>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<PanelRow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<PanelHistory>,
}

impl PanelPayload {
    pub fn new(title: &str, kind: &str) -> Self {
        Self {
            title: title.to_string(),
            kind: kind.to_string(),
            updated: None,
            metrics: Vec::new(),
            rows: Vec::new(),
            history: None,
        }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PanelMetric {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

impl PanelMetric {
    pub fn new(label: &str, value: &str) -> Self {
        Self {
            label: label.to_string(),
            value: value.to_string(),
            unit: None,
            status: None,
        }
    }
    pub fn unit(mut self, u: &str) -> Self {
        self.unit = Some(u.to_string());
        self
    }
    pub fn status(mut self, s: &str) -> Self {
        self.status = Some(s.to_string());
        self
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PanelRow {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
    pub cells: Vec<(String, String)>,
}

impl PanelRow {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            desc: None,
            cells: Vec::new(),
        }
    }
    pub fn desc(mut self, d: &str) -> Self {
        self.desc = Some(d.to_string());
        self
    }
    pub fn cell(mut self, k: &str, v: &str) -> Self {
        self.cells.push((k.to_string(), v.to_string()));
        self
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct PanelHistory {
    pub label: String,
    pub points: Vec<(String, f64)>,
}

pub fn panel_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".proto").join("panel")
    } else {
        PathBuf::from(PANEL_DIR)
    }
}

fn project_dir() -> PathBuf {
    let dir = panel_dir();
    if dir.join("package.json").exists() {
        return dir;
    }
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() && p.join("package.json").exists() {
                return p;
            }
        }
    }
    dir
}

pub fn default_port() -> u16 {
    DEFAULT_PORT
}

pub fn running(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/", port);
    match ureq::get(&url).timeout(std::time::Duration::from_secs(2)).call() {
        Ok(_) => true,
        Err(ureq::Error::Status(_, _)) => true,
        Err(_) => false,
    }
}

pub fn panel_url(port: u16, kind: &str) -> String {
    format!("http://localhost:{}/?type={}", port, kind)
}

pub fn open(port: u16, kind: &str) {
    let url = panel_url(port, kind);
    for opener in ["xdg-open", "open", "wslview"] {
        if crate::utils::which(opener) {
            let _ = Command::new(opener).arg(&url).spawn();
            return;
        }
    }
    println!("\n  Panel: {}", url);
}

pub fn ingest(port: u16, payload: &PanelPayload) -> bool {
    let url = format!(
        "http://127.0.0.1:{}/api/ingest?type={}",
        port,
        payload.kind
    );
    match serde_json::to_string(payload) {
        Ok(body) => ureq::post(&url)
            .timeout(std::time::Duration::from_secs(3))
            .send_string(&body)
            .is_ok(),
        Err(_) => false,
    }
}

pub fn ensure_app() -> Result<(), String> {
    let dir = panel_dir();
    if project_dir().join("package.json").exists() {
        return Ok(());
    }
    if !crate::utils::which("bun") {
        return Err(
            "bun not found. Install it (curl -fsSL https://bun.sh/install | bash) so the \
             panel can be scaffolded."
                .to_string(),
        );
    }
    let parent = dir
        .parent()
        .ok_or_else(|| "Cannot determine panel parent dir".to_string())?;
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }

    println!("  Scaffolding the proto panel (Next.js + shadcn)...");
    let status = Command::new("bunx")
        .current_dir(&dir)
        .args([
            "--bun",
            "shadcn@latest",
            "init",
            "--template",
            "next",
            "--preset",
            "nova",
            "--yes",
            "--force",
        ])
        .status()
        .map_err(|e| format!("Failed to run bunx: {}", e))?;
    if !status.success() {
        return Err("bunx shadcn init did not complete. The panel app may be missing.".to_string());
    }

    write_panel_files()?;
    Ok(())
}

pub fn write_panel_files() -> Result<(), String> {
    let dir = project_dir();
    let app_dir = if dir.join("src").join("app").exists() {
        dir.join("src").join("app")
    } else {
        dir.join("app")
    };
    std::fs::create_dir_all(app_dir.join("api").join("ingest"))
        .map_err(|e| e.to_string())?;

    let page = app_dir.join("page.tsx");
    std::fs::write(&page, PANEL_PAGE_TSX).map_err(|e| e.to_string())?;
    let route = app_dir.join("api").join("ingest").join("route.ts");
    std::fs::write(&route, PANEL_INGEST_TS).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn start(port: u16) -> Result<(), String> {
    ensure_app()?;
    write_panel_files()?;
    if running(port) {
        return Ok(());
    }
    let dir = project_dir();
    let mut child = Command::new("bun")
        .current_dir(&dir)
        .args(["--bun", "run", "dev"])
        .env("PORT", port.to_string())
        .env("HOSTNAME", "127.0.0.1")
        .env("BROWSER", "none")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start panel server: {}", e))?;

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(90);
    while !running(port) {
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            return Err(format!(
                "Panel server did not start on port {}. Run `bun run dev` in {} manually.",
                port,
                dir.display()
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(400));
    }
    Ok(())
}

const PANEL_INGEST_TS: &str = r##"const store = new Map<string, { data: unknown; ts: number }>();

export async function POST(req: Request) {
  const url = new URL(req.url);
  const type = url.searchParams.get("type") ?? "default";
  const body = await req.text();
  let json: unknown = body;
  try {
    json = JSON.parse(body);
  } catch {}
  store.set(type, { data: json, ts: Date.now() });
  return Response.json({ ok: true });
}

export async function GET(req: Request) {
  const url = new URL(req.url);
  const type = url.searchParams.get("type") ?? "default";
  const hit = store.get(type);
  if (!hit) return Response.json({});
  return Response.json(hit.data);
}
"##;

const PANEL_PAGE_TSX: &str = r##""use client";
import { useEffect, useState } from "react";

interface Metric {
  label: string;
  value: string;
  unit?: string;
  status?: string;
}
interface Row {
  name: string;
  desc?: string;
  cells: [string, string][];
}
interface History {
  label: string;
  points: [string, number][];
}
interface Payload {
  title: string;
  kind: string;
  updated?: string;
  metrics: Metric[];
  rows: Row[];
  history?: History;
}

export default function Panel() {
  const [type, setType] = useState("status");
  const [data, setData] = useState<Payload | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const t = new URLSearchParams(window.location.search).get("type") ?? "status";
    setType(t);
    let alive = true;
    const load = async () => {
      try {
        const r = await fetch(`/api/ingest?type=${t}`, { cache: "no-store" });
        const j = (await r.json()) as Payload;
        if (alive && j && j.title) {
          setData(j);
          setConnected(true);
        }
      } catch {}
    };
    load();
    const id = setInterval(load, 2000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  return (
    <main className="p-6 font-sans">
      <header className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">
            {data ? data.title : "proto panel"}
          </h1>
          <p className="text-sm text-muted-foreground">
            {type}
            {data && data.updated ? ` · ${data.updated}` : ""}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {!data && <span className="text-sm text-muted-foreground">waiting for data…</span>}
          <span className="relative flex h-3 w-3">
            {connected ? (
              <>
                <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-green-400 opacity-75" />
                <span className="relative inline-flex h-3 w-3 rounded-full bg-green-500" />
              </>
            ) : (
              <span className="relative inline-flex h-3 w-3 rounded-full bg-amber-500" />
            )}
          </span>
        </div>
      </header>

      {data && data.metrics.length > 0 && (
        <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          {data.metrics.map((m, i) => (
            <div key={i} className="rounded-lg border bg-card p-4 shadow-sm">
              <p className="text-xs uppercase tracking-wide text-muted-foreground">{m.label}</p>
              <p className={`mt-1 text-2xl font-bold ${statusColor(m.status)}`}>
                {m.value}
                {m.unit && (
                  <span className="ml-1 text-sm font-normal text-muted-foreground">{m.unit}</span>
                )}
              </p>
            </div>
          ))}
        </div>
      )}

      {data && data.history && <Sparkline h={data.history} />}

      {data && data.rows.length > 0 && (
        <div className="mt-4 overflow-x-auto rounded-lg border bg-card shadow-sm">
          <table className="w-full text-sm">
            <tbody>
              {data.rows.map((r, i) => (
                <tr key={i} className="border-b last:border-0">
                  <td className="p-2.5 font-medium">
                    {r.name}
                    {r.desc && <div className="text-xs text-muted-foreground">{r.desc}</div>}
                  </td>
                  {r.cells.map(([k, v], j) => (
                    <td key={j} className="p-2.5">
                      <span className="text-muted-foreground">{k}:</span>{" "}
                      <span className="font-mono">{v}</span>
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </main>
  );
}

function statusColor(s?: string) {
  switch (s) {
    case "bad":
      return "text-red-500";
    case "warn":
      return "text-amber-500";
    case "ok":
      return "text-green-500";
    default:
      return "text-foreground";
  }
}

function Sparkline({ h }: { h: History }) {
  const pts = h.points;
  if (pts.length < 2) return null;
  const W = 600;
  const H = 96;
  const PAD = 8;
  const max = Math.max(...pts.map((p) => p[1]), 1);
  const min = Math.min(...pts.map((p) => p[1]), 0);
  const range = max - min || 1;
  const stepX = (W - PAD * 2) / (pts.length - 1);
  const path = pts
    .map((p, i) => {
      const x = PAD + i * stepX;
      const y = H - PAD - ((p[1] - min) / range) * (H - PAD * 2);
      return `${i === 0 ? "M" : "L"}${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  const last = pts[pts.length - 1];
  return (
    <div className="mt-4 rounded-lg border bg-card p-4 shadow-sm">
      <p className="text-xs uppercase tracking-wide text-muted-foreground">{h.label}</p>
      <svg viewBox={`0 0 ${W} ${H}`} className="mt-2 w-full">
        <path d={path} fill="none" stroke="#22c55e" strokeWidth="2" strokeLinejoin="round" />
      </svg>
      <p className="mt-1 text-xs text-muted-foreground">
        now: {last[1]} · min: {min.toFixed(1)} · max: {max.toFixed(1)}
      </p>
    </div>
  );
}
"##;
