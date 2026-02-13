use crate::kernel::logica::memoria::{Base, PrimaFuncion};
use rayon::prelude::*;
use rhai::{Engine, Scope, AST};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct FormulaFnx {
    pub codigo: String,
    pub nombre: String,
    pub codigo_rhai: String,
    pub ast: AST,
    pub activo: Arc<AtomicBool>, // Bandera global thread-safe para desactivar si falla
}

pub struct SentinelEngine {
    engine: Engine,
    formulas: Arc<Vec<FormulaFnx>>,
}

impl SentinelEngine {
    pub fn new(primas: Vec<PrimaFuncion>) -> Self {
        println!("🚀 [SentinelEngine] Iniciando motor de cálculo...");
        let engine = Engine::new();
        let mut formulas = Vec::new();

        println!(
            "📦 [SentinelEngine] Cargando {} fórmulas base desde PrimaFuncion...",
            primas.len()
        );

        for prima in primas {
            // Asumimos que el código viene en Rhai válido o compatible
            // Solo limpiamos espacios y punto y coma final por seguridad
            let raw = prima.formula.trim().trim_end_matches(';').to_string();

            // Compilación
            match engine.compile(&raw) {
                Ok(ast) => {
                    // Marcador de interpretación exitosa
                    // println!("✅ [SentinelEngine] Fórmula interpretada: {} -> OK", prima.codigo);
                    formulas.push(FormulaFnx {
                        codigo: prima.codigo,
                        nombre: prima.nombre,
                        codigo_rhai: raw,
                        ast,
                        activo: Arc::new(AtomicBool::new(true)),
                    });
                }
                Err(e) => {
                    eprintln!(
                        "⚠️ [SentinelEngine] Error crítico interpretando fórmula [{}] '{}': {}",
                        prima.codigo, prima.formula, e
                    );
                }
            }
        }

        println!(
            "✅ [SentinelEngine] Carga completa. {} fórmulas listas para ejecución.",
            formulas.len()
        );

        Self {
            engine,
            formulas: Arc::new(formulas),
        }
    }

    /// Calcula la nómina para un lote de beneficiarios en paralelo
    pub fn calcular_nomina(&self, base: &Vec<Base>) -> Vec<(String, HashMap<String, f64>)> {
        // Rayon: Iterador paralelo
        base.par_iter()
            .map(|ben| {
                let mut scope = Scope::new();

                // 1. Inyectar Contexto del Militar
                self.llenar_scope(&mut scope, ben);

                let mut rs_base = HashMap::with_capacity(self.formulas.len());

                // 2. Ejecutar Fórmulas (Secuencial por militar, para mantener dependencias)
                // DEBUG: Solo para el primer item del lote (o uno específico si pudiéramos filtrar)
                // let debug = false; // Cambiar a true si quieres ver logs de un item al azar o el primero

                // Hack sucio para debugear el primero del thread (rayon lo hace difícil, pero imprimirá algunos)
                // if ben.sueldo_base > 600.0 { println!("🔍 Scope para SUELDO {}: Hijos={}, Antig={}, StProf={}", ben.sueldo_base, ben.n_hijos, ben.antiguedad, ben.st_profesion); }

                for formula in self.formulas.iter() {
                    // 1. Circuit Breaker: Si la fórmula falló antes, la ignoramos.
                    if !formula.activo.load(Ordering::Relaxed) {
                        continue;
                    }

                    // Evaluar AST
                    let resultado: f64 = match self
                        .engine
                        .eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &formula.ast)
                    {
                        Ok(val) => {
                            let r = if let Ok(f) = val.as_float() {
                                f
                            } else if let Ok(i) = val.as_int() {
                                i as f64
                            } else {
                                0.0
                            };
                            r
                        }
                        Err(e) => {
                            // Si falla, la desactivamos globalmente para no spamear logs ni perder tiempo
                            // Solo imprimimos el error la primera vez (cuando pasamos de true a false)
                            if formula.activo.swap(false, Ordering::Relaxed) {
                                let msg = format!(
                                    "Fórmula '{}' DESACTIVADA por error crítico: {}",
                                    formula.codigo, e
                                );
                                eprintln!("❌ [SentinelEngine] {}", msg);
                                // Log del sistema
                                crate::kernel::logica::logger::log_error("FORMULA", &msg);
                            }
                            0.0
                        }
                    };

                    // 3. Inyectar resultado como variable para siguientes fórmulas
                    scope.push(formula.codigo.clone(), resultado);

                    // Guardar resultado
                    rs_base.insert(formula.codigo.clone(), resultado);
                }

                (ben.patterns.clone(), rs_base)
            })
            .collect()
    }

    /// Prepara el Scope de Rhai con los datos del Beneficiario
    fn llenar_scope(&self, scope: &mut Scope, base: &Base) {
        // Mapeo de variables esperadas por las fórmulas SQL legacy

        // Sueldo y Datos Básicos
        scope.push("sueldo_base", base.sueldo_base);
        // Convertimos a i64 para compatibilidad con rangos de Rhai (ej: 1..=antiguedad)
        // Convertimos a i64 para compatibilidad con rangos de Rhai
        scope.push("antiguedad", base.antiguedad as i64); // Ahora usamos los años de servicio, no el monto
        scope.push("tiempo_servicio", base.antiguedad as i64);

        // Familiares
        scope.push("numero_hijos", base.n_hijos as i64);

        scope.push("st_profesionalizacion", base.st_profesion as i64);

        // Ascenso
        scope.push("no_ascenso", base.st_no_ascenso as i64);

        // Datos adicionales
        scope.push("grado_id", base.grado_id as i64);

        // Inicializar variables de primas conocidas en 0.0 para evitar errores si se referencian antes de calcular (o si fallan)
        // Esto es opcional, pero ayuda a la robustez
    }
}
