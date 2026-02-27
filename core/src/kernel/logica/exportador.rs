use super::logger;
use crate::kernel::logica::memoria::Beneficiario;
use std::fs::File;
use std::path::Path;

pub fn exportar_nomina_csv(
    beneficiarios: &Vec<Beneficiario>,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "> Exportando nómina a CSV en '{}' ({} registros)...",
        path.display(),
        beneficiarios.len()
    );

    let file = File::create(path)?;
    let mut wtr = csv::Writer::from_writer(file);

    // 1. Definir Cabeceras (Flattened)
    wtr.write_record(&[
        // IDENTIFICACION
        "cedula",
        "nombres",
        "apellidos",
        "sexo",
        "edo_civil",
        "n_hijos (base)", // Usamos base, que tiene mas sentido
        "componente_id",
        "grado_id (base)",
        "categoria",
        "status_id",
        "status",
        "st_no_ascenso",
        // FECHAS BASE
        "fecha_ingreso",
        "fecha_ultimo_ascenso",
        "fecha_retiro",
        // RECONOCIMIENTO
        "anio_reconocido",
        "mes_reconocido",
        "dia_reconocido",
        // TIEMPO SERVICIO CALCULADO
        "antiguedad_total (decimal)",
        "antiguedad_grado (anos)",
        // SUELDOS Y PRIMAS (BASE)
        "sueldo_base",
        "prima_antiguedad",
        "prima_hijos",
        "prima_profesionalizacion",
        "total_asignaciones_base",
        // MOVIMIENTOS
        "cap_banco",
        "anticipo",
        "f_cap_banco",
        "dif_asi_anti",
        "anticipo_retroactivo",
        "dep_adicional",
        "dep_garantia",
        // PATRON
        "patterns",
    ])?;

    // 2. Iterar y Escribir Filas
    for b in beneficiarios {
        // Preparar valores (manejo de Options y Defaults)
        let cedula = &b.cedula;
        let nombres = &b.nombres;
        let apellidos = &b.apellidos;
        let sexo = b.sexo.as_deref().unwrap_or("");
        let edo_civil = b.edo_civil.as_deref().unwrap_or("");

        // Base Data
        let n_hijos = b.base.n_hijos.to_string();
        let comp_id = b.componente_id.to_string(); // From Beneficiario
        let grado_id = b.base.grado_id.to_string();
        let cat = b.categoria.as_deref().unwrap_or("");
        let status_id = b.status_id.to_string();
        let status = b.status.to_string();
        let st_no_ascenso = b.st_no_ascenso.to_string();

        // Fechas (Preferimos Base si existe, sino Beneficiario)
        let f_ingreso = b
            .base
            .fecha_ingreso
            .as_deref()
            .unwrap_or(b.f_ingreso_sistema.as_deref().unwrap_or(""));
        let f_ascenso = b
            .base
            .f_ult_ascenso
            .as_deref()
            .unwrap_or(b.f_ult_ascenso.as_deref().unwrap_or(""));
        let f_retiro = b
            .base
            .f_retiro
            .as_deref()
            .unwrap_or(b.f_retiro.as_deref().unwrap_or(""));

        // Reconocimiento
        let a_recon = b.base.anio_reconocido.to_string();
        let m_recon = b.base.mes_reconocido.to_string();
        let d_recon = b.base.dia_reconocido.to_string();

        // Calculados
        let antig_total = format!("{:.4}", b.base.antiguedad); // Decimal presision
        let antig_grado = b.base.antiguedad_grado.to_string();

        // Financiero Base
        let sueldo = format!("{:.2}", b.base.sueldo_base);

        // Helper inline para extraer de calculos
        let get_calc = |key: &str| -> String {
            if let Some(map) = &b.base.calculos {
                if let Some(val) = map.get(key) {
                    return format!("{:.2}", val);
                }
            }
            "0.00".to_string()
        };

        // Extraemos las primas dinámicamente usando sus códigos reales en BD
        let p_antig = get_calc("prima_tiemposervicio"); // O prima_antiguedad según BD
        let p_hijos = get_calc("prima_hijos");
        // Agregamos otras importantes si quieres
        let p_profe = get_calc("prima_profesionalizacion");

        let total_asig = format!("{:.2}", b.base.total_asignaciones);

        // Movimientos
        let cap_banco = format!("{:.2}", b.movimientos.cap_banco);
        let anticipo = format!("{:.2}", b.movimientos.anticipo);
        let f_cap_banco = format!("{:.2}", b.movimientos.fcap_banco);
        let dif_asi = format!("{:.2}", b.movimientos.dif_asi_anti);
        let anti_r = format!("{:.2}", b.movimientos.anticipor);
        let dep_ad = format!("{:.2}", b.movimientos.dep_adicional);
        let dep_gar = format!("{:.2}", b.movimientos.dep_garantia);

        let patterns = &b.patterns;

        wtr.write_record(&[
            cedula,
            nombres,
            apellidos,
            sexo,
            edo_civil,
            &n_hijos,
            &comp_id,
            &grado_id,
            cat,
            &status_id,
            &status,
            &st_no_ascenso,
            f_ingreso,
            f_ascenso,
            f_retiro,
            &a_recon,
            &m_recon,
            &d_recon,
            &antig_total,
            &antig_grado,
            &sueldo,
            &p_antig,
            &p_hijos,
            &p_profe,
            &total_asig,
            &cap_banco,
            &anticipo,
            &f_cap_banco,
            &dif_asi,
            &anti_r,
            &dep_ad,
            &dep_gar,
            patterns,
        ])?;
    }

    wtr.flush()?;
    logger::log_info("EXPORT", "Archivo CSV generado correctamente.");
    Ok(())
}
