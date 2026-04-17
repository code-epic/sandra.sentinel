# SANDRA SENTINEL - Habilidad de Migración de Nómina

## Identificación

- **Nombre**: Sandra Sentinel - Migración de Nómina
- **Alias**: Loki, Sentinel
- **Proyecto**: `/rust/sandra.sentinel/`
- **Ruta Migración**: `/rust/sandra.sentinel/migracion/`
- **Versión**: 1.0.0
- **Fecha**: 17/04/2026
- **Ubicación**: `.agent/skills/sentinel-migracion.md`

---

## Arquitectura del Proyecto

### Directorios Principales

```
rust/sandra.sentinel/
├── migracion/
│   ├── nomina.pensiones/     # App CodeIgniter - Sistema de pensiones
│   │   ├── application/
│   │   │   ├── controllers/  # Lógica de control
│   │   │   ├── models/       # Modelos (kernel, fisico, logico)
│   │   │   ├── views/        # Vistas
│   │   │   └── config/       # Configuración
│   │   └── system/           # Framework CodeIgniter
│   └── nomina.prestaciones/  # App de prestaciones
│       ├── application/
│       │   └── modules/panel/
│       │       ├── controllers/
│       │       │   └── WServer.php  # ← Fórmulas de cálculo
│       │       ├── models/
│       │       └── views/
```

---

## Funciones de Cálculo de Pensión

### 1. Función Casos($t) - Artículo 38 (Año >= 2010)

**Ubicación**: `nomina.prestaciones/application/modules/panel/controllers/WServer.php`

```php
public function Casos($t){
    $v = 0;
    switch ($t) {
        case 15: $v = 50; break;
        case 16: $v = 52; break;
        case 17: $v = 54; break;
        case 18: $v = 56; break;
        case 19: $v = 59; break;
        case 20: $v = 62; break;
        case 21: $v = 65; break;
        case 22: $v = 68; break;
        case 23: $v = 72; break;
        case 24: $v = 76; break;
        case 25: $v = 80; break;
        case 26: $v = 84; break;
        case 27: $v = 89; break;
        case 28: $v = 94; break;
        case 29: $v = 99; break;
        case 30: $v = 100; break;
        default: if ($t>30) $v = 100; break;
    }
    return $v;
}
```

**Descripción**: Retorna el porcentaje de pension segun años de servicio para promociones del 2010 en adelante.

**Parámetros**:
- `$t`: Tiempo de servicio (años)
- **Retorna**: Porcentaje (0-100)

### 2. Función CasoMenor2010($t) - Tabla Especial

```php
public function CasoMenor2010($t){
    $v = 0;
    switch ($t) {
        case 15: $v = 60; break;
        case 16: $v = 63; break;
        case 17: $v = 66; break;
        case 18: $v = 69; break;
        case 19: $v = 72; break;
        case 20: $v = 75; break;
        case 21: $v = 80; break;
        case 22: $v = 84; break;
        case 23: $v = 88; break;
        case 24: $v = 92; break;
        case 25: $v = 99; break;
        default: if ($t>25) $v = 100; break;
    }
    return $v;
}
```

**Descripción**: Retorna el porcentaje especial para promociones anteriores a 2010.

---

## Tabla de Porcentajes

### Casos() - Año >= 2010 (Artículo 38)

| Años | Porcentaje | Ejemplo (Bs. 1.500.000) |
|------|------------|------------------------|
| 15 | 50% | Bs. 750.000 |
| 16 | 52% | Bs. 780.000 |
| 17 | 54% | Bs. 810.000 |
| 18 | 56% | Bs. 840.000 |
| 19 | 59% | Bs. 885.000 |
| 20 | 62% | Bs. 930.000 |
| 21 | 65% | Bs. 975.000 |
| 22 | 68% | Bs. 1.020.000 |
| 23 | 72% | Bs. 1.080.000 |
| 24 | 76% | Bs. 1.140.000 |
| 25 | 80% | Bs. 1.200.000 |
| 26 | 84% | Bs. 1.260.000 |
| 27 | 89% | Bs. 1.335.000 |
| 28 | 94% | Bs. 1.410.000 |
| 29 | 99% | Bs. 1.485.000 |
| 30+ | 100% | Bs. 1.500.000 |

### CasoMenor2010() - Tabla Especial

| Años | Porcentaje |
|------|-----------|
| 15 | 60% |
| 16 | 63% |
| 17 | 66% |
| 18 | 69% |
| 19 | 72% |
| 20 | 75% |
| 21 | 80% |
| 22 | 84% |
| 23 | 88% |
| 24 | 92% |
| 25 | 99% |
| 26+ | 100% |

---

## Flujo de Cálculo

```
1. Recibir datos del militar
   ↓
2. Verificar año de ingreso
   ↓
   ├─ Si año < 2010 → CasoMenor2010()
   ├─ Si año >= 2010 → Casos()
   ↓
3. Obtener porcentaje según años de servicio
   ↓
4. Calcular: Pension = Sueldo × (Porcentaje / 100)
   ↓
5. Generar respuesta JSON
```

### Código del Flujo

```php
if($ano[0] < 2010) {
    $p = $this->CasoMenor2010($this->MBeneficiario->tiempo_servicio);
}else{
    $p = $this->Casos($this->MBeneficiario->tiempo_servicio);
}

$rs = array('Persona' => [
    's_mensual' => $this->MBeneficiario->sueldo_global,
    's_pension_mensual' => ($this->MBeneficiario->sueldo_global * $p)/100,
    's_pension_integral' => ($this->MBeneficiario->sueldo_integral * $p)/100,
    'p_pension' => $p . '%',
    'f_ingreso' => $this->MBeneficiario->fecha_ingreso,
    't_servicio' => $this->MBeneficiario->tiempo_servicio
]);
```

---

## Glosario de Términos

| Término | Descripción |
|---------|------------|
| **Pensión** | Prestación económica mensual por años de servicio |
| **Tiempo de Servicio** | Años累计 de labor en la FANB |
| **Sueldo Global** | Sueldo base + primas + bonos |
| **Sueldo Integral** | Sueldo + aguinaldo + vacaciones |
| **Porcentaje** | Tasa aplicada según años de servicio |
| **Antigüedad** | Asignación por años de servicio |
| **Sobreviviente** | Familiar con derecho a pensión por fallecimiento |
| **Reserva Activa** | Estado del militar pensionado |
| **Directiva** | Norma de cálculo de nómina |
| **Retroactivo** | Diferencia de pago por ajustes |
| **Embargo** |Descuento por medida judicial |
| **Medida Judicial** | Orden judicial de descuento |

---

## Modelos del Sistema

### Modelos Kernel

| Archivo | Descripción |
|---------|-------------|
| `KCalculo.php` | Cálculos de nómina |
| `KPension.php` | Gestión de pensiones |
| `KDirectiva.php` | Gestión de directivas |
| `KConcepts.php` | Definición de conceptos |
| `KRetroactivo.php` | Cálculo de retroactivos |
| `KRecibo.php` | Generación de recibos |
| `KNomina.php` | Procesamiento de nómina |

### Modelos Físicos

| Archivo | Descripción |
|---------|-------------|
| `MBeneficiario.php` | Datos del beneficiario |
| `MGrado.php` | Grados militares |
| `MComponente.php` | Componentes FANB |
| `MMedidaJudicial.php` | Embargos judiciales |
| `MOrdenPago.php` | Órdenes de pago |
| `MAnticipo.php` | Anticipos de antigüedad |

---

## Endpoints Principales

| Endpoint | Descripción |
|----------|-------------|
| `/WServer/calcular` | Calcula pension |
| `/WServer/antiguedad` | Calcula antigüedad |
| `/WServer/retroactivo` | Calcula retroactivo |
| `/WServer/nomina` | Procesa nómina |
| `/WServer/datos` | Consulta datos |

---

## Referencias Legales

- **Ley Negro Primero** (Gaceta Oficial N° 6.209 del 29/12/2015)
  - Artículo 34: Jubilación con 15 años mínimos
  - Artículo 38: Porcentajes de pension
  - Artículo 43: Distribución de sobreviente
  - Artículo 56-59: Asignación de antigüedad

---

## Uso

Esta habilidad permite a Sandra Sentinel:

1. **Calcular pension** según años de servicio
2. **Distribución de porcentaje** para promociones del 2010
3. **Aplicacionespecial** para promociones anteriores al 2010
4. **Determinar antiguedad** y asignaciones

---

## Enlaces

- **Código Fuente**: `/rust/sandra.sentinel/migracion/`
- **Controlador**: `nomina.prestaciones/application/modules/panel/controllers/WServer.php`
- **Ley Negro Primero**: `.agent/skills/lokimd`
- **Habilidades**: `.agent/skills/`