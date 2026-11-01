// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  aro: "#16040f",      // aro oscuro exterior
  sombra: "#b0006f",   // sombra de un solo tono
  magenta: "#ff14a6",  // anillo principal
  luz: "#ff6fd8",      // luz del anillo
  pastel: "#ffd3f4",   // transición al núcleo
  blanco: "#ffffff",   // núcleo
}
const C = 512
setLight(-1, -1)

layer("halo", () => {
  // halo suave que se desvanece hasta el borde del lienzo
  circle(C, C, 440, { fill: P.magenta, opacity: 0.6, blur: 70 })
  circle(C, C, 420, { fill: P.magenta, opacity: 0.55, blur: 28 })
})

layer("aro", () => {
  // aro casi negro; más grueso abajo a la derecha (lado en sombra)
  circle(C, C, 396, { fill: P.aro })
})

layer("anillo", () => {
  const ring = circle(C - 7, C - 7, 362, {
    fill: radial(C - 7, C - 7, 362, [
      [0.45, P.pastel],
      [0.58, P.luz],
      [0.78, P.magenta],
      [1, "#e0008f"],
    ]),
  })
  clip(ring, () => {
    // sombra de un solo tono en media luna (abajo a la derecha)
    const mask = circle(C - 40, C - 40, 372, { hidden: true })
    const s = circle(C - 7, C - 7, 362, { fill: P.sombra, opacity: 0.85 })
    cut(s, [mask])
    // arcos de energía girando
    seed(7)
    for (let i = 0; i < 7; i++) {
      const a0 = i * 51 + rand(-10, 10)
      const r = rand(250, 330)
      brush(arc(C - 7, C - 7, r, a0, a0 + rand(35, 60), 8), {
        width: rand(8, 16), color: alpha(P.pastel, 0.75), taper: [0.5, 0.5],
      })
    }
  })
})

layer("nucleo", () => {
  circle(C - 7, C - 7, 188, {
    fill: radial(C - 7, C - 7, 188, [[0.6, P.blanco], [1, "#ffeefb"]]),
    glow: { blur: 40, color: P.blanco },
  })
})

layer("brillos", () => {
  // brillo de pegatina blanco y marcado arriba a la izquierda
  brush(arc(C - 7, C - 7, 318, 202, 252, 10), { width: 36, color: P.blanco, taper: [0.45, 0.45] })
  const d = polar(C - 7, C - 7, 318, 188)
  circle(d[0], d[1], 16, { fill: P.blanco })
})
