# Esquemas y Definiciones - Sistema de Pensiones Militares

Sistema de gestión de nóminas de pensiones militares para la Fuerza Armada Nacional Bolivariana.

## Índice

1. [Glosario de Términos](#glosario)
2. [Tipos de Nómina](#tipos-nomina)
3. [Fórmulas de Cálculo](#fórmulas-de-cálculo)
4. [Referencias Legales](#referencias-legales)
5. [Ejemplos Prácticos](#ejemplos-prácticos)
6. [Estructura de CSVs](#estructura-de-csvs)

---

## Glosario

### Términos Fundamentales

| Término | Descripción |
|---------|-------------|
| **Sueldo Integral** | Sueldo básico + todas las primas, bonos y asignaciones mensuales |
| **Pensión** | Prestación económica mensual permanente por servicios prestados |
| **Reserva Activa** | Situación del militar que ha cesado servicio activo pero mantiene vinculación |
| **Causahabiente** | El militar fallecido del cual derivan derechos los sobrevivientes |
| **Familiar Inmediato Calificado** | Cónyuge, hijos o padres que cumplen requisitos legales |

### Familiar Inmediato Calificado (Art. 8)

1. **Padres**: Con o sin afinidad/consanguinidad, que hayan tenido a cargo la asistencia material y cuidado del militar
2. **Cónyuge**: El o la cónyuge
3. **Hijos** que cumplan alguno de estos requisitos:
   - Menores de 18 años
   - Mayores de edad cursando estudios universitarios (máximo 26 años)
   - Con discapacidad o invalidez (sin límite de edad)
   - Con enfermedades catastróficas crónicas (sin límite de edad)

---

## Tipos de Nómina

### RCP - Retirados con Pensión

**Definición**: Personal militar en reserva activa que recibe pensión directa por sus años de servicio.

| Atributo | Valor |
|----------|-------|
| Tipo de beneficiario | Titular (militar retirado) |
| Es titular | ✅ Sí |
| Es sobreviviente | ❌ No |
| Base legal | Artículos 34-38 Ley Negro Primero |
| Mínimo años de servicio | 15 años |

**Características**:
- Recibe directamente su pensión mensual
- El monto depende de los años de servicio cumplidos
- Puede recibir el 50% (15 años) hasta 100% (30 años) del sueldo integral
- Tiene derecho a homologación automática con aumentos del personal en actividad

---

### FCP - Fallecidos con Pensión (Sobrevivientes)

**Definición**: Familiares del militar fallecido que reciben pensión por derecho derivado.

| Atributo | Valor |
|----------|-------|
| Tipo de beneficiario | Sobreviviente (familiares) |
| Es titular | ❌ No |
| Es sobreviviente | ✅ Sí |
| Base legal | Artículos 40-43 Ley Negro Primero |
| Requisito | Causahabiente fallecido en actividad o pensionado |

**Características**:
- No son beneficiarios directos, reciben por derivación del causahabiente
- El porcentaje se distribuye entre los familiares calificados
- Aplican reglas de acrecer ( Art. 45): si un beneficiario pierde el derecho, su porcentaje se redistribuye

**Familiares con derecho**:
- Cónyuge (viuda/viudo)
- Hijos calificados
- Padres

---

### PG - Pensión de Gracia

**Definición**: Beneficio otorgado por el Ejecutivo Nacional sin cumplir todos los requisitos legales regulares.

| Atributo | Valor |
|----------|-------|
| Tipo de beneficiario | Variable |
| Base legal | Discrecional del Ejecutivo |

---

### I - Inválido

**Definición**: Personal militar que adquiere invalidez y recibe pensión por discapacidad.

| Atributo | Valor |
|----------|-------|
| Tipo de beneficiario | Titular (militar inválido) |
| Base legal | Artículos 46-49 Ley Negro Primero |
| Evaluación | Junta Médica Militar |

**Grados de invalidez**:
- **Absoluta y permanente**: 100% del sueldo integral
- **Parcial y permanente**: 75% del sueldo integral

---

## Fórmulas de Cálculo

### RCP - Pensión de Reserva Activa

#### Base de Cálculo (Art. 37)

```
Sueldo_Integral = Sueldo_Básico + Prima_Antigüedad + Prima_Hijos + Prima_Profesionalización 
                + Prima_Transporte + Prima_Especial + Prima_No_Ascenso + Prima_Tiempo_Servicio
                + Bono_Transportista + Asignación_Antigüedad + Otros_Beneficios
```

#### Porcentajes según Años de Servicio (Art. 38)

| Años de Servicio | Porcentaje | Incremento Anual |
|------------------|------------|------------------|
| 15 | 50% | - |
| 16-18 | 50% + (2% × años_adicionales) | +2% por año |
| 19 | 59% | - |
| 20-22 | 59% + (3% × años_adicionales) | +3% por año |
| 23 | 72% | - |
| 24-26 | 72% + (4% × años_adicionales) | +4% por año |
| 27-29 | 84% + (5% × años_adicionales) | +5% por año |
| 30 | 100% | - |

#### Fórmula General RCP

```
Pensión_RCP = Sueldo_Integral × Porcentaje_Pensión(años_servicio)
```

**Donde**:
- `Sueldo_Integral`: Último sueldo mensual integral devengado
- `Porcentaje_Pensión`: Según tabla del Art. 38
- `Pensión_Mensual` = Σ(pensiones_mensuales_año) / 12 + (Bono_Recreacional / 12) + (Bonificación_Fin_Año / 12)

---

### FCP - Pensión de Sobreviviente

#### Base de Cálculo (Art. 42)

```
Si_Causahabiente_Actividad = Último_Sueldo_Integral_Percibido
Si_Causahabiente_Pensionado = Última_Pensión_Mensual_Percibida
```

#### Distribución por Familiar (Art. 43)

| Caso | Cónyuge | Hijos | Padres |
|------|---------|-------|--------|
| Con todos (cónyuge + hijos + padres) | 60% | 20% ÷ n | 20% |
| Sin padres | 60% | 40% ÷ n | - |
| Sin hijos | 50% | - | 50% |
| Sin padres ni hijos | 100% | - | - |
| Solo hijos (sin cónyuge) | - | 75% ÷ n | 25% |
| Solo padres (sin cónyuge ni hijos) | - | - | 100% |

**Donde `n` = número de hijos con derecho a pensión**

#### Fórmula General FCP

```
Beneficio_Total = Base_Cálculo × 100%

-- Con todos los familiares --
Cónyuge    = Beneficio_Total × 60%
Hijos      = Beneficio_Total × 20% ÷ n_hijos
Padres     = Beneficio_Total × 20%

-- Sin padres --
Cónyuge    = Beneficio_Total × 60%
Hijos      = Beneficio_Total × 40% ÷ n_hijos

-- Sin hijos --
Cónyuge    = Beneficio_Total × 50%
Padres     = Beneficio_Total × 50%

-- Sin padres ni hijos --
Cónyuge    = Beneficio_Total × 100%

-- Solo hijos (sin cónysisse) --
Hijos      = Beneficio_Total × 75% ÷ n_hijos
Padres     = Beneficio_Total × 25%

-- Solo padres --
Padres     = Beneficio_Total × 100%
```

---

### Cálculo de Primas y Asignaciones

#### Prima de Antigüedad (Horas de servicio)

```
Prima_Antigüedad = Sueldo_Básico × Factor_Antigüedad(tiempo_servicio)
```

#### Prima por Hijos

```
Prima_Hijos = Cantidad_Hijos × Monto_Fijo_Por_Hijo
```

#### Prima de Profesionalización

```
Prima_Profesionalización = Sueldo_Básico × Porcentaje_Profesionalización
```

#### Sueldo Integral

```
Sueldo_Integral = Sueldo_Básico + Prima_Antigüedad + Prima_Hijos + Prima_Profesionalización 
               + Prima_Transporte + Prima_Especial + Prima_No_Ascenso + Prima_Tiempo_Servicio
               + Asignaciones_Mensuales
```

#### Garantías (LGT)

```
Garantía = Sueldo_Integral × Días_Garantía / 360
```

---

## Referencias Legales

**Ley Orgánica de Seguridad Social de la Fuerza Armada Nacional Bolivariana - Ley Negro Primero**

- **Gaceta Oficial**: N° 6.209 Extraordinario
- **Fecha**: 29 de diciembre de 2015
- **Archivo fuente**: `ley.negro.primero.txt`

### Artículos Clave

| Artículo | Tema |
|----------|------|
| **Art. 8** | Definición de familiares inmediatos calificados |
| **Art. 34** | Derecho a pensión tras 15 años de servicio |
| **Art. 35-36** | Definición de pensión de reserva activa |
| **Art. 37** | Base de cálculo para pensión RCP |
| **Art. 38** | Tabla de porcentajes según años de servicio |
| **Art. 40** | Definición de pensión de sobreviviente |
| **Art. 42** | Base de cálculo para pensión FCP |
| **Art. 43** | Distribución porcentual entre familiares |
| **Art. 44** | Pérdida del derecho a pensión de sobreviviente |
| **Art. 45** | Derecho de acrecer |
| **Art. 46-49** | Pensión por invalidez |

### Extractos Relevantes

**Art. 38 - Montos de Pensión**:
> "Cumplidos los quince (15) primeros años de servicio, el cincuenta por ciento (50%) del último sueldo mensual integral devengado; Cumplido el tiempo de servicio, dentro del lapso comprendido entre los dieciséis (16) y los dieciocho (18) años, la pensión continuará incrementándose anualmente en un dos por ciento (2%) del último sueldo mensual integral devengado..."

**Art. 43 - Distribución**:
> "A la viuda o al viudo corresponde el sesenta por ciento (60%). A los hijos amparados por este Decreto... le corresponde el veinte por ciento (20%) distribuido proporcionalmente entre el número de ellos. A los padres del causante... le corresponde el veinte por ciento (20%) restante de dicha pensión."

---

## Ejemplos Prácticos

### Ejemplo 1: RCP - Coronel con 25 años de servicio

**Datos del beneficiario**:
- Grado: Coronel
- Años de servicio: 25
- Sueldo básico: Bs. 500.000
- Prima antigüedad: Bs. 150.000
- Prima hijos: Bs. 50.000
- Prima profesionalización: Bs. 75.000
- Otras asignaciones: Bs. 225.000

**Cálculo**:

1. **Sueldo Integral**:
   ```
   Sueldo_Integral = 500.000 + 150.000 + 50.000 + 75.000 + 225.000 = 1.000.000
   ```

2. **Porcentaje según Art. 38** (25 años = 72% + 4%):
   ```
   Años adicionales = 25 - 23 = 2
   Porcentaje = 72% + (2 × 4%) = 80%
   ```

3. **Pensión RCP**:
   ```
   Pensión = 1.000.000 × 80% = 800.000 Bs.
   ```

**Resultado**: El coronel recibe Bs. 800.000 mensuales

---

### Ejemplo 2: FCP - Familia con cónyuge y 2 hijos

**Datos del causahabiente**:
- Coronel fallecido en actividad
- Último sueldo integral: Bs. 1.500.000
- Hijos calificados: 2

**Cálculo**:

1. **Beneficio Total** (Art. 42):
   ```
   Beneficio_Total = 1.500.000 × 100% = 1.500.000
   ```

2. **Distribución (Art. 43)**:
   ```
   Cónyuge = 1.500.000 × 60% = 900.000 Bs.
   Hijos   = 1.500.000 × 20% ÷ 2 = 150.000 Bs. cada uno
   ```

**Resultado**:
- Cónyuge recibe: Bs. 900.000
- Hijo 1 recibe: Bs. 150.000
- Hijo 2 recibe: Bs. 150.000
- **Total distribuido**: Bs. 1.200.000

---

### Ejemplo 3: FCP - Solo hijos (sin cónysisse)

**Datos del causahabiente**:
- Sargento Mayor fallecido
- Hijos calificados: 3
- Sin cónysisse ni padres vivos
- Última pensión del causahabiente: Bs. 800.000

**Cálculo (Art. 43 caso 4)**:

```
Beneficio_Total = 800.000 × 100% = 800.000
Hijos = 800.000 × 75% ÷ 3 = 200.000 Bs. cada uno
```

**Resultado**:
- Hijo 1: Bs. 200.000
- Hijo 2: Bs. 200.000
- Hijo 3: Bs. 200.000
- **Total**: Bs. 600.000
- Nota: 25% (Bs. 200.000) no se distribuye por no existir padres

---

### Ejemplo 4: RCP - Capitán con 17 años de servicio

**Datos**:
- Capitán
- Años de servicio: 17
- Sueldo integral: Bs. 600.000

**Cálculo**:
```
Años adicionales = 17 - 15 = 2
Porcentaje = 50% + (2 × 2%) = 54%
Pensión = 600.000 × 54% = 324.000 Bs.
```

---

## Estructura de CSVs

Ver archivo `config.json` para esquemas detallados de:

- **csv_nomina**: Esquema de nómina de pensiones (Rust y PHP)
- **csv_prestaciones**: Esquema de fideicomiso/prestaciones laborales

---

## Archivos del Proyecto

| Archivo | Descripción |
|---------|-------------|
| `config.json` | Esquemas de CSV para exportación |
| `ley.negro.primero.txt` | Texto completo de la Ley Negro Primero |
| `README.md` | Este documento |

---

**Última actualización**: Marzo 2026
