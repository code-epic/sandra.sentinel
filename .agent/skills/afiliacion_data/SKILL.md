---
name: Manejo de Datos de Afiliación
description: Estructura de datos completa para afiliados (Militares y Familiares) y patrones de uso.
---

# Estructura de Datos de Afiliación

El objeto `Afiliado` representa la información completa de un miembro del sistema de seguridad social (militar) y sus familiares. Este objeto es complejo y contiene múltiples sub-entidades.

## Ubicación del Modelo

El modelo TypeScript principal se encuentra en:
`src/app/core/models/afiliacion/afiliado.model.ts`

## Entidades Principales

### 1. Afiliado (Raíz)

Contiene la información militar y administrativa del titular.

- **Identificadores**: `id` (Cédula), `_id` (Mongo ID).
- **Estatus**: `situacion` (ACT, RCP), `categoria` (EFE), `clase` (OFIT).
- **Militar**: `grado`, `componente`, `historialmilitar`.
- **Fechas Clave**: `fingreso`, `fascenso`, `fretiro` (Formato Mongo Date).

### 2. Persona (`persona`)

Datos biográficos y de contacto del individuo (aplica para Afiliado y Familiares).

- `datobasico`: Nombres, apellidos, nacionalidad, sexo.
- `datofisico` / `datofisionomico`: Rasgos físicos.
- `direccion`: Lista de direcciones.
- `telefono` / `correo` / `redsocial`.

### 3. Familiares (`familiar`)

Array de dependientes.

- Cada elemento contiene su propia estructura `persona`.
- `parentesco`: Código de relación (e.g., "PD" Padre, "HJ" Hijo).
- `esmilitar`: Booleano si el familiar también es militar.

### 4. Documentos y Servicios

- `tim`: Tarjeta de Identificación Militar.
- `pension`: Datos de pensión y prestaciones.
- `cis`: Beneficios socieconómicos (medicina, etc.).

## Patrón de Uso Recomendado

### Importación

```typescript
import { IAfiliado } from "src/app/core/models/afiliacion/afiliado.model";
```

### Manejo de Fechas Mongo

El backend retorna fechas en formato objeto Mongo:

```json
{ "$date": "2020-07-05T00:00:00.000Z" }
// o
{ "$date": { "$numberLong": "-62135596800000" } }
```

**Recomendación:** Crear un pipe o servicio de utilidad para convertir estas fechas a objetos `Date` nativos de JS o Strings legibles en el frontend inmediatamente al recibir la data.

### Validación

Al procesar el objeto `familiar`, verificar siempre `parentesco` y `esmilitar` para determinar reglas de negocio (ej. derecho a carnet).

## Ejemplo de Iteración

```typescript
if (afiliado.familiar && afiliado.familiar.length > 0) {
  afiliado.familiar.forEach((fam) => {
    console.log(
      `Familiar: ${fam.persona.datobasico.nombreprimero} - Parentesco: ${fam.parentesco}`,
    );
  });
}
```
