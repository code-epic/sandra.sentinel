use super::memoria::*;
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
                            // 🔥 CÁLCULO AL VUELO: TIEMPO + SUELDO
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

            println!(
                "✅ [DONE] '{}' completado y calculado en {:?}. Total: {} registros en {} lotes.",
                funcion,
                start_time.elapsed(),
                results.len(),
                chunks
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

            println!(
                "✅ [DONE] '{}' completado en {:?}. Total: {} registros en {} lotes.",
                funcion,
                start_time.elapsed(),
                results.len(),
                chunks
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
                        eprintln!(
                            "⚠️ [WARN] Error deserializando batch JSON en '{}': {}",
                            funcion, e
                        );
                    }
                }
            }
            let total_elapsed = start_time.elapsed();
            println!(
                "✅ [DONE] '{}' completado en {:?}. Total: {} registros en {} lotes.",
                funcion,
                total_elapsed,
                results.len(),
                chunks
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
