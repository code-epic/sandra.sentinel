---
name: PACE Kernel & Migration Logic (Views & JS)
description: Deep dive into the legacy frontend logic (JavaScript/jQuery) of the PACE Kernel system, focusing on key modules like Finiquito, Anticipos, and Medidas Judiciales.
---

# Lógica del Frontend Heredado (Views/JS)

Este documento es una extensión de la habilidad principal de migración, enfocada específicamente en la lógica contenida en `src/migracion/pace-kernel/views/js/`. Estos archivos contienen reglas de negocio críticas implementadas en el cliente (JavaScript/jQuery) que **deben** ser migradas al backend (Rust) o servicios robustos en Angular.

## 1. Análisis de Módulos Clave

### A. Registrar Finiquito (`registrar_finiquito.js`)

Este es el archivo más complejo, manejando el cálculo final y cierre de prestaciones.

- **Dependencia de SAMAN**: Interactúa con un sistema externo llamado "SAMAN" (`consultarSAMAN`, `insertarSAMAN`) para sincronizar datos de beneficiarios. Esto implica que la migración debe considerar puntos de integración o simulación de estos endpoints.
- **Distribución a Familiares**:
  - Funciones `CalcularAA` (Alicuota Aguinaldo) y `CalcularM` (Monto) distribuyen porcentajes de capital y asignaciones entre familiares en caso de fallecimiento.
  - **Validación Crítica**: La suma de porcentajes no puede exceder el 100% (`suma <= 100`).
- **Lógica de "Recuperación"**:
  - Calcula montos a recuperar si el beneficiario tiene deudas superiores a sus haberes (`monto_recuperar`, `Calculo.saldo_disponible_fini`).
- **Manejo de Actos de Servicio**:
  - Distingue entre fallecimiento en "Acto de Servicio" vs "Fuera de Servicio" para asignar montos específicos (36 vs 24 meses de sueldo global, según `KCalculo.php`).

### B. Anticipos (`anticipo.js`)

Maneja la solicitud de adelantos de prestaciones.

- **Regla del 75%**: Variable `porcentajeMaximo = 75`. El anticipo no puede exceder el 75% del saldo disponible.
- **Resguardo**: Calcula un `monto_resguardo` basado en medidas judiciales (`dem > mt ? (dem-mt) : 0`).
- **Validación de Cuenta Bancaria**: Impide anticipos si `numero_cuenta` está vacío o es '0'.

### C. Medidas Judiciales (`medidajudicial.js`)

Administra embargos y retenciones legales.

- **Tipos de Medida**: Manutención (`220`), Intereses, etc.
- **Cálculo de Embargo**: Puede ser por monto fijo o porcentaje del sueldo.
- **Estados**: Activa, Suspendida, Ejecutada.

## 2. Patrones de Código Identificados

- **Uso Extensivo de jQuery**: Manipulación directa del DOM (`$("#id").val()`) mezclada con lógica de negocio.
  - _Reflexión_: En Angular, esto debe separarse estrictamente. El componente no debe calcular montos, solo mostrarlos.
- **"Spaghetti Code" en Consultas**: Las funciones `consultar()` hacen llamadas AJAX anidadas y manipulan la UI basándose en múltiples condiciones (fechas, estatus) dentro del callback.
  - _Estrategia_: Usar RxJS (`switchMap`, `forkJoin`) para orquestar estas llamadas de manera limpia.
- **Hardcoding de Fechas Legales**: Se observan comparaciones directas de fechas string (e.g., `'2018-08-20'`, `'2021-10-01'`) para determinar reconversiones monetarias.
  - _Acción_: Estas fechas deben moverse a un archivo de configuración o constantes del sistema "Business Rules".

## 3. Reflexión sobre el "Perceptrón" en el Frontend

Aunque el archivo `KPerceptron.php` está en el backend, el frontend refleja el consumo de estos datos "memorizados".

- La estructura de respuesta JSON de `consultarBeneficiario` devuelve un objeto `Calculo` pre-procesado (`data.Calculo`), lo que confirma que el backend (posiblemente usando el Perceptrón) hace el trabajo pesado antes de enviar los datos a la vista.
- **Conclusión**: El frontend es "tonto" en cuanto a fórmulas de sueldo base, pero "inteligente" en cuanto a la distribución y validación final (distribución de herencia, tope de anticipos).

## 4. Índice de Funciones Relevantes para Migración

| Archivo JS               | Función                      | Descripción                                    | Importancia |
| :----------------------- | :--------------------------- | :--------------------------------------------- | :---------- |
| `registrar_finiquito.js` | `CalcularAA`                 | Distribuye aguinaldos entre familiares         | Alta        |
| `registrar_finiquito.js` | `consultarBeneficiarioFecha` | Logica condicional para reconversión monetaria | Crítica     |
| `anticipo.js`            | `calcularMonto`              | Aplica tope del 75% y resguardo                | Alta        |
| `medidajudicial.js`      | `guardarMedida`              | Prepara objeto JSON complejo para el backend   | Media       |

## 5. Próximos Pasos Sugeridos

1.  Crear **Servicios Angular** dedicados para cada módulo (`FiniquitoService`, `AnticipoService`).
2.  Implementar las **Reglas de Negocio** (tope 75%, reconversión por fecha) como utilidades puras (`BusinessRulesUtils`).
3.  Diseñar los **Tipos de Datos** (Interfaces) basándose en los objetos JSON que se ven en los `$.getJSON` de estos scripts.
