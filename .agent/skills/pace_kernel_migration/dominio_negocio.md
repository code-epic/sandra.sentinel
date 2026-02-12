---
name: Dominio de Negocio y Reglas PACE
description: Diccionario de términos, reglas de negocio y estrategias de migración para el sistema de Prestaciones Sociales (PACE).
---

# Dominio y Reglas del Sistema PACE

Esta guía establece el lenguaje ubicuo (Ubiquitous Language) y las reglas fundamentales para el sistema de cálculo de prestaciones. Debe ser utilizada como referencia principal durante la migración a Rust.

## 1. Mapeo de Términos (Diccionario de Dominio)

La siguiente tabla traduce la terminología del código legado (Legacy PHP) a los conceptos de negocio actuales que usaremos en la nueva arquitectura.

| Término Legado (PHP)     | Concepto de Negocio (Rust/Domain) | Descripción                                                                                             |
| :----------------------- | :-------------------------------- | :------------------------------------------------------------------------------------------------------ |
| **MBeneficiario**        | **Afiliado / Militar**            | La entidad raíz. Un efectivo militar con derechos a prestaciones.                                       |
| **MDirectiva**           | **Tabulador de Sueldos**          | Tabla oficial que dicta el _Sueldo Base_ según Grado y Antigüedad. Vinculada a Decretos Presidenciales. |
| **DetalleDirectiva**     | **Escala Salarial**               | La fila específica del tabulador (Grado + Años en el grado = Monto).                                    |
| **MPrima**               | **Asignación / Bono**             | Pagos adicionales al sueldo base (Transporte, Descendencia, Profesionalización, etc.).                  |
| **FNX (Calculo)**        | **Fórmula de Cálculo**            | Ecuación dinámica para determinar el valor de una Prima (ej. `SueldoBase * 0.10`).                      |
| **KCalculo**             | **Motor de Prestaciones**         | El orquestador que ejecuta la secuencia de cálculo (Sueldo Global -> Integral -> Prestaciones).         |
| **KPerceptron**          | **Memoization / Caché**           | Mecanismo para almacenar resultados intermedios y evitar recálculos costosos en bucles.                 |
| **MHistorialMovimiento** | **Ledger / Movimientos**          | Historial financiero: Anticipos recibidos, Intereses pagados, Garantías.                                |
| **AsignacionAntiguedad** | **Prestación Social**             | El capital acumulado por el militar al final de su servicio (`Sueldo Integral * Años de Servicio`).     |
| **Finiquito**            | **Liquidación Final**             | Cálculo definitivo al momento del retiro, congelando variables a la `Fecha de Retiro`.                  |

---

## 2. Reglas de Negocio Fundamentales

A continuación se describen las invariantes del sistema que **no pueden** violarse.

### A. Jerarquía del Sueldo (El "Sueldo Integral")

El cálculo de prestaciones se basa en una construcción en cascada del sueldo. Es vital respetar este orden:

1.  **Sueldo Base (SB)**: Determinado por el **Tabulador** (Grado + Antigüedad en el Grado).
2.  **Sueldo Global (SG)**: `SB + Sumatoria(Primas)`.
3.  **Sueldo Integral (SI)**: `SG + Alicuota de Aguinaldos + Alicuota de Vacaciones`.
4.  **Prestación de Antigüedad**: `SI * Tiempo de Servicio General`.

### B. Reglas de Tiempo (Fechas Críticas)

El tiempo es el factor más sensible.

- **Tiempo de Servicio**: `Fecha de Retiro (o Actual) - Fecha de Ingreso`.
  - _Excepción_: Se deben sumar tiempos "Reconocidos" (servicios previos en administración pública) si existen (`ano_reconocido`, etc.).
- **Antigüedad en el Grado**: `Fecha Actual - Fecha Último Ascenso`.
  - _Importante_: Determina qué columna del Tabulador se usa.
  - _Bloqueo_: Si `st_no_ascenso` es TRUE, la antigüedad en el grado puede tener un tope o comportamiento especial.

### C. Alicuotas (Factores Variables)

Las alicuotas no son fijas, dependen de condiciones históricas y legislativas:

- **Aguinaldos**: Históricamente eran 90 días, luego 105, actualmente 120 días (variable por decreto). La fórmula es:
  `((Días * Sueldo Global) / 30) / 12`
- **Vacaciones**: Dependen de los años de servicio (Escalafón: 40, 45, 50 días).
  `((Días * Sueldo Global) / 30) / 12`

---

## 3. Estrategias de Implementación (Rust)

### Patrón "Componente-Entidad"

En `core/src/model`, evitaremos un solo struct gigante.

- **`Afiliado`**: Struct principal (Datos básicos, fechas).
- **`HojaDeServicio`**: Datos militares (Grado, Componente, Estatus).
- **`HojaFinanciera`**: Historial de pagos, cuentas bancarias.

### Motor de Cálculo (Traits y Pipelines)

En `core/src/calc`, usaremos Traits para definir comportamientos de cálculo.

```rust
pub trait Calculable {
    fn calcular(&self, contexto: &Contexto) -> Decimal;
}

// Pipeline
let sueldo_base = Tabulador::resolver(grado, antiguedad);
let primas = GestorPrimas::calcular(sueldo_base, perfil_usuario);
let integral = CalculadoraIntegral::computar(sueldo_base, primas);
```

### Manejo de Precisión

**Prohibido usar `f64` para dinero.**

- Usaremos la crate `rust_decimal` o similar para garantizar precisión financiera (2-4 decimales exactos).
- PHP usaba `round($val, 2)`. Debemos replicar este redondeo en cada etapa intermedia si es necesario para mantener compatibilidad histórica ("Bit-exactness" con el sistema legado), o mejorarla si se autoriza.

### El Perceptrón (Cache)

Implementaremos `Perceptron` en `core/src/kernel` como un `HashMap` thread-safe (o simple si es per-request) que almacene:

- `Key`: Hash de (Grado + Antigüedad + Año).
- `Value`: Sueldo Base calculado.
  Esto es vital para operaciones por lote (ej. calcular nómina de 5,000 efectivos).

---

## 4. Tipos de Operación

1.  **Simulación/Proyección**: Calcular prestaciones a fecha futura.
2.  **Calcular**: Calcular a fecha actual (cierre de mes).
3.  **Finiquitar**: Calcular a fecha de retiro (congelar).

---

**Nota Final**: Este documento debe evolucionar. Si encuentras una regla oscura en el código PHP, agrégala aquí.
