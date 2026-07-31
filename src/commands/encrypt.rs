use clap::Subcommand;
use owo_colors::OwoColorize;
use crate::style;

#[derive(Subcommand, Debug, Clone)]
pub enum EncryptAction {
    #[command(about = "Base64 encode or decode text")]
    Base64 {
        #[command(subcommand)]
        action: CodecAction,
    },
    #[command(about = "Hex encode or decode text")]
    Hex {
        #[command(subcommand)]
        action: CodecAction,
    },
    #[command(about = "Hash text (MD5, SHA-1, SHA-256, SHA-512)")]
    Hash {
        #[arg(value_name = "ALGO", help = "md5, sha1, sha256, sha512")]
        algo: String,
        #[arg(required = true, value_name = "TEXT")]
        text: String,
    },
    #[command(about = "Generate a UUID v4")]
    Uuid,
    #[command(about = "Bcrypt hash a password")]
    Bcrypt {
        #[arg(required = true, value_name = "PASSWORD")]
        password: String,
        #[arg(short, long, default_value = "12", value_name = "ROUNDS")]
        rounds: u32,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum CodecAction {
    #[command(about = "Encode text")]
    Encode {
        #[arg(required = true, value_name = "TEXT")]
        text: String,
    },
    #[command(about = "Decode text")]
    Decode {
        #[arg(required = true, value_name = "TEXT")]
        text: String,
    },
}

pub fn run(action: &EncryptAction) {
    match action {
        EncryptAction::Base64 { action } => b64(action),
        EncryptAction::Hex { action } => hex_cmd(action),
        EncryptAction::Hash { algo, text } => hash(algo, text),
        EncryptAction::Uuid => gen_uuid(),
        EncryptAction::Bcrypt { password, rounds } => do_bcrypt(password, *rounds),
    }
}

fn b64(action: &CodecAction) {
    use base64::Engine;
    match action {
        CodecAction::Encode { text } => {
            let out = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
            println!("{}", style::label_value("Base64", &out));
        }
        CodecAction::Decode { text } => {
            use base64::Engine;
            match base64::engine::general_purpose::STANDARD.decode(text.trim()) {
                Ok(bytes) => println!("{}", style::label_value("Decoded", &String::from_utf8_lossy(&bytes))),
                Err(e) => eprintln!("{} Invalid base64: {}", style::error(""), e),
            }
        }
    }
}

fn hex_cmd(action: &CodecAction) {
    match action {
        CodecAction::Encode { text } => {
            let hex: String = text.as_bytes().iter().map(|b| format!("{:02x}", b)).collect();
            println!("{}", style::label_value("Hex", &hex));
        }
        CodecAction::Decode { text } => {
            let cleaned: String = text.chars().filter(|c| !c.is_whitespace()).collect();
            if cleaned.len() % 2 != 0 {
                eprintln!("{} Invalid hex length", style::error(""));
                return;
            }
            let bytes: Vec<u8> = (0..cleaned.len()).step_by(2)
                .filter_map(|i| u8::from_str_radix(&cleaned[i..i+2], 16).ok())
                .collect();
            if bytes.len() * 2 == cleaned.len() {
                println!("{}", style::label_value("Decoded", &String::from_utf8_lossy(&bytes)));
            } else {
                eprintln!("{} Invalid hex string", style::error(""));
            }
        }
    }
}

fn hash(algo: &str, text: &str) {
    use sha2::{Sha256, Sha512, Digest};
    use sha1::Sha1;
    use md5::Md5;

    let (name, output): (String, String) = match algo.to_lowercase().as_str() {
        "md5" => {
            let mut h = Md5::new();
            h.update(text.as_bytes());
            ("MD5".into(), format!("{:x}", h.finalize()))
        }
        "sha1" => {
            let mut h = Sha1::new();
            h.update(text.as_bytes());
            ("SHA-1".into(), format!("{:x}", h.finalize()))
        }
        "sha256" => {
            let mut h = Sha256::new();
            h.update(text.as_bytes());
            ("SHA-256".into(), format!("{:x}", h.finalize()))
        }
        "sha512" => {
            let mut h = Sha512::new();
            h.update(text.as_bytes());
            ("SHA-512".into(), format!("{:x}", h.finalize()))
        }
        _ => {
            eprintln!("{} Unknown algorithm: '{}'. Use: md5, sha1, sha256, sha512", style::error(""), algo);
            return;
        }
    };
    println!("{}", style::label_value(&name, &output));
}

fn gen_uuid() {
    let id = uuid::Uuid::new_v4();
    println!("{}", style::label_value("UUID v4", &id.to_string()));
}

fn do_bcrypt(password: &str, rounds: u32) {
    let sp = style::Spinner::new(&format!("Hashing with bcrypt ({} rounds)...", rounds));
    match bcrypt::hash(password, rounds) {
        Ok(hash) => {
            sp.done("Hashed");
            println!("\n{}", style::label_value("Bcrypt", &hash));
        }
        Err(e) => {
            sp.fail(&format!("Bcrypt error: {}", e));
        }
    }
}
