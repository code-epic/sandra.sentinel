# Optimización de Código Rust — Sandra Sentinel
## Patrones Avanzados para Dominio Financiero-Crítico

> Documento técnico de referencia para ingenieros del ecosistema Sandra.
> Última actualización: 2026-06-02.

---

## 1. Aritmética Monetaria con `rust_decimal::Decimal`

### Prohibición

Los tipos `f32` y `f64` están **prohibidos** en todo módulo que opere montos, salarios, primas, alícuotas o retroactivos.

### Rationale

- `0.1 + 0.2 != 0.3` en IEEE-754.
- La acumulación de errores de redondeo en ~500.000 registros genera discrepancias legales detectables por la Contraloría.

### Patrón Idiomático

```rust
use rust_decimal::Decimal;
use rust_decimal::prelude::*;

// Parsing desde string (exacto)
let sueldo_base = Decimal::from_str("1250.00")?;
let prima_antiguedad = Decimal::from_str("87.50")?;

// Operaciones centesimales exactas
let sueldo_integral = sueldo_base + prima_antiguedad;
assert_eq!(sueldo_integral, Decimal::from_str("1337.50")?);

// División con redondeo explícito (half-up, 2 decimales)
let alicuota = (sueldo_integral * Decimal::from(120))
    .checked_div(Decimal::from(360))
    .unwrap()
    .round_dp(2);

// Serialización a PostgreSQL sin pérdida
// El driver `rust-postgres` mapea Decimal ↔ NUMERIC nativamente.
```

### Anti-patrón

```rust
let total: f64 = 1250.00 + 87.50;  // ❌ Prohibido en Sentinel
```

---

## 2. Zero-Copy Parsing para CSV Masivos

### Problema

Archivos de nómina con 500.000+ filas exigen lectura sin copias de buffer.

### Solución: `memmap2` + `csv` crate

```rust
use memmap2::Mmap;
use std::fs::File;

pub fn mmap_file(path: &str) -> io::Result<Mmap> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}

// El parser `csv::Reader` opera directamente sobre &[u8] del mmap.
// Zero allocations para el buffer de lectura.
```

### Ventajas

- El kernel mapea el archivo a páginas de memoria virtual; no se duplica en el heap del proceso.
- El parser avanza por el slice sin `memcpy`.
- El sistema operativo descarga páginas bajo presión de memoria automáticamente.

### Pipeline de Índice

```rust
// Fase Build: construir HashMap en O(N)
let csv_index: HashMap<String, CsvRecord> = build_index(&mmap, ';', true)?;

// Fase Probe: búsqueda O(1) por cédula normalizada
if let Some(record) = csv_index.get(&cedula_norm) { ... }
```

---

## 3. Streaming Concurrente con `tokio` + `tonic`

### Patrón: Productor-Consumidor con Back-pressure

```rust
use tokio::sync::mpsc;
use tonic::Streaming;

const CHANNEL_CAPACITY: usize = 64; // Limita memoria bajo burst

let (tx, mut rx) = mpsc::channel::<Vec<u8>>(CHANNEL_CAPACITY);

// Task productora: recibe del gRPC, emite bytes crudos
tokio::spawn(async move {
    while let Some(msg) = stream.message().await? {
        if tx.send(msg.rows).await.is_err() { break; }
    }
});

// Task consumidora: parsea NDJSON en paralelo
while let Some(rows_data) = rx.recv().await {
    let items: Vec<Value> = serde_json::from_slice(&rows_data)?;
    // procesar...
}
```

### gRPC Max Decoding Size

```rust
let mut client = SentinelDynamicServiceClient::connect(url).await?
    .max_decoding_message_size(usize::MAX);  // Aceptar payloads > 4 MiB
```

### Tolerancia a Fallos en Stream

```rust
while let Ok(Some(msg)) = stream.message().await {
    // ...
} else {
    // Stream roto: modo degradado, procesar CSV como huérfanos.
    eprintln!("[WARN] Stream gRPC interrumpido. Continuando con CSV.");
}
```

---

## 4. Concurrencia Determinista con Tipos Atómicos

### Métricas en Tiempo Real

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug)]
pub struct LiveMetrics {
    pub records_matched: AtomicU64,
    pub records_partial: AtomicU64,
    pub records_not_found_csv: AtomicU64,
    pub records_not_found_grpc: AtomicU64,
}

// Multiples tareas tokio actualizan sin Mutex
metrics.records_matched.fetch_add(1, Ordering::Relaxed);
```

### Por qué `Relaxed` es seguro aquí

Las métricas son **monótonas** (solo incrementan). No requieren ordenamiento de memoria entre threads porque no hay dependencias de lectura-escritura cruzadas. Esto elimina el costo de barreras de memoria en x86_64 y ARM64.

---

## 5. Circuit Breaker para Fórmulas Rhai

### Contexto

El motor de cálculo ejecuta fórmulas de nómina en tiempo de ejecución. Una fórmula mal escrita no debe detener el procesamiento de 500.000 registros.

### Implementación

```rust
use std::sync::atomic::{AtomicBool, Ordering};

static FORMULA_DISABLED: AtomicBool = AtomicBool::new(false);

fn evaluar_formula(scope: &mut rhai::Scope, formula: &str) -> Option<Decimal> {
    if FORMULA_DISABLED.load(Ordering::Acquire) {
        return None;  // Circuit breaker abierto
    }

    match engine.eval_expression_with_scope(scope, formula) {
        Ok(val) => Some(val),
        Err(e) => {
            log::error!("Fórmula fallida: {}. Contexto: {:?}", e, scope);
            FORMULA_DISABLED.store(true, Ordering::Release);
            None
        }
    }
}
```

---

## 6. Normalización de Fechas sin Allocaciones

```rust
use chrono::NaiveDate;

// Truncar ISO-8601 a YYYY-MM-DD sin heap allocation
fn truncate_iso_date(src: &str) -> &str {
    src.split('T').next().unwrap_or(src)
}

let fecha: NaiveDate = NaiveDate::parse_from_str(
    truncate_iso_date("2024-07-05T00:00:00Z"),
    "%Y-%m-%d"
)?;
```

---

## 7. Checksums Criptográficos para Inmutabilidad

```rust
use blake3::Hasher;

fn fingerprint_dataset(data: &[u8]) -> String {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize().to_hex().to_string()
}

// Cada archivo de salida se acompaña de su BLAKE3 en resumen-procesos.toml
```

---

## Referencias

- [rust_decimal](https://docs.rs/rust_decimal) — Aritmética decimal de precisión arbitraria.
- [memmap2](https://docs.rs/memmap2) — Memory-mapped I/O.
- [tokio](https://tokio.rs) — Runtime asíncrono.
- [tonic](https://github.com/hyperium/tonic) — gRPC para Rust.
- [BLAKE3](https://github.com/BLAKE3-team/BLAKE3) — Hashing paralelo y verificable.
