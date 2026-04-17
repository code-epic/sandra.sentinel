# Changelog - Sandra Sentinel

Todos los cambios notables de este proyecto se documentarán en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/).

---

## [1.1.0] - 2026-04-17

### Agregado

- **Nuevo sistema de generación de archivos apertura/aporte conjunto**
  
  Implementada nueva lógica para dividir beneficiarios entre:
  - **Aporte**: Beneficiarios con movimientos cargados (cap_banco + anticipo + dep_adicional + dep_garantia + anticipor > 0)
  - **Apertura**: Beneficiarios sin movimientos en el ciclo actual
  
  Esta característica permite:
  - Procesar nóminas donde no todos los beneficiarios tienen movimientos en el período
  - Generar automáticamente archivos de apertura para nuevos pensionados
  - Optimizar el proceso al separar flujos según corresponda

- **Generación de TXT Apertura para Banco de Venezuela**
  
  Nuevo formato TXT específico para apertures bancarias con estructura de 97 caracteres:
  - Plan (5): 03487
  - Nacionalidad (1): V
  - Cédula (9): padded con 0
  - Nombre (60): apellidos + nombres
  - Estado Civil (1): S/C/D/V
  - Reservado (22): ceros
  - Monto (13): sin decimales

- **Bandera `generar_apertura_con_aporte` en Manifiesto**
  
  Nueva configuración en el bloque `aportes`:
  ```json
  {
    "aportes": {
      "habilitar": true,
      "monto_aprobado_garantias": 40000000.00,
      "generar_apertura_con_aporte": true
    }
  }
  ```

- **Funciones nuevas en exportador**
  - `exportar_aporte_y_apertura_txt()`: Genera CSV de aporte + TXT de apertura
  - `generar_linea_apertura()`: Formatea líneas para archivo de apertura banco Venezuela

### Modificado

- **Lógica de división Beneficiarios → Aporte/Apertura**
  
  Cambiado de usar campos calculados (`garantia_original + garantia_anticipo`) a usar campos de movimientos:
  ```rust
  // Antes (incorrecto - siempre 0 para algunos casos)
  let total_mov = b.base.garantia_original + b.base.garantia_anticipo;
  
  // Ahora (correcto - usa datos de movimientos PACE)
  let m = &b.movimientos;
  let total_mov = m.cap_banco + m.anticipo + m.dep_adicional + m.dep_garantia + m.anticipor;
  ```

- **Flujo de exportación en start.rs**
  
  El flujo ahora genera ambos archivos simultáneamente cuando `generar_apertura_con_aporte: true`:
  1. Divide beneficiarios por presencia de movimientos
  2. Genera `aporte_{ciclo}.csv` (beneficiarios con movimientos)
  3. Genera `APERT{ciclo}.txt.zst` (beneficiarios sin movimientos)
  4. Muestra métricas de ambos archivos

### Caso de Uso

**Escenario**: Nómina donde hay más beneficiarios (113,422) que movimientos (87,626)

| Beneficiarios | Movimientos | Diferencia | Resultado |
|--------------|-------------|-----------|-----------|
| 113,422 | 87,626 | 25,796 | Sin movimientos → Apertura |

**Antes**: Todos iban a aporte aunque no tuvieran movimientos
**Ahora**: 
- 87,626 → `aporte_2026-01.csv`
- 25,796 → `APERT2026-01.txt` (apertura bancaria)

---

## [1.0.0] - 2026-01-15

### Agregado

- **Lanzamiento inicial de Sandra Sentinel**
- Motor de cálculo estocástico-determinista
- Integración gRPC con Sandra Server
- Géneración de nóminas (NPR, NACT, NRCP, NFCP)
- Exportación CSV y TXT bancario
- Sistema de distribución de garantías
- Compresión Zstd
- Sellado SHA-256

---

## Formato TXT Banco de Venezuela

### Formato Apertura (APERT)

| Posición | Campo | Longitud | Descripción |
|----------|-------|---------|-------------|
| 1-5 | Plan | 5 | Código-plan (03487) |
| 6 | Nacionalidad | 1 | V = Venezolano |
| 7-15 | Cédula | 9 | 9 dígitos, padded 0 |
| 16-75 | Nombre | 60 | Apellidos + Nombres |
| 76 | Edo Civil | 1 | S/C/D/V |
| 77-98 | Reservado | 22 | 22 ceros |
| 99-111 | Monto BS | 13 | Sin decimales |

**Ejemplo**:
```
03487V020178906CARLOS EDUARDO MONTERO CASTILLO                                        S0000000000000000000000000000000029322
```

### Formato Aporte (APORT)

| Posición | Campo | Longitud |
|----------|-------|----------|
| 1-5 | Plan | 5 |
| 6 | Nacionalidad | 1 |
| 7-15 | Cédula | 9 |
| 16 | Tipo Tran | 1 |
| 17-18 | Tipo Prod | 2 |
| 19 | Frm Pago | 1 |
| 20-32 | Monto | 13 |
| ... | ... | ... |

### Formato Retiro (RETIR)

Similar a Aporte pero con tipo de transacción 3 y monto negativo.

---

## Referencias

- [Sandra Sentinel/README.md](README.md) - Documentación principal
- [docs/RELEASE.md](docs/RELEASE.md) - Guía de releases
- [schema/README.md](schema/README.md) - Esquemas de datos