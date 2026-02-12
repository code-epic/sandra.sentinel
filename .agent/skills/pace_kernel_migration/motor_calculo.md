# Arquitectura de Integración: PACE Hybrid Engine (Go-Rust)

## 1. Definición de Roles (System Roles)

- [cite_start]**Data Layer (Golang):** Responsable de la introspección de base de datos, generación de queries dinámicas y orquestación de API[cite: 5, 12].
- [cite_start]**Compute Layer (Rust):** Motor de ejecución para algoritmos de alta precisión (Garantías, Intereses Caídos, Finiquitos)[cite: 9, 101].

## 2. Protocolo de Intercambio (Data Pipeline)

- **Formato:** Protocol Buffers (Binary Serialization).
- **Transporte:** gRPC Stream (Bidireccional para grandes volúmenes).
- [cite_start]**Carga Nominal:** 100,000 registros por lote de cálculo[cite: 195, 203].

## 3. Lógica del Motor de Cálculo (Rust Skills)

[cite_start]El motor de Rust debe implementar las fórmulas del Decreto-Ley Negro Primero[cite: 1, 9]:

### [cite_start]A. Cálculo de Disponibilidad [cite: 150]

$$Anticipos = ((\sum Aportes + \sum Garantia) - \sum Anticipos) \times 0.75$$

### [cite_start]B. Regla de Finiquito [cite: 241]

- **Condición:** $MAX(\text{Asignación de Antigüedad}, \sum \text{Garantías})$.
- [cite_start]**Salida:** El monto mayor debe ser retornado a Golang para la interfaz[cite: 241].

## 4. Optimizaciones de Rendimiento

- **Zero-Copy:** En Rust, utilizar `Arc` y buffers compartidos para minimizar la latencia de memoria al recibir datos de Go.
- **Batch Processing:** Procesar los 100k registros en micro-lotes paralelos para aprovechar todos los núcleos del procesador.

## [cite_start]5. Manejo de Errores y Auditoría [cite: 131, 134]

- [cite_start]Cualquier discrepancia en el cálculo de Rust debe generar un log de error compatible con el formato CSV/PDF de la Gerencia de Informática[cite: 156, 176].
