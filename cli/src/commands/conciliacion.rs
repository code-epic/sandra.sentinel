use sandra_core::System;

pub async fn execute(archivo: Option<String>) {
    let _system = System::init(); // Core necesario para cálculos
    match archivo {
        Some(path) => println!("Procesando conciliación desde: {}", path),
        None => println!("Procesando conciliación estándar..."),
    }
}
