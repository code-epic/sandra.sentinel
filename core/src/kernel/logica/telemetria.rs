use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;

static SENSORS_ENABLED: AtomicBool = AtomicBool::new(false);
static METRICS_STORE: Mutex<Vec<Metric>> = Mutex::new(Vec::new());

#[derive(Debug, Clone)]
struct Metric {
    category: String, // "CARGA", "MOTOR", "EXPORT", etc.
    name: String,     // "IPSFA_CBase", "Cálculo Nómina"
    duration: Duration,
    records: usize,
    extra_info: String, // Ejemplo: "2 lotes", "15MB"
}

pub fn init(enabled: bool) {
    SENSORS_ENABLED.store(enabled, Ordering::Relaxed);
    if enabled {
        println!("📡 [Sensors] Telemetría activada. Se generará reporte al finalizar.");
    }
}

pub fn is_enabled() -> bool {
    SENSORS_ENABLED.load(Ordering::Relaxed)
}

pub fn record(category: &str, name: &str, duration: Duration, records: usize, extra: &str) {
    if !is_enabled() {
        return;
    }

    let m = Metric {
        category: category.to_string(),
        name: name.to_string(),
        duration,
        records,
        extra_info: extra.to_string(),
    };

    if let Ok(mut store) = METRICS_STORE.lock() {
        store.push(m);
    }
}

pub fn generate_report() {
    if !is_enabled() {
        return;
    }

    let report_path = "sandra_metrics_report.txt";
    println!("📊 Generando reporte de sensores en '{}'...", report_path);

    if let Ok(store) = METRICS_STORE.lock() {
        let mut file = File::create(report_path).expect("No se pudo crear reporte");

        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(file, "SANDRA SENTINEL - REPORTE DE TELEMETRIA").unwrap();
        writeln!(file, "Fecha: {}\n", now).unwrap();

        writeln!(file, "DESGLOSE DE OPERACIONES").unwrap();
        writeln!(file, "{:-<140}", "").unwrap();
        writeln!(
            file,
            "{:<25} {:<10} {:<8} {:<10} {:<12} {:<12} {}",
            "OPERACION", "CATEGORIA", "ESTATUS", "T-SEG", "TIEMPO", "REGISTROS", "DETALLES"
        )
        .unwrap();
        writeln!(file, "{:-<140}", "").unwrap();

        let mut total_time = Duration::new(0, 0);
        let mut max_time = Duration::new(0, 0);
        let mut max_op = "";
        let mut last_cat = String::new();

        for m in store.iter() {
            // Separador si cambia de categoría (ej: de CARGA a SISTEMA)
            if !last_cat.is_empty() && last_cat == "CARGA" && m.category != "CARGA" {
                writeln!(file, "{:-<140}", "").unwrap();
            }
            last_cat = m.category.clone();

            total_time += m.duration;
            if m.duration > max_time {
                max_time = m.duration;
                max_op = &m.name;
            }

            let seg = m.duration.as_secs_f64();
            let status = if m.records > 0 { "OK" } else { "FALLO" };

            writeln!(
                file,
                "{:<25} {:<10} {:<8} {:<10.4} {:<12?} {:<12} {}",
                m.name, m.category, status, seg, m.duration, m.records, m.extra_info
            )
            .unwrap();
        }

        writeln!(file, "\n{:-<140}", "").unwrap();
        writeln!(file, "RESUMEN ESTADISTICO").unwrap();
        writeln!(file, "Tiempo Total Acumulado : {:.2?}", total_time).unwrap();
        writeln!(
            file,
            "Operación más lenta    : {} ({:.2?})",
            max_op, max_time
        )
        .unwrap();

        println!("✨ Reporte generado exitosamente.");
    }
}
