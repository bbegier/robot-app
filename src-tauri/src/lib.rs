use std::process::Command;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[tauri::command]
fn set_secret(key: String, value: String) -> Result<(), String> {
    let entry = keyring::Entry::new("teleop-ui", &key).map_err(|e| e.to_string())?;
    entry.set_password(&value).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_secret(key: String) -> Result<String, String> {
    let entry = keyring::Entry::new("teleop-ui", &key).map_err(|e| e.to_string())?;
    entry.get_password().map_err(|e| e.to_string())
}

#[tauri::command]
fn clear_secret(key: String) -> Result<(), String> {
    let entry = keyring::Entry::new("teleop-ui", &key).map_err(|e| e.to_string())?;
    entry.delete_password().map_err(|e| e.to_string())
}

#[tauri::command]
fn preflight_gstreamer() -> Result<String, String> {
    Command::new("gst-launch-1.0")
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())
        .and_then(|o| String::from_utf8(o.stdout).map_err(|e| e.to_string()))
}

#[tauri::command]
fn preflight_tailscale_status() -> Result<String, String> {
    // Step 0 only needs presence; use `version` and try common macOS paths
    let try_paths = [
        "tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
        "/usr/local/bin/tailscale",
        "/opt/homebrew/bin/tailscale",
    ];
    for p in try_paths {
        let out = Command::new(p).arg("version").output();
        if let Ok(o) = out {
            if o.status.success() {
                return String::from_utf8(o.stdout).map_err(|e| e.to_string());
            }
        }
    }
    Err("tailscale CLI not found".into())
}

#[tauri::command]
fn tailscale_status_json() -> Result<String, String> {
    let try_paths = [
        "tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
        "/usr/local/bin/tailscale",
        "/opt/homebrew/bin/tailscale",
    ];
    for p in try_paths {
        let out = Command::new(p).args(["status", "--json"]).output();
        if let Ok(o) = out {
            if o.status.success() {
                return String::from_utf8(o.stdout).map_err(|e| e.to_string());
            }
        }
    }
    Err("tailscale CLI not found".into())
}

#[tauri::command]
fn tailscale_netcheck_json() -> Result<String, String> {
    let try_paths = [
        "tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
        "/usr/local/bin/tailscale",
        "/opt/homebrew/bin/tailscale",
    ];
    for p in try_paths {
        // Prefer --json; fall back to --format=json for older versions
        let out = Command::new(p).args(["netcheck", "--json"]).output();
        if let Ok(o) = out {
            if o.status.success() {
                return String::from_utf8(o.stdout).map_err(|e| e.to_string());
            }
        }
        let out2 = Command::new(p).args(["netcheck", "--format=json"]).output();
        if let Ok(o2) = out2 {
            if o2.status.success() {
                return String::from_utf8(o2.stdout).map_err(|e| e.to_string());
            }
        }
    }
    Err("tailscale CLI not found".into())
}

#[tauri::command]
fn tailscale_up(auth_key: String, reset: Option<bool>) -> Result<String, String> {
    let try_paths = [
        "tailscale",
        "/Applications/Tailscale.app/Contents/MacOS/Tailscale",
        "/usr/local/bin/tailscale",
        "/opt/homebrew/bin/tailscale",
    ];
    for p in try_paths {
        let mut args = vec!["up", "--authkey", &auth_key];
        if reset.unwrap_or(false) {
            args.insert(1, "--reset");
        }
        let out = Command::new(p).args(&args).output();
        if let Ok(o) = out {
            let mut s = String::new();
            s.push_str(&String::from_utf8_lossy(&o.stdout));
            s.push_str(&String::from_utf8_lossy(&o.stderr));
            if o.status.success() {
                return Ok(s);
            } else {
                return Err(s);
            }
        }
    }
    Err("tailscale CLI not found".into())
}

#[tauri::command]
fn tailscale_quit_gui() -> Result<String, String> {
    let out = Command::new("osascript")
        .args(["-e", "tell application \"Tailscale\" to quit"])
        .output()
        .map_err(|e| e.to_string())?;
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok(s)
}

#[tauri::command]
fn tailscale_install() -> Result<String, String> {
    // Minimal installer: download official pkg and run installer
    let pkg_path = "/tmp/Tailscale.pkg";
    let url = "https://pkgs.tailscale.com/stable/Tailscale.pkg";
    let curl = Command::new("curl")
        .args(["-L", "-o", pkg_path, url])
        .output()
        .map_err(|e| e.to_string())?;
    if !curl.status.success() {
        let mut s = String::new();
        s.push_str(&String::from_utf8_lossy(&curl.stdout));
        s.push_str(&String::from_utf8_lossy(&curl.stderr));
        return Err(format!("download failed: {}", s));
    }
    let inst = Command::new("/usr/sbin/installer")
        .args(["-pkg", pkg_path, "-target", "/"])
        .output()
        .map_err(|e| e.to_string())?;
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&inst.stdout));
    s.push_str(&String::from_utf8_lossy(&inst.stderr));
    if inst.status.success() {
        Ok(s)
    } else {
        Err(s)
    }
}

#[tauri::command]
fn pip_install_pyzmq() -> Result<String, String> {
    let out = Command::new("python3")
        .args(["-m", "pip", "install", "pyzmq"])
        .output()
        .map_err(|e| e.to_string())?;
    let mut s = String::new();
    s.push_str(&String::from_utf8_lossy(&out.stdout));
    s.push_str(&String::from_utf8_lossy(&out.stderr));
    Ok(s)
}

#[tauri::command]
fn preflight_zmq() -> Result<String, String> {
    Command::new("python3")
        .args(["-c", "import zmq,sys;print(zmq.__version__)\n"])
        .output()
        .map_err(|e| e.to_string())
        .and_then(|o| String::from_utf8(o.stdout).map_err(|e| e.to_string()))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            preflight_gstreamer,
            preflight_tailscale_status,
            preflight_zmq,
            pip_install_pyzmq,
            tailscale_status_json,
            tailscale_netcheck_json,
            tailscale_up,
            tailscale_quit_gui,
            tailscale_install,
            install_gstreamer,
            auto_install_all,
            auto_bootstrap_offline,
            get_log_dir,
            get_log_file,
            append_log,
            reveal_log_dir,
            export_logs_to_desktop,
            set_secret,
            get_secret,
            clear_secret
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn logs_dir_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let mut p = PathBuf::from(home);
    p.push("Library/Logs/teleop-ui");
    Ok(p)
}

fn log_file_path() -> Result<PathBuf, String> {
    let mut p = logs_dir_path()?;
    p.push("new_user_debug.log");
    Ok(p)
}

#[tauri::command]
fn get_log_dir() -> Result<String, String> { logs_dir_path().map(|p| p.to_string_lossy().into_owned()) }

#[tauri::command]
fn get_log_file() -> Result<String, String> { log_file_path().map(|p| p.to_string_lossy().into_owned()) }

#[tauri::command]
fn append_log(line: String) -> Result<(), String> {
    let dir = logs_dir_path()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file = log_file_path()?;
    let mut f = OpenOptions::new().create(true).append(true).open(&file).map_err(|e| e.to_string())?;
    let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    let entry = format!("[{}] {}\n", ts, line);
    f.write_all(entry.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
fn reveal_log_dir() -> Result<(), String> {
    let dir = logs_dir_path()?;
    Command::new("open").arg(dir).output().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
fn export_logs_to_desktop() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let mut dest = PathBuf::from(&home);
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    dest.push(format!("Desktop/new_user_debug_{}.log", ts));
    let src = log_file_path()?;
    if src.exists() {
        fs::copy(&src, &dest).map_err(|e| e.to_string())?;
    } else {
        fs::create_dir_all(dest.parent().unwrap()).ok();
        fs::write(&dest, b"(empty)").map_err(|e| e.to_string())?;
    }
    Ok(dest.to_string_lossy().into_owned())
}

fn which(cmd: &str) -> bool {
    Command::new("/usr/bin/which").arg(cmd).output().map(|o| o.status.success()).unwrap_or(false)
}

fn brew_path() -> Option<String> {
    if std::path::Path::new("/opt/homebrew/bin/brew").exists() { return Some("/opt/homebrew/bin/brew".into()); }
    if std::path::Path::new("/usr/local/bin/brew").exists() { return Some("/usr/local/bin/brew".into()); }
    if which("brew") { return Some("brew".into()); }
    None
}

#[tauri::command]
fn install_gstreamer() -> Result<String, String> {
    // If already present, return
    if Command::new("gst-launch-1.0").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return Ok("gstreamer already present".into());
    }
    // Try Homebrew
    let brew = match brew_path() { Some(p) => p, None => {
        // Attempt non-interactive install of Homebrew
        let script = "https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh";
        let out = Command::new("/bin/bash")
            .env("NONINTERACTIVE", "1")
            .args(["-c", &format!("/bin/bash -c \"$(curl -fsSL {})\"", script)])
            .output()
            .map_err(|e| e.to_string())?;
        if !out.status.success() {
            let mut s = String::new(); s.push_str(&String::from_utf8_lossy(&out.stdout)); s.push_str(&String::from_utf8_lossy(&out.stderr));
            return Err(format!("homebrew install failed: {}", s));
        }
        brew_path().ok_or_else(|| "brew not found after install".to_string())?
    }};
    // Install gstreamer and plugins
    let pkgs = ["gstreamer", "gst-plugins-base", "gst-plugins-good", "gst-plugins-bad", "gst-libav"];
    let mut log = String::new();
    let upd = Command::new(&brew).args(["update"]).output().map_err(|e| e.to_string())?;
    log.push_str(&String::from_utf8_lossy(&upd.stdout)); log.push_str(&String::from_utf8_lossy(&upd.stderr));
    for p in pkgs {
        let out = Command::new(&brew).args(["install", p]).output().map_err(|e| e.to_string())?;
        log.push_str(&String::from_utf8_lossy(&out.stdout));
        log.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    Ok(log)
}

#[tauri::command]
fn auto_install_all() -> Result<String, String> {
    let mut log = String::new();
    // tailscale if missing
    let ts_ok = Command::new("tailscale").arg("version").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("/Applications/Tailscale.app/Contents/MacOS/Tailscale").arg("version").output().map(|o| o.status.success()).unwrap_or(false);
    if !ts_ok {
        match tailscale_install() { Ok(s) => { log.push_str("tailscale_install ok\n"); log.push_str(&s); }, Err(e) => { log.push_str("tailscale_install err: "); log.push_str(&e); log.push('\n'); } }
    }
    match install_gstreamer() { Ok(s) => { log.push_str("gstreamer ok\n"); log.push_str(&s); }, Err(e) => { log.push_str("gstreamer err: "); log.push_str(&e); log.push('\n'); } }
    match pip_install_pyzmq() { Ok(s) => { log.push_str("pyzmq ok\n"); log.push_str(&s); }, Err(e) => { log.push_str("pyzmq err: "); log.push_str(&e); log.push('\n'); } }
    Ok(log)
}

fn current_app_resources_dir() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    // macOS bundle: My.app/Contents/MacOS/teleop-ui → Resources is ../Resources
    let macos_dir = exe.parent().ok_or_else(|| "no parent".to_string())?;
    let contents_dir = macos_dir.parent().ok_or_else(|| "no contents".to_string())?;
    let mut res = contents_dir.to_path_buf();
    res.push("Resources");
    Ok(res)
}

fn find_pkg_in(dir: &PathBuf, prefix: &str) -> Option<PathBuf> {
    if let Ok(rd) = fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if let Some(ext) = p.extension() { if ext == "pkg" {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    if name.to_lowercase().contains(&prefix.to_lowercase()) { return Some(p); }
                }
            }}
        }
    }
    None
}

#[tauri::command]
fn auto_bootstrap_offline() -> Result<String, String> {
    let mut log = String::new();
    // Bootstrap guard
    let mut support = PathBuf::from(std::env::var("HOME").map_err(|e| e.to_string())?);
    support.push("Library/Application Support/Teleop");
    fs::create_dir_all(&support).ok();
    let mut marker = support.clone(); marker.push("bootstrap_done");
    if marker.exists() { return Ok("already_bootstrapped".into()); }

    let res = current_app_resources_dir()?;
    // Tailscale
    let ts_ok = Command::new("tailscale").arg("version").output().map(|o| o.status.success()).unwrap_or(false)
        || Command::new("/Applications/Tailscale.app/Contents/MacOS/Tailscale").arg("version").output().map(|o| o.status.success()).unwrap_or(false);
    if !ts_ok {
        let mut ts_dir = res.clone(); ts_dir.push("resources/tailscale");
        if let Some(pkg) = find_pkg_in(&ts_dir, "tailscale") {
            let out = Command::new("/usr/sbin/installer").args(["-pkg", pkg.to_string_lossy().as_ref(), "-target", "/"]).output().map_err(|e| e.to_string())?;
            log.push_str(&String::from_utf8_lossy(&out.stdout)); log.push_str(&String::from_utf8_lossy(&out.stderr));
        } else {
            log.push_str("tailscale pkg not bundled\n");
        }
    }
    // GStreamer
    let gst_ok = Command::new("gst-launch-1.0").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    if !gst_ok {
        let mut gs_dir = res.clone(); gs_dir.push("resources/gstreamer");
        if let Some(pkg) = find_pkg_in(&gs_dir, "gstreamer") {
            let out = Command::new("/usr/sbin/installer").args(["-pkg", pkg.to_string_lossy().as_ref(), "-target", "/"]).output().map_err(|e| e.to_string())?;
            log.push_str(&String::from_utf8_lossy(&out.stdout)); log.push_str(&String::from_utf8_lossy(&out.stderr));
        } else { log.push_str("gstreamer pkg not bundled\n"); }
    }
    // Python venv + pyzmq wheel
    let mut venv = support.clone(); venv.push("venv");
    if !venv.exists() {
        let out = Command::new("python3").args(["-m", "venv", venv.to_string_lossy().as_ref()]).output().map_err(|e| e.to_string())?;
        log.push_str(&String::from_utf8_lossy(&out.stdout)); log.push_str(&String::from_utf8_lossy(&out.stderr));
    }
    let mut wheels = res.clone(); wheels.push("resources/python-wheels");
    // Pick first pyzmq wheel
    let mut wheel_path: Option<PathBuf> = None;
    if let Ok(rd) = fs::read_dir(&wheels) { for e in rd.flatten() { let p = e.path(); if let Some(name) = p.file_name().and_then(|s| s.to_str()) { if name.to_lowercase().starts_with("pyzmq") && p.extension().map(|e| e=="whl").unwrap_or(false) { wheel_path = Some(p); break; } } } }
    if let Some(whl) = wheel_path {
        let mut pip = venv.clone(); pip.push("bin/pip");
        let out = Command::new(pip).args(["install", whl.to_string_lossy().as_ref()]).output().map_err(|e| e.to_string())?;
        log.push_str(&String::from_utf8_lossy(&out.stdout)); log.push_str(&String::from_utf8_lossy(&out.stderr));
    } else { log.push_str("pyzmq wheel not bundled\n"); }

    fs::write(&marker, b"ok").ok();
    Ok(log)
}
