# Animaciones de referencia

Los `anim.js` y `base.png` de las ejecuciones de pxcli en `reference/runs/`
(las que tienen animación), copiados aquí porque `reference/` no está en git.

`pxcli.json` lo genera el motor de pxcli (`internal/anim`, Go 1.26 + goja)
con `anim.Render` y `anim.Check`: hash FNV-1a de 64 bits de cada imagen
(bytes RGBA por filas, tal como quedan en memoria en Go, con el alfa
premultiplicado de `image.RGBA`), las líneas de las acciones, los píxeles
que se salen de la celda y los avisos.

Se regenera con `scripts/anim-golden/` (ver su `LEEME.md`).
