// use serde_json::Value; // Asegúrate de tener serde_json en Cargo.toml
use tonic::{transport::Channel, Request};
// use prost_types::Struct;

// Importamos el código generado por tonic
pub mod sandra {
    tonic::include_proto!("sandra.sentinel.v1");
}

use sandra::sentinel_dynamic_service_client::SentinelDynamicServiceClient;
use sandra::{DynamicRequest, DynamicResponse};

pub mod logica;

use logica::cargador;
use logica::memoria;

// El "Perceptrón" (Cache/Memoization)
#[derive(Debug)]
pub struct Perceptron {
    // Almacenamos el cliente gRPC para realizar llamadas
    pub client: Option<SentinelDynamicServiceClient<Channel>>,

    // Memoria de Trabajo
    pub directiva: Vec<memoria::Directiva>,
    pub primas_funciones: Vec<memoria::PrimaFuncion>,
    pub conceptos: Vec<memoria::Concepto>,
    pub base: Vec<memoria::Base>,
    pub movimientos: Vec<memoria::Movimiento>,
    pub beneficiarios: Vec<memoria::Beneficiario>,
}

impl Default for Perceptron {
    fn default() -> Self {
        Perceptron {
            client: None,
            directiva: Vec::new(),
            primas_funciones: Vec::new(),
            conceptos: Vec::new(),
            base: Vec::new(),
            movimientos: Vec::new(),
            beneficiarios: Vec::new(),
        }
    }
}

impl Perceptron {
    pub fn new() -> Self {
        Perceptron::default()
    }

    /// Conecta al servidor de Sandra (Golang)
    pub async fn connect_to_sandra(
        &mut self,
        url: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Conectando a Sandra Server en {}...", url);
        // Aumentar limite de mensaje para recibir batches grandes (ej: 10k filas)
        let client = SentinelDynamicServiceClient::connect(url)
            .await?
            .max_decoding_message_size(usize::MAX); // Sin limite practico
        self.client = Some(client);
        println!("Conexión establecida con Sandra Server.");
        Ok(())
    }

    pub async fn solicitar_ejecucion(
        &mut self,
        funcion: String,
        parametros: String,
        valores: String,
    ) -> Result<tonic::Streaming<DynamicResponse>, Box<dyn std::error::Error + Send + Sync>> {
        // ... (misma lógica existente) ...
        if let Some(client) = &mut self.client {
            let request = Request::new(DynamicRequest {
                // query_id: "".to_string(),
                funcion,
                parametros,
                valores,
            });
            let response = client.execute_dynamic(request).await?;
            Ok(response.into_inner())
        } else {
            Err("No hay conexión".into())
        }
    }

    /// Orquestador Principal del Ciclo de Nómina
    pub async fn ejecutar_ciclo_carga(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!(">>> INICIANDO CICLO DE CARGA DE NÓMINA <<<");
        let client = self.client.clone().ok_or("Cliente gRPC no conectado")?;

        // 1. Cargar Directiva y Primas (Fase Inicial)
        println!("1. Cargando Directiva y Primas Funciones (Paralelo)...");

        let c_dir_client = client.clone();
        let c_primas_client = client.clone();

        let task_directiva = tokio::spawn(async move {
            let mut c = cargador::Cargador {
                client: Some(c_dir_client),
            };
            c.cargar_directiva().await
        });

        let task_primas = tokio::spawn(async move {
            let mut c = cargador::Cargador {
                client: Some(c_primas_client),
            };
            c.cargar_primas_funciones().await
        });

        let (res_dir, res_primas) = tokio::join!(task_directiva, task_primas);

        // Unwrap de JoinError y luego de Result
        self.directiva = res_dir??;
        self.primas_funciones = res_primas??;

        // --- INSTANCIAR MOTOR (Fase 1.5) ---
        // Creamos el motor UNA VEZ con las primas cargadas
        println!("1.5. Inicializando SentinelEngine...");
        // Clonamos las primas porque el motor toma ownership o hacemos un clone explícito
        let motor = crate::calc::motor::SentinelEngine::new(self.primas_funciones.clone());
        // Envolvemos en Arc para compartirlo thread-safe con la tarea asíncrona de Base
        let motor_arc = std::sync::Arc::new(motor);

        println!(
            "   ✅ Directiva: {} | Primas: {} | Motor Listo",
            self.directiva.len(),
            self.primas_funciones.len()
        );

        // 2. Carga Paralela (Base, Conceptos, Movimientos)
        println!("2. Iniciando Carga Paralela (Base, Conceptos, Movimientos)...");

        // Clones para tareas
        let c1 = client.clone();
        let c2 = client.clone();
        let c3 = client.clone();

        // Clonamos recursos para pasar a threads
        let directivas_clone = self.directiva.clone();
        let motor_ref = motor_arc.clone(); // Arc clone es barato

        let task_base = tokio::spawn(async move {
            let mut c = cargador::Cargador { client: Some(c1) };
            // Pasamos referencia a la copia local de directivas y al motor
            c.cargar_base(&directivas_clone, &motor_ref).await
        });

        let task_conc = tokio::spawn(async move {
            let mut c = cargador::Cargador { client: Some(c2) };
            c.cargar_conceptos().await
        });

        let task_mov = tokio::spawn(async move {
            let mut c = cargador::Cargador { client: Some(c3) };
            c.cargar_movimientos().await
        });

        // Esperar a todos
        let (res_base, res_conc, res_mov) = tokio::join!(task_base, task_conc, task_mov);

        // Procesar resultados
        self.base = res_base??;
        println!(
            "   - Base cargada y calculada (Pipeline): {} registros.",
            self.base.len()
        );

        self.conceptos = res_conc??;
        println!(
            "   - Conceptos cargados: {} registros.",
            self.conceptos.len()
        );

        self.movimientos = res_mov??;
        println!(
            "   - Movimientos cargados: {} registros.",
            self.movimientos.len()
        );

        // 3. Cargar Beneficiarios (Final - Cruce)
        println!("3. Cargando y Calculando Beneficiarios...");
        let mut c_ben = cargador::Cargador {
            client: Some(client.clone()),
        };
        self.beneficiarios = c_ben
            .cargar_beneficiarios(&self.base, &self.movimientos)
            .await?;
        println!("   Beneficiarios procesados: {}.", self.beneficiarios.len());

        // if let Some(primero) = self.base.first() {
        //     println!("Ejemplo Base: {:?}", primero);
        // }

        println!(">>> CICLO COMPLETADO EXITOSAMENTE <<<");
        Ok(())
    }

    /// Convierte y mapea los resultados dinámicos a una lista de Structs tipados
    pub fn mapear_resultados<T: serde::de::DeserializeOwned>(
        &self,
        rows: Vec<prost_types::Struct>,
    ) -> Vec<T> {
        rows.into_iter()
            .filter_map(|row| {
                if let Ok(json_value) = proto_struct_to_json(row) {
                    serde_json::from_value(json_value).ok()
                } else {
                    None
                }
            })
            .collect()
    }
}

// --- Helpers para conversión Protobuf -> JSON ---

pub fn proto_value_to_json(val: prost_types::Value) -> serde_json::Value {
    use prost_types::value::Kind;
    match val.kind {
        Some(Kind::NullValue(_)) => serde_json::Value::Null,
        Some(Kind::NumberValue(n)) => serde_json::json!(n),
        Some(Kind::StringValue(s)) => serde_json::Value::String(s),
        Some(Kind::BoolValue(b)) => serde_json::Value::Bool(b),
        Some(Kind::StructValue(s)) => {
            if let Ok(v) = proto_struct_to_json(s) {
                v
            } else {
                serde_json::Value::Null
            }
        }
        Some(Kind::ListValue(l)) => {
            let list: Vec<serde_json::Value> =
                l.values.into_iter().map(proto_value_to_json).collect();
            serde_json::Value::Array(list)
        }
        None => serde_json::Value::Null,
    }
}

pub fn proto_struct_to_json(s: prost_types::Struct) -> Result<serde_json::Value, String> {
    let mut map = serde_json::Map::new();
    for (k, v) in s.fields {
        map.insert(k, proto_value_to_json(v));
    }
    Ok(serde_json::Value::Object(map))
}
