use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static LOG_ENABLED: AtomicBool = AtomicBool::new(false);
static LOG_FILE_LOCK: Mutex<()> = Mutex::new(());
static LOG_DESTINO: Mutex<String> = Mutex::new(String::new());

pub fn init(enabled: bool, destino: &str) {
    LOG_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        if let Ok(mut dest) = LOG_DESTINO.lock() {
            *dest = destino.to_string();
        }
        println!("[Logger] Sistema de logs activado. Archivo: sandra_sentinel.log");
        log_system("Sistema de Logs Iniciado");
    }
}

pub fn is_enabled() -> bool {
    LOG_ENABLED.load(Ordering::Relaxed)
}

fn get_log_path() -> String {
    if let Ok(dest) = LOG_DESTINO.lock() {
        if dest.is_empty() || *dest == "." {
            return "sandra_sentinel.log".to_string();
        }
        return format!("{}/sandra_sentinel.log", dest);
    }
    "sandra_sentinel.log".to_string()
}

fn write_log(level: &str, category: &str, message: &str) {
    if !is_enabled() {
        return;
    }

    if level == "INFO" && category == "CARGA" {
        return;
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("{} | {:<5} | {:<10} | {}\n", now, level, category, message);

    if level == "ERROR" {
        eprint!("{}", log_line);
    }

    let _guard = LOG_FILE_LOCK.lock().unwrap();
    let log_path = get_log_path();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&log_path)
    {
        let _ = file.write_all(log_line.as_bytes());
    }
}

pub fn log_error(category: &str, message: &str) {
    write_log("ERROR", category, message);
}

pub fn log_warn(category: &str, message: &str) {
    write_log("WARN", category, message);
}

pub fn log_info(category: &str, message: &str) {
    write_log("INFO", category, message);
}

pub fn log_system(message: &str) {
    write_log("SYSTEM", "CORE", message);
}
