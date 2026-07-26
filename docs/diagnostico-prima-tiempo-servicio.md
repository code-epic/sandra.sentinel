# Diagnóstico: Prima por Tiempo de Servicio

> Guía de comandos y notas de la corrección aplicada a `prima_tiemposervicio` en Sandra Sentinel.
> Fecha: 2026-07-25

## Resumen del problema

La prima por tiempo de servicio (`prima_tiemposervicio`) devolvía `31697.76` en lugar del esperado `24090.30`. La fórmula Rhai estaba correcta, pero la variable `tiempo_servicio` llegaba con valor `28` (calculado desde `fecha_ingreso` hasta la fecha actual) en vez de `18` (la antigüedad real del beneficiario, limitada por `f_retiro`).

## Causa raíz

1. `IPSFA_CBase` se carga sin `f_retiro` (`Base.f_retiro` es `None` durante el primer cálculo).
2. `procesar_registro_base()` recalcula `antiguedad` usando `Local::now()` como límite superior, produciendo 28 años.
3. El motor Rhai calcula primas con `tiempo_servicio = 28`.
4. Más tarde, en `cargar_beneficiarios()`, `f_retiro` sí llega desde `IPSFA_CBeneficiarios`, pero las primas Rhai no se recalculaban.

## Archivos modificados

| Archivo | Cambio |
|---|---|
| `core/src/kernel/logica/cargador.rs` | `cargar_beneficiarios()` ahora recibe el `SentinelEngine` y recalcula primas Rhai después de corregir `f_retiro`. |
| `core/src/kernel/mod.rs` | Se pasa `motor_arc.as_ref()` a `cargar_beneficiarios()`. |
| `core/src/calc/mod.rs` | `is_debug()` ahora es pública para usarla en `motor.rs`. |
| `core/src/calc/motor.rs` | El debug `[DEBUG-PTS]` de fórmulas ahora se activa bajo demanda con el flag `--debug`. |

## Comandos de ejecución

### 1. Ejecutar nómina NPR sin debug

Salida limpia. Útil para producción o verificación final.

```bash
cargo run -p sandra_sentinel -- start \
  -x \
  --tipo npr \
  -m manifest_unic.json \
  --json \
  --log
```

### 2. Ejecutar nómina NPR con diagnóstico de fórmulas

Activa `SANDRA_DEBUG=1` y muestra el bloque `[DEBUG-PTS]` con la fórmula Rhai completa, variables del scope y resultados.

```bash
cargo run -p sandra_sentinel -- start \
  -x \
  --tipo npr \
  -m manifest_unic.json \
  --json \
  --log \
  --debug
```

### 3. Ejecutar y capturar stderr en archivo

El debug `[DEBUG-PTS]` va a `stderr`. Redirigirlo permite revisarlo sin mezclarlo con el JSON de salida.

```bash
cargo run -p sandra_sentinel -- start \
  -x \
  --tipo npr \
  -m manifest_unic.json \
  --json \
  --log \
  --debug 2>/tmp/debug_prima.log
```

### 4. Filtrar mensajes de debug

```bash
grep "DEBUG-PTS" /tmp/debug_prima.log
```

### 5. Ver resultado final con Python

Extrae del JSON los campos relevantes para confirmar la corrección:

```bash
cargo run -p sandra_sentinel -- start \
  -x --tipo npr \
  -m manifest_unic.json \
  --json --log 2>/dev/null | python3 -c "
import json, sys
d = json.load(sys.stdin)[0]
print(json.dumps({
  'antiguedad': d['base']['antiguedad'],
  'antiguedad_grado': d['base']['antiguedad_grado'],
  'sueldo_base': d['base']['sueldo_base'],
  'prima_tiemposervicio': d['base']['calculos']['prima_tiemposervicio'],
  'sueldo_mensual': d['base']['sueldo_mensual'],
  'sueldo_integral': d['base']['sueldo_integral'],
  'asignacion_antiguedad': d['base']['asignacion_antiguedad'],
}, indent=2))
"
```

Resultado esperado tras la corrección:

```json
{
  "antiguedad": 18,
  "antiguedad_grado": 4,
  "sueldo_base": 105659.19,
  "prima_tiemposervicio": 24090.3,
  "sueldo_mensual": 143490.59,
  "sueldo_integral": 203278.33,
  "asignacion_antiguedad": 3659009.94
}
```

## Comandos de compilación

### Compilar solo el paquete Sentinel

```bash
rtk cargo build -p sandra_sentinel
```

### Ejecutar tests de cálculo

```bash
rtk cargo test -p sandra_core --lib calc
```

## Notas de mantenimiento

- El debug `[DEBUG-PTS]` está centrado en `prima_tiemposervicio`. Si se necesita rastrear otra fórmula, el matcher `es_pts` en `core/src/calc/motor.rs` puede parametrizarse.
- El flag `--debug` (`-d`) ya existía en el CLI. Ahora controla correctamente el diagnóstico de fórmulas.
- `f_retiro` debe seguir llegando correctamente desde `IPSFA_CBeneficiarios` para que el recálculo funcione.
- Si se cambia el flujo de carga de beneficiarios, verificar que `cargar_beneficiarios()` siga recibiendo el motor y reprocesando las primas Rhai tras corregir `f_retiro`.

## Referencias

- `core/src/calc/mod.rs` — cálculo de tiempo de servicio y `is_debug()`.
- `core/src/calc/motor.rs` — motor Rhai y `SentinelEngine`.
- `core/src/kernel/logica/cargador.rs` — fusión de beneficiarios y recálculo con `f_retiro`.
- `core/src/kernel/mod.rs` — orquestación del pipeline de cálculo.
