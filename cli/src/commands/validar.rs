pub fn execute(clave: String) {
    println!("Validando clave: {}", clave);
    // Lógica de validación dummy
    if clave == "1234" {
        println!("Clave VALIDA");
    } else {
        println!("Clave INVALIDA");
    }
}
