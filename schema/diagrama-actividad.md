# Diagrama de Actividad - Sandra Sentinel

```mermaid
flowchart TD
    START([Inicio]) --> CLI
    
    subgraph CLI["1. CLI - main.rs"]
        CLI["parsear argumentos<br/>sandra start -x -t nrcp -m manifest.json"]
    end
    
    CLI --> INIT
    
    subgraph INIT["2. Inicializacion - start.rs"]
        INIT["System::init()"]
        INIT --> LOAD_MANIFEST
        LOAD_MANIFEST["Cargar manifest.json"]
        LOAD_MANIFEST --> CONNECT
        CONNECT["Conectar a Sandra Server (gRPC)"]
    end
    
    CONNECT --> KERNEL
    
    subgraph KERNEL["3. Kernel - ejecutar_ciclo_carga()"]
        
        subgraph P1["PASO 1: Carga de Referencias"]
            P1A["Cargar Directiva"]
            P1A --> P1B
            P1B["Cargar Primas Funciones"]
            P1B --> P1C
            P1C["Crear SentinelEngine"]
            P1C --> P1D
            P1D(["Motor listo"])
        end
        
        subgraph P2["PASO 2: Carga Masiva"]
            P2A["Cargar Movimientos"]
            P2A --> P2B
            P2B{"Ejecutar en<br/>paralelo?"}
            P2B -->|Si| P2C
            P2B -->|Si| P2D
            P2C["Cargar Base<br/>(+ calcular sueldo_integral)"]
            P2D["Cargar Conceptos"]
            P2C --> P2E
            P2D --> P2E
            P2E(["Datos cargados"])
        end
        
        subgraph P2_5["PASO 2.5: Conceptos Dinamicos"]
            P2_5A["Crear EjecutorConceptos"]
            P2_5A --> P2_5B
            P2_5B["Ejecutar formulas Rhai<br/>(en paralelo con Rayon)"]
            P2_5B --> P2_5C
            P2_5C["HashMap cedula → Conceptos"]
        end
        
        subgraph P3["PASO 3: Fusion de Beneficiarios"]
            P3A["cargar_beneficiarios()"]
            P3A --> P3B
            P3B["Fusionar base + movimientos"]
            P3B --> P3C
            P3C(["Beneficiarios listos"])
        end
        
        subgraph P3_5["PASO 3.5: Calcular Neto"]
            P3_5A{"Para cada<br/>beneficiario"}
            P3_5A -->|Si| P3_5B
            P3_5A -->|No| P3_5E
            
            P3_5B["Buscar conceptos calculados"]
            P3_5B --> P3_5C
            P3_5C["Sumar asignaciones<br/>Sumar deducciones"]
            P3_5C --> P3_5D
            
            P3_5D{"tipo_nomina?"}
            P3_5D -->|NPR| NPR["neto = garantias"]
            P3_5D -->|NACT| NACT["neto = integral + asig - deduc"]
            P3_5D -->|NRCP| NRCP["neto = (integral × pct/100)<br/>+ asig - deduc"]
            P3_5D -->|NFCP| NFCP["neto = (integral × pct/100)<br/>+ asig - deduc"]
            
            NPR & NACT & NRCP & NFCP --> P3_5A
            
            P3_5E(["Neto calculado<br/>para todos"])
        end
    end
    
    KERNEL --> EXPORT
    
    subgraph EXPORT["4. Exportacion - exportador.rs"]
        EXP1["exportar_nomina_dinamica()"]
        EXP1 --> EXP2["nomina_[tipo]_[ciclo].csv"]
        
        EXP2 --> EXP3{"aportes<br/>habilitado?"}
        EXP3 -->|Si| EXP4["exportar_aporte_csv()"]
        EXP3 -->|No| EXP5
        EXP4 --> EXP5
        
        EXP5{"generar<br/>TXT bancos?"}
        EXP5 -->|Si| EXP6["Generar TXT<br/>Venezuela / Banfanb / Bicentenario"]
        EXP5 -->|No| EXP7
        EXP6 --> EXP7
        
        EXP7["generar_manifest()"]
        EXP7 --> EXP8["manifest.json"]
    end
    
    EXPORT --> FINISH
    FINISH([Fin]) --> REPORT
    
    subgraph REPORT["5. Reporte"]
        REPORT1["Reporte de telemetria"]
        REPORT1 --> REPORT2["sandra_metrics_report.txt"]
    end

    style START fill:#4CAF50,color:#fff
    style FINISH fill:#f44336,color:#fff
    style CLI fill:#2196F3,color:#fff
    style KERNEL fill:#FF9800,color:#fff
    style EXPORT fill:#9C27B0,color:#fff
    style P2_5 fill:#E91E63,color:#fff
    style P3_5 fill:#00BCD4,color:#fff
```

