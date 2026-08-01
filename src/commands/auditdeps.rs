use crate::style;
use owo_colors::OwoColorize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const OSV_BATCH: &str = "https://api.osv.dev/v1/querybatch";
const OSV_VULN: &str = "https://api.osv.dev/v1/vulns";
const AST_ISSUES: &str = "https://security.archlinux.org/issues/all.json";
const OSV_CHUNK: usize = 150;

#[derive(Debug, Clone)]
struct PackageQuery {
    ecosystem: &'static str,
    name: String,
    version: String,
    source: String,
}

#[derive(Debug, Clone)]
struct Vuln {
    id: String,
    alias: Option<String>,
    severity: Option<String>,
    summary: Option<String>,
    fixed: Option<String>,
}

#[derive(Debug, Clone)]
struct Finding {
    source: String,
    package: String,
    version: String,
    vulns: Vec<Vuln>,
}

struct Checked {
    source: String,
    packages: usize,
}

pub fn run(dir: &str) {
    println!("{}", style::header("Dependency Audit"));
    println!("{}", style::divider());
    println!("  {} Scanning {} — lockfiles & system packages\n", style::muted(""), dir);

    let agent = new_agent();
    let mut findings: Vec<Finding> = Vec::new();
    let mut checked: Vec<Checked> = Vec::new();

    let dir_path = PathBuf::from(dir);
    let files = find_lockfiles(&dir_path);
    if files.is_empty() {
        println!(
            "  {} No supported lockfiles found in '{}'.",
            style::muted(""),
            dir
        );
    }

    let mut queries: Vec<PackageQuery> = Vec::new();
    for f in &files {
        let Some(ec) = ecosystem_for(f) else { continue };
        let list = parse_lockfile(f);
        if list.is_empty() {
            continue;
        }
        let label = rel_label(&dir_path, f);
        for (name, version) in list {
            queries.push(PackageQuery {
                ecosystem: ec,
                name,
                version,
                source: label.clone(),
            });
        }
    }

    if !queries.is_empty() {
        let spin = style::Spinner::new(&format!(
            "Querying {} OSV databases...",
            queries.len()
        ));
        let hits = osv_batch(&agent, &queries);
        let details = vuln_details(&agent, &hits);
        spin.done("OSV check complete");

        for q in &queries {
            let key = query_key(q);
            let Some(ids) = hits.get(&key) else { continue };
            if ids.is_empty() {
                continue;
            }
            let vulns: Vec<Vuln> = dedup_vulns(
                ids.iter()
                    .filter_map(|id| details.get(id).cloned())
                    .collect(),
            );
            if vulns.is_empty() {
                continue;
            }
            findings.push(Finding {
                source: q.source.clone(),
                package: q.name.clone(),
                version: q.version.clone(),
                vulns,
            });
        }

        let mut per_source: BTreeMap<String, usize> = BTreeMap::new();
        for q in &queries {
            *per_source.entry(q.source.clone()).or_insert(0) += 1;
        }
        for (source, n) in per_source {
            checked.push(Checked { source, packages: n });
        }
    }

    if crate::utils::which("pacman") {
        let spin = style::Spinner::new("Checking Arch package advisories...");
        let (sys, sys_count) = scan_system(&agent);
        spin.done("Arch advisory check complete");
        checked.push(Checked {
            source: "system (pacman)".to_string(),
            packages: sys_count,
        });
        findings.extend(sys);
    } else {
        println!(
            "  {} pacman not found — skipping system package audit.",
            style::muted("")
        );
    }

    println!("{}", style::divider());
    print_findings(&findings);
    print_summary(&findings, &checked);
}

fn new_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(20))
        .build()
}

fn query_key(q: &PackageQuery) -> String {
    format!("{}|{}|{}", q.ecosystem, q.name, q.version)
}

fn find_lockfiles(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack: Vec<(PathBuf, usize)> = vec![(dir.to_path_buf(), 0)];
    let max_depth = 6;
    while let Some((d, depth)) = stack.pop() {
        if depth > max_depth {
            continue;
        }
        let rd = match std::fs::read_dir(&d) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let p = e.path();
            if p.is_dir() {
                if matches!(
                    name.as_str(),
                    "node_modules" | "target" | ".git" | "vendor" | "dist" | "build" | ".cache"
                ) {
                    continue;
                }
                stack.push((p, depth + 1));
            } else if matches!(
                name.as_str(),
                "package-lock.json"
                    | "yarn.lock"
                    | "Cargo.lock"
                    | "go.sum"
                    | "requirements.txt"
                    | "Pipfile.lock"
                    | "poetry.lock"
                    | "Gemfile.lock"
                    | "composer.lock"
            ) {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn ecosystem_for(path: &Path) -> Option<&'static str> {
    match path.file_name()?.to_str()? {
        "package-lock.json" | "yarn.lock" => Some("npm"),
        "Cargo.lock" => Some("crates.io"),
        "go.sum" => Some("Go"),
        "requirements.txt" | "Pipfile.lock" | "poetry.lock" => Some("PyPI"),
        "Gemfile.lock" => Some("RubyGems"),
        "composer.lock" => Some("Packagist"),
        _ => None,
    }
}

fn rel_label(dir: &Path, path: &Path) -> String {
    path.strip_prefix(dir)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn parse_lockfile(path: &Path) -> Vec<(String, String)> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    match name {
        "package-lock.json" => {
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            if let Some(pkgs) = v.get("packages").and_then(|p| p.as_object()) {
                for (key, val) in pkgs {
                    let Some(version) = val.get("version").and_then(|x| x.as_str()) else {
                        continue;
                    };
                    if version == "0.0.0" || version.is_empty() {
                        continue;
                    }
                    let pkg = key
                        .split("node_modules/")
                        .last()
                        .unwrap_or(key)
                        .to_string();
                    if !pkg.is_empty() {
                        out.push((pkg, version.to_string()));
                    }
                }
            } else if let Some(deps) = v.get("dependencies").and_then(|d| d.as_object()) {
                for (name, val) in deps {
                    if let Some(version) = val.get("version").and_then(|x| x.as_str()) {
                        out.push((name.clone(), version.to_string()));
                    }
                }
            }
        }
        "yarn.lock" => parse_yarn(&content, &mut out),
        "Cargo.lock" | "poetry.lock" => parse_toml_lock(&content, &mut out),
        "go.sum" => {
            for line in content.lines() {
                if line.contains("/go.mod") {
                    continue;
                }
                let mut it = line.split_whitespace();
                let (Some(module), Some(version)) = (it.next(), it.next()) else {
                    continue;
                };
                out.push((module.to_string(), version.to_string()));
            }
        }
        "requirements.txt" => {
            for line in content.lines() {
                let line = line.split('#').next().unwrap_or("").trim();
                if line.is_empty() {
                    continue;
                }
                let mut it = line.splitn(2, "==");
                if let (Some(pkg), Some(version)) = (it.next(), it.next()) {
                    let version = version.trim();
                    if version.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') {
                        out.push((pkg.trim().to_string(), version.to_string()));
                    }
                }
            }
        }
        "Pipfile.lock" => {
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            for section in ["default", "develop"] {
                if let Some(obj) = v.get(section).and_then(|o| o.as_object()) {
                    for (name, val) in obj {
                        if let Some(version) = val.get("version").and_then(|x| x.as_str()) {
                            let v = version
                                .trim_start_matches("==")
                                .trim_start_matches('=')
                                .trim();
                            if !v.is_empty() {
                                out.push((name.clone(), v.to_string()));
                            }
                        }
                    }
                }
            }
        }
        "Gemfile.lock" => {
            for line in content.lines() {
                let line = line.trim();
                if let Some(open) = line.find('(') {
                    if line.starts_with("    ") || line.starts_with("  ") || line.starts_with(" ") {
                        if let Some(close) = line.find(')') {
                            if close > open + 1 {
                                let name = line[..open].trim();
                                let version = &line[open + 1..close];
                                if !name.is_empty() && !version.contains(' ') {
                                    out.push((name.to_string(), version.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
        "composer.lock" => {
            let v: serde_json::Value = serde_json::from_str(&content).unwrap_or_default();
            for section in ["packages", "packages-dev"] {
                if let Some(arr) = v.get(section).and_then(|a| a.as_array()) {
                    for p in arr {
                        let Some(name) = p.get("name").and_then(|x| x.as_str()) else {
                            continue;
                        };
                        let Some(version) = p.get("version").and_then(|x| x.as_str()) else {
                            continue;
                        };
                        out.push((name.to_string(), version.to_string()));
                    }
                }
            }
        }
        _ => {}
    }
    out.sort();
    out.dedup();
    out
}

fn parse_yarn(content: &str, out: &mut Vec<(String, String)>) {
    let mut pending: Option<String> = None;
    for line in content.lines() {
        let line = line.trim_end();
        if line.starts_with("  ") {
            if let Some(name) = pending.take() {
                if let Some(rest) = line.trim_start().strip_prefix("version ") {
                    let version = rest.trim().trim_matches('"');
                    if !version.is_empty() {
                        out.push((name, version.to_string()));
                    }
                }
            }
            continue;
        }
        if line.ends_with(':') && line.contains('@') {
            let key = line.trim_end_matches(':').trim();
            let name = key
                .split(',')
                .next()
                .unwrap_or(key)
                .trim()
                .trim_matches('"');
            let name = name
                .split('@')
                .take_while(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("@");
            pending = if name.is_empty() { None } else { Some(name) };
        } else if !line.is_empty() {
            pending = None;
        }
    }
}

fn parse_toml_lock(content: &str, out: &mut Vec<(String, String)>) {
    let val: toml::Value = toml::from_str(content).unwrap_or_else(|_| toml::Value::Table(Default::default()));
    let Some(pkgs) = val.get("package").and_then(|p| p.as_array()) else {
        return;
    };
    for p in pkgs {
        let Some(name) = p.get("name").and_then(|n| n.as_str()) else {
            continue;
        };
        let Some(version) = p.get("version").and_then(|v| v.as_str()) else {
            continue;
        };
        out.push((name.to_string(), version.to_string()));
    }
}

fn osv_batch(agent: &ureq::Agent, queries: &[PackageQuery]) -> BTreeMap<String, Vec<String>> {
    let mut hits: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for chunk in queries.chunks(OSV_CHUNK) {
        let body = serde_json::json!({
            "queries": chunk.iter().map(|q| {
                serde_json::json!({
                    "package": { "ecosystem": q.ecosystem, "name": q.name },
                    "version": q.version,
                })
            }).collect::<Vec<_>>(),
        });
        let resp = match agent.post(OSV_BATCH).send_json(&body) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  {} OSV query failed: {}", style::error(""), e);
                continue;
            }
        };
        let value: serde_json::Value = match resp.into_json() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let results = value.get("results").and_then(|r| r.as_array());
        if let Some(results) = results {
            for (i, res) in results.iter().enumerate() {
                let Some(q) = chunk.get(i) else { continue };
                let ids: Vec<String> = res
                    .get("vulns")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.get("id").and_then(|x| x.as_str()))
                            .map(|s| s.to_string())
                            .collect()
                    })
                    .unwrap_or_default();
                if !ids.is_empty() {
                    hits.insert(query_key(q), ids);
                }
            }
        }
    }
    hits
}

fn vuln_details(agent: &ureq::Agent, hits: &BTreeMap<String, Vec<String>>) -> BTreeMap<String, Vuln> {
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for list in hits.values() {
        for id in list {
            ids.insert(id.clone());
        }
    }
    let mut out: BTreeMap<String, Vuln> = BTreeMap::new();
    let ids: Vec<String> = ids.into_iter().collect();
    let wave = 10;
    for chunk in ids.chunks(wave) {
        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for id in chunk {
                let agent = agent.clone();
                let id = id.clone();
                handles.push(s.spawn(move || {
                    let url = format!("{}/{}", OSV_VULN, id);
                    let resp = match agent.get(&url).call() {
                        Ok(r) => r,
                        Err(_) => return None,
                    };
                    let value: serde_json::Value = match resp.into_json() {
                        Ok(v) => v,
                        Err(_) => return None,
                    };
                    Some((
                        id.clone(),
                        Vuln {
                            id,
                            alias: value
                                .get("aliases")
                                .and_then(|a| a.as_array())
                                .and_then(|a| a.first())
                                .and_then(|x| x.as_str())
                                .map(|s| s.to_string()),
                            severity: vuln_severity(&value),
                            summary: value
                                .get("summary")
                                .and_then(|s| s.as_str())
                                .map(|s| s.to_string()),
                            fixed: vuln_fixed(&value),
                        },
                    ))
                }));
            }
            for h in handles {
                if let Ok(Some((id, vuln))) = h.join() {
                    out.insert(id, vuln);
                }
            }
        });
    }
    out
}

fn vuln_severity(rec: &serde_json::Value) -> Option<String> {
    if let Some(s) = rec
        .pointer("/database_specific/severity")
        .and_then(|v| v.as_str())
    {
        return Some(s.to_uppercase());
    }
    if let Some(arr) = rec.get("severity").and_then(|s| s.as_array()) {
        for sev in arr {
            if let Some(score) = sev.get("score").and_then(|s| s.as_str()) {
                let score = score.trim();
                if score.starts_with("CVSS:3") {
                    if let Some(n) = cvss3_base(score) {
                        return Some(qualitative(n));
                    }
                } else if let Some(num) = score.split_whitespace().next() {
                    if let Ok(n) = num.trim_end_matches(['.', ',']).parse::<f64>() {
                        return Some(qualitative(n));
                    }
                }
            }
        }
    }
    None
}

fn cvss3_base(vector: &str) -> Option<f64> {
    let mut m: BTreeMap<String, String> = BTreeMap::new();
    for part in vector.split('/') {
        let Some(idx) = part.find(':') else {
            continue;
        };
        let (k, v) = (&part[..idx], &part[idx + 1..]);
        m.insert(k.to_string(), v.to_string());
    }
    let av = match m.get("AV").map(|s| s.as_str()) {
        Some("N") => 0.85,
        Some("A") => 0.62,
        Some("L") => 0.55,
        Some("P") => 0.2,
        _ => return None,
    };
    let ac = match m.get("AC").map(|s| s.as_str()) {
        Some("L") => 0.77,
        Some("H") => 0.44,
        _ => return None,
    };
    let scope_c = m.get("S").map(|s| s.as_str()) == Some("C");
    let pr = match (scope_c, m.get("PR").map(|s| s.as_str())) {
        (false, Some("N")) => 0.85,
        (false, Some("L")) => 0.62,
        (false, Some("H")) => 0.27,
        (true, Some("N")) => 0.85,
        (true, Some("L")) => 0.68,
        (true, Some("H")) => 0.5,
        _ => return None,
    };
    let ui = match m.get("UI").map(|s| s.as_str()) {
        Some("N") => 0.85,
        Some("R") => 0.62,
        _ => return None,
    };
    let impact = |x: &str| -> Option<f64> {
        match x {
            "H" => Some(0.56),
            "L" => Some(0.22),
            "N" => Some(0.0),
            _ => None,
        }
    };
    let c = impact(m.get("C").map(|s| s.as_str())?)?;
    let i = impact(m.get("I").map(|s| s.as_str())?)?;
    let a = impact(m.get("A").map(|s| s.as_str())?)?;
    let isc = 1.0 - (1.0 - c) * (1.0 - i) * (1.0 - a);
    let impact = if scope_c {
        7.52 * (isc - 0.029) - 3.25 * (isc - 0.02).powi(15)
    } else {
        6.42 * isc
    };
    let expl = 8.22 * av * ac * pr * ui;
    let base = (impact + expl).min(10.0);
    Some((base * 10.0).ceil() / 10.0)
}

fn qualitative(score: f64) -> String {
    if score >= 9.0 {
        "CRITICAL".to_string()
    } else if score >= 7.0 {
        "HIGH".to_string()
    } else if score >= 4.0 {
        "MODERATE".to_string()
    } else {
        "LOW".to_string()
    }
}

fn vuln_fixed(rec: &serde_json::Value) -> Option<String> {
    let mut best: Option<String> = None;
    if let Some(affected) = rec.get("affected").and_then(|a| a.as_array()) {
        for a in affected {
            if let Some(ranges) = a.get("ranges").and_then(|r| r.as_array()) {
                for range in ranges {
                    if let Some(events) = range.get("events").and_then(|e| e.as_array()) {
                        for ev in events {
                            if let Some(f) = ev.get("fixed").and_then(|x| x.as_str()) {
                                best = Some(f.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
    best
}

fn scan_system(agent: &ureq::Agent) -> (Vec<Finding>, usize) {
    let mut findings = Vec::new();

    let out = match Command::new("pacman").arg("-Q").output() {
        Ok(o) if o.status.success() => o,
        _ => return (findings, 0),
    };
    let mut installed: Vec<(String, String)> = Vec::new();
    let mut foreign: BTreeSet<String> = BTreeSet::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let mut it = line.split_whitespace();
        if let (Some(name), Some(version)) = (it.next(), it.next()) {
            installed.push((name.to_string(), version.to_string()));
        }
    }
    if let Ok(fout) = Command::new("pacman").args(["-Qm"]).output() {
        for line in String::from_utf8_lossy(&fout.stdout).lines() {
            if let Some(name) = line.split_whitespace().next() {
                foreign.insert(name.to_string());
            }
        }
    }

    let issues_value = match get_json(agent, AST_ISSUES) {
        Some(v) => v,
        None => {
            eprintln!("  {} Arch Security Tracker unreachable.", style::error(""));
            return (findings, installed.len());
        }
    };
    let Some(issues) = issues_value.as_array() else {
        return (findings, installed.len());
    };

    let mut by_pkg: BTreeMap<String, Vec<&serde_json::Value>> = BTreeMap::new();
    for issue in issues {
        let Some(pkgs) = issue.get("packages").and_then(|p| p.as_array()) else {
            continue;
        };
        for p in pkgs {
            if let Some(name) = p.as_str() {
                by_pkg.entry(name.to_string()).or_default().push(issue);
            }
        }
    }

    for (name, version) in &installed {
        let Some(matches) = by_pkg.get(name) else {
            continue;
        };
        let mut vulns: Vec<Vuln> = Vec::new();
        for issue in matches {
            let Some(fixed) = issue.get("fixed").and_then(|f| f.as_str()) else {
                continue;
            };
            if fixed.is_empty() || !arch_version_lt(version, fixed) {
                continue;
            }
            let mut id = issue
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("AVG")
                .to_string();
            if let Some(cves) = issue.get("issues").and_then(|i| i.as_array()) {
                let cves: Vec<String> = cves
                    .iter()
                    .filter_map(|i| i.as_str())
                    .map(|s| s.to_string())
                    .collect();
                if !cves.is_empty() {
                    id = format!("{} ({})", id, cves.join(", "));
                }
            }
            let severity = issue
                .get("severity")
                .and_then(|s| s.as_str())
                .map(|s| s.to_uppercase());
            let atype = issue.get("type").and_then(|t| t.as_str()).unwrap_or("");
            vulns.push(Vuln {
                id,
                alias: None,
                severity,
                summary: if atype.is_empty() {
                    None
                } else {
                    Some(format!("Arch advisory: {}", atype))
                },
                fixed: Some(fixed.to_string()),
            });
        }
        if !vulns.is_empty() {
            let src = if foreign.contains(name) {
                "system (AUR)"
            } else {
                "system (pacman)"
            };
            findings.push(Finding {
                source: src.to_string(),
                package: name.clone(),
                version: version.clone(),
                vulns,
            });
        }
    }
    (findings, installed.len())
}

fn dedup_vulns(vulns: Vec<Vuln>) -> Vec<Vuln> {
    let mut map: BTreeMap<String, Vuln> = BTreeMap::new();
    for v in vulns {
        let key = v.alias.clone().unwrap_or_else(|| v.id.clone());
        let better = match map.get(&key) {
            None => true,
            Some(existing) => existing.severity.is_none() && v.severity.is_some(),
        };
        if better {
            map.insert(key, v);
        }
    }
    map.into_values().collect()
}

fn get_json(agent: &ureq::Agent, url: &str) -> Option<serde_json::Value> {
    let resp = agent.get(url).call().ok()?;
    resp.into_json().ok()
}

fn arch_version_lt(installed: &str, fixed: &str) -> bool {
    if crate::utils::which("vercmp") {
        if let Ok(out) = Command::new("vercmp").args([installed, fixed]).output() {
            let text = String::from_utf8_lossy(&out.stdout);
            if let Ok(n) = text.trim().parse::<i32>() {
                return n < 0;
            }
        }
    }
    vercmp_manual(installed, fixed) < 0
}

fn vercmp_manual(a: &str, b: &str) -> i32 {
    let split = |s: &str| -> (u64, String, String) {
        let s = s.trim();
        let mut rest = s;
        let mut epoch = 0u64;
        if let Some(idx) = s.find(':') {
            if let Ok(e) = s[..idx].parse::<u64>() {
                epoch = e;
                rest = &s[idx + 1..];
            }
        }
        let (version, release) = match rest.find('-') {
            Some(idx) => (rest[..idx].to_string(), rest[idx + 1..].to_string()),
            None => (rest.to_string(), "0".to_string()),
        };
        (epoch, version, release)
    };
    let (ea, va, ra) = split(a);
    let (eb, vb, rb) = split(b);
    if ea != eb {
        return if ea < eb { -1 } else { 1 };
    }
    let cv = cmp_segments(&va, &vb);
    if cv != 0 {
        return cv;
    }
    cmp_segments(&ra, &rb)
}

fn cmp_segments(x: &str, y: &str) -> i32 {
    let mut xi = x.split('.');
    let mut yi = y.split('.');
    loop {
        let (xa, ya) = (xi.next(), yi.next());
        match (xa, ya) {
            (None, None) => return 0,
            (None, Some(_)) => return -1,
            (Some(_), None) => return 1,
            (Some(xs), Some(ys)) => {
                let c = cmp_token(xs, ys);
                if c != 0 {
                    return c;
                }
            }
        }
    }
}

fn cmp_token(x: &str, y: &str) -> i32 {
    let xnum: Option<u64> = x.parse().ok();
    let ynum: Option<u64> = y.parse().ok();
    match (xnum, ynum) {
        (Some(a), Some(b)) => {
            if a < b {
                -1
            } else if a > b {
                1
            } else {
                0
            }
        }
        (Some(_), None) => 1,
        (None, Some(_)) => -1,
        (None, None) => {
            if x < y {
                -1
            } else if x > y {
                1
            } else {
                0
            }
        }
    }
}

fn print_findings(findings: &[Finding]) {
    if findings.is_empty() {
        return;
    }
    let mut by_source: BTreeMap<String, Vec<&Finding>> = BTreeMap::new();
    for f in findings {
        by_source.entry(f.source.clone()).or_default().push(f);
    }
    for (source, list) in &by_source {
        println!("  {}", format!("SOURCE: {}", source).style(style::Theme::LABEL));
        for f in list {
            println!(
                "    {} {}",
                f.package.style(style::Theme::BOLD),
                f.version.style(style::Theme::MUTED)
            );
            for v in &f.vulns {
                let sev = v
                    .severity
                    .as_deref()
                    .map(sev_style)
                    .unwrap_or_else(|| "[?]".style(style::Theme::MUTED).to_string());
                let alias = v
                    .alias
                    .as_ref()
                    .map(|a| format!(" ({})", a.style(style::Theme::MUTED)))
                    .unwrap_or_default();
                let summary = v
                    .summary
                    .as_ref()
                    .map(|s| format!(" — {}", s.chars().take(90).collect::<String>()))
                    .unwrap_or_default();
                let fix = v
                    .fixed
                    .as_ref()
                    .map(|f| format!(" → fix {}", f.style(style::Theme::SUCCESS)))
                    .unwrap_or_default();
                println!(
                    "      {} {} {}{}{}{}",
                    style::warn(""),
                    sev,
                    v.id.style(style::Theme::BOLD),
                    alias,
                    fix,
                    summary
                );
            }
        }
        println!();
    }
}

fn sev_style(s: &str) -> String {
    let up = s.to_uppercase();
    if up.starts_with("CRIT") || up.starts_with("HIGH") {
        format!("[{}]", up.style(style::Theme::ERROR))
    } else if up.starts_with("MOD") || up.starts_with("MED") {
        format!("[{}]", up.style(style::Theme::WARN))
    } else if up.starts_with("LOW") {
        format!("[{}]", up.style(style::Theme::MUTED))
    } else {
        format!("[{}]", up.style(style::Theme::VALUE))
    }
}

fn print_summary(findings: &[Finding], checked: &[Checked]) {
    let total_pkgs: usize = checked.iter().map(|c| c.packages).sum();
    let vuln_pkgs = findings.len();
    println!("{}", style::divider());
    for c in checked {
        println!(
            "  {}",
            style::label_value(
                &c.source,
                &format!(
                    "{} package(s) checked",
                    c.packages.style(style::Theme::VALUE)
                )
            )
        );
    }
    println!("  {}", style::label_value("Total", &format!("{} package(s)", total_pkgs)));
    println!();
    if vuln_pkgs == 0 {
        println!("  {} No known vulnerabilities found.", style::success(""));
    } else {
        println!(
            "  {} {} vulnerable package(s) found — check the advisories above and update.",
            style::warn(""),
            vuln_pkgs.style(style::Theme::ERROR)
        );
    }
}
