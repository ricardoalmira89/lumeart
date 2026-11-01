#!/usr/bin/env bash
# Compila LumeArt en release y lo instala para el usuario actual (Linux):
#   ~/.local/bin/lumeart                       el ejecutable (también la CLI)
#   ~/.local/share/icons/hicolor/*/apps/       los iconos
#   ~/.local/share/applications/lumeart.desktop el lanzador del menú
#
# Se puede repetir: sustituye lo instalado. Una ventana abierta sigue con la
# versión de antes hasta que se cierra y se vuelve a abrir.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin_dir="${HOME}/.local/bin"
icons="${HOME}/.local/share/icons/hicolor"
apps="${HOME}/.local/share/applications"

cargo build --release -p lumeart --manifest-path "${root}/Cargo.toml"

mkdir -p "${bin_dir}" "${apps}"
# Se copia aparte y se renombra: nunca queda un ejecutable a medias.
install -m 755 "${root}/target/release/lumeart" "${bin_dir}/.lumeart.nuevo"
mv -f "${bin_dir}/.lumeart.nuevo" "${bin_dir}/lumeart"

for size in 16 24 32 48 64 128 256 512; do
    mkdir -p "${icons}/${size}x${size}/apps"
    install -m 644 "${root}/assets/brand/icon-${size}.png" "${icons}/${size}x${size}/apps/lumeart.png"
done
mkdir -p "${icons}/scalable/apps"
install -m 644 "${root}/assets/brand/icon.svg" "${icons}/scalable/apps/lumeart.svg"

# El lanzador del repositorio, con la ruta absoluta del ejecutable.
sed "s|^Exec=lumeart$|Exec=${bin_dir}/lumeart|" \
    "${root}/assets/linux/lumeart.desktop" > "${apps}/lumeart.desktop"
chmod 644 "${apps}/lumeart.desktop"

command -v update-desktop-database > /dev/null && update-desktop-database "${apps}" || true
# Sin index.theme la caché no sirve (y avisa): el menú encuentra igual el icono.
if [ -f "${icons}/index.theme" ] && command -v gtk-update-icon-cache > /dev/null; then
    gtk-update-icon-cache -q -t "${icons}" || true
fi

echo "LumeArt instalado: ${bin_dir}/lumeart ($("${bin_dir}/lumeart" --version))"
