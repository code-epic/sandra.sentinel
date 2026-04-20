use crate::banco::tipos::CampoBanco;
use crate::kernel::logica::exportador::{comprimir_y_sellar, generar_hash, ResultadoExport};
use crate::kernel::logica::memoria::Beneficiario;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const PLAN_LOKI: &str = "03487";

pub struct GeneradorLoki;

impl GeneradorLoki {
    pub fn generar_linea_apertura(b: &Beneficiario, porcentaje: f64) -> String {
        let mut campo = CampoBanco::default();
        campo.plan = PLAN_LOKI.to_string();
        campo.nac = "V".to_string();
        campo.cedula = Self::limpiar_cedula(&b.cedula);

        let nombre_completo = format!("{} {}", b.nombres, b.apellidos);
        campo.nombre = Some(Self::pad_right(&nombre_completo, 60, ' '));
        campo.edocivil = Some("S".to_string());

        let m = &b.movimientos;
        let monto_raw = (m.cap_banco + m.anticipo + m.dep_adicional + m.dep_garantia + m.anticipor)
            * porcentaje
            / 100.0;
        campo.monto = monto_raw;

        let monto_str = Self::pad_left(&Self::remove_decimal(monto_raw), 13, '0');
        let nombre_str = campo.nombre.as_deref().unwrap_or("");
        let edocivil_str = campo.edocivil.as_deref().unwrap_or("S");

        format!(
            "{}{}{}{}{}{}{}",
            campo.plan,
            campo.nac,
            campo.cedula,
            nombre_str,
            edocivil_str,
            "00000000000000000000000",
            monto_str
        )
    }

    pub fn generar_txt_apertura(
        beneficiarios: &[Beneficiario],
        ciclo: &str,
        destino: &str,
        porcentaje: f64,
        comprimir: bool,
        nivel_compresion: i32,
    ) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
        let nombre_archivo = format!("LOKI_APERT_{}.txt", ciclo);
        let ruta = if destino == "." || destino.is_empty() {
            PathBuf::from(&nombre_archivo)
        } else {
            PathBuf::from(destino).join(&nombre_archivo)
        };

        println!(
            "> Generando TXT LOKI Apertura en '{}' ({} registros, {}% monto)...",
            ruta.display(),
            beneficiarios.len(),
            porcentaje
        );

        let mut archivo = File::create(&ruta)?;
        let mut cantidad = 0;
        let mut suma_total = 0.0;

        for b in beneficiarios {
            let m = &b.movimientos;
            let tiene_mov =
                m.cap_banco + m.anticipo + m.dep_adicional + m.dep_garantia + m.anticipor > 0.0;

            if !tiene_mov {
                let linea = Self::generar_linea_apertura(b, porcentaje);
                if !linea.is_empty() {
                    writeln!(archivo, "{}", linea)?;
                    cantidad += 1;
                    let monto_mov =
                        m.cap_banco + m.anticipo + m.dep_adicional + m.dep_garantia + m.anticipor;
                    suma_total += monto_mov * porcentaje / 100.0;
                }
            }
        }

        archivo.flush()?;
        archivo.sync_all()?;

        let datos = std::fs::read(&ruta)?;
        let tamano_original = datos.len() as u64;
        let hash_original = generar_hash(&datos);

        if comprimir {
            println!(
                "    > Comprimiendo archivo LOKI con zstd (nivel {})...",
                nivel_compresion
            );

            let (comprimido, hash) = comprimir_y_sellar(&datos, nivel_compresion);

            let ruta_zst = ruta.with_extension("txt.zst");
            let mut archivo_zst = File::create(&ruta_zst)?;
            archivo_zst.write_all(&comprimido)?;

            std::fs::remove_file(&ruta)?;

            let tamano_comprimido = comprimido.len() as u64;
            let nombre_final = ruta_zst
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();

            println!(
                "    > LOKI Apertura generado: Original: {} bytes, Comprimido: {} bytes, Hash: {}",
                tamano_original,
                tamano_comprimido,
                &hash[..16]
            );

            Ok(ResultadoExport {
                ruta: nombre_final,
                tipo: "loki_apertura".to_string(),
                tamano_original,
                tamano_comprimido: Some(tamano_comprimido),
                hash_sha256: Some(hash),
                hash_sha256_original: Some(hash_original),
                compresion_aplicada: true,
            })
        } else {
            println!(
                "    > LOKI Apertura generado: {} bytes, {} registros, Total: {:.2}, Hash: {}",
                tamano_original,
                cantidad,
                suma_total,
                &hash_original[..16]
            );

            Ok(ResultadoExport {
                ruta: ruta.display().to_string(),
                tipo: "loki_apertura".to_string(),
                tamano_original,
                tamano_comprimido: None,
                hash_sha256: Some(hash_original),
                hash_sha256_original: None,
                compresion_aplicada: false,
            })
        }
    }

    fn limpiar_cedula(cedula: &str) -> String {
        cedula
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
    }

    fn pad_left(input: &str, target_len: usize, pad_char: char) -> String {
        let len = input.len();
        if len >= target_len {
            return input[..target_len].to_string();
        }
        let pad_len = target_len - len;
        let mut result = String::with_capacity(target_len);
        for _ in 0..pad_len {
            result.push(pad_char);
        }
        result.push_str(input);
        result
    }

    fn pad_right(input: &str, target_len: usize, pad_char: char) -> String {
        let len = input.len();
        if len >= target_len {
            return input[..target_len].to_string();
        }
        let pad_len = target_len - len;
        let mut result = String::with_capacity(target_len);
        result.push_str(input);
        for _ in 0..pad_len {
            result.push(pad_char);
        }
        result
    }

    fn remove_decimal(value: f64) -> String {
        (value * 100.0).round().to_string()
    }
}
