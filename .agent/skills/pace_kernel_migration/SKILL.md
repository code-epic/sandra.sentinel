---
name: Lógica de Migración y Kernel PACE
description: Guía integral sobre el Kernel PACE heredado (PHP), su arquitectura, fórmulas de cálculo y el concepto 'Perceptrón' para propósitos de migración.
---

# Guía de Kernel PACE y Migración

Esta habilidad documenta el sistema PHP heredado ubicado en `src/migracion/pace-kernel/`. Es fundamental para comprender la lógica de negocio de las Prestaciones Sociales de la Fuerza Armada Nacional Bolivariana (FANB) durante la migración a la nueva arquitectura Angular/Rust.

## 1. Arquitectura del Sistema

El sistema heredado sigue un patrón **Modelo-Vista-Controlador (MVC)**, probablemente basado en CodeIgniter.

### Estructura de Directorios

- **`models/`**: Contiene la lógica de negocio principal.
  - **`kernel/`**: El corazón del sistema. Contiene los motores de cálculo y el "Perceptrón".
  - **`motor/`**: Motores de inferencia (ej. `RInferencia.php`).
- **`views/`**: Plantillas PHP y archivos JavaScript para el frontend.
  - **`js/`**: Lógica del lado del cliente (ej. `registrar_finiquito.js`, `anticipo.js`).

## 2. El Concepto "Perceptrón" (`KPerceptron.php`)

En este contexto, el **Perceptrón** es una convención de nomenclatura metafórica para un **Mecanismo de Aprendizaje y Memoria de Patrones**. **No** es una red neuronal en el sentido moderno de ML, sino una utilidad de gestión de caché/estado.

### Modelo Conceptual

- **Neurona**: Un array que actúa como memoria a corto plazo o caché.
- **Aprender**: Almacenar un valor asociado con una clave (Patrón).
- **Recordar**: Recuperar un valor basado en una clave (Pensamiento).

```php
// Uso Metafórico
$this->KPerceptron->Aprender('sueldo_base', 1500.00);
$sueldo = $this->KPerceptron->Recordar('sueldo_base');
```

Esta abstracción permite que el sistema "aprenda" valores calculados durante una sesión para evitar recalcularlos, actuando como una capa de memoización.

## 3. Interfaz del Kernel e Iconos

La **Interfaz del Kernel** se refiere al conjunto de APIs Públicas expuestas por las clases en `models/kernel/`.

- **`KCalculo`**: Interfaz para cálculos individuales de beneficiarios.
- **`KCalculoLote`**: Interfaz para procesamiento por lotes.

**Iconos y Visuales**:

- El sistema heredado utiliza `ipsfa.png` (branding).
- Los archivos JavaScript en `views/js/` manejan la interactividad de la interfaz de usuario, a menudo manipulando elementos del DOM para mostrar/ocultar resultados de cálculos.

## 4. Objetos y Métodos Clave

### `MBeneficiario` (El Objeto Central)

Representa a un afiliado militar. Propiedades clave:

- `id`, `cedula`: Identificadores.
- `fingreso`, `fultimo_ascenso`, `fretiro`: Fechas críticas para cálculos.
- `grado_codigo`, `antiguedad_grado`: Determinantes para el sueldo.
- `Componente`, `Grado`, `Prima`: Objetos anidados.

### `MDirectiva` y `MPrima`

- **MDirectiva**: Configuración para sueldos y unidades (UT) por período.
- **MPrima**: Lógica para primas específicas (Descendencia, Transporte, etc.).

## 5. Pautas de Desarrollo (Heredado)

- **Convención de Nombres**: `K{NombreClase}` para modelos del Kernel (ej. `KCalculo`), `M{NombreClase}` para modelos de Datos (ej. `MBeneficiario`).
- **Indentación**: Indentación de 2 o 4 espacios (mixta en el código heredado).
- **Tipado**: PHP es de tipado dinámico, pero los métodos esperan estructuras de objetos específicas (ej. `MBeneficiario`).

## 6. Estrategia de Migración

Al migrar a Rust/Angular:

1.  **Mapear Objetos**: Convertir la clase PHP `MBeneficiario` a interfaces TypeScript y Structs de Rust.
2.  **Portar Fórmulas**: Copiar la lógica de los métodos de `KCalculo.php` a funciones Rust (para cálculo en backend) o servicios TypeScript (para estimación en frontend).
3.  **Preservar Precisión**: PHP utiliza `round($val, 2)`. Asegurar que la implementación en Rust/JS maneje correctamente la aritmética de punto flotante (considerar tipos `Decimal`).
