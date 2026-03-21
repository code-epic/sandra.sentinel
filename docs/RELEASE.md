# Release Guide

Guía para crear releases de **Sandra Sentinel**.

---

## Método Automático (GitHub Actions)

### Prerrequisitos

1. **Crear GitHub Personal Access Token**
   ```
   https://github.com/settings/tokens/new
   ```
   - **Scopes:** ☑️ `repo` (Full control)
   - **Expiration:** 30 días o personalizado

2. **Añadir Secret en el repositorio**
   ```
   https://github.com/code-epic/sandra.sentinel/settings/secrets/actions
   ```
   - **Name:** `HOMEBREW_TOKEN`
   - **Secret:** Pegar el token generado

3. **Subir cambios del workflow**
   ```bash
   git add .github/workflows/release.yml cli/Cargo.toml
   git commit -m "Add GitHub Actions release workflow"
   git push origin main
   ```

### Crear Release

```bash
# 1. Asegurarse de estar en main y tener cambios
git checkout main
git pull origin main

# 2. Actualizar versión en Cargo.toml
# Editar cli/Cargo.toml: version = "0.1.0" → version = "0.2.0"

# 3. Commitear cambios de versión
git add cli/Cargo.toml
git commit -m "Bump version to 0.2.0"
git push origin main

# 4. Crear y pushear tag
git tag v0.2.0
git push origin main --tags
```

### Flujo del Workflow

```
┌──────────────────────────────────────────────────────────────────┐
│  push tag v* → GitHub Actions                                     │
│                                                                  │
│  ┌─────────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │   BUILD     │───▶│ CREATE       │───▶│ UPDATE          │   │
│  │  (3 targets)│    │ RELEASE      │    │ HOMEBREW TAP    │   │
│  └─────────────┘    └──────────────┘    └─────────────────┘   │
│                                                                  │
│  Targets:                                                         │
│  • macOS ARM64 (aarch64-apple-darwin)                            │
│  • macOS Intel  (x86_64-apple-darwin)                           │
│  • Linux        (x86_64-unknown-linux-gnu)                      │
└──────────────────────────────────────────────────────────────────┘
```

---

## Método Manual

### Compilar Localmente

```bash
# macOS ARM64
cargo build --release --target aarch64-apple-darwin --package sandra_sentinel
mv target/aarch64-apple-darwin/release/sandra-sentinel sandra-sentinel-aarch64-apple-darwin

# macOS Intel
cargo build --release --target x86_64-apple-darwin --package sandra_sentinel
mv target/x86_64-apple-darwin/release/sandra-sentinel sandra-sentinel-x86_64-apple-darwin

# Linux
cargo build --release --target x86_64-unknown-linux-gnu --package sandra_sentinel
mv target/x86_64-unknown-linux-gnu/release/sandra-sentinel sandra-sentinel-x86_64-unknown-linux-gnu
```

### Subir a GitHub Releases

1. Ir a: `https://github.com/code-epic/sandra.sentinel/releases/new`
2. Seleccionar tag `v0.2.0`
3. Título: `v0.2.0`
4. Arrastrar binarios compilados
5. Click **"Publish release"**

### Actualizar Homebrew Tap Manualmente

```bash
cd ~/dev/homebrew-sandra

# 1. Actualizar versión
sed -i 's/version ".*"/version "0.2.0"/' Formula/sandra-sentinel.rb

# 2. Descargar y calcular SHA256
VERSION="0.2.0"
for binary in sandra-sentinel-aarch64-apple-darwin sandra-sentinel-x86_64-apple-darwin sandra-sentinel-x86_64-unknown-linux-gnu; do
  curl -sL "https://github.com/code-epic/sandra.sentinel/releases/download/v$VERSION/$binary" -o "/tmp/$binary"
  sha=$(sha256sum "/tmp/$binary" | awk '{print $1}')
  sed -i "s|sha256 \".*\" # $binary|sha256 \"$sha\" # $binary|" Formula/sandra-sentinel.rb
done

# 3. Commit y push
git add Formula/sandra-sentinel.rb
git commit -m "Release v0.2.0"
git push origin main
```

---

## Instalación

### macOS/Linux con Homebrew

```bash
# Añadir tap (si no está)
brew tap code-epic/sandra https://github.com/code-epic/homebrew-sandra

# Instalar
brew install code-epic/sandra/sandra-sentinel

# Verificar
sandra-sentinel --version
```

### Binarios Directos

```bash
# macOS ARM64
sudo curl -L https://github.com/code-epic/sandra.sentinel/releases/latest/download/sandra-sentinel-aarch64-apple-darwin \
  -o /usr/local/bin/sandra-sentinel
sudo chmod +x /usr/local/bin/sandra-sentinel

# Linux
sudo curl -L https://github.com/code-epic/sandra.sentinel/releases/latest/download/sandra-sentinel-x86_64-unknown-linux-gnu \
  -o /usr/local/bin/sandra-sentinel
sudo chmod +x /usr/local/bin/sandra-sentinel
```

---

## Verificación Post-Release

```bash
# 1. Verificar que el release existe
gh release view v0.2.0

# 2. Verificar que los assets están subidos
gh release view v0.2.0 --json assets

# 3. Verificar Homebrew
brew install code-epic/sandra/sandra-sentinel
sandra-sentinel --version
```

---

## Troubleshooting

### Error: `HOMEBREW_TOKEN` not found
- Ir a `https://github.com/code-epic/sandra.sentinel/settings/secrets/actions`
- Crear secret `HOMEBREW_TOKEN`

### Error: Permission denied (homebrew-tap)
- Verificar que el token tiene scope `repo`
- Verificar que el token tiene acceso al repo `code-epic/homebrew-sandra`

### Error: Build fails
- Verificar que Rust está instalado: `rustc --version`
- Verificar cargo: `cargo --version`

---

## Changelog

| Versión | Fecha | Descripción |
|---------|-------|-------------|
| v0.1.0 | 2026-03-21 | Release inicial con GitHub Actions |
