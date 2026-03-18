use crate::kernel::logica::exportador::{comprimir_y_sellar, generar_hash, ResultadoExport};
use crate::kernel::logica::memoria::Beneficiario;
use chrono::Local;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const CODIGO_EMPRESA_BANFANB: &str = "0131";

pub fn generar_linea_banfanb(b: &Beneficiario) -> String {
    let monto = b.base.garantias * 100.0;
    let monto_str = pad_left(&remove_decimal(monto), 12, '0');
    let cuenta = pad_left(&b.numero_cuenta, 20, '0');
    let cedula = pad_left(
        &b.cedula.replace(|c: char| !c.is_ascii_digit(), ""),
        10,
        '0',
    );

    format!(
        "{}{}{}{}00000{}00\r\n",
        CODIGO_EMPRESA_BANFANB, monto_str, cuenta, cedula, "0"
    )
}

pub fn generar_cabecera_banfanb(cantidad: u32, monto_total: f64, fecha: &str) -> String {
    let monto_str = pad_left(&remove_decimal(monto_total), 17, '0');
    let cant_str = pad_left(&cantidad.to_string(), 4, '0');

    format!(
        "{}{}{}{}\r\n",
        CODIGO_EMPRESA_BANFANB, fecha, monto_str, cant_str
    )
}

pub fn generar_txt_banfanb(
    beneficiarios: &[Beneficiario],
    ciclo: &str,
    destino: &str,
    _codigo_empresa: &str,
    comprimir: bool,
    nivel_compresion: i32,
) -> Result<ResultadoExport, Box<dyn std::error::Error>> {
    let nombre_archivo = format!("banfanb_{}.txt", ciclo);
    let ruta = if destino == "." || destino.is_empty() {
        PathBuf::from(&nombre_archivo)
    } else {
        PathBuf::from(destino).join(&nombre_archivo)
    };

    println!(
        "> Generando TXT Banfanb en '{}' ({} registros)...",
        ruta.display(),
        beneficiarios.len()
    );

    let fecha = Local::now().format("%d%m%y").to_string();
    let mut archivo = File::create(&ruta)?;
    let mut cantidad = 0;
    let mut suma_total = 0.0;
    let mut lineas_detalle = String::new();

    for b in beneficiarios {
        if b.base.garantias > 0.0 && !b.numero_cuenta.is_empty() {
            let monto = b.base.garantias * 100.0;
            let monto_str = pad_left(&remove_decimal(monto), 12, '0');
            let cuenta = pad_left(&b.numero_cuenta, 20, '0');
            let cedula = pad_left(
                &b.cedula.replace(|c: char| !c.is_ascii_digit(), ""),
                10,
                '0',
            );

            lineas_detalle.push_str(&format!(
                "{}{}{}{}00000{}00\r\n",
                CODIGO_EMPRESA_BANFANB, monto_str, cuenta, cedula, "0"
            ));

            cantidad += 1;
            suma_total += b.base.garantias;
        }
    }

    let cabecera = generar_cabecera_banfanb(cantidad as u32, suma_total, &fecha);
    archivo.write_all(cabecera.as_bytes())?;
    archivo.write_all(lineas_detalle.as_bytes())?;
    archivo.flush()?;
    archivo.sync_all()?;

    let datos = std::fs::read(&ruta)?;
    let tamano_original = datos.len() as u64;
    let hash_original = generar_hash(&datos);

    if comprimir {
        println!(
            "    > Comprimiendo archivo Banfanb con zstd (nivel {})...",
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
            "    > Banfanb generado: Original: {} bytes, Comprimido: {} bytes, Hash: {}",
            tamano_original,
            tamano_comprimido,
            &hash[..16]
        );

        Ok(ResultadoExport {
            ruta: nombre_final,
            tipo: "banfanb".to_string(),
            tamano_original,
            tamano_comprimido: Some(tamano_comprimido),
            hash_sha256: Some(hash),
            hash_sha256_original: Some(hash_original),
            compresion_aplicada: true,
        })
    } else {
        println!(
            "    > Banfanb generado: {} bytes, {} registros, Total: {:.2}, Hash: {}",
            tamano_original,
            cantidad,
            suma_total,
            &hash_original[..16]
        );

        Ok(ResultadoExport {
            ruta: ruta.display().to_string(),
            tipo: "banfanb".to_string(),
            tamano_original,
            tamano_comprimido: None,
            hash_sha256: Some(hash_original),
            hash_sha256_original: None,
            compresion_aplicada: false,
        })
    }
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

fn remove_decimal(value: f64) -> String {
    value.round().to_string()
}
