pub mod calculos;
pub mod motor;
use crate::kernel::logica::memoria::{Base, Directiva};
use chrono::{Datelike, Local, NaiveDate};

#[derive(Debug, Clone)]
pub struct TiempoServicio {
    pub anos: u32,
    pub meses: u32,
    pub dias: u32,
    pub antiguedad: u32, //Para cálculos numéricos
    pub antiguedad_grados: u32,
}

pub fn is_debug() -> bool {
    std::env::var("SANDRA_DEBUG").is_ok()
}

/// Procesa un registro de Base individualmente:
/// 1. Calcula tiempo de servicio (y actualiza antigüedad).
/// 2. Busca y asigna el sueldo base correcto según Directiva.
pub fn procesar_registro_base(base: &mut Base, directivas: &Vec<Directiva>) {
    // 1. Calcular Tiempo y actualizar antigüedad decimal
    let tiempo = calcular_tiempo_servicio(base);
    base.antiguedad = tiempo.antiguedad;
    base.antiguedad_grado = tiempo.antiguedad_grados;

    if is_debug() {
        eprintln!(
            "[DEBUG] procesar_registro_base: grado_id={}, anos_servicio={}",
            base.grado_id, tiempo.anos
        );
    }

    // 2. Determinar y asignar Sueldo Base y variables de directiva
    if let Some(directiva) = obtener_directiva(base.grado_id, base.antiguedad_grado, directivas) {
        if is_debug() {
            eprintln!(
                "[DEBUG]   -> Sueldo base asignado: {}",
                directiva.sueldo_base
            );
        }
        base.sueldo_base = directiva.sueldo_base;
        base.unidad_tributaria = directiva.unidad_tributaria;
        base.salario_minimo = directiva.salario_minimo;
    } else {
        if is_debug() {
            eprintln!(
                "[DEBUG]   -> NO SE ENCONTRO SUELDO para grado_id={}, anos={}",
                base.grado_id, tiempo.anos
            );
            eprintln!(
                "[DEBUG]   -> Directivas disponibles para grado_id={}: {} registros",
                base.grado_id,
                directivas
                    .iter()
                    .filter(|d| d.grado_id == base.grado_id)
                    .count()
            );

            // Mostrar las directivas disponibles para este grado
            for d in directivas
                .iter()
                .filter(|d| d.grado_id == base.grado_id)
                .take(3)
            {
                eprintln!(
                    "[DEBUG]      Directiva: grado={}, antiguedad={}, sueldo={}",
                    d.grado_id, d.antiguedad, d.sueldo_base
                );
            }
        }
    }
}

/// Equivalente a __fechaReconocida y TiempoServicios
/// Fecha de corte: `f_retiro_efectiva` (prioridad) → `f_retiro` (fallback) → hoy
pub fn calcular_tiempo_servicio(base: &Base) -> TiempoServicio {
    // f_retiro_efectiva es la fecha canónica de corte cuando existe.
    // Si es None o vacía, se cae en f_retiro (retrocompatibilidad).
    let fecha_corte_str = base
        .f_retiro_efectiva
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .or(base.f_retiro.as_deref());
    let f_retiro = parsear_fecha_o_hoy(fecha_corte_str);

    let fecha_ing_str = base.fecha_ingreso.as_deref().unwrap_or("");
    // Limpiamos la fecha por si viene con hora (ISO 8601: "YYYY-MM-DDTHH:MM:SSZ")
    let fecha_limpia = fecha_ing_str.split('T').next().unwrap_or("");

    // Fecha Ingreso por defecto (si falla parseo se usa HOY -> 0 antigüedad)
    let f_ingreso_defecto = NaiveDate::parse_from_str(fecha_limpia, "%Y-%m-%d")
        .unwrap_or_else(|_| Local::now().date_naive());

    // Determinamos fecha ultimo ascenso para antigüedad de grado
    let f_ascenso_str = base.f_ult_ascenso.as_deref().unwrap_or("");
    let f_ascenso_clean = f_ascenso_str.split('T').next().unwrap_or("");
    let f_ascenso =
        NaiveDate::parse_from_str(f_ascenso_clean, "%Y-%m-%d").unwrap_or(f_ingreso_defecto); // Si no tiene fecha ascenso, se asume fecha ingreso

    // Caso 1: Tiene años reconocidos (Ajuste de fecha de ingreso)
    if base.anio_reconocido > 0 || base.mes_reconocido > 0 || base.dia_reconocido > 0 {
        if let Ok(f_ingreso) = NaiveDate::parse_from_str(fecha_limpia, "%Y-%m-%d") {
            // Lógica PHP: $anoR = $ano - $this->baseeficiario->ano_reconocido; ...
            // Restar periodo reconocido a la fecha de ingreso para obtener "fecha ficticia"
            // Esta lógica es compleja de replicar exactamente igual con chrono ops directos
            // por el manejo de "meses de 30 dias" vs calendario real.
            // Usaremos una aproximación robusta o la lógica manual si se requiere exactitud contable militar.

            // Implementación estilo PHP manual para coincidir con lógica heredada
            let (y, m, d) = (f_ingreso.year(), f_ingreso.month(), f_ingreso.day());

            let mut y_r = y - (base.anio_reconocido as i32);
            let mut m_r = (m as i32) - (base.mes_reconocido as i32);
            let mut d_r = (d as i32) - (base.dia_reconocido as i32);

            if d_r < 1 {
                m_r -= 1;
                d_r += 30; // Asumiendo mes comercial/militar de 30 días para ajuste? O mes anterior real?
                           // PHP code: $diaR = 30 + $diaR; (Esto sugiere mes comercial o simplificación)
            }
            if m_r < 1 {
                y_r -= 1;
                m_r += 12;
            }

            // Construir fecha reconocida aproximada
            // Nota: Si d_r > 28/30/31 puede fallar NaiveDate, hay que clamp
            let d_r_safe = d_r.max(1).min(30) as u32;
            let m_r_safe = m_r.max(1).min(12) as u32;

            if let Some(fecha_reconocida) = NaiveDate::from_ymd_opt(y_r, m_r_safe, d_r_safe) {
                return restar_fechas(fecha_reconocida, f_retiro, f_ascenso);
            }
        }
    }

    // Caso 2: Ingreso normal
    restar_fechas(f_ingreso_defecto, f_retiro, f_ascenso)
}

/// Busca la Directiva según Grado y Tiempo de Servicio
pub fn obtener_directiva<'a>(
    grado_id: u32,
    anos_servicio: u32,
    directivas: &'a Vec<Directiva>,
) -> Option<&'a Directiva> {
    if is_debug() {
        eprintln!(
            "[DEBUG] obtener_directiva: grado_id={}, anos_servicio={}",
            grado_id, anos_servicio
        );
    }

    // 1. Filtrar directiva por código de grado (o ID)
    let directivas_grado: Vec<&Directiva> = directivas
        .iter()
        .filter(|d| d.grado_id == grado_id)
        .collect();

    if is_debug() {
        eprintln!(
            "[DEBUG]   Directivas para grado_id={}: {} registros",
            grado_id,
            directivas_grado.len()
        );
    }

    if directivas_grado.is_empty() {
        if is_debug() {
            eprintln!("[DEBUG]   -> NO hay directivas para este grado_id!");
            eprintln!(
                "[DEBUG]   -> Total de directivas disponibles: {}",
                directivas.len()
            );

            // Mostrar algunos ejemplos de grados disponibles
            let grados_disponibles: std::collections::HashSet<_> =
                directivas.iter().map(|d| d.grado_id).collect();
            let grados_vec: Vec<_> = grados_disponibles.iter().take(10).collect();
            eprintln!("[DEBUG]   -> Grados disponibles: {:?}", grados_vec);
        }
        return None;
    }

    // 2. Buscar el rango de antigüedad que corresponde
    let mut candidato = directivas_grado
        .iter()
        .filter(|d| d.antiguedad <= anos_servicio)
        .max_by_key(|d| d.antiguedad);

    if candidato.is_none() {
        candidato = directivas_grado.iter().min_by_key(|d| d.antiguedad);
    }

    if is_debug() {
        if let Some(d) = candidato {
            eprintln!(
                "[DEBUG]   Candidato: antiguedad={}, sueldo={}",
                d.antiguedad, d.sueldo_base
            );
        } else {
            eprintln!("[DEBUG]   -> NO se encontro candidato");
        }
    }

    candidato.copied()
}

// --- UTILIDADES ---

fn parsear_fecha_o_hoy(fecha: Option<&str>) -> NaiveDate {
    match fecha {
        Some(s) if !s.is_empty() => {
            // Limpiamos formato ISO ("YYYY-MM-DDTHH:MM:SSZ") -> "YYYY-MM-DD"
            let s_clean = s.split('T').next().unwrap_or("");
            NaiveDate::parse_from_str(s_clean, "%Y-%m-%d")
                .unwrap_or_else(|_| Local::now().date_naive())
        }
        _ => Local::now().date_naive(),
    }
}

/// Equivalente a __restarFecha
/// Retorna tiempo transcurrido entre f_inicio y f_fin.
/// También calcula la antigüedad en el grado (años transcurridos
/// desde el último ascenso hasta la fecha de corte) de forma calendárica.
fn restar_fechas(f_inicio: NaiveDate, f_fin: NaiveDate, f_grado: NaiveDate) -> TiempoServicio {
    let (anos, meses, dias) = diferencia_calendario(f_inicio, f_fin);

    // Calcular decimal para comparaciones (ej: 15.5 años)
    let decimal = anos as f64 + (meses as f64 / 12.0) + (dias as f64 / 360.0);

    // Antigüedad en el grado: diferencia calendárica entre f_grado y f_fin.
    // Se calcula con el mismo algoritmo de préstamo (años/meses/días) para
    // evitar truncamientos por división de días / 365.25 (ej: 2 años exactos
    // = 730 días / 365.25 = 1.9986 -> truncaría a 1). Con diferencia
    // calendárica el resultado es exacto.
    let (anos_grado, ..) = diferencia_calendario(f_grado, f_fin);

    TiempoServicio {
        anos: anos.max(0) as u32,
        meses: meses.max(0) as u32,
        dias: dias.max(0) as u32,
        antiguedad: decimal as u32,
        antiguedad_grados: anos_grado.max(0) as u32,
    }
}

/// Calcula la diferencia calendárica (años, meses, días) entre dos fechas
/// usando préstamo de días del mes anterior (30 días) y meses del año
/// anterior (12 meses), equivalente a la lógica legacy __restarFecha.
fn diferencia_calendario(desde: NaiveDate, hasta: NaiveDate) -> (i32, i32, i32) {
    let (d1, m1, y1) = (
        desde.day() as i32,
        desde.month() as i32,
        desde.year(),
    );
    let (d2, m2, y2) = (
        hasta.day() as i32,
        hasta.month() as i32,
        hasta.year(),
    );

    let mut anos = y2 - y1;
    let mut meses = m2 - m1;
    let mut dias = d2 - d1;

    if dias < 0 {
        // Préstamo de días del mes anterior
        // PHP legacy: $dia_dif = ($dia_r + 30) - $dia;
        dias += 30;
        meses -= 1;
    }

    if meses < 0 {
        // Préstamo de meses del año anterior
        meses += 12;
        anos -= 1;
    }

    (anos, meses, dias)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construye una Base con las fechas del caso de prueba reportado.
    fn crear_base(case: &str, f_ingreso: &str, f_ult_ascenso: &str, f_retiro: &str) -> Base {
        let mut b = Base::default();
        b.fecha_ingreso = Some(f_ingreso.to_string());
        b.f_ult_ascenso = Some(f_ult_ascenso.to_string());
        b.f_retiro = Some(f_retiro.to_string());
        b.patterns = format!("test-{}", case);
        b
    }

    /// Caso reportado por el usuario: retiro 2011-07-05, ascenso 2009-07-05.
    /// La antigüedad en el grado debe ser exactamente 2 años.
    #[test]
    fn antiguedad_grado_dos_anos_exactos() {
        let base = crear_base(
            "dos-anos",
            "1990-01-01",
            "2009-07-05",
            "2011-07-05",
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 2,
            "antigüedad en grado debe ser 2 años (2009-07-05 a 2011-07-05), fue {}",
            t.antiguedad_grados
        );
    }

    /// Un año exacto NO bisiesto (365 días): la división 365/365.25=0.999
    /// truncaría a 0; con diferencia calendárica debe ser 1.
    #[test]
    fn antiguedad_grado_un_ano_exacto_no_bisiesto() {
        let base = crear_base(
            "un-ano",
            "1990-01-01",
            "2008-03-15",
            "2009-03-15",
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 1,
            "antigüedad en grado debe ser 1 año, fue {}",
            t.antiguedad_grados
        );
    }

    /// Rango que cruza un año bisiesto (2012): 2010-06-01 a 2013-06-01 = 3 años.
    #[test]
    fn antiguedad_grado_tres_anos_con_bisiesto() {
        let base = crear_base(
            "tres-anos-bisiesto",
            "1990-01-01",
            "2010-06-01",
            "2013-06-01",
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 3,
            "antigüedad en grado debe ser 3 años, fue {}",
            t.antiguedad_grados
        );
    }

    /// Meses parciales: 2015-02-10 a 2016-11-20 = 1 año, 9 meses, 10 días.
    /// antiguedad_grados debe ser 1 (años completos).
    #[test]
    fn antiguedad_grado_meses_parciales_trunca_anos() {
        let base = crear_base(
            "meses-parciales",
            "1990-01-01",
            "2015-02-10",
            "2016-11-20",
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 1,
            "con 1 año 9 meses la antigüedad en grado debe ser 1, fue {}",
            t.antiguedad_grados
        );
    }

    /// Días parciales: 2018-01-01 a 2018-06-30 (6 meses, sin año completo).
    /// antiguedad_grados debe ser 0.
    #[test]
    fn antiguedad_grado_menos_de_un_ano() {
        let base = crear_base(
            "menos-un-ano",
            "1990-01-01",
            "2018-01-01",
            "2018-06-30",
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 0,
            "con 6 meses la antigüedad en grado debe ser 0, fue {}",
            t.antiguedad_grados
        );
    }

    /// Verificamos que la antigüedad (tiempo de servicio total) también se
    /// calcula bien: f_ingreso 1990-01-01 a f_retiro 2011-07-05 = 21 años.
    #[test]
    fn antiguedad_total_se_mantiene() {
        let base = crear_base(
            "total",
            "1990-01-01",
            "2009-07-05",
            "2011-07-05",
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.anos, 21,
            "antigüedad total debe ser 21 años (1990-01-01 a 2011-07-05), fue {}",
            t.anos
        );
    }

    // =========================================================================
    // Tests de f_retiro_efectiva como fecha de corte con prioridad
    // =========================================================================

    /// Construye una Base con f_retiro_efectiva para tests de prioridad.
    fn crear_base_con_efectiva(
        case: &str,
        f_ingreso: &str,
        f_ult_ascenso: &str,
        f_retiro: &str,
        f_retiro_efectiva: Option<&str>,
    ) -> Base {
        let mut b = Base::default();
        b.fecha_ingreso = Some(f_ingreso.to_string());
        b.f_ult_ascenso = Some(f_ult_ascenso.to_string());
        b.f_retiro = Some(f_retiro.to_string());
        b.f_retiro_efectiva = f_retiro_efectiva.map(|s| s.to_string());
        b.patterns = format!("test-{}", case);
        b
    }

    /// Caso 1: f_retiro_efectiva existe y difiere de f_retiro.
    /// Debe usar f_retiro_efectiva como fecha de corte.
    /// f_retiro = 2013-07-05, f_retiro_efectiva = 2011-07-05, ascenso = 2009-07-05
    /// Antigüedad en grado con f_retiro_efectiva: 2009-07-05 → 2011-07-05 = 2 años
    /// Antigüedad total con f_retiro_efectiva: 1990-01-01 → 2011-07-05 = 21 años
    #[test]
    fn f_retiro_efectiva_tiene_prioridad_sobre_f_retiro() {
        let base = crear_base_con_efectiva(
            "efectiva-prioridad",
            "1990-01-01",
            "2009-07-05",
            "2013-07-05",     // f_retiro (ignorada)
            Some("2011-07-05"), // f_retiro_efectiva (usada)
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 2,
            "con f_retiro_efectiva=2011-07-05, antigüedad en grado debe ser 2 (desde 2009-07-05), fue {}",
            t.antiguedad_grados
        );
        assert_eq!(
            t.anos, 21,
            "con f_retiro_efectiva=2011-07-05, antigüedad total debe ser 21 (desde 1990-01-01), fue {}",
            t.anos
        );
    }

    /// Caso 2: f_retiro_efectiva es None (null) → fallback a f_retiro.
    /// Comportamiento idéntico al actual: f_retiro = 2011-07-05.
    #[test]
    fn f_retiro_efectiva_null_usa_f_retiro_fallback() {
        let base = crear_base_con_efectiva(
            "fallback",
            "1990-01-01",
            "2009-07-05",
            "2011-07-05", // f_retiro (usada como fallback)
            None,         // f_retiro_efectiva es null
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 2,
            "sin f_retiro_efectiva, debe usar f_retiro: antigüedad en grado = 2, fue {}",
            t.antiguedad_grados
        );
        assert_eq!(
            t.anos, 21,
            "sin f_retiro_efectiva, debe usar f_retiro: antigüedad total = 21, fue {}",
            t.anos
        );
    }

    /// Caso 3: Ambas fechas iguales (caso reportado por usuario).
    /// f_retiro = 2011-07-05, f_retiro_efectiva = 2011-07-05.
    /// Resultado debe ser 2 años de antigüedad en grado.
    #[test]
    fn f_retiro_efectiva_igual_a_f_retiro() {
        let base = crear_base_con_efectiva(
            "ambas-iguales",
            "1990-01-01",
            "2009-07-05",
            "2011-07-05",
            Some("2011-07-05"),
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 2,
            "con ambas fechas iguales a 2011-07-05, antigüedad en grado = 2, fue {}",
            t.antiguedad_grados
        );
    }

    /// Caso 4: f_retiro_efectiva vacía (string vacío) → fallback a f_retiro.
    #[test]
    fn f_retiro_efectiva_vacia_usa_f_retiro_fallback() {
        let base = crear_base_con_efectiva(
            "efectiva-vacia",
            "1990-01-01",
            "2009-07-05",
            "2011-07-05",
            Some(""), // f_retiro_efectiva es string vacío
        );
        let t = calcular_tiempo_servicio(&base);
        assert_eq!(
            t.antiguedad_grados, 2,
            "con f_retiro_efectiva vacía, debe usar f_retiro: antigüedad en grado = 2, fue {}",
            t.antiguedad_grados
        );
    }

    /// Caso 5: Solo f_retiro_efectiva sin f_retiro.
    /// f_retiro = None, f_retiro_efectiva = 2011-07-05.
    #[test]
    fn solo_f_retiro_efectiva_sin_f_retiro() {
        let mut b = Base::default();
        b.fecha_ingreso = Some("1990-01-01".to_string());
        b.f_ult_ascenso = Some("2009-07-05".to_string());
        b.f_retiro = None;
        b.f_retiro_efectiva = Some("2011-07-05".to_string());
        b.patterns = "test-solo-efectiva".to_string();
        let t = calcular_tiempo_servicio(&b);
        assert_eq!(
            t.antiguedad_grados, 2,
            "con solo f_retiro_efectiva=2011-07-05, antigüedad en grado = 2, fue {}",
            t.antiguedad_grados
        );
    }
}
