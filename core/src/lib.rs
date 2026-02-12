pub mod kernel;
pub mod model;
pub mod calc;
pub mod system;

// Punto de entrada del sistema
pub struct System {
    pub kernel: kernel::Perceptron,
    pub config: system::config::Config,
}

impl System {
    pub fn init() -> Self {
        println!("Iniciando Core...");

        // 0. Cargar Configuración
        let config = system::config::Config::load();
        println!("Configuración Cargada (v{})", config.version);

        // 1. Cargar Modelos de Datos
        // Aquí se cargan estructuras, validaciones, etc.
        println!("Modelos Cargados");

        // 2. Inicializar Kernel (Perceptrón/Memoria)
        let kernel = kernel::Perceptron::new();
        println!("Kernel Cargado");

        // 3. Cargar Motor de Cálculo
        // Aquí se verifican fórmulas, tablas, etc.
        println!("Calculos Cargado");

        System {
            kernel,
            config,
        }
    }

    pub async fn connect_sandra(&mut self, url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
         self.kernel.connect_to_sandra(url).await
    }
}
