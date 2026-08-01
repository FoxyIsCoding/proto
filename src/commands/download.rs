use crate::style;
use clap::{Subcommand, ValueEnum};
use owo_colors::OwoColorize;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const SOUNDCLOUD_LOG: &str = "download_log.txt";
const SOUNDCLOUD_ERROR_LOG: &str = "download_log_ERROR.txt";

#[derive(Subcommand, Debug, Clone)]
pub enum DownloadAction {
    #[command(
        about = "Interactive yt-dlp wrapper for downloading videos"
    )]
    Video {
        #[arg(value_name = "URL", help = "Video/playlist URL (default: interactive prompt)")]
        url: Option<String>,
        #[arg(long, value_enum, help = "Output format")]
        format: Option<VideoFormat>,
        #[arg(long, value_name = "DIR", help = "Output directory")]
        dir: Option<String>,
        #[arg(long, help = "Download subtitles when available")]
        subtitles: bool,
        #[arg(long, help = "Skip embedding metadata/thumbnail")]
        no_metadata: bool,
    },
    #[command(
        about = "Music downloader for YouTube Music playlists and SoundCloud (with download log)"
    )]
    Music {
        #[arg(value_name = "URL", help = "Playlist URL (default: interactive prompt)")]
        url: Option<String>,
        #[arg(long, value_name = "DIR", default_value = "downloads/music", help = "Output directory")]
        dir: String,
        #[arg(long, value_name = "BROWSER", help = "Browser to pull cookies from (default: opera)")]
        browser: Option<String>,
        #[arg(long, value_name = "N", help = "Limit how many new tracks to download")]
        amount: Option<usize>,
        #[arg(long, help = "Sort newest-first instead of playlist order")]
        newest: bool,
        #[arg(long, help = "SoundCloud artist profile page (downloads the whole catalog)")]
        artist: bool,
        #[arg(long, help = "Skip all confirmation prompts")]
        yes: bool,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq)]
pub enum VideoFormat {
    Best,
    P1080,
    P720,
    P480,
    AudioMp3,
}

pub fn run(action: &DownloadAction) {
    if !crate::utils::which("yt-dlp") {
        eprintln!(
            "{} yt-dlp required. Install it: {}",
            style::error(""),
            "python3 -m pip install -U yt-dlp".dimmed()
        );
        return;
    }

    match action {
        DownloadAction::Video {
            url,
            format,
            dir,
            subtitles,
            no_metadata,
        } => video(url.as_deref(), *format, dir.as_deref(), *subtitles, *no_metadata),
        DownloadAction::Music {
            url,
            dir,
            browser,
            amount,
            newest,
            artist,
            yes,
        } => music(url.as_deref(), dir, browser.as_deref(), *amount, *newest, *artist, *yes),
    }
}

// ---------------------------------------------------------------- video flow

fn video(url_opt: Option<&str>, format_opt: Option<VideoFormat>, dir_opt: Option<&str>, subtitles: bool, no_metadata: bool) {
    println!("{}", style::header("Video Downloader"));
    println!("{}", style::divider());

    let url = url_opt
        .map(|u| u.to_string())
        .unwrap_or_else(|| prompt_input("Video or playlist URL"));
    if url.trim().is_empty() {
        eprintln!("{} No URL provided.", style::error(""));
        return;
    }

    let fmt = format_opt.unwrap_or_else(|| {
        use dialoguer::Select;
        let items = vec![
            format!("Best quality (merged video+audio)"),
            format!("1080p (mp4)"),
            format!("720p (mp4)"),
            format!("480p (mp4)"),
            format!("Audio only (mp3 256k)"),
        ];
        let idx = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Select format")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(0);
        match idx {
            1 => VideoFormat::P1080,
            2 => VideoFormat::P720,
            3 => VideoFormat::P480,
            4 => VideoFormat::AudioMp3,
            _ => VideoFormat::Best,
        }
    });

    let default_dir = home_dir().join("Downloads").display().to_string();
    let dir = dir_opt.map(|d| d.to_string()).unwrap_or_else(|| {
        let entered = prompt_input(&format!("Output directory (default: {})", default_dir));
        if entered.trim().is_empty() {
            default_dir.clone()
        } else {
            entered
        }
    });
    let dir = expand_tilde(&dir);

    let metadata = !no_metadata;

    let mut args: Vec<String> = Vec::new();
    match fmt {
        VideoFormat::Best => {
            args.push("-f".into());
            args.push("bv*+ba/b".into());
            args.push("--merge-output-format".into());
            args.push("mp4".into());
        }
        VideoFormat::P1080 => {
            args.push("-f".into());
            args.push("bv*[height<=1080]+ba/b[height<=1080]".into());
            args.push("--merge-output-format".into());
            args.push("mp4".into());
        }
        VideoFormat::P720 => {
            args.push("-f".into());
            args.push("bv*[height<=720]+ba/b[height<=720]".into());
            args.push("--merge-output-format".into());
            args.push("mp4".into());
        }
        VideoFormat::P480 => {
            args.push("-f".into());
            args.push("bv*[height<=480]+ba/b[height<=480]".into());
            args.push("--merge-output-format".into());
            args.push("mp4".into());
        }
        VideoFormat::AudioMp3 => {
            args.push("-x".into());
            args.push("--audio-format".into());
            args.push("mp3".into());
            args.push("--audio-quality".into());
            args.push("1".into());
        }
    }
    if subtitles {
        args.push("--write-subs".into());
        args.push("--sub-langs".into());
        args.push("en,en.*".into());
    }
    if metadata && fmt != VideoFormat::AudioMp3 {
        args.push("--embed-metadata".into());
        args.push("--embed-thumbnail".into());
    }
    if metadata && fmt == VideoFormat::AudioMp3 {
        args.push("--embed-metadata".into());
        args.push("--embed-thumbnail".into());
        args.push("--write-thumbnail".into());
    }
    args.push("-o".into());
    args.push(format!("{}/%(title)s.%(ext)s", dir));
    args.push("--no-playlist".into());
    args.push(url.clone());

    println!();
    println!("  {}", style::label_value("URL", &url));
    println!("  {}", style::label_value("Format", format_label(fmt)));
    println!("  {}", style::label_value("Directory", &dir));
    println!();
    println!(
        "  {} {}",
        "▶".style(style::Theme::WARN),
        "Downloading (yt-dlp)...".dimmed()
    );

    if let Err(e) = run_ytdlp_inherit(&args) {
        eprintln!("  {} {}", style::error(""), e);
    }
}

fn format_label(f: VideoFormat) -> &'static str {
    match f {
        VideoFormat::Best => "best (merged)",
        VideoFormat::P1080 => "1080p",
        VideoFormat::P720 => "720p",
        VideoFormat::P480 => "480p",
        VideoFormat::AudioMp3 => "audio only (mp3 256k)",
    }
}

// ---------------------------------------------------------------- music flow

fn music(
    url_opt: Option<&str>,
    dir: &str,
    browser: Option<&str>,
    amount: Option<usize>,
    newest: bool,
    artist: bool,
    yes: bool,
) {
    let url = url_opt
        .map(|u| u.to_string())
        .unwrap_or_else(|| prompt_input("Playlist URL (SoundCloud or YouTube)"));
    if url.trim().is_empty() {
        eprintln!("{} No URL provided.", style::error(""));
        return;
    }

    let platform = if artist {
        Platform::Soundcloud
    } else {
        detect_platform(&url)
    };
    match platform {
        Platform::Soundcloud => soundcloud(&url, dir, browser, amount, newest, artist),
        Platform::Youtube => youtube_music(&url, dir, amount, newest, yes),
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Platform {
    Soundcloud,
    Youtube,
}

fn detect_platform(url: &str) -> Platform {
    let lower = url.to_lowercase();
    if lower.contains("soundcloud.com") {
        Platform::Soundcloud
    } else {
        Platform::Youtube
    }
}

fn sc_artist_name(url: &str) -> Option<&str> {
    let lower = url.to_lowercase();
    let idx = lower.find("soundcloud.com/")? + "soundcloud.com/".len();
    let rest = &url[idx..];
    let name = rest.split(['/', '?', '#']).next().unwrap_or("");
    if name.is_empty() || matches!(name, "likes" | "you" | "search" | "discover" | "upload") {
        None
    } else {
        Some(name)
    }
}

fn youtube_music(url: &str, dir: &str, amount: Option<usize>, newest: bool, yes: bool) {
    println!("{}", style::header("Music Downloader — YouTube"));
    println!("{}", style::divider());

    let dir = if dir == "downloads/music" {
        dir.to_string()
    } else {
        expand_tilde(dir)
    };

    let amount = amount.unwrap_or_else(|| {
        use dialoguer::Select;
        let items = vec![
            "Download entire playlist".to_string(),
            "First 10 tracks".to_string(),
            "First 25 tracks".to_string(),
            "First 50 tracks".to_string(),
            "Custom number".to_string(),
        ];
        let idx = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("How many tracks?")
            .items(&items)
            .default(0)
            .interact()
            .unwrap_or(0);
        match idx {
            1 => 10,
            2 => 25,
            3 => 50,
            4 => {
                let n = prompt_input("Number of tracks");
                n.trim().parse().unwrap_or(10)
            }
            _ => 0,
        }
    });

    let sort = if newest {
        SortOrder::Newest
    } else {
        prompt_sort()
    };

    let metadata = if yes {
        true
    } else {
        use dialoguer::Confirm;
        Confirm::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt("Embed metadata & cover art?")
            .default(true)
            .interact()
            .unwrap_or(true)
    };

    let mut args: Vec<String> = vec![
        "-f".into(),
        "bestaudio".into(),
        "-x".into(),
        "--audio-format".into(),
        "mp3".into(),
        "--audio-quality".into(),
        "1".into(),
    ];
    if metadata {
        args.push("--embed-metadata".into());
        args.push("--embed-thumbnail".into());
        args.push("--write-thumbnail".into());
    }
    args.push("-o".into());
    args.push(format!("{}/%(uploader)s/%(title)s.%(ext)s", dir));
    if amount > 0 {
        args.push("--playlist-items".into());
        args.push(format!("1:{}", amount));
    }
    match sort {
        SortOrder::Newest => {
            args.push("--playlist-reverse".into());
        }
        SortOrder::Oldest => {}
        SortOrder::Shuffle => {
            args.push("--playlist-random".into());
        }
        SortOrder::Order => {}
    }
    args.push("--yes-playlist".into());
    args.push(url.to_string());

    println!();
    println!("  {}", style::label_value("URL", url));
    println!("  {}", style::label_value("Amount", &if amount > 0 { amount.to_string() } else { "all".into() }));
    println!("  {}", style::label_value("Directory", &dir));
    println!();
    println!(
        "  {} {}",
        "▶".style(style::Theme::WARN),
        "Downloading (yt-dlp)...".dimmed()
    );

    if let Err(e) = run_ytdlp_inherit(&args) {
        eprintln!("  {} {}", style::error(""), e);
    }
}

fn prompt_sort() -> SortOrder {
    use dialoguer::Select;
    let items = vec![
        "Playlist order".to_string(),
        "Newest first".to_string(),
        "Oldest first".to_string(),
        "Shuffle".to_string(),
    ];
    let idx = Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("Sort order")
        .items(&items)
        .default(0)
        .interact()
        .unwrap_or(0);
    match idx {
        1 => SortOrder::Newest,
        2 => SortOrder::Oldest,
        3 => SortOrder::Shuffle,
        _ => SortOrder::Order,
    }
}

enum SortOrder {
    Order,
    Newest,
    Oldest,
    Shuffle,
}

// ------------------------------------------------- soundcloud (Rust port)

fn soundcloud(
    url: &str,
    dir: &str,
    browser: Option<&str>,
    amount: Option<usize>,
    newest: bool,
    artist: bool,
) {
    let header_title = if artist {
        "Music Downloader — SoundCloud Artist"
    } else {
        "Music Downloader — SoundCloud"
    };
    println!("{}", style::header(header_title));
    println!("{}", style::divider());

    let dir = if dir == "downloads/music" {
        dir.to_string()
    } else {
        expand_tilde(dir)
    };
    let dir_path = PathBuf::from(&dir);
    if let Err(e) = std::fs::create_dir_all(&dir_path) {
        eprintln!("{} Cannot create {}: {}", style::error(""), dir, e);
        return;
    }

    if artist {
        if let Some(name) = sc_artist_name(url) {
            println!(
                "  {}",
                style::label_value("Artist", name)
            );
        }
    }

    let browser = browser.unwrap_or("opera").to_string();

    let mut previous = std::collections::HashSet::new();
    read_download_log(&dir_path.join(SOUNDCLOUD_LOG), &mut previous);
    println!(
        "  {}",
        style::label_value("Previously logged", &previous.len().to_string())
    );

    let sp = style::Spinner::new("Fetching playlist entries...");
    let json = match fetch_entries(url, &browser) {
        Some(j) => j,
        None => {
            sp.fail("Could not read the playlist. Is the URL private or the browser cookie missing?");
            return;
        }
    };
    let entries = parse_entries(&json);
    sp.done(&format!("{} entries found", entries.len()));

    let mut new_songs: Vec<(&str, &str, Option<u64>, String)> = Vec::new();
    for e in &entries {
        let title = e.get("title").and_then(|t| t.as_str()).unwrap_or("");
        let uploader = e
            .get("uploader")
            .or_else(|| e.get("channel"))
            .and_then(|u| u.as_str())
            .unwrap_or("Unknown Uploader");
        let duration = e.get("duration").and_then(|d| d.as_u64());
        if title.is_empty() {
            continue;
        }
        if let Some(d) = duration {
            if d > 900 {
                continue;
            }
        }
        let song_id = normalize_string(&format!("{} - {}", title, uploader));
        if previous.contains(&song_id) {
            continue;
        }
        let track_url = e
            .get("url")
            .and_then(|u| u.as_str())
            .unwrap_or("")
            .to_string();
        if track_url.is_empty() {
            continue;
        }
        new_songs.push((title, uploader, duration, track_url));
    }

    if newest {
        new_songs.reverse();
    }

    println!(
        "  {} new track(s) after dedupe + 15min filter.",
        style::label_value("New tracks", &new_songs.len().to_string()),
    );

    if new_songs.is_empty() {
        println!("{} Nothing new to download.", style::success(""));
        return;
    }

    let amount = amount.unwrap_or(0);
    let take = if amount > 0 {
        new_songs.len().min(amount)
    } else {
        new_songs.len()
    };
    if take < new_songs.len() {
        println!(
            "  {} Limiting to the first {} track(s).",
            style::warn(""),
            take
        );
    }
    let selected: Vec<&(&str, &str, Option<u64>, String)> = new_songs.iter().take(take).collect();

    for (i, (title, uploader, _, _)) in selected.iter().enumerate() {
        println!(
            "    {} {}",
            format!("{:>3}", i + 1).style(style::Theme::MUTED),
            format!("{} — {}", title, uploader).dimmed()
        );
    }

    if selected.is_empty() {
        return;
    }

    let batch = std::env::temp_dir().join(format!(
        "proto-dl-{}.txt",
        std::process::id()
    ));
    let mut file = match std::fs::File::create(&batch) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{} Cannot write batch file: {}", style::error(""), e);
            return;
        }
    };
    for (_, _, _, track_url) in &selected {
        let _ = writeln!(file, "{}", track_url);
    }
    let _ = file.flush();

    let mut args: Vec<String> = Vec::new();
    args.push("--batch-file".into());
    args.push(batch.to_string_lossy().to_string());
    args.push("--cookies-from-browser".into());
    args.push(browser.clone());
    args.push("-f".into());
    args.push("bestaudio".into());
    args.push("-x".into());
    args.push("--audio-format".into());
    args.push("mp3".into());
    args.push("--audio-quality".into());
    args.push("1".into());
    args.push("--embed-metadata".into());
    args.push("--embed-thumbnail".into());
    args.push("--write-thumbnail".into());
    args.push("--ignore-errors".into());
    args.push("-o".into());
    args.push(format!("{}/%(uploader)s/%(title)s.%(ext)s", dir));

    println!();
    println!(
        "  {} {}",
        "▶".style(style::Theme::WARN),
        "Downloading (yt-dlp)...".dimmed()
    );

    let (status, lines) = match run_ytdlp_capture(&args) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("  {} {}", style::error(""), e);
            let _ = std::fs::remove_file(&batch);
            return;
        }
    };
    let _ = std::fs::remove_file(&batch);

    let successes: Vec<String> = lines
        .iter()
        .filter_map(|l| {
            l.strip_prefix("[ExtractAudio] Destination: ")
                .map(|p| p.to_string())
        })
        .collect();
    let failures: Vec<String> = lines
        .iter()
        .filter(|l| l.starts_with("ERROR:"))
        .cloned()
        .collect();

    append_log(&dir_path.join(SOUNDCLOUD_LOG), &successes);
    if !failures.is_empty() {
        append_error_log(&dir_path.join(SOUNDCLOUD_ERROR_LOG), &failures);
    }

    println!();
    println!("{}", style::divider());
    if status.success() && failures.is_empty() {
        println!(
            "  {} {} track(s) downloaded.",
            style::success(""),
            successes.len()
        );
    } else {
        println!(
            "  {} {} track(s) downloaded, {} failed (logged).",
            style::success(""),
            successes.len(),
            failures.len()
        );
    }
}

fn read_download_log(path: &Path, out: &mut std::collections::HashSet<String>) {
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with("Failed Downloads") {
                out.insert(normalize_string(line));
            }
        }
    }
}

fn fetch_entries(url: &str, browser: &str) -> Option<serde_json::Value> {
    let out = Command::new("yt-dlp")
        .args([
            "--flat-playlist",
            "--dump-single-json",
            "--cookies-from-browser",
            browser,
            "--no-warnings",
        ])
        .arg(url)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    serde_json::from_slice(&out.stdout).ok()
}

fn parse_entries(json: &serde_json::Value) -> Vec<&serde_json::Value> {
    json.get("entries")
        .and_then(|e| e.as_array())
        .map(|arr| arr.iter().collect())
        .unwrap_or_default()
}

fn normalize_string(s: &str) -> String {
    s.trim().to_lowercase().replace(' ', "_")
}

fn append_log(path: &Path, destinations: &[String]) {
    if destinations.is_empty() {
        return;
    }
    let mut content = String::new();
    if path.exists() {
        if let Ok(existing) = std::fs::read_to_string(path) {
            content.push_str(&existing);
        }
    }
    for dest in destinations {
        let p = PathBuf::from(dest);
        let uploader = p.parent().and_then(|d| d.file_name()).map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let title = p.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let song_id = normalize_string(&format!("{} - {}", title, uploader));
        content.push_str(&song_id);
        content.push('\n');
    }
    let _ = std::fs::write(path, content);
}

fn append_error_log(path: &Path, failures: &[String]) {
    let mut content = String::new();
    if path.exists() {
        if let Ok(existing) = std::fs::read_to_string(path) {
            content.push_str(&existing);
        }
    }
    for f in failures {
        content.push_str(f);
        content.push('\n');
    }
    let _ = std::fs::write(path, content);
}

// ------------------------------------------------------------ yt-dlp runners

fn run_ytdlp_inherit(args: &[String]) -> std::io::Result<()> {
    Command::new("yt-dlp")
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?;
    Ok(())
}

fn run_ytdlp_capture(args: &[String]) -> std::io::Result<(std::process::ExitStatus, Vec<String>)> {
    let mut child = Command::new("yt-dlp")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stderr = child.stderr.take().unwrap();
    let mut lines = Vec::new();
    let reader = BufReader::new(stderr);
    for line in reader.lines() {
        let line = line.unwrap_or_default();
        println!("  {}", line.dimmed());
        lines.push(line);
    }
    let status = child.wait()?;
    if let Some(mut stdout) = child.stdout.take() {
        use std::io::Read;
        let mut buf = String::new();
        let _ = stdout.read_to_string(&mut buf);
        for line in buf.lines() {
            println!("  {}", line.dimmed());
            lines.push(line.to_string());
        }
    }
    Ok((status, lines))
}

// ---------------------------------------------------------------- helpers

fn prompt_input(prompt: &str) -> String {
    use dialoguer::Input;
    Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .unwrap_or_default()
}

fn expand_tilde(path: &str) -> String {
    if path == "~" {
        home_dir().display().to_string()
    } else if let Some(rest) = path.strip_prefix("~/") {
        home_dir().join(rest).display().to_string()
    } else {
        path.to_string()
    }
}

fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"))
}
