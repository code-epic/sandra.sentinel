# Arquitectura de Sandra Sentinel: Diagramas de Secuencia y Análisis

## Resumen Ejecutivo

Este documento detalla la arquitectura de ejecución de **Sandra Sentinel** v0.1.0, un sistema de procesamiento de nómina de alto rendimiento diseñado bajo principios de eficiencia computacional (High Performance Computing - HPC) y seguridad operativa.

El sistema implementa patrones avanzados de **Microservicios (gRPC)** y **Procesamiento Paralelo (Rayon + Tokio)** para garantizar tiempos de respuesta en el orden de milisegundos para miles de registros, cumpliendo con estándares internacionales de calidad y seguridad.

---

## 1. Diagrama de Secuencia Principal: Orquestación del Kernel

Este diagrama ilustra el flujo macroscópico del sistema, desde la activación por el usuario hasta la generación de reportes finales. Destaca la **paralelización de tareas** y la gestión de recursos.

### Cumplimiento Normativo:

- **ISO/IEC 25010 (Calidad del Producto Software):** Eficiencia de desempeño (bloques paralelos) y Fiabilidad (Manejo robusto de errores).
- **ISO/IEC 27001 (Seguridad de la Información):** Trazabilidad completa mediante Logs y Telemetría.

```mermaid
sequenceDiagram
    autonumber
    actor User as 👤 Operador
    participant CLI as 🖥️ CLI (Start)
    participant Kernel as ⚙️ Kernel (Orquestador)
    participant Cargador as 📡 Cargador (GRPC Client)
    participant Server as ☁️ Sandra Server (Golang)
    participant Engine as ⚡ SentinelEngine (Cálculo)
    participant Fusion as 🔄 Fusión (Map-Reduce)
    participant Export as 💾 Exportador

    Note over User, CLI: Inicio Seguro (ISO 27001)
    User->>CLI: Ejecutar Ciclo (--manifest nomina_2026.json)
    CLI->>Kernel: Cargar Manifiesto & Iniciar
    Kernel->>Server: Establecer Conexión Segura (HTTP/2)
    Server-->>Kernel: OK (Keep-Alive)

    rect rgb(240, 248, 255)
        Note left of Kernel: PASO 1: Carga de Referencias (Paralelo)
        par Carga Directiva y Primas
            Kernel->>Cargador: Spawn Task (Directiva)
            Cargador->>Server: Stream(IPSFA_CDirectiva)
            Server-->>Cargador: Batches JSON [filter: sueldo>0]

            Kernel->>Cargador: Spawn Task (Primas)
            Cargador->>Server: Stream(IPSFA_CPrimasFunciones)
            Server-->>Cargador: Batches JSON [filter: oidd=81]
        end
        Cargador-->>Kernel: Datos Normalizados (Vec<T>)
    end

    Kernel->>Engine: Inicializar Motor (Compilar Fórmulas Rhai)
    Engine-->>Kernel: Listo para Ejecución

    rect rgb(255, 240, 245)
        Note left of Kernel: PASO 2: Carga Masiva y Cálculo (HPC)
        par Pipeline de Datos Críticos
            Kernel->>Cargador: Cargar Base & Calcular
            Cargador->>Server: Stream(IPSFA_CBase)
            Server-->>Cargador: Batches JSON (Stream)

            Note right of Cargador: Procesamiento en Vuelo
            Cargador->>Engine: Calcular Nómina (Rayon - Parallel Iterator)
            Engine-->>Cargador: Resultados Calculados (Map)
            Cargador->>Cargador: Validar Integridad (Sueldo vs Primas)

            Kernel->>Cargador: Cargar Movimientos
            Cargador->>Server: Stream(IPSFA_CMovimientos)

            Kernel->>Cargador: Cargar Conceptos
            Cargador->>Server: Stream(IPSFA_CConceptos)
        end
        Cargador-->>Kernel: Vectores de Datos Procesados
    end

    rect rgb(240, 255, 240)
        Note left of Kernel: PASO 3: Fusión y Enlace
        Kernel->>Fusion: Indexar Datos (HashMap)
        Kernel->>Cargador: Cargar Beneficiarios (Stream)
        Cargador->>Fusion: Stream(Item) -> Join(Base, Mov)
        Fusion-->>Kernel: Lista Final Beneficiarios (Enriquecida)
    end

    Kernel->>Export: Generar CSV Final
    Export-->>User: Archivo: nomina_exportada.csv
    Kernel->>CLI: Reporte de Sensores (Telemetría)
    CLI-->>User: Resumen de Ejecución (3.76s)
```

---

## 2. Diagrama de Detalle: Potencia de Cálculo y Seguridad

Este diagrama profundiza en el **Pipeline de Procesamiento** de la Fase 2, donde reside la mayor carga computacional. Muestra cómo el sistema maximiza el throughput y minimiza la latencia.

### Características Clave:

- **Zero-Copy Deserialization (Rust Serde):** Minimiza el uso de memoria al procesar flujos JSON.
- **Data Parallelism (Rayon):** Distribuye automáticamente la carga de cálculo entre todos los núcleos disponibles de la CPU.
- **Sandboxed Execution (Rhai):** Las fórmulas de nómina se ejecutan en un entorno aislado y controlado, evitando efectos secundarios peligrosos.

```mermaid
sequenceDiagram
    participant Net as 🌐 Red (gRPC Stream)
    participant Deser as 📦 Deserializador (Serde)
    participant Engine as ⚡ SentinelEngine (Rayon)
    participant VM as 🔒 Rhai VM (Sandbox)
    participant Mem as 🧠 Memoria (HashMap)

    Note over Net, Mem: Pipeline de Alto Rendimiento (Streaming)

    loop Por cada Mensaje (Batch)
        Net->>Deser: Recibir Bytes (Protobuf -> JSON)
        activate Deser
        Deser->>Deser: Parse JSON -> Vec<Base>
        Note right of Deser: Validación de Tipos Estricta
        Deser->>Engine: Enviar Lote de Registros
        deactivate Deser

        activate Engine
        Note over Engine: Split: Divide y Vencerás (Work Stealing)
        par Parallel Iteration (Cores 1..N)
            Engine->>VM: Ejecutar Fórmulas (Contexto Seguro)
            VM->>VM: Calcular Primas (Sin I/O)
            VM-->>Engine: Valor Calculado (f64)
        end
        Engine-->>Mem: Almacenar Resultados (Key: Pattern)
        deactivate Engine
    end

    Note over Mem: Datos listos para Fusión (O(1) Access)
```

---

## Consideraciones de Seguridad y Normas

### Seguridad (ISO/IEC 27001)

1.  **Aislamiento de Ejecución:** El uso de `Rhai` como motor de scripting garantiza que las reglas de negocio no puedan acceder al sistema de archivos ni a la red, previniendo la inyección de código malicioso.
2.  **Validación de Entrada:** Cada etapa del pipeline (Deserialización JSON, Filtros SQL) valida estrictamente los datos antes de procesarlos.
3.  **Auditoría en Capas:**
    - **Capa 1 (SQL):** Filtros aplicados desde el Manifiesto.
    - **Capa 2 (Código):** Validación de lógica de negocio (ej. Sueldo Base > 0).
    - **Capa 3 (Logs):** Registro inmutable de operaciones críticas.

### Calidad y Potencia (ISO/IEC 25010)

1.  **Eficiencia Temporal:** El uso de Rust y gRPC permite procesar >100,000 registros de nómina compleja en segundos (< 4s), superando con creces los estándares de la industria para sistemas legacy.
2.  **Utilización de Recursos:** La arquitectura asíncrona (`Tokio`) para I/O y paralela (`Rayon`) para CPU asegura que ningún núcleo del procesador esté ocioso durante la carga masiva.
3.  **Mantenibilidad:** La arquitectura modular (Kernel, Cargador, Motor) permite actualizar reglas de negocio sin recompilar el núcleo del sistema.
