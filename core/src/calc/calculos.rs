use crate::kernel::logica::memoria::{Base, Movimiento};
use chrono::{Datelike, NaiveDate};

/// Redondea un valor flotante a 2 decimales (centésimas).
/// Se aplica en cada paso intermedio de la cadena de cálculo para evitar
/// acumulación de errores de punto flotante.
pub fn redondear_dos(valor: f64) -> f64 {
    (valor * 100.0).round() / 100.0
}

/// Trunca un valor flotante a 2 decimales (centésimas), descartando los decimales
/// posteriores sin redondear. Se usa para la columna de anticipos para reflejar
/// exactamente el valor almacenado en base de datos (ej. 0.295 -> 0.29).
pub fn truncar_dos(valor: f64) -> f64 {
    (valor * 100.0).trunc() / 100.0
}

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
        base.sueldo_mensual = redondear_dos(base.sueldo_base + prima_total);

        // 2. Alicuota Aguinaldo
        base.aguinaldos = redondear_dos(calcular_alicuota_aguinaldo(base.sueldo_mensual, f_retiro));

        // 3. Alicuota Vacaciones
        let (vacaciones, dias_vac) =
            calcular_alicuota_vacaciones(base.sueldo_mensual, f_retiro, tiempo_servicio);
        base.vacaciones = redondear_dos(vacaciones);
        base.dia_vacaciones = dias_vac;

        // 4. Sueldo Integral = Sueldo Mensual + Vacaciones + Aguinaldos
        base.sueldo_integral = redondear_dos(base.sueldo_mensual + base.vacaciones + base.aguinaldos);

        // 5. Asignacion Antiguedad = Sueldo Integral * Tiempo de Servicio
        base.asignacion_antiguedad = redondear_dos(base.sueldo_integral * tiempo_servicio as f64);

        // 6. Garantias = (Sueldo Integral / 30) * 15
        base.garantias = redondear_dos((base.sueldo_integral / 30.0) * 15.0);
        base.garantia_original = base.garantias;

        // 7. Dias Adicionales
        base.dias_adicionales = redondear_dos(calcular_dias_adicionales(base.sueldo_mensual, tiempo_servicio));

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
        base.no_depositado_banco = redondear_dos(no_depositado);
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

/// Algoritmo de distribución exacta para evitar errores de punto flotante
/// El último registro absorbe la diferencia para cuadrar exactamente el monto aprobado
fn aplicar_distribucion_exacta(bases: &mut [Base], monto_aprobado: f64) {
    // Calcular suma total de garantías originales
    let suma_garantias: f64 = bases.iter().map(|b| b.garantia_original).sum();

    if suma_garantias == 0.0 {
        return;
    }

    // Factor global: monto_aprobado / suma_total_garantias
    let factor_global = monto_aprobado / suma_garantias;

    // Convertir a multiplicado para evitar errores de punto flotante
    let monto_aprobado_multiplicado = (monto_aprobado * 100.0).round() as i64;

    let mut acumulado: i64 = 0;
    let n = bases.len();

    for (i, base) in bases.iter_mut().enumerate() {
        // Calcular anticipo basado en factor global
        let anticipo_calculado = base.garantia_original * factor_global;
        let anticipo_multiplicado = (anticipo_calculado * 100.0).round() as i64;

        if i < n - 1 {
            // Primeros N-1: truncado a multiplicado (redondeo hacia abajo)
            base.garantia_anticipo = anticipo_multiplicado as f64 / 100.0;
            acumulado += anticipo_multiplicado;
        } else {
            // Último registro: cuadra exactamente el monto aprobado
            let anticipo_final_multiplicado = monto_aprobado_multiplicado - acumulado;
            base.garantia_anticipo = anticipo_final_multiplicado as f64 / 100.0;
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

    // Orden de operaciones alineado con PHP: (sueldo / 30) * (dias / 12)
    // Evita diferencias de 0.01 por punto flotante vs ((dias * sueldo) / 30) / 12
    (sueldo_mensual / 30.0) * (dias as f64 / 12.0)
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

    // Orden de operaciones alineado con PHP: (sueldo / 30) * (dias / 12)
    // Evita diferencias de 0.01 por punto flotante vs ((dias * sueldo) / 30) / 12
    let monto = (sueldo_mensual / 30.0) * (dias as f64 / 12.0);
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
            unidad_tributaria: 0.0,
            salario_minimo: 0.0,
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

    #[test]
    fn test_generar_calculos_sin_anticipo() {
        let mut bases = [create_test_base()];
        let movimientos = vec![];

        generar_calculos(&mut bases, &movimientos, 0.0);

        assert!(bases[0].sueldo_mensual > 0.0);
        assert!(bases[0].aguinaldos > 0.0);
        assert!(bases[0].vacaciones > 0.0);
        assert!(bases[0].garantias > 0.0);
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

    #[test]
    fn test_sueldo_mensual_no_duplica_prima_descendencia() {
        // Escenario: prima_descendencia existe, prima_hijos NO existe en calculos
        // sueldo_mensual debe ser sueldo_base + prima_descendencia (una sola vez)
        let mut base = create_test_base();
        base.sueldo_base = 1000.0;
        base.n_hijos = 2;
        let mut calculos = std::collections::HashMap::new();
        calculos.insert("prima_descendencia".to_string(), 25.0);
        base.calculos = Some(calculos);

        let mut bases = [base];
        let movimientos = vec![];

        generar_calculos(&mut bases, &movimientos, 0.0);

        // sueldo_mensual = 1000 + 25 = 1025 (NO 1050)
        assert!(
            (bases[0].sueldo_mensual - 1025.0).abs() < 0.01,
            "sueldo_mensual debe ser 1025.0, fue {}",
            bases[0].sueldo_mensual
        );
    }

    #[test]
    fn test_alicuota_vacaciones_coincide_con_php() {
        // Caso reportado: sueldo_mensual=939.06, dias=50
        // Con orden PHP: (939.06 / 30) * (50 / 12) = 130.43
        // Con orden anterior: ((50 * 939.06) / 30) / 12 = 130.42
        let mut base = create_test_base();
        base.sueldo_base = 939.06;
        base.calculos = Some(std::collections::HashMap::new());

        let mut bases = [base];
        let movimientos = vec![];

        generar_calculos(&mut bases, &movimientos, 0.0);

        assert!(
            (bases[0].vacaciones - 130.43).abs() < 0.01,
            "alicuota vacaciones debe ser 130.43, fue {}",
            bases[0].vacaciones
        );
        assert_eq!(bases[0].dia_vacaciones, 50);
    }

    #[test]
    fn test_alicuota_aguinaldo_coincide_con_php() {
        // Caso límite: sueldo_mensual=939.06, dias=120
        // Con orden PHP: (939.06 / 30) * (120 / 12) = 313.02
        // Con orden anterior: ((120 * 939.06) / 30) / 12 = 313.01999999999998 -> 313.02
        let mut base = create_test_base();
        base.sueldo_base = 939.06;
        base.calculos = Some(std::collections::HashMap::new());

        let mut bases = [base];
        let movimientos = vec![];

        generar_calculos(&mut bases, &movimientos, 0.0);

        assert!(
            (bases[0].aguinaldos - 313.02).abs() < 0.01,
            "alicuota aguinaldo debe ser 313.02, fue {}",
            bases[0].aguinaldos
        );
    }

    #[test]
    fn test_truncar_dos_no_redondea() {
        // Caso reportado: valor en BD 0.295 no debe redondear a 0.30, debe quedar en 0.29
        assert!((truncar_dos(0.295) - 0.29).abs() < f64::EPSILON);
        assert!((truncar_dos(1557.585) - 1557.58).abs() < 0.0001);
        assert!((truncar_dos(0.30) - 0.30).abs() < f64::EPSILON);
        assert!((truncar_dos(0.01) - 0.01).abs() < f64::EPSILON);
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

    // Convertir a multiplicado
    let monto_aprobado_multiplicado = (monto_aprobado * 100.0).round() as i64;

    let mut acumulado: i64 = 0;
    let n = beneficiarios.len();

    for (i, ben) in beneficiarios.iter_mut().enumerate() {
        let garantia_original = ben.base.garantia_original;
        let anticipo_calculado = garantia_original * factor_global;
        let anticipo_multiplicado = (anticipo_calculado * 100.0).round() as i64;

        if i < n - 1 {
            ben.base.garantia_anticipo = anticipo_multiplicado as f64 / 100.0;
            acumulado += anticipo_multiplicado;
        } else {
            // Último: cuadra exacto
            let anticipo_final_multiplicado = monto_aprobado_multiplicado - acumulado;
            ben.base.garantia_anticipo = anticipo_final_multiplicado as f64 / 100.0;
        }

        ben.base.factor_aplicado = factor_global;
    }
}
