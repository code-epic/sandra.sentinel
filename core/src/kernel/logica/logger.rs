use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

// Bandera global para habilitar/deshabilitar logs
static LOG_ENABLED: AtomicBool = AtomicBool::new(false);

// Mutex para escritura segura en archivo (evita condiciones de carrera entre threads)
static LOG_FILE_LOCK: Mutex<()> = Mutex::new(());

pub fn init(enabled: bool) {
    LOG_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        println!("📝 [Logger] Sistema de logs activado. Archivo: sandra_sentinel.log");
        log_system("Sistema de Logs Iniciado");
    }
}

pub fn is_enabled() -> bool {
    LOG_ENABLED.load(Ordering::Relaxed)
}

fn write_log(level: &str, category: &str, message: &str) {
    if !is_enabled() {
        return;
    }

    // Filtro: No registrar INFO de CARGA (reducir ruido, ya está en telemetría)
    if level == "INFO" && category == "CARGA" {
        return;
    }

    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("{} | {:<5} | {:<10} | {}\n", now, level, category, message);

    // Salida a consola (opcional, solo errores o warns)
    if level == "ERROR" {
        eprint!("{}", log_line);
    }

    // Escritura en archivo
    let _guard = LOG_FILE_LOCK.lock().unwrap();
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open("sandra_sentinel.log")
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
