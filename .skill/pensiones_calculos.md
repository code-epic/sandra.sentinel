# Skill: Cálculos de Pensiones Militares

## Propósito
Proporcionar fórmulas, validaciones y funciones para el cálculo de nóminas de pensiones militares según la Ley Negro Primero.

---

## Tabla de Contenidos

1. [Constantes y Parámetros](#constantes-y-parámetros)
2. [Funciones de Cálculo RCP](#funciones-de-cálculo-rcp)
3. [Funciones de Cálculo FCP](#funciones-de-cálculo-fcp)
4. [Validaciones](#validaciones)
5. [Componentes del Sueldo](#componentes-del-sueldo)

---

## Constantes y Parámetros

### Porcentajes de Pensión RCP (Art. 38)

| Años Min | Años Max | Porcentaje Base | Incremento Anual |
|----------|----------|-----------------|-----------------|
| 15 | 15 | 50% | - |
| 16 | 18 | 50% | +2% |
| 19 | 19 | 59% | - |
| 20 | 22 | 59% | +3% |
| 23 | 23 | 72% | - |
| 24 | 26 | 72% | +4% |
| 27 | 29 | 84% | +5% |
| 30 | 30+ | 100% | - |

### Distribución FCP (Art. 43)

| Caso | Cónyuge | Hijos | Padres |
|------|---------|-------|--------|
| Default (todos) | 60% | 20% | 20% |
| Sin padres | 60% | 40% | 0% |
| Sin hijos | 50% | 0% | 50% |
| Sin padres ni hijos | 100% | 0% | 0% |
| Solo hijos | 0% | 75% | 25% |
| Solo padres | 0% | 0% | 100% |

### Requisitos Legales

```
MIN_ANTIGUEDAD_RCP = 15  # años
MAX_PENSION_RCP = 100    # porcentaje
MIN_PENSION_RCP = 50     # porcentaje
EDAD_MAX_HIJOS_UNIVERSITARIOS = 26  # años
EDAD_MAYORIA = 18        # años
```

---

## Funciones de Cálculo RCP

### `calcular_porcentaje_rcp(anos_servicio)`

```python
def calcular_porcentaje_rcp(anos_servicio: int) -> float:
    """
    Calcula el porcentaje de pensión según años de servicio.
    Art. 38 Ley Negro Primero
    
    Args:
        anos_servicio: Años completos de servicio militar
        
    Returns:
        Porcentaje de pensión (0.0 - 1.0)
    """
    if anos_servicio < 15:
        return 0.0  # Sin derecho a pensión
    elif anos_servicio == 15:
        return 0.50
    elif 16 <= anos_servicio <= 18:
        return 0.50 + (anos_servicio - 15) * 0.02
    elif anos_servicio == 19:
        return 0.59
    elif 20 <= anos_servicio <= 22:
        return 0.59 + (anos_servicio - 19) * 0.03
    elif anos_servicio == 23:
        return 0.72
    elif 24 <= anos_servicio <= 26:
        return 0.72 + (anos_servicio - 23) * 0.04
    elif 27 <= anos_servicio <= 29:
        return 0.84 + (anos_servicio - 26) * 0.05
    else:  # 30+
        return 1.0
```

### `calcular_sueldo_integral(datos)`

```python
def calcular_sueldo_integral(
    sueldo_base: float,
    prima_antiguedad: float,
    prima_hijos: float,
    prima_profesionalizacion: float,
    prima_transporte: float = 0.0,
    prima_especial: float = 0.0,
    prima_no_ascenso: float = 0.0,
    prima_tiempo_servicio: float = 0.0,
    otros_beneficios: float = 0.0
) -> float:
    """
    Calcula el sueldo integral sumando todos los componentes.
    Art. 37 Ley Negro Primero
    """
    return (
        sueldo_base
        + prima_antiguedad
        + prima_hijos
        + prima_profesionalizacion
        + prima_transporte
        + prima_especial
        + prima_no_ascenso
        + prima_tiempo_servicio
        + otros_beneficios
    )
```

### `calcular_pension_rcp(sueldo_integral, anos_servicio)`

```python
def calcular_pension_rcp(sueldo_integral: float, anos_servicio: int) -> float:
    """
    Calcula la pensión mensual para RCP.
    Fórmula: Pensión = Sueldo Integral × Porcentaje
    """
    porcentaje = calcular_porcentaje_rcp(anos_servicio)
    return sueldo_integral * porcentaje
```

### `calcular_pension_mensual_rcp(pensiones_mensuales, bono_recreacional, bonificacion_fin_ano)`

```python
def calcular_pension_mensual_rcp(
    pensiones_mensuales: list[float],
    bono_recreacional: float,
    bonificacion_fin_ano: float
) -> float:
    """
    Calcula la pensión mensual promedio.
    Art. 36: (Σ pensiones / 12) + (bono_recreacional / 12) + (bonificacion / 12)
    """
    suma_pensiones = sum(pensiones_mensuales)
    return (suma_pensiones / 12) + (bono_recreacional / 12) + (bonificacion_fin_ano / 12)
```

---

## Funciones de Cálculo FCP

### `obtener_base_calculo_fcp(causahabiente)`

```python
def obtener_base_calculo_fcp(
    tipo_causahabiente: str,
    ultimo_sueldo_integral: float = 0.0,
    ultima_pension_mensual: float = 0.0
) -> float:
    """
    Obtiene la base de cálculo para pensión FCP.
    Art. 42 Ley Negro Primero
    
    Args:
        tipo_causahabiente: "actividad" o "pensionado"
    """
    if tipo_causahabiente == "actividad":
        return ultimo_sueldo_integral
    else:  # pensionado
        return ultima_pension_mensual
```

### `calcular_distribucion_fcp(base_calculo, n_conyuge, n_hijos, n_padres)`

```python
def calcular_distribucion_fcp(
    base_calculo: float,
    n_conyuge: int,
    n_hijos: int,
    n_padres: int
) -> dict:
    """
    Calcula la distribución de pensión FCP según Art. 43.
    
    Returns:
        dict con { "conyuge": float, "hijos": list[float], "padres": float }
    """
    resultado = {"conyuge": 0.0, "hijos": [], "padres": 0.0}
    
    tiene_conyuge = n_conyuge > 0
    tiene_hijos = n_hijos > 0
    tiene_padres = n_padres > 0
    
    if tiene_conyuge and tiene_hijos and tiene_padres:
        # Caso 1: Todos presentes
        resultado["conyuge"] = base_calculo * 0.60
        por_hijo = (base_calculo * 0.20) / n_hijos
        resultado["hijos"] = [por_hijo] * n_hijos
        resultado["padres"] = base_calculo * 0.20
        
    elif tiene_conyuge and tiene_hijos and not tiene_padres:
        # Caso 2: Sin padres
        resultado["conyuge"] = base_calculo * 0.60
        por_hijo = (base_calculo * 0.40) / n_hijos
        resultado["hijos"] = [por_hijo] * n_hijos
        
    elif tiene_conyuge and not tiene_hijos and tiene_padres:
        # Caso 5: Sin hijos
        resultado["conyuge"] = base_calculo * 0.50
        resultado["padres"] = base_calculo * 0.50
        
    elif tiene_conyuge and not tiene_hijos and not tiene_padres:
        # Caso 6: Solo cónysisse
        resultado["conyuge"] = base_calculo * 1.0
        
    elif not tiene_conyuge and tiene_hijos and tiene_padres:
        # Caso 4: Solo hijos y padres
        por_hijo = (base_calculo * 0.75) / n_hijos
        resultado["hijos"] = [por_hijo] * n_hijos
        resultado["padres"] = base_calculo * 0.25
        
    elif not tiene_conyuge and tiene_hijos and not tiene_padres:
        # Caso 8: Solo hijos
        por_hijo = base_calculo / n_hijos
        resultado["hijos"] = [por_hijo] * n_hijos
        
    elif not tiene_conyuge and not tiene_hijos and tiene_padres:
        # Caso 7: Solo padres
        resultado["padres"] = base_calculo * 1.0
        
    return resultado
```

### `calcular_acrecimiento_fcp(distribucion_actual, porcentaje_perdido, tipo_perdido)`

```python
def calcular_acrecimiento_fcp(
    distribucion_actual: dict,
    porcentaje_perdido: float,
    tipo_perdido: str  # "conyuge", "hijos", "padres"
) -> dict:
    """
    Calcula el acrecimiento cuando un beneficiario pierde el derecho.
    Art. 45 Ley Negro Primero
    
    Redistribuye el porcentaje perdido proporcionalmente entre los demás.
    """
    pass  # Implementar según necesidad
```

---

## Validaciones

### `validar_calificacion_hijo(edad, es_estudiante, tiene_discapacidad, tiene_enfermedad_catastrofica)`

```python
def validar_calificacion_hijo(
    edad: int,
    es_estudiante: bool,
    tiene_discapacidad: bool,
    tiene_enfermedad_catastrofica: bool
) -> bool:
    """
    Valida si un hijo cumple requisitos para ser familiar calificado.
    Art. 8 numeral 3
    """
    if edad < 18:
        return True  # Menor de edad siempre califica
    
    if tiene_discapacidad:
        return True
        
    if tiene_enfermedad_catastrofica:
        return True
        
    if es_estudiante and edad <= 26:
        return True
        
    return False
```

### `validar_derecho_pension_rcp(anos_servicio)`

```python
def validar_derecho_pension_rcp(anos_servicio: int) -> tuple[bool, str]:
    """
    Valida si cumple requisitos para pensión RCP.
    Art. 34
    """
    if anos_servicio < 15:
        return False, "Mínimo 15 años de servicio requeridos"
    return True, "Derecho a pensión confirmado"
```

### `validar_perdida_derecho_fcp(motivo: str)`

```python
MOTIVOS_PERDIDA = [
    "fallecimiento",           # Art. 44.1
    "hijos_dejan_calificar",  # Art. 44.2
    "conyuge_nuevas_nupcias"  # Art. 44.3
]

def validar_perdida_derecho_fcp(motivo: str) -> bool:
    """
    Verifica si el motivo es válido para pérdida de derecho.
    Art. 44 Ley Negro Primero
    """
    return motivo in MOTIVOS_PERDIDA
```

---

## Componentes del Sueldo

### Estructura del Sueldo Integral

```
Sueldo_Integral
├── Sueldo_Básico
│   └── Determinado por grado y jerarquía
├── Prima_Antigüedad
│   └── Basada en años de servicio
├── Prima_Hijos
│   └── Cantidad_hijos × Monto_por_hijo
├── Prima_Profesionalización
│   └── Porcentaje según nivel educativo
├── Prima_Transporte
│   └── Monto fijo establecido
├── Prima_Especial
│   └── Según componente y situación
├── Prima_No_Ascenso
│   └── Para personal sin ascenso
├── Prima_Tiempo_Servicio
│   └── Acumulado por años
└── Otros_Beneficios
    └── Asignaciones especiales
```

### Cálculo de Garantías (LGT)

```python
def calcular_garantia(sueldo_integral: float, dias_garantia: int = 30) -> float:
    """
    Calcula la garantía de vacaciones.
    Fórmula: Sueldo Integral × Días / 360
    """
    return sueldo_integral * dias_garantia / 360
```

---

## Ejemplo de Uso Completo

```python
# RCP: Calcular pensión de Capitán con 22 años
sueldo_base = 350000.00
prima_antiguedad = 85000.00
prima_hijos = 50000.00
prima_prof = 45000.00
otros = 65000.00

# 1. Calcular sueldo integral
sueldo_integral = calcular_sueldo_integral(
    sueldo_base, prima_antiguedad, prima_hijos, prima_prof, otros
)
# Resultado: 595.000

# 2. Calcular porcentaje (22 años = 59% + 9% = 68%)
porcentaje = calcular_porcentaje_rcp(22)
# Resultado: 0.68

# 3. Calcular pensión
pension = calcular_pension_rcp(sueldo_integral, 22)
# Resultado: 404.600
```

```python
# FCP: Calcular distribución para viuda con 2 hijos
base = 800000.00
distribucion = calcular_distribucion_fcp(
    base_calculo=base,
    n_conyuge=1,
    n_hijos=2,
    n_padres=0
)
# Resultado:
# {
#     "conyuge": 480000.00,  # 60%
#     "hijos": [160000.00, 160000.00],  # 40% / 2
#     "padres": 0.0
# }
```

---

## Notas de Implementación

1. **Precisión**: Usar decimales para cálculos financieros (no floats)
2. **Redondeo**: Aplicar a 2 decimales para montos monetarios
3. **Homologación**: La pensión se ajusta automáticamente según Art. 39
4. **Límites**: Verificar que el porcentaje no exceda 100%
5. **溯源**: Guardar referencia al artículo legal de cada cálculo

---

**Skill Version**: 1.0  
**Última actualización**: Marzo 2026  
**Referencia legal**: Ley Negro Primero (Gaceta Oficial N° 6.209 Extraordinario, 29/12/2015)
