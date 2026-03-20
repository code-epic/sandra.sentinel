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

use crate::nomina::{calcular_totales_conceptos, EjecutorConceptos};
use crate::tipos::TipoNomina;
use logica::cargador;
use logica::memoria::{self, Base, Beneficiario, Movimiento};

// El "Perceptrón" (Cache/Memoization)
#[derive(Debug)]
pub struct Perceptron {
    // Almacenamos el cliente gRPC para realizar llamadas
    pub client: Option<SentinelDynamicServiceClient<Channel>>,

    // Memoria de Trabajo
    pub directiva: Vec<memoria::Directiva>,
    pub primas_funciones: Vec<memoria::PrimaFuncion>,
    pub conceptos_nomina: Vec<memoria::ConceptoNomina>,
    pub base: Vec<memoria::Base>,
    pub movimientos: Vec<memoria::Movimiento>,
    pub beneficiarios: Vec<memoria::Beneficiario>,

    // Configuración de Ejecución (Manifiesto)
    pub config: crate::model::Manifiesto,
}

impl Default for Perceptron {
    fn default() -> Self {
        Perceptron {
            client: None,
            directiva: Vec::new(),
            primas_funciones: Vec::new(),
            conceptos_nomina: Vec::new(),
            base: Vec::new(),
            movimientos: Vec::new(),
            beneficiarios: Vec::new(),
            config: crate::model::Manifiesto::default(),
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
        tipo_nomina: TipoNomina,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // println!(">>> INICIANDO CICLO DE CARGA DE NÓMINA <<<"); // REEMPLAZADO POR HEADER DE CLI

        // Clonar configuración para pasarla a los hilos
        let config_ref = self.config.clone();
        let client = self.client.clone().ok_or("Cliente gRPC no conectado")?;

        // ---------------------------------------------------------------------
        // PASO 1: CARGA DE REFERENCIAS
        // ---------------------------------------------------------------------
        println!("PASO 1: CARGA DE REFERENCIAS");
        println!("{:-<80}", "");

        let t_p1 = std::time::Instant::now();
        let c_dir_client = client.clone();
        let c_primas_client = client.clone();

        let conf_1 = config_ref.clone();
        let conf_2 = config_ref.clone();

        let task_directiva = tokio::spawn(async move {
            let mut c = cargador::Cargador::new(conf_1);
            c.client = Some(c_dir_client);
            c.cargar_directiva().await
        });

        let task_primas = tokio::spawn(async move {
            let mut c = cargador::Cargador::new(conf_2);
            c.client = Some(c_primas_client);
            c.cargar_primas_funciones().await
        });

        let (res_dir, res_primas) = tokio::join!(task_directiva, task_primas);

        self.directiva = res_dir??;
        self.primas_funciones = res_primas??;

        // Helper para imprimir filtros debajo del resumen
        let print_filtro = |cfg: &crate::model::Manifiesto, func: &str| {
            if let Some(c) = cfg.cargas.get(func) {
                if let Some(f) = &c.sql_filter {
                    println!("      - Filtro: {}", f);
                }
            }
        };

        // Imprimir Resumen Paso 1
        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Directiva",
            self.directiva.len()
        );
        print_filtro(&self.config, "IPSFA_CDirectiva");

        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Primas Funciones",
            self.primas_funciones.len()
        );
        print_filtro(&self.config, "IPSFA_CPrimasFunciones");

        println!("    (Tiempo Paso 1: {:.2?})", t_p1.elapsed());
        println!();

        // --- INSTANCIAR MOTOR (Fase 1.5) ---
        // println!("1.5. Inicializando SentinelEngine..."); // Oculto
        let motor = crate::calc::motor::SentinelEngine::new(self.primas_funciones.clone());
        let motor_arc = std::sync::Arc::new(motor);
        println!(
            "  • {:<20} : {:>10} | LISTO",
            "Motor de Cálculo", "Inicializado"
        );
        println!();

        // ---------------------------------------------------------------------
        // PASO 2: CARGA MASIVA Y CÁLCULO
        // ---------------------------------------------------------------------
        println!("PASO 2: CARGA MASIVA Y CÁLCULO (PARALELO)");
        println!("{:-<80}", "");

        let t_p2 = std::time::Instant::now();

        let c1 = client.clone();
        let c2 = client.clone();
        let c3 = client.clone();

        let conf_3 = config_ref.clone();
        let conf_4 = config_ref.clone();
        let conf_5 = config_ref.clone();

        let directivas_clone = self.directiva.clone();
        let motor_ref = motor_arc.clone();

        // Obtener monto aprobado de garantías de la config
        let monto_aprobado = config_ref.aportes.monto_aprobado_garantias;
        println!(
            "  > Aporte Config: habilitar={}, monto_aprobado={}",
            config_ref.aportes.habilitar,
            monto_aprobado
        );

        // Paso 1: Cargar movimientos primero (necesarios para base)
        println!("  • {:<20} : {:>10} registros", "Movimientos", "cargando...");
        let mut c_mov = cargador::Cargador::new(conf_5);
        c_mov.client = Some(c3);
        let movimientos_cargados = c_mov.cargar_movimientos().await?;
        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Movimientos",
            movimientos_cargados.len()
        );

        // Clonar para pasar al closure
        let movs_clone = movimientos_cargados.clone();

        // Paso 2: Cargar base (con movimientos) y conceptos en paralelo
        let task_base = tokio::spawn(async move {
            let mut c = cargador::Cargador::new(conf_3);
            c.client = Some(c1);
            c.cargar_base(&directivas_clone, &motor_ref, &movs_clone, monto_aprobado).await
        });

        let task_conc = tokio::spawn(async move {
            let mut c = cargador::Cargador::new(conf_4);
            c.client = Some(c2);
            c.cargar_conceptos().await
        });

        let (res_base, res_conc) = tokio::join!(task_base, task_conc);

        // Procesar resultados
        self.base = res_base??;
        self.conceptos_nomina = res_conc??;
        self.movimientos = movimientos_cargados;

        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Base (Personal)",
            self.base.len()
        );
        print_filtro(&self.config, "IPSFA_CBase");

        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Movimientos",
            self.movimientos.len()
        );
        print_filtro(&self.config, "IPSFA_CMovimientos");

        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Conceptos",
            self.conceptos_nomina.len()
        );
        print_filtro(&self.config, "IPSFA_CConceptos");

        println!("    (Tiempo Paso 2: {:.2?})", t_p2.elapsed());
        println!();

        // ---------------------------------------------------------------------
        // PASO 2.5: EJECUTAR CONCEPTOS DINÁMICOS
        // ---------------------------------------------------------------------
        println!("PASO 2.5: EJECUTAR CONCEPTOS DINÁMICOS");
        println!("{:-<80}", "");

        let t_conceptos = std::time::Instant::now();
        
        let conceptos_nomina = self.conceptos_nomina.clone();

        let ejecutor = EjecutorConceptos::new(conceptos_nomina);
        let conceptos_calculados_map = ejecutor.ejecutar(&self.base);

        let total_conceptos_cargados = self.conceptos_nomina.len();
        let total_beneficiarios_procesados = conceptos_calculados_map.len();

        if total_conceptos_cargados == 0 {
            println!(
                "  • {:<20} : {:>10} registros | {}",
                "Conceptos Definidos",
                0,
                "ADVERTENCIA: No se cargaron formulas de IPSFA_CConceptos"
            );
            logica::logger::log_warn(
                "CONCEPTOS",
                "IPSFA_CConceptos devolvio 0 registros - No se aplicaran conceptos dinamicos",
            );
        } else {
            println!(
                "  • {:<20} : {:>10} registros | OK",
                "Conceptos Definidos",
                total_conceptos_cargados
            );
        }
        println!(
            "  • {:<20} : {:>10} beneficiarios | OK",
            "Beneficiarios Procesados",
            total_beneficiarios_procesados
        );
        println!("    (Tiempo conceptos: {:.2?})", t_conceptos.elapsed());
        println!();

        // ---------------------------------------------------------------------
        // PASO 3: FUSIÓN DE BENEFICIARIOS
        // ---------------------------------------------------------------------
        println!("PASO 3: FUSIÓN DE BENEFICIARIOS");
        println!("{:-<80}", "");

        let t_p3 = std::time::Instant::now();

        let mut c_ben = cargador::Cargador::new(config_ref.clone());
        c_ben.client = Some(client.clone());

        // Obtener monto aprobado para distribuir garantías después de fusión
        let monto_aprobado = config_ref.aportes.monto_aprobado_garantias;

        self.beneficiarios = c_ben
            .cargar_beneficiarios(&self.base, &self.movimientos, monto_aprobado)
            .await?;

        // ---------------------------------------------------------------------
        // PASO 3.5: APLICAR CONCEPTOS Y CALCULAR NETO
        // ---------------------------------------------------------------------
        println!("PASO 3.5: APLICAR CONCEPTOS Y CALCULAR NETO");
        println!("{:-<80}", "");

        for beneficiario in &mut self.beneficiarios {
            // Buscar conceptos calculados para este beneficiario
            if let Some(conceptos) = conceptos_calculados_map.get(&beneficiario.base.patterns) {
                let (total_asig, total_ded) = calcular_totales_conceptos(conceptos);
                
                beneficiario.conceptos_calculados = Some(
                    conceptos.iter()
                        .map(|c| (c.codigo.clone(), c.clone()))
                        .collect()
                );
                beneficiario.total_asignaciones = total_asig;
                beneficiario.total_deducciones = total_ded;

                // Calcular neto según tipo de nómina
                match tipo_nomina {
                    TipoNomina::Npr => {
                        // NPR: neto = garantías (lógica actual de prestaciones)
                        beneficiario.neto = beneficiario.base.garantias;
                        beneficiario.porcentaje = 100.0;
                    }
                    TipoNomina::Nact => {
                        // NACT: neto = sueldo integral (100%)
                        beneficiario.porcentaje = 100.0;
                        let base_neto = beneficiario.base.sueldo_integral;
                        beneficiario.neto = base_neto + total_asig - total_ded;
                    }
                    TipoNomina::Nrcp => {
                        // NRCP: neto = (integral × porcentaje/100) + asig - deduc
                        let pct = beneficiario.porcentaje / 100.0;
                        let base_neto = beneficiario.base.sueldo_integral * pct;
                        beneficiario.neto = base_neto + total_asig - total_ded;
                    }
                    TipoNomina::Nfcp => {
                        // NFCP: se calcula después con lógica especial de familiares
                        beneficiario.neto = 0.0;
                    }
                }
            } else {
                // Si no hay conceptos, inicializar en 0
                beneficiario.conceptos_calculados = None;
                beneficiario.total_asignaciones = 0.0;
                beneficiario.total_deducciones = 0.0;
                beneficiario.neto = match tipo_nomina {
                    TipoNomina::Npr => beneficiario.base.garantias,
                    TipoNomina::Nact => beneficiario.base.sueldo_integral,
                    TipoNomina::Nrcp => {
                        let pct = beneficiario.porcentaje / 100.0;
                        beneficiario.base.sueldo_integral * pct
                    }
                    TipoNomina::Nfcp => 0.0,
                };
                beneficiario.porcentaje = 100.0;
            }
        }

        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Beneficiarios",
            self.beneficiarios.len()
        );
        print_filtro(&self.config, "IPSFA_CBeneficiarios");

        println!("    (Tiempo Paso 3: {:.2?})", t_p3.elapsed());
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

    pub async fn calcular_nfcp(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("PASO NFCP: CARGANDO FAMILIARES SOBREVIVIENTES");
        println!("{:-<80}", "");

        let mut c_fam = cargador::Cargador::new(self.config.clone());
        c_fam.client = self.client.clone();

        let familiares = c_fam.cargar_familiares().await?;
        
        println!(
            "  • {:<20} : {:>10} registros | OK",
            "Familiares",
            familiares.len()
        );

        let mut familiares_beneficiarios = Vec::new();

        for familiar in familiares {
            let porcentaje = familiar.porcentaje / 100.0;
            let monto_familiar = 0.0 * porcentaje;

            let bnf = Beneficiario {
                cedula: familiar.cedula.clone(),
                nombres: familiar.nombres.clone(),
                apellidos: familiar.apellidos.clone(),
                porcentaje: familiar.porcentaje,
                neto: monto_familiar,
                total_asignaciones: 0.0,
                total_deducciones: 0.0,
                conceptos_calculados: None,
                es_familiar: true,
                cedula_titular: Some(familiar.titular.clone()),
                parentesco: familiar.parentesco.clone(),
                nombre_autorizado: familiar.nombre_autorizado.clone(),
                base: Base::default(),
                movimientos: Movimiento::default(),
                asignaciones: Vec::new(),
                deducciones: Vec::new(),
                patterns: String::new(),
                componente_id: 0,
                f_ingreso_sistema: None,
                f_ult_ascenso: None,
                f_retiro: None,
                f_retiro_efectiva: None,
                edo_civil: None,
                sexo: familiar.sexo.clone(),
                status_id: 0,
                st_no_ascenso: 0,
                categoria: None,
                status: 0,
                numero_cuenta: familiar.numero_cuenta.clone().unwrap_or_default(),
                f_creacion: None,
                usr_creacion: None,
                f_ult_modificacion: None,
                usr_modificacion: None,
                observ_ult_modificacion: None,
                motivo_paralizacion: None,
                f_reincorporacion: None,
            };

            familiares_beneficiarios.push(bnf);
        }

        self.beneficiarios = familiares_beneficiarios;
        
        println!(
            "  • {:<20} : {:>10} beneficiarios NFCP",
            "Total NFCP",
            self.beneficiarios.len()
        );

        Ok(())
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
