# Sandra Sentinel

**Sandra Sentinel** es un núcleo de procesamiento de alto rendimiento desarrollado en **Rust**, diseñado para la auditoría, fusión computacional y proyección de nóminas masivas en entornos jerárquicos complejos.

Actúa como un **auditor determinista**: consume datos crudos de fuentes legadas, aplica reglas de negocio modernas y genera una estructura de datos unificada y validada.

---

## Arquitectura

Sentinel está diseñado bajo principios de _Zero-Cost Abstractions_ y seguridad de memoria (_Memory Safety_), operando bajo un patrón de **Arquitectura Hexagonal (Ports & Adapters)**. El núcleo lógico (`core`) está totalmente desacoplado de las interfaces de entrada (gRPC Streams) y salida (CLI/CSV).

### Stack Tecnológico

- **Lenguaje:** Rust (Edición 2021) sobre el runtime asíncrono `Tokio`.
- **Protocolo:** gRPC con Protobuf v3 para transporte de alta eficiencia.
- **Serialización:** NDJSON (Newline Delimited JSON) sobre bytes crudos para maximizar el throughput.
- **Algoritmos:** Hash-Join en memoria para fusión de entidades y Pipeline Asíncrono para concurrencia I/O.

---

## El Motor de Cálculo (Computation Engine)

El corazón de Sentinel es su **Motor de Cálculo Estocástico-Determinista**. A diferencia de los sistemas tradicionales que realizan consultas SQL complejas (JOINs costosos), Sentinel descarga los datos "crudos" y realiza la lógica de negocio en la memoria de la aplicación (`In-Memory Computing`), aprovechando la velocidad de la CPU moderna y evitando la latencia de la base de datos.

### 1. Modelo de Datos Unificado

El motor trabaja sobre tres entidades fundamentales que se fusionan para crear un "Expediente Digital Completo" (`Beneficiario`):

1.  **Entidad Base (The Blueprint):** Contiene la información estructural del afiliado: Nivel Jerárquico (`Grado`), Grupo Organizacional (`Componente`), y Tiempos de Servicio.
2.  **Entidad Financiera (Movements):** Representa el estado transaccional dinámico: cuentas bancarias, pasivos, y variaciones monetarias.
3.  **Directivas (The Ruleset):** Tablas maestras que dictan las reglas salariales vigentes (tabuladores).

### 2. Algoritmo de Fusión (In-Memory Hash Join)

Para unir estas entidades masivamente (500k+ registros) en milisegundos, Sentinel implementa una variante del algoritmo **Hash Join**:

- **Fase de Indexación (Build Phase):**
  - Se cargan las _Entidades Base_ y _Movimientos_ en memoria.
  - Se construyen tablas hash (`HashMap<Key, &Entity>`) optimizadas. La clave de búsqueda suele ser un `Pattern` (identificador compuesto) o un ID único (Cédula).
  - _Complejidad:_ O(N).

- **Fase de Sondeo (Probe Phase):**
  - El stream de _Beneficiarios_ entra al sistema.
  - Para cada beneficiario, se realiza una búsqueda O(1) en los índices para encontrar su Base y Movimientos correspondientes.
  - **Resultado:** Un objeto `Beneficiario` enriquecido con toda su historia financiera y jerárquica sin realizar una sola consulta extra a la base de datos.

### 3. Lógica de Tiempo y Jerarquía

El motor no confía en los cálculos heredados; los recalcula al vuelo.

- **Cálculo de Antigüedad:** Utiliza aritmética de fechas (`chrono`) para determinar con precisión de días el tiempo transcurrido desde el `Ingreso al Sistema` y el `Último Ascenso`.
- **Normalización de Rangos:** Convierte identificadores jerárquicos legados en un sistema de tipos estricto, permitiendo comparaciones válidas para la asignación de primas y beneficios.

---

## Ingeniería de Rendimiento (Pipeline Asíncrono)

Uno de los logros técnicos más notables de esta implementación es su capacidad para procesar **~500,000+ registros complejos en segundos**. Esto se logra mediante una arquitectura de tubería (Pipeline) que elimina los tiempos muertos.

### El Problema "Stop-and-Wait" (Superado)

En implementaciones ingenuas, el sistema haría:
`Descargar Lote -> Pausar Red -> Deserializar (CPU) -> Procesar -> Repetir`.
Esto desperdicia el 50% del tiempo esperando I/O.

### La Solución: Async Streaming & Zero-Copy Deserialization

Sentinel implementa un flujo continuo:

1.  **Transporte Optimizado (Bytes vs Structs):**
    - Se migró el protocolo gRPC de enviar estructuras complejas (`google.protobuf.Struct`) a enviar **bloques de bytes crudos (JSON/NDJSON)**.
    - Esto elimina la costosa reflexión y asignación de memoria en el servicio upstream (Golang), reduciendo la latencia de serialización drásticamente.

2.  **Parallel Parsing (Back-pressure):**
    - El hilo principal (`Main Thread`) se dedica exclusivamente a recibir paquetes de red.
    - Inmediatamente delega la deserialización y el parsing a un _pool de hilos_ (`tokio::spawn`).
    - Mientras la CPU procesa el Lote N, la tarjeta de red ya está descargando el Lote N+1.

3.  **SIMD JSON Parsing:**
    - Al usar `serde_json::from_slice`, Rust puede utilizar instrucciones vectoriales (SIMD) para escanear el buffer de bytes y mapearlo a las estructuras en memoria (`Struct Beneficiario`) a velocidades cercanas a la del ancho de banda de la memoria RAM.

---

## Proyección y Auditoría

El resultado final no es solo una copia de datos, sino una **Nómina Auditada**. Al recalcular atributos críticos (como la antigüedad o el derecho a primas) basándose en la data cruda, Sentinel actúa como un sistema de detección de anomalías:

- **Inconsistencia de Datos:** Si un registro no tiene `Base` (fusión fallida), el sistema aplica `Defaults` seguros (`trait Default`) y lo marca implícitamente, permitiendo identificar afiliados "húerfanos" en la data origen.
- **Exportación Lineal:** La capacidad de "aplanar" (`Flattening`) estructuras jerárquicas complejas en un formato lineal (CSV) facilita la integración con herramientas de Business Intelligence (BI) y auditoría externa.

---

> **Sandra Sentinel** es un ejemplo de ingeniería de sistemas moderna: tipado fuerte, concurrencia segura y optimización a bajo nivel para resolver problemas de gestión de datos a gran escala.
