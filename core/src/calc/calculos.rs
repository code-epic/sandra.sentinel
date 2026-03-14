use crate::kernel::logica::memoria::{Base, Movimiento};
use chrono::{Datelike, NaiveDate};

pub fn generar_calculos(bases: &mut [Base], movimientos: &[Movimiento]) {
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

        // 7. Dias Adicionales
        base.dias_adicionales = calcular_dias_adicionales(base.sueldo_mensual, tiempo_servicio);

        // 8. Deposito banco ya viene seteado desde la fusion en Beneficiario
        // Solo si no se estableció, usamos 0
        if base.deposito_banco == 0.0 {
            base.deposito_banco = buscar_deposito_banco(&base.patterns, movimientos);
        }

        // 9. No Depositado Banco = Asignacion Antiguedad - Deposito Banco - Garantias - Dias Adicionales
        let mut no_depositado = base.asignacion_antiguedad
            - base.deposito_banco
            - base.garantias
            - base.dias_adicionales;

        if no_depositado < 0.0 {
            no_depositado = 0.0;
        }
        base.no_depositado_banco = (no_depositado * 100.0).round() / 100.0;
    }
}

fn calcular_tiempo_servicio_aux(anos: u32) -> u32 {
    // Equivalente a $anos['n'] en PHP
    // Si meses > 5, suma 1 año
    // Por ahora simplificado - retorna el mismo valor
    anos
}

fn parsear_fecha(fecha: &str) -> Option<NaiveDate> {
    if fecha.is_empty() {
        return None;
    }
    let limpia = fecha.split('T').next().unwrap_or(fecha);
    NaiveDate::parse_from_str(limpia, "%Y-%m-%d").ok()
}

/// Equivalente a GenerarAlicuotaAguinaldo
fn calcular_alicuota_aguinaldo(sueldo_mensual: f64, f_retiro: &str) -> f64 {
    let f_retiro_date = parsear_fecha(f_retiro);

    let dias = if f_retiro_date.is_none() {
        120 // Activo
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

/// Equivalente a GenerarAlicuotaVacaciones
fn calcular_alicuota_vacaciones(
    sueldo_mensual: f64,
    f_retiro: &str,
    tiempo_servicio: u32,
) -> (f64, u32) {
    let f_retiro_date = parsear_fecha(f_retiro);

    let (dias, factor) = if f_retiro_date.is_none() || f_retiro.is_empty() {
        (50, 1) // Activo
    } else if let Some(fr) = f_retiro_date {
        if fr.year() > 2016 {
            (50, 1)
        } else if fr.year() == 2016 && fr.month() <= 12 {
            // Retirado en 2016 o antes - depende del tiempo de servicio
            if tiempo_servicio > 0 && tiempo_servicio <= 14 {
                (40, 1)
            } else if tiempo_servicio > 14 && tiempo_servicio <= 24 {
                (45, 1)
            } else {
                (50, 1)
            }
        } else {
            (50, 1)
        }
    } else {
        (50, 1)
    };

    let monto = ((dias as f64 * sueldo_mensual) / 30.0) / 12.0;
    (monto, dias * factor)
}

/// Equivalente a GenerarDiasAdicionales
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

/// Busca el deposito banco del movimiento
fn buscar_deposito_banco(_patterns: &str, _movimientos: &[Movimiento]) -> f64 {
    // patterns es la clave, pero en Movimiento la clave es cedula
    // Por ahora retornamos 0 - se necesita mapeo correcto
    // TODO: Mapear correctamente
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sueldo_mensual() {
        let mut base = Base {
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
        };

        let movimientos = vec![];
        calcular_nomina_completa(&mut base, &movimientos);

        assert!(base.sueldo_mensual > 0.0);
        assert!(base.aguinaldos > 0.0);
        assert!(base.vacaciones > 0.0);
    }
}
