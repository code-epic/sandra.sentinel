# Estrategia de Mensajería Dinámica: PACE Go-Rust

## 1. Definición de Contratos de Datos

Para resolver la discrepancia entre SQL Dinámico y Protobuf Estático, se implementan dos niveles de abstracción:

### A. Mensaje de Cálculo (Strict Type)

[cite_start]Utilizado para el motor de cálculos de antigüedad y garantías[cite: 105, 120].

- **Regla:** El `SELECT` de Go debe incluir obligatoriamente el set mínimo de variables financieras.
- [cite_start]**Campos Críticos:** `sueldo_base`, `fecha_ingreso`, `numero_hijos`, `grado_militar`[cite: 31, 33].

### B. Mensaje de Consulta (Flexible Type)

Utilizado para reportes y visualización en la UI.

- **Estructura:** Uso de `map<string, string>` o `google.protobuf.Struct` para representar proyecciones variables de SQL.

## 2. Manejo de Valores Nulos y Errores de Carga

[cite_start]Para evitar el error histórico de "Hijos en 0" [cite_start]y fallos en el grado de "Primer Teniente"[cite: 212, 213]:

- **Validación en Go:** Antes de enviar el stream a Rust, Go debe validar que los campos obligatorios para el cálculo no sean nulos, incluso en consultas `SELECT *`.
- **Defaulting:** Si un campo opcional no viene en el `SELECT`, se marca como `optional` en el `.proto` para que Rust lo trate como un `Option<T>`.

## 3. Lógica de Anticipos con Datos Dinámicos

Independientemente de la consulta, el cálculo de disponibilidad de anticipos debe recibir:
[cite_start]$$\text{Disponibilidad} = ((\sum \text{Aportes} + \sum \text{Garantía}) - \sum \text{Anticipos}) \times 0.75$$[cite: 150].
El motor en Rust rechazará el stream si alguno de estos sumatorios no puede ser calculado por falta de datos en la consulta dinámica.
