use super::memoria::*;
use crate::kernel::logica::logger;
use crate::kernel::sandra::sentinel_dynamic_service_client::SentinelDynamicServiceClient;
use crate::kernel::sandra::DynamicRequest;
use tonic::transport::Channel;

#[derive(Debug)]
pub struct Cargador {
    pub client: Option<SentinelDynamicServiceClient<Channel>>,
}

impl Cargador {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub async fn connect(
        &mut self,
        url: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = SentinelDynamicServiceClient::connect(url).await?;
        self.client = Some(client);
        Ok(())
    }

    // --- CARGA DE REFERENCIAS ---

    pub async fn cargar_directiva(
        &mut self,
    ) -> Result<Vec<Directiva>, Box<dyn std::error::Error + Send + Sync>> {
        // En tu arquitectura, esto podría ser un 'Ejecutar' simple o stream
        self.fetch_stream("IPSFA_CDirectiva").await
    }

    pub async fn cargar_primas_funciones(
        &mut self,
    ) -> Result<Vec<PrimaFuncion>, Box<dyn std::error::Error + Send + Sync>> {
        self.fetch_stream("IPSFA_CPrimasFunciones").await
    }

    pub async fn cargar_conceptos(
        &mut self,
    ) -> Result<Vec<Concepto>, Box<dyn std::error::Error + Send + Sync>> {
        // self.fetch_stream("IPSFA_CConceptos").await
        self.fetch_stream("IPSFA_CConceptos").await
    }

    pub async fn cargar_movimientos(
        &mut self,
    ) -> Result<Vec<Movimiento>, Box<dyn std::error::Error + Send + Sync>> {
        self.fetch_stream("IPSFA_CMovimientos").await
    }

    pub async fn cargar_base(
        &mut self,
        directivas: &Vec<Directiva>,
        engine: &crate::calc::motor::SentinelEngine,
    ) -> Result<Vec<Base>, Box<dyn std::error::Error + Send + Sync>> {
        let funcion = "IPSFA_CBase";
        println!("🚀 [INIT] Iniciando carga inteligente para: '{}'", funcion);

        if let Some(client) = &mut self.client {
            let request = tonic::Request::new(DynamicRequest {
                funcion: funcion.to_string(),
                parametros: "\"%\"".to_string(),
                valores: "null".to_string(),
            });

            let start_time = std::time::Instant::now();
            let mut stream = client.execute_dynamic(request).await?.into_inner();

            let mut results = Vec::with_capacity(50000); // Pre-allocate memory estimate
            let mut chunks = 0;

            while let Some(msg) = stream.message().await? {
                chunks += 1;
                // Deserializar array completo de bytes (JSON)
                match serde_json::from_slice::<Vec<Base>>(&msg.rows) {
                    Ok(items) => {
                        for mut item in items {
                            // 1. 🔥 CÁLCULO PREVIO: TIEMPO + SUELDO BASE (Requisito para el motor)
                            crate::calc::procesar_registro_base(&mut item, directivas);
                            results.push(item);
                        }
                    }
                    Err(e) => {
                        if results.len() == 0 && chunks <= 5 {
                            eprintln!("⚠️ [Base Error] Deserializing batch: {}", e);
                        }
                    }
                }
            }

            // 2. ⚡️ INVOCACIÓN DEL MOTOR SENTINEL (Cálculo de Nómina Masivo)
            println!(
                "⚙️ [SentinelEngine] Procesando nómina para {} registros...",
                results.len()
            );

            // El motor usa Rayon internamente para calcular en paralelo
            let calculos = engine.calcular_nomina(&results);

            // 3. 💾 FUSIÓN DE RESULTADOS (Map-Reduce: Volcar cálculos al struct Base)
            // Optimizamos creando un mapa temporal para acceso rápido por patrón/key
            let mapa_calculos: std::collections::HashMap<_, _> = calculos.into_iter().collect();

            let mut match_count = 0;
            let mut count_zeros_primas = 0;
            for base in &mut results {
                // Usamos patterns como clave de enlace según tu lógica en motor.rs
                if let Some(valores) = mapa_calculos.get(&base.patterns) {
                    match_count += 1;

                    // A) ALMACENAMIENTO DINÁMICO (Único y Definitivo)
                    base.calculos = Some(valores.clone());

                    // 2. Calcular Total Asignaciones
                    let sum_primas: f64 = valores.values().sum();
                    base.total_asignaciones = base.sueldo_base + sum_primas;

                    // Integridad: Si tiene sueldo pero 0 primas, es sospechoso
                    if base.sueldo_base > 0.0 && sum_primas == 0.0 {
                        count_zeros_primas += 1;
                    }
                }
            }

            if count_zeros_primas > 0 {
                logger::log_warn(
                    "CALCULO",
                    &format!(
                        "Atención: {} registros tienen Sueldo Base pero 0.0 en Primas calculadas.",
                        count_zeros_primas
                    ),
                );
            }

            println!(
                "✅ [DONE] '{}' completado. Base: {:?} | Motor: {} procesados. Total tiempo: {:?}",
                funcion,
                start_time.elapsed(),
                match_count,
                start_time.elapsed()
            );
            logger::log_info(
                "CARGA",
                &format!(
                    "'{}' completado. Base: {:?} registros. Motor: {} procesados. Tiempo: {:?}",
                    funcion,
                    results.len(),
                    match_count,
                    start_time.elapsed()
                ),
            );
            // Telemetría
            crate::kernel::logica::telemetria::record(
                "CARGA",
                funcion,
                start_time.elapsed(),
                results.len(),
                &format!("Lotes: {}", chunks),
            );
            Ok(results)
        } else {
            Err("Cliente no conectado".into())
        }
    }

    pub async fn cargar_beneficiarios(
        &mut self,
        bases: &Vec<Base>,
        movimientos: &Vec<Movimiento>,
    ) -> Result<Vec<Beneficiario>, Box<dyn std::error::Error + Send + Sync>> {
        let funcion = "IPSFA_CBeneficiarios";
        println!("🚀 [INIT] Iniciando carga FUSIONADA para: '{}'", funcion);

        // 1. Indexar Base y Movimientos para búsqueda rápida
        println!("   - Indexando {} registros Base...", bases.len());
        let mut map_base: std::collections::HashMap<String, &Base> =
            std::collections::HashMap::with_capacity(bases.len());
        for b in bases {
            if !b.patterns.is_empty() {
                map_base.insert(b.patterns.clone(), b);
            }
        }

        println!(
            "   - Indexando {} registros Movimientos...",
            movimientos.len()
        );
        let mut map_mov: std::collections::HashMap<String, Vec<Movimiento>> =
            std::collections::HashMap::new();
        for m in movimientos {
            map_mov.entry(m.cedula.clone()).or_default().push(m.clone());
        }

        if let Some(client) = &mut self.client {
            let request = tonic::Request::new(DynamicRequest {
                funcion: funcion.to_string(),
                parametros: "\"%\"".to_string(),
                valores: "null".to_string(),
            });

            let start_time = std::time::Instant::now();
            let mut stream = client.execute_dynamic(request).await?.into_inner();

            let size_aprox = 120_000;
            let mut results = Vec::with_capacity(size_aprox);
            let mut chunks = 0;
            let mut huerfanos_count = 0; // Contador de integridad

            let mut t_last = std::time::Instant::now();
            let mut net_time = std::time::Duration::new(0, 0);

            // Tareas de Deserialización (Pipeline)
            let mut tasks = Vec::new();

            while let Some(msg) = stream.message().await? {
                net_time += t_last.elapsed();
                chunks += 1;

                // Spawnear tarea de CPU (Parsing JSON masivo)
                // Ahora msg.rows es Vec<u8> (JSON Array bytes)
                let rows_data = msg.rows;
                let task = tokio::spawn(async move {
                    if rows_data.is_empty() {
                        return Vec::new();
                    }
                    // Deserializamos el array completo de golpe
                    match serde_json::from_slice::<Vec<Beneficiario>>(&rows_data) {
                        Ok(items) => items,
                        Err(e) => {
                            eprintln!("⚠️ Error deserializando batch JSON (Beneficiarios): {}", e);
                            Vec::new()
                        }
                    }
                });

                tasks.push(task);
                t_last = std::time::Instant::now();
            }

            println!(
                "   -> Descarga completada (Red: {:.2?}). Procesando fusión...",
                net_time
            );

            // Recolectar y Fusionar (Main Thread)
            for task in tasks {
                match task.await {
                    Ok(batch_items) => {
                        for mut item in batch_items {
                            // --- FUSIÓN ---
                            // 1. Unir con Base por patterns
                            if !item.patterns.is_empty() {
                                if let Some(base_encontrada) = map_base.get(&item.patterns) {
                                    item.base = (*base_encontrada).clone();
                                } else {
                                    huerfanos_count += 1;
                                }
                            }

                            // 2. Unir con Movimiento por cedula
                            if let Some(movs_encontrados) = map_mov.get(&item.cedula) {
                                if let Some(ultimo_mov) = movs_encontrados.last() {
                                    item.movimientos = ultimo_mov.clone();
                                }
                            }

                            results.push(item);
                        }
                    }
                    Err(e) => eprintln!("⚠️ Error en tarea de parsing: {}", e),
                }
            }

            if huerfanos_count > 0 {
                logger::log_warn("INTEGRIDAD", &format!("Detectados {} beneficiarios sin registro Base asociado (Posible inconsistencia)", huerfanos_count));
            }

            let msg_done = format!(
                "'{}' completado en {:?}. Total: {} registros en {} lotes.",
                funcion,
                start_time.elapsed(),
                results.len(),
                chunks
            );
            println!("✅ [DONE] {}", msg_done);
            logger::log_info("CARGA", &msg_done);

            // Telemetría
            crate::kernel::logica::telemetria::record(
                "CARGA",
                funcion,
                start_time.elapsed(),
                results.len(),
                &format!("Lotes: {}", chunks),
            );

            Ok(results)
        } else {
            Err("Cliente no conectado".into())
        }
    }

    // --- HELPER GENÉRICO PARA STREAM ---

    async fn fetch_stream<T: serde::de::DeserializeOwned>(
        &mut self,
        funcion: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 [INIT] Iniciando stream para: '{}'", funcion);

        if let Some(client) = &mut self.client {
            let request = tonic::Request::new(DynamicRequest {
                funcion: funcion.to_string(),
                parametros: "\"%\"".to_string(),
                valores: "null".to_string(),
            });

            let start_time = std::time::Instant::now();
            // Usamos el stream
            let mut stream = client.execute_dynamic(request).await?.into_inner();
            // let elapsed = start_time.elapsed();
            // println!(
            //     "    ⏱️  [{}] Conexión establecida en {:?}",
            //     funcion, elapsed
            // );

            let mut results = Vec::new();
            let mut chunks = 0;

            let mut json_error_logged = false;

            while let Some(msg) = stream.message().await? {
                chunks += 1;
                // msg.rows es Vec<u8> (JSON Array)
                if msg.rows.is_empty() {
                    continue;
                }

                match serde_json::from_slice::<Vec<T>>(&msg.rows) {
                    Ok(items) => {
                        results.extend(items);
                    }
                    Err(e) => {
                        if !json_error_logged {
                            logger::log_error(
                                "JSON",
                                &format!("Error deserializando batch en '{}': {}", funcion, e),
                            );
                            json_error_logged = true;
                        }
                        eprintln!(
                            "⚠️ [WARN] Error deserializando batch JSON en '{}': {}",
                            funcion, e
                        );
                    }
                }
            }

            if results.is_empty() {
                logger::log_warn(
                    "DATA",
                    &format!("Servicio '{}' devolvió 0 registros", funcion),
                );
            }

            let total_elapsed = start_time.elapsed();
            let msg_done = format!(
                "'{}' completado en {:?}. Total: {} registros en {} lotes.",
                funcion,
                total_elapsed,
                results.len(),
                chunks
            );
            println!("✅ [DONE] {}", msg_done);
            logger::log_info("CARGA", &msg_done);

            // Telemetría
            crate::kernel::logica::telemetria::record(
                "CARGA",
                funcion,
                total_elapsed,
                results.len(),
                &format!("Lotes: {}", chunks),
            );

            Ok(results)
        } else {
            println!(
                "❌ [ERROR] Intento de carga '{}' fallido: Cliente no conectado",
                funcion
            );
            Err("Cliente no conectado".into())
        }
    }
}
