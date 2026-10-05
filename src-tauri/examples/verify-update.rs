use base64::{engine::general_purpose::STANDARD, Engine};
use minisign_verify::{PublicKey, Signature};
use serde_json::Value;
use std::{error::Error, fs, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err("Usage: verify-update <tauri.conf.json> <latest.json> <installer.exe>".into());
    }
    let config: Value = serde_json::from_slice(&fs::read(&args[0])?)?;
    let feed: Value = serde_json::from_slice(&fs::read(&args[1])?)?;
    let version = config["version"].as_str().ok_or("Missing app version")?;
    if feed["version"].as_str() != Some(version) {
        return Err("Updater feed version does not match the application".into());
    }
    let public_key = config["plugins"]["updater"]["pubkey"]
        .as_str()
        .ok_or("Missing updater public key")?;
    let public_key = String::from_utf8(STANDARD.decode(public_key)?)?;
    let public_key = PublicKey::decode(&public_key)?;
    let installer = fs::read(&args[2])?;
    let filename = Path::new(&args[2])
        .file_name()
        .ok_or("Missing installer filename")?
        .to_str()
        .ok_or("Invalid installer filename")?;
    for platform in ["windows-x86_64", "windows-x86_64-nsis"] {
        let artifact = &feed["platforms"][platform];
        let url = artifact["url"]
            .as_str()
            .ok_or("Missing Windows installer URL")?;
        let expected = format!(
            "https://github.com/Gumayushuo/VibeCal/releases/download/v{version}/{filename}"
        );
        if url != expected {
            return Err(format!("Unexpected {platform} installer URL: {url}").into());
        }
        let signature = artifact["signature"]
            .as_str()
            .ok_or("Missing updater signature")?;
        let signature = String::from_utf8(STANDARD.decode(signature)?)?;
        let signature = Signature::decode(&signature)?;
        public_key.verify(&installer, &signature, true)?;
        println!(
            "Verified {platform}: v{version}, trusted updater key, installer signature and URL"
        );
    }
    Ok(())
}
