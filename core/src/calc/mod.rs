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

/// Procesa un registro de Base individualmente:
/// 1. Calcula tiempo de servicio (y actualiza antigüedad).
/// 2. Busca y asigna el sueldo base correcto según Directiva.
pub fn procesar_registro_base(base: &mut Base, directivas: &Vec<Directiva>) {
    // 1. Calcular Tiempo y actualizar antigüedad decimal
    let tiempo = calcular_tiempo_servicio(base);
    base.antiguedad = tiempo.antiguedad;
    base.antiguedad_grado = tiempo.antiguedad_grados;

    // 2. Determinar y asignar Sueldo Base
    if let Some(sueldo) = obtener_sueldo_base(base.grado_id, tiempo.anos, directivas) {
        base.sueldo_base = sueldo;
    }
}

/// Equivalente a __fechaReconocida y TiempoServicios
pub fn calcular_tiempo_servicio(base: &Base) -> TiempoServicio {
    let f_retiro = parsear_fecha_o_hoy(base.f_retiro.as_deref());

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

/// Busca el sueldo en la Directiva según Grado y Tiempo de Servicio
pub fn obtener_sueldo_base(
    grado_id: u32,
    anos_servicio: u32,
    directivas: &Vec<Directiva>,
) -> Option<f64> {
    // 1. Filtrar directiva por código de grado (o ID)
    let directivas_grado: Vec<&Directiva> = directivas
        .iter()
        .filter(|d| d.grado_id == grado_id)
        .collect();

    // 2. Buscar el rango de antigüedad que corresponde
    let mut candidato = directivas_grado
        .iter()
        .filter(|d| d.antiguedad <= anos_servicio)
        .max_by_key(|d| d.antiguedad);

    if candidato.is_none() {
        candidato = directivas_grado.iter().min_by_key(|d| d.antiguedad);
    }

    candidato.map(|d| d.sueldo_base)
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
/// Retorna tiempo transcurrido entre f_inicio y f_fin
fn restar_fechas(f_inicio: NaiveDate, f_fin: NaiveDate, f_grado: NaiveDate) -> TiempoServicio {
    let (d1, m1, y1) = (
        f_inicio.day() as i32,
        f_inicio.month() as i32,
        f_inicio.year() as i32,
    );
    let (d2, m2, y2) = (
        f_fin.day() as i32,
        f_fin.month() as i32,
        f_fin.year() as i32,
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

    // Calcular decimal para comparaciones (ej: 15.5 años)
    let decimal = anos as f64 + (meses as f64 / 12.0) + (dias as f64 / 360.0);

    // Calcular antigüedad en grado (diferencia f_fin - f_grado)
    // Usamos lógica simplificada de diferencia de días / 365
    let diff_grado_dias = f_fin.signed_duration_since(f_grado).num_days();
    let antiguedad_grado_anos = (diff_grado_dias as f64 / 365.25).max(0.0) as u32;

    TiempoServicio {
        anos: anos.max(0) as u32,
        meses: meses.max(0) as u32,
        dias: dias.max(0) as u32,
        antiguedad: decimal as u32,
        antiguedad_grados: antiguedad_grado_anos,
    }
}
