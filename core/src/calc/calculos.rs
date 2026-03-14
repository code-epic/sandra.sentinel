use crate::kernel::logica::memoria::{Base, Movimiento};
use chrono::{Datelike, NaiveDate};

pub fn generar_calculos(
    bases: &mut [Base],
    movimientos: &[Movimiento],
    monto_aprobado_garantias: f64,
) {
    // Primera pasada: calcular todos los valores base
    for base in bases.iter_mut() {
        let tiempo_servicio = base.antiguedad;
        let _tiempo_servicio_aux = calcular_tiempo_servicio_aux(tiempo_servicio);
        let f_retiro = base.f_retiro.as_deref().unwrap_or("");
        let _f_retiro_date = parsear_fecha(f_retiro);

        // 1. Sueldo Mensual = Sueldo Base + Total Primas
        let prima_total = base
            .calculos
            .as_ref()
            .map(|c| c.values().sum::<f64>())
            .unwrap_or(0.0);
        base.sueldo_mensual = base.sueldo_base + prima_total;

        // 2. Alicuota Aguinaldo
        base.aguinaldos = calcular_alicuota_aguinaldo(base.sueldo_mensual, f_retiro);

        // 3. Alicuota Vacaciones
        let (vacaciones, dias_vac) =
            calcular_alicuota_vacaciones(base.sueldo_mensual, f_retiro, tiempo_servicio);
        base.vacaciones = vacaciones;
        base.dia_vacaciones = dias_vac;

        // 4. Sueldo Integral = Sueldo Mensual + Vacaciones + Aguinaldos
        base.sueldo_integral = base.sueldo_mensual + base.vacaciones + base.aguinaldos;

        // 5. Asignacion Antiguedad = Sueldo Integral * Tiempo de Servicio
        base.asignacion_antiguedad = base.sueldo_integral * tiempo_servicio as f64;

        // 6. Garantias = (Sueldo Integral / 30) * 15
        base.garantias = (base.sueldo_integral / 30.0) * 15.0;
        base.garantia_original = base.garantias;

        // 7. Dias Adicionales
        base.dias_adicionales = calcular_dias_adicionales(base.sueldo_mensual, tiempo_servicio);

        // 8. Deposito banco ya viene seteado desde la fusion en Beneficiario
        // Solo si no se estableció, usamos 0
        if base.deposito_banco == 0.0 {
            base.deposito_banco = buscar_deposito_banco(&base.patterns, movimientos);
        }

        // 9. No Depositado Banco = Asignacion Antiguedad - Deposito Banco - Garantias Originales - Dias Adicionales
        let mut no_depositado = base.asignacion_antiguedad
            - base.deposito_banco
            - base.garantia_original
            - base.dias_adicionales;

        if no_depositado < 0.0 {
            no_depositado = 0.0;
        }
        base.no_depositado_banco = (no_depositado * 100.0).round() / 100.0;
    }

    // Segunda pasada: distribución exacta de garantías con anticipo
    if monto_aprobado_garantias > 0.0 {
        let suma_garantias: f64 = bases.iter().map(|b| b.garantia_original).sum();
        println!(
            "    > DISTRIBUCION: monto_aprobado={}, suma_garantias={}, factor={}",
            monto_aprobado_garantias,
            suma_garantias,
            monto_aprobado_garantias / suma_garantias
        );
        aplicar_distribucion_exacta(bases, monto_aprobado_garantias);
    }
}

/// Algoritmo de distribución exacta usando centavos para evitar errores de punto flotante
/// El último registro absorbe la diferencia para cuadrar exactamente el monto aprobado
fn aplicar_distribucion_exacta(bases: &mut [Base], monto_aprobado: f64) {
    // Calcular suma total de garantías originales
    let suma_garantias: f64 = bases.iter().map(|b| b.garantia_original).sum();

    if suma_garantias == 0.0 {
        return;
    }

    // Factor global: monto_aprobado / suma_total_garantias
    let factor_global = monto_aprobado / suma_garantias;

    // Convertir a centavos para evitar errores de punto flotante
    let monto_aprobado_centavos = (monto_aprobado * 100.0).round() as i64;

    let mut acumulado: i64 = 0;
    let n = bases.len();

    for (i, base) in bases.iter_mut().enumerate() {
        // Calcular anticipo basado en factor global
        let anticipo_calculado = base.garantia_original * factor_global;
        let anticipo_centavos = (anticipo_calculado * 100.0).round() as i64;

        if i < n - 1 {
            // Primeros N-1: truncado a centavos (redondeo hacia abajo)
            base.garantia_anticipo = anticipo_centavos as f64 / 100.0;
            acumulado += anticipo_centavos;
        } else {
            // Último registro: cuadra exactamente el monto aprobado
            let anticipo_final_centavos = monto_aprobado_centavos - acumulado;
            base.garantia_anticipo = anticipo_final_centavos as f64 / 100.0;
        }

        // Guardar factor global aplicado (para referencia/auditoría)
        base.factor_aplicado = factor_global;
    }
}

fn calcular_tiempo_servicio_aux(anos: u32) -> u32 {
    anos
}

fn parsear_fecha(fecha: &str) -> Option<NaiveDate> {
    if fecha.is_empty() {
        return None;
    }
    let limpia = fecha.split('T').next().unwrap_or(fecha);
    NaiveDate::parse_from_str(limpia, "%Y-%m-%d").ok()
}

fn calcular_alicuota_aguinaldo(sueldo_mensual: f64, f_retiro: &str) -> f64 {
    let f_retiro_date = parsear_fecha(f_retiro);

    let dias = if f_retiro_date.is_none() {
        120
    } else if let Some(fr) = f_retiro_date {
        let anio_retiro = fr.year();
        if anio_retiro < 2016 {
            90
        } else if anio_retiro == 2016 && fr.month() >= 10 && fr.month() <= 12 {
            105
        } else {
            120
        }
    } else {
        120
    };

    ((dias as f64 * sueldo_mensual) / 30.0) / 12.0
}

fn calcular_alicuota_vacaciones(
    sueldo_mensual: f64,
    f_retiro: &str,
    tiempo_servicio: u32,
) -> (f64, u32) {
    let f_retiro_date = parsear_fecha(f_retiro);

    let dias = if f_retiro_date.is_none() || f_retiro.is_empty() {
        50
    } else if let Some(fr) = f_retiro_date {
        if fr.year() > 2016 {
            50
        } else if fr.year() == 2016 && fr.month() <= 12 {
            if tiempo_servicio > 0 && tiempo_servicio <= 14 {
                40
            } else if tiempo_servicio > 14 && tiempo_servicio <= 24 {
                45
            } else {
                50
            }
        } else {
            50
        }
    } else {
        50
    };

    let monto = ((dias as f64 * sueldo_mensual) / 30.0) / 12.0;
    (monto, dias)
}

fn calcular_dias_adicionales(sueldo_mensual: f64, tiempo_servicio: u32) -> f64 {
    if tiempo_servicio == 0 {
        return 0.0;
    }

    let factor = if tiempo_servicio < 16 {
        tiempo_servicio as f64
    } else {
        15.0
    };

    ((sueldo_mensual / 30.0) * 2.0) * factor
}

fn buscar_deposito_banco(_patterns: &str, _movimientos: &[Movimiento]) -> f64 {
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_base() -> Base {
        Base {
            grado_id: 1,
            componente_id: 1,
            n_hijos: 0,
            fecha_ingreso: Some("2000-01-01".to_string()),
            f_ult_ascenso: Some("2020-01-01".to_string()),
            anio_reconocido: 0,
            mes_reconocido: 0,
            dia_reconocido: 0,
            st_no_ascenso: 0,
            st_profesion: 0.0,
            patterns: "1-1-0".to_string(),
            f_retiro: None,
            sueldo_base: 500.0,
            total_asignaciones: 0.0,
            antiguedad: 10,
            antiguedad_grado: 5,
            calculos: Some(std::collections::HashMap::new()),
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
            garantia_original: 0.0,
            garantia_anticipo: 0.0,
            factor_aplicado: 0.0,
        }
    }

    #[test]
    fn test_generar_calculos_sin_anticipo() {
        let mut base = create_test_base();
        let movimientos = vec![];

        generar_calculos(&mut [&mut base], &movimientos, 0.0);

        assert!(base.sueldo_mensual > 0.0);
        assert!(base.aguinaldos > 0.0);
        assert!(base.vacaciones > 0.0);
        assert!(base.garantias > 0.0);
    }

    #[test]
    fn test_distribucion_exacta() {
        // Crear 3 registros con garantías iguales
        let mut bases: Vec<Base> = (0..3)
            .map(|_| {
                let mut b = create_test_base();
                b.garantia_original = 100.0; // Cada una 100 Bs
                b
            })
            .collect();

        let movimientos = vec![];

        // Aprobar solo 150 Bs (la mitad)
        generar_calculos(&mut bases, &movimientos, 150.0);

        // Verificar que suma exacta es 150
        let suma: f64 = bases.iter().map(|b| b.garantia_anticipo).sum();
        assert!(
            (suma - 150.0).abs() < 0.01,
            "Suma debe ser 150, fue {}",
            suma
        );

        // Verificar que todos tienen factor aplicado
        for b in &bases {
            assert!(b.factor_aplicado > 0.0);
        }
    }
}

/// Aplica distribución de garantías sobre Beneficiarios (después de fusión)
pub fn generar_calculos_beneficiarios(
    beneficiarios: &mut [crate::kernel::logica::memoria::Beneficiario],
    monto_aprobado: f64,
) {
    // Extraer las bases de los beneficiarios para calcular suma total
    let suma_garantias: f64 = beneficiarios.iter().map(|b| b.base.garantia_original).sum();

    if suma_garantias == 0.0 {
        return;
    }

    // Factor global
    let factor_global = monto_aprobado / suma_garantias;

    println!(
        "    > DISTRIBUCION: monto_aprobado={}, suma_garantias={}, factor={}",
        monto_aprobado, suma_garantias, factor_global
    );

    // Convertir a centavos
    let monto_aprobado_centavos = (monto_aprobado * 100.0).round() as i64;

    let mut acumulado: i64 = 0;
    let n = beneficiarios.len();

    for (i, ben) in beneficiarios.iter_mut().enumerate() {
        let garantia_original = ben.base.garantia_original;
        let anticipo_calculado = garantia_original * factor_global;
        let anticipo_centavos = (anticipo_calculado * 100.0).round() as i64;

        if i < n - 1 {
            ben.base.garantia_anticipo = anticipo_centavos as f64 / 100.0;
            acumulado += anticipo_centavos;
        } else {
            // Último: cuadra exacto
            let anticipo_final_centavos = monto_aprobado_centavos - acumulado;
            ben.base.garantia_anticipo = anticipo_final_centavos as f64 / 100.0;
        }

        ben.base.factor_aplicado = factor_global;
    }
}
