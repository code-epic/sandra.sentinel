use regex::Regex;

/// Completa con ceros (u otro caracter) a la izquierda hasta alcanzar la longitud deseada.
pub fn pad_left(input: &str, target_len: usize, pad_char: char) -> String {
    let len = input.chars().count();
    if len >= target_len {
        return input.to_string();
    }
    let pad_len = target_len - len;
    let mut result = String::with_capacity(target_len);
    for _ in 0..pad_len {
        result.push(pad_char);
    }
    result.push_str(input);
    result
}

/// Completa con ceros (u otro caracter) a la derecha hasta alcanzar la longitud deseada.
pub fn pad_right(input: &str, target_len: usize, pad_char: char) -> String {
    let len = input.chars().count();
    if len >= target_len {
        return input.to_string();
    }
    let pad_len = target_len - len;
    let mut result = String::with_capacity(target_len);
    result.push_str(input);
    for _ in 0..pad_len {
        result.push(pad_char);
    }
    result
}

/// Elimina caracteres especiales de una cadena, manteniendo solo alfanuméricos, espacios y guiones.
/// Útil para nombres de archivos o identificadores.
pub fn clean_special_chars(input: &str) -> String {
    // Regex para mantener solo a-z, A-Z, 0-9, espacio, guion y guion bajo
    let re = Regex::new(r"[^a-zA-Z0-9\s\-_]").unwrap();
    re.replace_all(input, "").to_string()
}
