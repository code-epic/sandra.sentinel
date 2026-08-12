use crate::kernel::logica::memoria::{Base, PrimaFuncion};
use rayon::prelude::*;
use rhai::{Engine, Scope, AST};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// DEBUG de fórmulas bajo demanda.
/// Controlado por el flag CLI --debug (o variable de entorno SANDRA_DEBUG=1).
/// Marcador: [DEBUG-PTS] — grep con: grep "DEBUG-PTS" /tmp/debug_prima.log
/// IMPORTANTE: redirigir stderr: comando 2>/tmp/debug_prima.log

#[derive(Debug, Clone)]
pub struct FormulaFnx {
    pub codigo: String,
    pub nombre: String,
    pub codigo_rhai: String,
    pub ast: AST,
    pub activo: Arc<AtomicBool>, // Bandera global thread-safe para desactivar si falla
    pub monto_nominal: f64,
}

pub struct SentinelEngine {
    engine: Engine,
    formulas: Arc<Vec<FormulaFnx>>,
}

impl SentinelEngine {
    pub fn new(primas: Vec<PrimaFuncion>) -> Self {
        // println!("[SentinelEngine] Iniciando motor de cálculo...");
        let engine = Engine::new();
        let mut formulas = Vec::new();

        // println!(
        //     "[SentinelEngine] Cargando {} fórmulas base desde PrimaFuncion...",
        //     primas.len()
        // );

        for prima in primas {
            // Asumimos que el código viene en Rhai válido o compatible
            // Solo limpiamos espacios y punto y coma final por seguridad
            let raw = prima.formula.trim().trim_end_matches(';').to_string();

            // Compilación
            match engine.compile(&raw) {
                Ok(ast) => {
                    // Marcador de interpretación exitosa
                    // println!("[SentinelEngine] Fórmula interpretada: {} -> OK", prima.codigo);
                    formulas.push(FormulaFnx {
                        codigo: prima.codigo,
                        nombre: prima.nombre,
                        codigo_rhai: raw,
                        ast,
                        activo: Arc::new(AtomicBool::new(true)),
                        monto_nominal: prima.monto_nominal,
                    });
                }
                Err(e) => {
                    eprintln!(
                        "[SentinelEngine] Error crítico interpretando fórmula [{}] '{}': {}",
                        prima.codigo, prima.formula, e
                    );
                }
            }
        }

        // println!(
        //     "[SentinelEngine] Carga completa. {} fórmulas listas para ejecución.",
        //     formulas.len()
        // );

        Self {
            engine,
            formulas: Arc::new(formulas),
        }
    }

    /// Calcula las primas para un lote de beneficiarios en paralelo
    pub fn calcular_primas(&self, base: &Vec<Base>) -> Vec<(String, HashMap<String, f64>)> {
        // ═══════════════════════════════════════════════════════════════════
        //  [DEBUG-PTS] Bajo demanda: activar con flag --debug (SANDRA_DEBUG=1)
        //  Grepear con: grep "DEBUG-PTS" (redirigir stderr con 2>&1)
        // ═══════════════════════════════════════════════════════════════════
        let debug_formula = crate::calc::is_debug();
        if debug_formula {
            eprintln!("[DEBUG-PTS] ============================================================");
            eprintln!("[DEBUG-PTS] MOTOR INICIALIZADO — {} fórmulas cargadas", self.formulas.len());
            for (idx, f) in self.formulas.iter().enumerate() {
                let activo_str = if f.activo.load(Ordering::Relaxed) { "ACT" } else { "OFF" };
                eprintln!(
                    "[DEBUG-PTS]   #{} [{}] {} -> {} (monto_nom={:.2})",
                    idx, activo_str, f.codigo, f.nombre, f.monto_nominal
                );
            }
            eprintln!("[DEBUG-PTS] ============================================================");

            // Buscar la fórmula prima_tiemposervicio por código O por índice (#5)
            let pts_idx = self.formulas.iter().position(|f| {
                f.codigo == "prima_tiemposervicio"
                    || f.codigo.to_lowercase().contains("tiempo")
                    || f.nombre == "P_TIEMPOSERVICIO"
                    || f.nombre.to_lowercase().contains("tiempo")
            });
            if let Some(idx) = pts_idx {
                let f = &self.formulas[idx];
                eprintln!("[DEBUG-PTS] ============================================================");
                eprintln!("[DEBUG-PTS] FÓRMULA prima_tiemposervicio ENCONTRADA en índice #{}", idx);
                eprintln!("[DEBUG-PTS] CODIGO:        {}", f.codigo);
                eprintln!("[DEBUG-PTS] NOMBRE:        {}", f.nombre);
                eprintln!("[DEBUG-PTS] MONTO_NOMINAL: {:.2}", f.monto_nominal);
                eprintln!("[DEBUG-PTS] ACTIVO:        {}", f.activo.load(Ordering::Relaxed));
                eprintln!("[DEBUG-PTS] ─── FÓRMULA RHAI COMPLETA ───");
                eprintln!("[DEBUG-PTS] |{}", f.codigo_rhai);
                eprintln!("[DEBUG-PTS] ───────────────────────────────");
                eprintln!("[DEBUG-PTS] ============================================================");
            } else {
                eprintln!("[DEBUG-PTS] ⚠️  prima_tiemposervicio NO ENCONTRADA en las {} fórmulas", self.formulas.len());
                // Fallback: mostrar la última fórmula (índice 5 o len-1)
                if let Some(last) = self.formulas.last() {
                    eprintln!("[DEBUG-PTS] Última fórmula: {} -> {} | Rhai: {}",
                        last.codigo, last.nombre, last.codigo_rhai);
                }
            }
            eprintln!("[DEBUG-PTS] ============================================================");
        }

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
                // if ben.sueldo_base > 600.0 { println!("[DEBUG] Scope para SUELDO {}: Hijos={}, Antig={}, StProf={}", ben.sueldo_base, ben.n_hijos, ben.antiguedad, ben.st_profesion); }

                for formula in self.formulas.iter() {
                    // 1. Circuit Breaker: Si la fórmula falló antes, la ignoramos.
                    if !formula.activo.load(Ordering::Relaxed) {
                        continue;
                    }

                    // ── [DEBUG-PTS] Bajo demanda: activar con flag --debug ──────
                    // Match: por código exacto, por nombre, o por índice (#5 = última fórmula típica)
                    let es_pts = debug_formula && (
                        formula.codigo == "prima_tiemposervicio"
                        || formula.codigo.to_lowercase().contains("tiempo")
                        || formula.nombre == "P_TIEMPOSERVICIO"
                        || formula.nombre.to_lowercase().contains("tiempo")
                    );

                    if es_pts {
                        eprintln!();
                        eprintln!("[DEBUG-PTS] ═══════════════════════════════════════════════════════");
                        eprintln!("[DEBUG-PTS] ANTES DE EVALUAR — beneficiario: {}", ben.patterns);
                        eprintln!("[DEBUG-PTS] CODIGO:        {}", formula.codigo);
                        eprintln!("[DEBUG-PTS] NOMBRE:        {}", formula.nombre);
                        eprintln!("[DEBUG-PTS] MONTO_NOMINAL: {:.2}", formula.monto_nominal);
                        eprintln!("[DEBUG-PTS] ─── FÓRMULA RHAI COMPLETA ───");
                        eprintln!("[DEBUG-PTS] |{}", formula.codigo_rhai);
                        eprintln!("[DEBUG-PTS] ───────────────────────────────");
                        eprintln!("[DEBUG-PTS] ─── VARIABLES DEL SCOPE (valores reales) ───");
                        eprintln!("[DEBUG-PTS] antiguedad:         {} (u32→i64={})", ben.antiguedad, ben.antiguedad as i64);
                        eprintln!("[DEBUG-PTS] tiempo_servicio:    {} (mismo que antiguedad)", ben.antiguedad as i64);
                        eprintln!("[DEBUG-PTS] sueldo_base:        {:.2}", ben.sueldo_base);
                        eprintln!("[DEBUG-PTS] unidad_tributaria:  {:.2}", ben.unidad_tributaria);
                        eprintln!("[DEBUG-PTS] salario_minimo:     {:.2}", ben.salario_minimo);
                        eprintln!("[DEBUG-PTS] grado_id:           {}", ben.grado_id);
                        eprintln!("[DEBUG-PTS] componente_id:      {}", ben.componente_id);
                        eprintln!("[DEBUG-PTS] n_hijos:            {}", ben.n_hijos);
                        eprintln!("[DEBUG-PTS] st_profesion:       {:.2}", ben.st_profesion);
                        eprintln!("[DEBUG-PTS] st_no_ascenso:      {}", ben.st_no_ascenso);
                        eprintln!("[DEBUG-PTS] antiguedad_grado:   {}", ben.antiguedad_grado);
                        eprintln!("[DEBUG-PTS] fecha_ingreso:      {:?}", ben.fecha_ingreso);
                        eprintln!("[DEBUG-PTS] f_ult_ascenso:      {:?}", ben.f_ult_ascenso);
                        eprintln!("[DEBUG-PTS] f_retiro:           {:?}", ben.f_retiro);
                        eprintln!("[DEBUG-PTS] ═══════════════════════════════════════════════════════");
                    }
                    // ── FIN DEBUG PTS ──────────────────────────────────────────

                    // Inyectar el monto nominal propio de esta fórmula
                    scope.push("monto_nominal", formula.monto_nominal);

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
                                eprintln!("[ERROR] [SentinelEngine] {}", msg);
                                // Log del sistema
                                crate::kernel::logica::logger::log_error("FORMULA", &msg);
                            }
                            0.0
                        }
                    };

                    // 3. Redondear a 2 decimales para evitar propagación de errores
                    let resultado = (resultado * 100.0).round() / 100.0;

                    // ── [DEBUG-PTS] DESPUÉS DE EVALUAR ─────────────────────────
                    if es_pts {
                        eprintln!();
                        eprintln!("[DEBUG-PTS] ═══════════════════════════════════════════════════════");
                        eprintln!("[DEBUG-PTS] DESPUÉS DE EVALUAR — beneficiario: {}", ben.patterns);
                        eprintln!("[DEBUG-PTS] CODIGO:        {}", formula.codigo);
                        eprintln!("[DEBUG-PTS] RESULTADO CRUDO (antes de r2d2): {:.6}", resultado);
                        eprintln!("[DEBUG-PTS] RESULTADO R2D2 (round 2 dec):   {:.2}", resultado);
                        if formula.monto_nominal > 0.0 {
                            let pct = (resultado / formula.monto_nominal) * 100.0;
                            eprintln!("[DEBUG-PTS] PORCENTAJE:    {:.4}% = ({:.2} / {:.2}) * 100", pct, resultado, formula.monto_nominal);
                        }
                        eprintln!("[DEBUG-PTS] SCOPE post-eval — {} será visible para fórmulas siguientes", formula.codigo);
                        eprintln!("[DEBUG-PTS] ═══════════════════════════════════════════════════════");
                        eprintln!();
                    }
                    // ── FIN DEBUG PTS ──────────────────────────────────────────

                    // 4. Inyectar resultado como variable para siguientes fórmulas
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
        scope.push("unidad_tributaria", base.unidad_tributaria);
        scope.push("ut", base.unidad_tributaria);
        scope.push("salario_minimo", base.salario_minimo);
        scope.push("s_minimo", base.salario_minimo);

        // Convertimos a i64 para compatibilidad con rangos de Rhai (ej: 1..=antiguedad)
        // Convertimos a i64 para compatibilidad con rangos de Rhai
        scope.push("antiguedad", base.antiguedad as i64); // Ahora usamos los años de servicio, no el monto
        scope.push("tiempo_servicio", base.antiguedad as i64);

        // Familiares
        scope.push("numero_hijos", base.n_hijos as i64);
        scope.push("n_hijos", base.n_hijos as i64); // alias

        scope.push("st_profesionalizacion", base.st_profesion as i64);
        scope.push("st_profesion", base.st_profesion as i64); // alias

        // Ascenso
        scope.push("no_ascenso", base.st_no_ascenso as i64);

        // Datos adicionales
        scope.push("grado_id", base.grado_id as i64);

        // Inicializar variables de primas conocidas en 0.0 para evitar errores si se referencian antes de calcular (o si fallan)
        // Esto es opcional, pero ayuda a la robustez
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::logica::memoria::{Base, PrimaFuncion};

    fn crear_base_test(n_hijos: u32, sueldo_base: f64, unidad_tributaria: f64) -> Base {
        Base {
            grado_id: 1,
            componente_id: 1,
            n_hijos,
            fecha_ingreso: Some("2000-01-01".to_string()),
            f_ult_ascenso: Some("2020-01-01".to_string()),
            anio_reconocido: 0,
            mes_reconocido: 0,
            dia_reconocido: 0,
            st_no_ascenso: 0,
            st_profesion: 0.0,
            patterns: format!("1-1-{}", n_hijos),
            f_retiro: None,
            sueldo_base,
            unidad_tributaria,
            salario_minimo: 0.0,
            total_asignaciones: 0.0,
            antiguedad: 10,
            antiguedad_grado: 5,
            calculos: None,
            sueldo_mensual: 0.0,
            aguinaldos: 0.0,
            vacaciones: 0.0,
            dia_vacaciones: 0,
            sueldo_integral: 0.0,
            asignacion_antiguedad: 0.0,
            garantias: 0.0,
            dias_adicionales: 0.0,
            no_depositado_banco: 0.0,
            deposito_banco: 0.0,
            depositado_en_banco: 0.0,
            total_aportados: 0.0,
            porcentaje_cancelado: 0.0,
            saldo_disponible: 0.0,
            diferencia_asignacion: 0.0,
            garantia_original: 0.0,
            garantia_anticipo: 0.0,
            factor_aplicado: 0.0,
        }
    }

    fn crear_prima(codigo: &str, nombre: &str, formula: &str, monto_nominal: f64) -> PrimaFuncion {
        PrimaFuncion {
            codigo: codigo.to_string(),
            nombre: nombre.to_string(),
            descripcion: format!("Prueba {}", codigo),
            formula: formula.to_string(),
            monto_nominal,
        }
    }

    #[test]
    fn test_prima_descendencia_fija_por_hijo() {
        let engine = SentinelEngine::new(vec![crear_prima(
            "prima_descendencia",
            "P_DESCENDECIA",
            "12.50 * numero_hijos;",
            0.0,
        )]);

        let base = crear_base_test(2, 500.0, 0.0);
        let resultados = engine.calcular_primas(&vec![base]);

        assert_eq!(resultados.len(), 1);
        let (_, calculos) = resultados.into_iter().next().unwrap();
        assert_eq!(calculos.get("prima_descendencia"), Some(&25.0));
    }

    #[test]
    fn test_prima_descendencia_con_unidad_tributaria() {
        let engine = SentinelEngine::new(vec![crear_prima(
            "prima_descendencia",
            "P_DESCENDECIA",
            "monto_nominal * unidad_tributaria * numero_hijos;",
            2.0,
        )]);

        let base = crear_base_test(3, 1000.0, 10.0);
        let resultados = engine.calcular_primas(&vec![base]);

        assert_eq!(resultados.len(), 1);
        let (_, calculos) = resultados.into_iter().next().unwrap();
        assert_eq!(calculos.get("prima_descendencia"), Some(&60.0));
    }

    #[test]
    fn test_prima_descendencia_cero_sin_hijos() {
        let engine = SentinelEngine::new(vec![crear_prima(
            "prima_descendencia",
            "P_DESCENDECIA",
            "12.50 * numero_hijos;",
            0.0,
        )]);

        let base = crear_base_test(0, 500.0, 0.0);
        let resultados = engine.calcular_primas(&vec![base]);

        assert_eq!(resultados.len(), 1);
        let (_, calculos) = resultados.into_iter().next().unwrap();
        assert_eq!(calculos.get("prima_descendencia"), Some(&0.0));
    }
}
