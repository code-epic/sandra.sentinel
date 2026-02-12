# Guía de Fórmulas y Cálculos (PACE Kernel)

Esta guía detalla la lógica de negocio extraída de `src/migracion/pace-kernel/models/kernel/KCalculo.php` y `KCalculoLote.php`. Estas fórmulas son la verdad absoluta para el cálculo de prestaciones.

## Variables Globales

- **SG**: Sueldo Global
- **SI**: Sueldo Integral
- **AA**: Alicuota de Aguinaldos
- **AV**: Alicuota de Vacaciones
- **TS**: Tiempo de Servicio (Años)
- **UT**: Unidad Tributaria

---

## 1. Sueldo Global (SG)

Es la base de todos los cálculos. Suma del sueldo base más todas las primas.

**Fórmula:**
`SG = SueldoBase + Σ(Primas)`

**Primas Incluidas:**

- Prima de Transporte
- Prima de Año de Servicio
- Prima de Descendencia
- Prima de No Ascenso
- Prima Especial
- Prima de Profesionalización

> **Nota**: `round(Result, 2)` se aplica al final de la suma.

---

## 2. Alicuota de Aguinaldos (AA)

Parte proporcional del bono de fin de año distribuida mensualmente.

**Fórmula General:**
`AA = ((Días_Aguinaldo * SG) / 30) / 12`

**Condiciones de Días:**

- **Estándar**: 120 días.
- **Retiro < 29/10/2016**: 90 días.
- **Retiro entre 29/10/2016 y 31/12/2016**: 105 días.

---

## 3. Alicuota de Vacaciones (AV)

Parte proporcional del bono vacacional.

**Fórmula General:**
`AV = ((Días_Vacaciones * SG) / 30) / 12`

**Días según Tiempo de Servicio (TS):**

- **TS <= 14 años**: 40 días (o 50 según fecha retiro > 2016).
- **14 < TS <= 24 años**: 45 días (o 50 según fecha retiro > 2016).
- **TS > 24 años**: 50 días.

> **Regla Especial**: Si `Fecha_Retiro > 31/12/2016` o es Activo, se usan **50 días** fijos.

---

## 4. Sueldo Integral (SI)

El sueldo utilizado para calcular las prestaciones sociales.

**Fórmula:**
`SI = SG + AA + AV`

---

## 5. Asignación de Antigüedad (Prestaciones)

El monto total acumulado por el militar.

**Fórmula:**
`Asignacion = SI * TS`

---

## 6. Finiquito y Saldos

Cálculos para determinar cuánto dinero real recibe el beneficiario al retirarse, descontando deudas y anticipos.

### 6.1 Total Aportados

Dinero ya recibido o garantizado.
`Total_Aportados = DepositoBanco + Garantias + Dias_Adicionales`

### 6.2 Anticipos Netos

`Anticipos_Netos = Movimiento[5] - Movimiento[25]` (Anticipos menos Reversos).

### 6.3 Saldo Disponible (Finiquito)

`Saldo = ( (DepositoBanco - Anticipos) + Garantias + ComisionServicio ) - ( Embargos + Monto_Recuperar )`

### 6.4 Monto a Recuperar (Deuda)

Si el cálculo final es menor a lo que ya se le dio.
`Diferencia = AsignacionFiniquito - (Asignacion_Depositada + Dias_Adicionales)`
Si `Diferencia < 0`, entonces `Monto_Recuperar = ABS(Diferencia)`.

---

## 7. Garantías y Días Adicionales

### Garantías

`Garantias = (SI / 30) * 15`

### Días Adicionales

Se otorgan después des cierto tiempo de servicio.
`Dias_Adic = (SM / 30 * 2) * Factor`

- **Factor**: `TS` (Años de servicio), con tope de 15.
- `SM`: Sueldo Mensual (`SueldoBase + Primas`).
