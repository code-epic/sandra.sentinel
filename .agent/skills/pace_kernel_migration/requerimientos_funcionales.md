---
name: Requerimientos Funcionales y Reglas de Negocio PACE
description: Documentación detallada de los módulos, flujos de trabajo, reglas de cálculo y adaptaciones legales del sistema PACE, basada en los requerimientos de reingeniería.
---

# Requerimientos Funcionales y Reglas de Negocio PACE

Este documento detalla la estructura funcional y las reglas de negocio críticas para la reingeniería del sistema PACE en Rust/Angular. Se basa en los requerimientos de migración y adaptación legal.

## 1. Arquitectura Modular Funcional

El sistema se divide en 6 módulos principales con responsabilidades claras:

### Módulo 1: Beneficiario (Fideicomitente)

Gestión integral de los datos del afiliado y su historial.

- **Carga Masiva (TXT/CSV)**: Procesamiento de nuevos ingresos y actualizaciones.
  - _Regla_: Rechazar registros si un "nuevo" ya existe en histórico (generar error).
  - _Cruce SAMAN_: Validación estricta de Grado, Componente, Hijos y Estatus (Activo/Retirado).
- **Hoja de Vida**: Vista unificada 360° (Datos Militares + Financieros).
- **Historial Completo**: Sueldos, Anticipos, Medidas Judiciales, Movimientos (Fideicomiso).

### Módulo 2: Cálculos y Finanzas (Core)

El corazón del sistema. Ejecuta la lógica financiera compleja.

- **Aporte de Capital**: Cálculo por lotes/grupos.
- **Intereses Caídos**:
  - _Nueva Regla_: Capitalizar deuda pendiente de semestres anteriores en el cálculo actual.
  - _Regla_: Distribuir intereses considerando Medidas Judiciales activas en la fecha de corte.
- **Asignación de Antigüedad (AA)**: Cálculo mensual/anual.
- **Garantías y Días Adicionales**:
  - _Garantía_: 15 días trimestrales (Base: Último Sueldo Integral).
  - _Días Adicionales_: Después del año 1, +2 días/año (Max 30 días/acumulados).

### Módulo 3: Finiquitos (Liquidación)

Proceso de cierre de cuenta al retiro o fallecimiento.

- **Regla de Oro (Ley 2015)**: El finiquito es el **MAYOR** valor entre:
  1. `Asignación de Antigüedad` (Cálculo tradicional)
  2. `∑ Aportes de Garantía` (Nuevo régimen)
- **Fallecimiento (Indemnizaciones)**:
  - _Acto de Servicio_: 36 meses de remuneración (o salarios mínimos para tropa/cadetes).
  - _Fuera de Servicio_: 24 meses.
- **Recuperación**: Detectar si se pagó de más en anticipos vs. lo real (Saldo Negativo = Monto a Recuperar).

### Módulo 4: Anticipos

Gestión de retiros parciales de fondos.

- **Regla del 75%**:
  `Disponible = (∑Aportes Capital + ∑Garantías) - ∑Anticipos`
  `Max Anticipo = Disponible * 0.75`
- **Excepciones**: Anticipos > 75% requieren autorización (Jefe Dpto/Gerente).
- **Motivos Nuevos**: Liquidación Conyugal, Manutención, Vivienda.

### Módulo 5: Órdenes de Pago y Banca

Interfaz con tesorería y bancos.

- **Generación de Archivos**: Formatos TXT específicos (Banco de Venezuela).
- **Órdenes de Pago**: Flujo de aprobación (Pendiente -> Revisión -> Autorizada -> Ejecutada).

### Módulo 6: Administración y Auditoría

Control y trazabilidad.

- **Auditoría**: Trazabilidad completa de quién cargó/modificó cada registro.
- **Tablas Maestras**: Directivas, Tasas BCV, Componentes.

---

## 2. Fórmulas y Algoritmos Críticos

### Cálculo de Garantías (Trimestral)

```rust
fn calcular_garantia(sueldo_integral: Decimal) -> Decimal {
    // 15 días de Sueldo Integral
    (sueldo_integral / 30) * 15
}
```

### Cálculo de Días Adicionales (Anual)

```rust
fn calcular_dias_adicionales(anos_servicio: u32, dias_acumulados: u32) -> u32 {
    if anos_servicio > 1 {
        let nuevos = 2;
        min(dias_acumulados + nuevos, 30) // Tope de 30 días total acumulado
    } else {
        0
    }
}
```

### Saldo Disponible para Anticipo

```rust
fn calcular_disponible_anticipo(capital: Decimal, garantias: Decimal, anticipos_previos: Decimal) -> Decimal {
    let base = (capital + garantias) - anticipos_previos;
    base * 0.75 // 75%
}
```

---

## 3. Integración Externa

- **SAMAN (RRHH)**: Fuente de verdad para datos personales/militares. Requiere sincronización robusta para detectar cambios de estatus.
- **SIGESP (Contabilidad)**: El sistema PACE debe generar asientos contables automáticos (transferencia de montos) hacia SIGESP.
- **Bancos**: Generación de TXT planos con estructuras rígidas (Header, Detalle, Footer).

---

## 4. Estrategia de Migración (Técnica)

1.  **Refactorización del Modelo de Datos**:
    - Unificar `capital_no_depositado` -> `garantia_asignacion`.
    - Crear tipos de movimiento explícitos para: `Garantias`, `DiasAdicionales`.
2.  **Optimización de Lotes**:
    - Los cálculos de Intereses Caídos son lentos en PHP. En Rust, usaremos `Rayon` (paralelismo) y `Streaming` para procesar miles de registros en segundos.
3.  **Validación de Datos**:
    - Implementar "Sanity Checks" en la carga masiva para evitar inconsistencias SAMAN vs PACE (ej. Grado Tte vs Ptte).
