#!/usr/bin/env bash
#======================================================================
# INSTALL-MAN.SH - Instala las paginas de manual de Sandra Ecosystem
# Autor: Odin / Sandra Fabrica de Software
#======================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MAN_SRC="${SCRIPT_DIR}"

PREFIX="${1:-/usr/local}"
MAN1_DIR="${PREFIX}/share/man/man1"
MAN7_DIR="${PREFIX}/share/man/man7"

echo "Instalando paginas de manual de Sandra Ecosystem..."
echo "  Origen:  ${MAN_SRC}"
echo "  Destino man1: ${MAN1_DIR}"
echo "  Destino man7: ${MAN7_DIR}"

mkdir -p "${MAN1_DIR}" "${MAN7_DIR}"

for page in sandra-sentinel.1 sandra-reconciler.1 sandra-conciliate.1; do
    src="${MAN_SRC}/${page}"
    if [[ -f "${src}" ]]; then
        cp "${src}" "${MAN1_DIR}/${page}"
        chmod 644 "${MAN1_DIR}/${page}"
        echo "  [OK] ${page}"
    else
        echo "  [SKIP] ${page} no encontrado en ${MAN_SRC}"
    fi
done

src="${MAN_SRC}/sandra.7"
if [[ -f "${src}" ]]; then
    cp "${src}" "${MAN7_DIR}/sandra.7"
    chmod 644 "${MAN7_DIR}/sandra.7"
    echo "  [OK] sandra.7"
else
    echo "  [SKIP] sandra.7 no encontrado en ${MAN_SRC}"
fi

if command -v mandb >/dev/null 2>&1; then
    echo "Actualizando cache de man pages..."
    mandb -q "${PREFIX}/share/man" 2>/dev/null || true
elif command -v makewhatis >/dev/null 2>&1; then
    echo "Actualizando whatis database..."
    makewhatis "${PREFIX}/share/man" 2>/dev/null || true
fi

echo ""
echo "Listo. Probar con:"
echo "  man sandra-sentinel"
echo "  man sandra-reconciler"
echo "  man sandra-conciliate"
echo "  man sandra"
echo ""
echo "Si man no las encuentra, agregar a ~/.zshrc o ~/.bashrc:"
echo "  export MANPATH=${PREFIX}/share/man:\$MANPATH"
