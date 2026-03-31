use wiclive_lib::core;
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "install-path" => cmd_install_path(),
        "version" => cmd_version(&args[2..]),
        "laa" => cmd_laa(&args[2..]),
        "set-laa" => cmd_set_laa(&args[2..]),
        "cdkey" => cmd_cdkey(),
        "set-cdkey" => cmd_set_cdkey(&args[2..]),
        "request-cdkey" => cmd_request_cdkey(),
        "vcredist" => cmd_vcredist(),
        "proxy" => cmd_proxy(&args[2..]),
        "soviet-assault" => cmd_soviet_assault(&args[2..]),
        "check" => cmd_check_all(&args[2..]),
        "reset" => cmd_reset(&args[2..]),
        "variants" => cmd_variants(&args[2..]),
        "maps" => cmd_maps(),
        "sync" => cmd_sync(),
        "download-test" => cmd_download_test(&args[2..]),
        "help" | "--help" | "-h" => usage(),
        other => {
            eprintln!("Unknown command: {}", other);
            usage();
            process::exit(1);
        }
    }
}

fn usage() {
    eprintln!("wiclive-cli — WIC LIVE game diagnostics

USAGE:
    wiclive-cli <command> [args]

COMMANDS:
    install-path          Detect game install path from registry
    version [exe_path]    Read PE version from exe (default: wic.exe from registry)
    laa [exe_path]        Check LAA flag on exe
    set-laa [exe_path]    Set LAA flag on exe
    cdkey                 Read CD key from registry
    set-cdkey <key>       Write CD key to registry
    vcredist              Check if VC++ Redistributable is installed
    proxy [game_dir]      Check proxy install status and version
    soviet-assault [dir]  Check if Soviet Assault is installed
    check [exe_path]      Run all readiness checks
    variants [game_dir]   List available wic.exe variants
    reset <variant>       Reset wic.exe to a variant (e.g. wic.1.0.0.nolaa.exe)
    maps                  List maps and their status (missing/outdated/current)
    sync                  Download all missing and outdated maps");
}

fn cmd_install_path() {
    match core::get_install_path() {
        Some(path) => println!("{}", path),
        None => {
            eprintln!("Game install path not found in registry");
            process::exit(1);
        }
    }
}

fn cmd_version(args: &[String]) {
    let path = resolve_exe(args);
    match core::read_pe_version(&path) {
        Ok(v) => println!("{}", v),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_laa(args: &[String]) {
    let path = resolve_exe(args);
    match core::check_laa(&path) {
        Ok(true) => println!("LAA: enabled"),
        Ok(false) => println!("LAA: disabled"),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_set_laa(args: &[String]) {
    let path = resolve_exe(args);
    match core::apply_laa(&path) {
        Ok(_) => println!("LAA flag set on {}", path),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_cdkey() {
    match core::read_cd_key() {
        Ok(key) if !key.is_empty() => println!("{}", key),
        Ok(_) => {
            println!("(not set)");
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_request_cdkey() {
    let api = env::var("API_URL").unwrap_or_else(|_| "http://localhost:3243".into());
    let url = format!("{}/cdkey/generate", api);
    println!("POST {}", url);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let client = reqwest::Client::new();
        let resp = client.post(&url).send().await.unwrap();
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        println!("HTTP {}", status);
        println!("{}", body);
    });
}

fn cmd_set_cdkey(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: wiclive-cli set-cdkey <key>");
        process::exit(1);
    }
    match core::write_cd_key(&args[0]) {
        Ok(()) => println!("CD key written"),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn cmd_vcredist() {
    if core::check_vcredist() {
        println!("VC++ Redistributable: installed");
    } else {
        println!("VC++ Redistributable: missing");
        process::exit(1);
    }
}

fn cmd_proxy(args: &[String]) {
    let dir = resolve_dir(args);
    if core::check_proxy(&dir) {
        match core::read_proxy_version(&dir) {
            Ok(ver) => println!("Proxy: {}", ver.trim()),
            Err(_) => println!("Proxy: installed (version unknown)"),
        }
    } else {
        println!("Proxy: not installed");
        process::exit(1);
    }
}

fn cmd_soviet_assault(args: &[String]) {
    let dir = resolve_dir(args);
    if core::is_soviet_assault(&dir) {
        println!("Soviet Assault: yes");
    } else {
        println!("Soviet Assault: no");
    }
}

fn cmd_check_all(args: &[String]) {
    let (dir, exe_path) = if !args.is_empty() {
        let p = args[0].clone();
        let exe = if p.ends_with(".exe") {
            p.clone()
        } else {
            PathBuf::from(&p).join("wic.exe").to_string_lossy().to_string()
        };
        let dir = std::path::Path::new(&exe)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(p);
        (dir, exe)
    } else {
        match core::get_install_path() {
            Some(p) => {
                let exe = PathBuf::from(&p).join("wic.exe").to_string_lossy().to_string();
                (p, exe)
            }
            None => {
                eprintln!("Install path not found. Pass a path as argument.");
                process::exit(1);
            }
        }
    };

    println!("Game dir: {}", dir);
    println!();

    // Version
    match core::read_pe_version(&exe_path) {
        Ok(v) => {
            let ok = v.patch == 1 && v.build == 1;
            println!("Version:  {} {}", v, if ok { "(ok)" } else { "(needs patch)" });
        }
        Err(e) => println!("Version:  error ({})", e),
    }

    // LAA
    match core::check_laa(&exe_path) {
        Ok(true) => println!("LAA:      enabled"),
        Ok(false) => println!("LAA:      disabled (needs fix)"),
        Err(e) => println!("LAA:      error ({})", e),
    }

    // VC++
    if core::check_vcredist() {
        println!("VC++:     installed");
    } else {
        println!("VC++:     missing");
    }

    // CD Key
    match core::read_cd_key() {
        Ok(key) if !key.is_empty() => println!("CD Key:   {}", key),
        Ok(_) => println!("CD Key:   (not set)"),
        Err(e) => println!("CD Key:   error ({})", e),
    }

    // Proxy
    if core::check_proxy(&dir) {
        match core::read_proxy_version(&dir) {
            Ok(ver) => println!("Proxy:    {}", ver.trim()),
            Err(_) => println!("Proxy:    installed"),
        }
    } else {
        println!("Proxy:    not installed");
    }

    // Soviet Assault
    if core::is_soviet_assault(&dir) {
        println!("Edition:  Soviet Assault");
    } else {
        println!("Edition:  Vanilla");
    }
}

fn cmd_variants(args: &[String]) {
    let dir = resolve_dir(args);
    let variants = core::list_variants(&dir);
    if variants.is_empty() {
        println!("No variants found in {}", dir);
        return;
    }
    for v in &variants {
        let path = PathBuf::from(&dir).join(v);
        let path_str = path.to_string_lossy();
        let ver = core::read_pe_version(&path_str)
            .map(|v| format!("{}", v))
            .unwrap_or_else(|_| "?".into());
        let laa = core::check_laa(&path_str)
            .map(|b| if b { "LAA" } else { "no-LAA" })
            .unwrap_or("?");
        println!("  {}  (v{}, {})", v, ver, laa);
    }
}

fn cmd_reset(args: &[String]) {
    if args.is_empty() {
        let dir = resolve_dir(&[]);
        eprintln!("Usage: wiclive-cli reset <variant>\n");
        eprintln!("Available variants:");
        cmd_variants(&[dir]);
        process::exit(1);
    }

    let variant = &args[0];
    let dir = resolve_dir(&args[1..]);

    // Show before state
    let exe_path = PathBuf::from(&dir).join("wic.exe").to_string_lossy().to_string();
    if let Ok(v) = core::read_pe_version(&exe_path) {
        let laa = core::check_laa(&exe_path).unwrap_or(false);
        println!("Before: v{} {}", v, if laa { "LAA" } else { "no-LAA" });
    }

    match core::reset_exe(&dir, variant) {
        Ok(()) => {
            // Show after state
            if let Ok(v) = core::read_pe_version(&exe_path) {
                let laa = core::check_laa(&exe_path).unwrap_or(false);
                println!("After:  v{} {}", v, if laa { "LAA" } else { "no-LAA" });
            }
            println!("Reset to {}", variant);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

/// Resolve exe path: use arg if given, otherwise find from registry.
fn resolve_exe(args: &[String]) -> String {
    if let Some(path) = args.first() {
        path.clone()
    } else {
        match core::require_exe_path() {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(e) => {
                eprintln!("{}. Pass an exe path as argument.", e);
                process::exit(1);
            }
        }
    }
}

/// Resolve game dir: use arg if given, otherwise find from registry.
fn resolve_dir(args: &[String]) -> String {
    if let Some(path) = args.first() {
        path.clone()
    } else {
        match core::require_install_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("{}. Pass a game directory as argument.", e);
                process::exit(1);
            }
        }
    }
}

#[derive(serde::Deserialize)]
struct RemoteMap {
    name: String,
    size: u64,
    hash: String,
    #[allow(dead_code)] date: String,
    #[allow(dead_code)] uploader: String,
    #[allow(dead_code)] version: u32,
}

struct MapStatus {
    name: String,
    remote_hash: String,
    size: u64,
    status: &'static str, // "missing", "outdated", "current"
}

fn fetch_map_status() -> Vec<MapStatus> {
    let api = env::var("API_URL").unwrap_or_else(|_| "http://localhost:3243".into());
    let rt = tokio::runtime::Runtime::new().unwrap();

    let remote: std::collections::HashMap<String, RemoteMap> = rt.block_on(async {
        let resp = reqwest::get(&format!("{}/maps/data", api)).await.unwrap();
        resp.json().await.unwrap()
    });

    let local_files: Vec<String> = core::list_map_files().unwrap_or_default();
    let local_set: std::collections::HashSet<String> = local_files.iter().map(|f| f.to_lowercase()).collect();

    let mut result: Vec<MapStatus> = Vec::new();
    for info in remote.values() {
        let key = info.name.to_lowercase();
        let status = if !local_set.contains(&key) {
            "missing"
        } else {
            match core::get_map_hash(&info.name) {
                Ok(h) if h == info.hash => "current",
                _ => "outdated",
            }
        };
        result.push(MapStatus {
            name: info.name.clone(),
            remote_hash: info.hash.clone(),
            size: info.size,
            status,
        });
    }

    result.sort_by(|a, b| {
        let ord = |s: &str| match s { "missing" => 0, "outdated" => 1, _ => 2 };
        ord(a.status).cmp(&ord(b.status)).then(a.name.cmp(&b.name))
    });

    result
}

fn cmd_maps() {
    let maps_dir = match core::get_maps_dir() {
        Ok(d) => d,
        Err(e) => { eprintln!("Maps dir: {}", e); process::exit(1); }
    };
    println!("Maps dir: {}", maps_dir.display());

    let statuses = fetch_map_status();
    let missing = statuses.iter().filter(|m| m.status == "missing").count();
    let outdated = statuses.iter().filter(|m| m.status == "outdated").count();
    let current = statuses.iter().filter(|m| m.status == "current").count();
    println!("{} maps: {} current, {} missing, {} outdated\n", statuses.len(), current, missing, outdated);

    for m in &statuses {
        let size_mb = m.size as f64 / 1024.0 / 1024.0;
        let tag = match m.status {
            "missing" => "MISSING ",
            "outdated" => "OUTDATED",
            _ => "   ok   ",
        };
        println!("  [{}]  {:30}  {:>7.1} MB", tag, m.name, size_mb);
    }
}

fn cmd_sync() {
    let api = env::var("API_URL").unwrap_or_else(|_| "http://localhost:3243".into());
    let maps_dir = match core::get_maps_dir() {
        Ok(d) => d,
        Err(e) => { eprintln!("Maps dir: {}", e); process::exit(1); }
    };
    println!("Maps dir: {}", maps_dir.display());

    let statuses = fetch_map_status();
    let need: Vec<&MapStatus> = statuses.iter().filter(|m| m.status != "current").collect();

    if need.is_empty() {
        println!("All maps up to date.");
        return;
    }

    println!("{} maps to download\n", need.len());

    let rt = tokio::runtime::Runtime::new().unwrap();
    for (i, m) in need.iter().enumerate() {
        let size_mb = m.size as f64 / 1024.0 / 1024.0;
        println!("[{}/{}] {} ({:.1} MB)", i + 1, need.len(), m.name, size_mb);

        let url = format!("{}/maps/download/{}", api, m.name);
        let dest = maps_dir.join(&m.name);
        let last_print = std::cell::Cell::new(std::time::Instant::now());

        let ok = rt.block_on(async {
            core::download_file(&url, &dest, |downloaded, total| {
                if last_print.get().elapsed().as_millis() >= 500 || downloaded == total {
                    let pct = if total > 0 { downloaded * 100 / total } else { 0 };
                    eprint!("\r  {} / {} ({}%)    ", downloaded, total, pct);
                    last_print.set(std::time::Instant::now());
                }
            }).await
        });

        match ok {
            Ok(()) => {
                eprintln!();
                // verify hash
                match core::get_map_hash(&m.name) {
                    Ok(h) if h == m.remote_hash => println!("  OK"),
                    Ok(h) => println!("  HASH MISMATCH: got {} expected {}", h, m.remote_hash),
                    Err(e) => println!("  hash check failed: {}", e),
                }
            }
            Err(e) => {
                eprintln!();
                println!("  FAILED: {}", e);
            }
        }
    }

    println!("\nDone.");
}

fn cmd_download_test(args: &[String]) {
    let url = if let Some(u) = args.first() {
        u.clone()
    } else {
        let api = env::var("API_URL").unwrap_or_else(|_| "http://localhost:13243".into());
        format!("{}/patches/patch-p11.zip", api)
    };

    let dest = std::env::temp_dir().join("download-test.zip");
    println!("URL:  {}", url);
    println!("Dest: {}", dest.display());

    let rt = tokio::runtime::Runtime::new().unwrap();
    let last_print = std::cell::Cell::new(std::time::Instant::now());

    rt.block_on(async {
        // First check headers manually
        let client = reqwest::Client::new();
        let resp = client.head(&url).send().await.unwrap();
        println!("\nHEAD response headers:");
        for (k, v) in resp.headers() {
            println!("  {}: {}", k, v.to_str().unwrap_or("?"));
        }
        println!("  content_length(): {:?}", resp.content_length());

        // Now do the actual download
        println!("\nDownloading...");
        core::download_file(&url, &dest, |downloaded, total| {
            if last_print.get().elapsed().as_millis() >= 500 || downloaded == total {
                let pct = if total > 0 { downloaded * 100 / total } else { 0 };
                println!("  {} / {} ({}%)", downloaded, total, pct);
                last_print.set(std::time::Instant::now());
            }
        }).await.unwrap();
        println!("Done.");
    });

    // Cleanup
    let _ = std::fs::remove_file(&dest);
}
