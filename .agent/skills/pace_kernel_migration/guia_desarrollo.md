# Guía de Desarrollo y Estándares (PACE Kernel Migration)

Esta guía está diseñada para los desarrolladores encargados de migrar o mantener la lógica del Kernel PACE.

## 1. Convenciones de Nombres (Naming Conventions)

El sistema legacy usa una notación mixta (CamelCase y underscore), pero con prefijos distintivos:

- **Clases del Kernel**: Prefijo `K`.
  - Ejemplo: `KCalculo`, `KPerceptron`, `KGenerador`.
- **Modelos de Datos**: Prefijo `M` (implícito en uso, ej. `MBeneficiario`).
- **Métodos**: CamelCase.
  - Ejemplo: `iniciarCalculosBeneficiario`, `SueldoGlobal`.
- **Variables**: snake_case.
  - Ejemplo: `sueldo_global`, `fecha_retiro`, `tiempo_servicio`.

### Tipado y Formatos

- **Moneda**: Todos los montos monetarios se manejan como `float` o `double` en PHP.
- **Formato Visual**: Se usa `number_format($val, 2, ',', '.')` para la vista (separador de miles con punto, decimales con coma).
  - **Importante**: Al migrar, NUNCA guardar el formato visual en la base de datos o en variables de cálculo. Solo formatear en el último paso (UI).
- **Fechas**: Formato `YYYY-MM-DD` (ISO 8601).

## 2. Estructura de Objetos (Legacy -> Moderno)

Al migrar a TypeScript/Rust, mapear las propiedades de la siguiente manera:

### Objeto `Beneficiario`

| Propiedad PHP     | Tipo TypeScript Sugerido | Descripción                |
| :---------------- | :----------------------- | :------------------------- |
| `cedula`          | `string`                 | ID único                   |
| `sueldo_global`   | `number`                 | Sueldo Base + Primas       |
| `sueldo_integral` | `number`                 | Base para prestaciones     |
| `fecha_ingreso`   | `Date` or `string`       | Fecha ingreso FANB         |
| `Componente`      | `IComponente`            | Objeto anidado             |
| `Prima`           | `Record<string, number>` | Lista de primas calculadas |

## 3. Patrones de Cálculo

El sistema separa cálculo individual (`KCalculo`) de por lote (`KCalculoLote`).

- **Migración**: En la nueva arquitectura, se recomienda unificar la lógica en un **Servicio de Cálculo** (Rust/WASM preferiblemente para rendimiento en lotes masivos) que acepte un array de beneficiarios.

## 4. Manejo de Errores y Excepciones

- El legacy usa poca validación de excepciones (`try-catch`).
- **Nuevo Estándar**: Implementar `Result<T, E>` en Rust o manejo estricto de nulos en TypeScript. Verificar siempre que `fecha_retiro` sea válida antes de calcular tiempos.

## 5. Glosario de Términos (Dominio Militar)

- **Antigüedad en el Grado**: Tiempo pasado desde el último ascenso.
- **Tiempo de Servicio**: Tiempo total desde el ingreso.
- **Reconocido**: Tiempo de servicio prestado en otras instituciones públicas, sumado al tiempo FANB.
- **Finiquito**: Cierre final de la cuenta de prestaciones al retirarse.
