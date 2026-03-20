use crate::kernel::logica::memoria::{Base, ConceptoCalculado, ConceptoNomina, TipoConcepto};
use rayon::prelude::*;
use rhai::{Engine, Scope, AST};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ConceptoFnx {
    pub codigo: String,
    pub descripcion: String,
    pub tipo: u8,
    pub codigo_rhai: String,
    pub ast: AST,
    pub activo: Arc<AtomicBool>,
    pub estructura: String,
    pub cuenta: String,
    pub partida: String,
}

pub struct EjecutorConceptos {
    engine: Engine,
    conceptos: Arc<Vec<ConceptoFnx>>,
}

impl EjecutorConceptos {
    pub fn new(conceptos_nomina: Vec<ConceptoNomina>) -> Self {
        let engine = Engine::new();
        let mut conceptos = Vec::new();

        for concepto in conceptos_nomina {
            let raw = concepto.codigo_rhai.trim().trim_end_matches(';').to_string();

            match engine.compile(&raw) {
                Ok(ast) => {
                    conceptos.push(ConceptoFnx {
                        codigo: concepto.codigo.clone(),
                        descripcion: concepto.descripcion.clone(),
                        tipo: concepto.tipo as u8,
                        codigo_rhai: raw,
                        ast,
                        activo: Arc::new(AtomicBool::new(true)),
                        estructura: concepto.estructura.clone(),
                        cuenta: concepto.cuenta.clone(),
                        partida: concepto.partida.clone(),
                    });
                }
                Err(e) => {
                    eprintln!(
                        "[EjecutorConceptos] Error compilando concepto [{}] '{}': {}",
                        concepto.codigo, concepto.codigo_rhai, e
                    );
                }
            }
        }

        Self {
            engine,
            conceptos: Arc::new(conceptos),
        }
    }

    pub fn ejecutar(&self, bases: &[Base]) -> HashMap<String, Vec<ConceptoCalculado>> {
        bases
            .par_iter()
            .map(|base| {
                let mut scope = self.crear_scope(base);
                let mut calculados = Vec::new();

                for concepto in self.conceptos.iter() {
                    if !concepto.activo.load(Ordering::Relaxed) {
                        continue;
                    }

                    let resultado: f64 = match self
                        .engine
                        .eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &concepto.ast)
                    {
                        Ok(val) => {
                            if let Ok(f) = val.as_float() {
                                f
                            } else if let Ok(i) = val.as_int() {
                                i as f64
                            } else {
                                0.0
                            }
                        }
                        Err(e) => {
                            if concepto.activo.swap(false, Ordering::Relaxed) {
                                let msg = format!(
                                    "Concepto '{}' DESACTIVADO por error: {}",
                                    concepto.codigo, e
                                );
                                eprintln!("[ERROR] [EjecutorConceptos] {}", msg);
                            }
                            0.0
                        }
                    };

                    scope.push(concepto.codigo.clone(), resultado);

                    calculados.push(ConceptoCalculado {
                        codigo: concepto.codigo.clone(),
                        descripcion: concepto.descripcion.clone(),
                        tipo: match concepto.tipo {
                            1 | 2 => TipoConcepto::Asignacion,
                            3 | 4 | 5 => TipoConcepto::Deduccion,
                            _ => {
                                if concepto.codigo.to_lowercase().contains("ded")
                                    || concepto.codigo.to_lowercase().starts_with("desc")
                                    || concepto.codigo.to_lowercase().starts_with("ret")
                                {
                                    TipoConcepto::Deduccion
                                } else {
                                    TipoConcepto::Asignacion
                                }
                            }
                        },
                        valor: resultado,
                        estructura: concepto.estructura.clone(),
                        cuenta: concepto.cuenta.clone(),
                        partida: concepto.partida.clone(),
                    });
                }

                (base.patterns.clone(), calculados)
            })
            .collect()
    }

    fn crear_scope(&self, base: &Base) -> Scope<'_> {
        let mut scope = Scope::new();

        scope.push("sueldo_base", base.sueldo_base);
        scope.push("sueldo_mensual", base.sueldo_mensual);
        scope.push("sueldo_integral", base.sueldo_integral);
        scope.push("asignacion_antiguedad", base.asignacion_antiguedad);
        scope.push("garantias", base.garantias);
        scope.push("vacaciones", base.vacaciones);
        scope.push("aguinaldos", base.aguinaldos);
        scope.push("cantidad_hijos", base.n_hijos as i64);
        scope.push("antiguedad", base.antiguedad as i64);
        scope.push("tiempo_servicio", base.antiguedad as i64);
        scope.push("numero_hijos", base.n_hijos as i64);
        scope.push("grado_id", base.grado_id as i64);
        scope.push("componente_id", base.componente_id as i64);

        if let Some(calculos) = &base.calculos {
            for (key, value) in calculos {
                scope.push(key.clone(), *value);
            }
        }

        scope
    }

    pub fn get_codigos(&self) -> Vec<String> {
        self.conceptos.iter().map(|c| c.codigo.clone()).collect()
    }
}

pub fn calcular_totales_conceptos(conceptos: &[ConceptoCalculado]) -> (f64, f64) {
    let asignaciones: f64 = conceptos
        .iter()
        .filter(|c| matches!(c.tipo, TipoConcepto::Asignacion))
        .map(|c| c.valor)
        .sum();

    let deducciones: f64 = conceptos
        .iter()
        .filter(|c| matches!(c.tipo, TipoConcepto::Deduccion))
        .map(|c| c.valor)
        .sum();

    (asignaciones, deducciones)
}
