// dibujo.js: asteroide lejano (fondo) — trozo desprendido de la colmena
canvas({ width: 1024, height: 1024, background: null })
seed(7)

const P = {
  base: "#283040",    // roca media
  shadow: "#171b25",  // lado en sombra
  deep: "#0f1219",    // sombra casi negra
  light: "#343d50",   // cara iluminada
  top: "#3e4860",     // luz más alta (arriba izquierda)
  cellA: "#26352f",   // celda hexagonal apagada
  cellB: "#18221e",   // interior de celda
  cellC: "#31443a",   // borde de celda con algo de luz
}

// Silueta: alargada en diagonal, irregular
const SIL = [
  [95, 250, 0.5], [175, 130], [300, 95, 0.5], [420, 140], [520, 175, 0.5],
  [660, 250], [780, 330, 0.5], [880, 420, 0], [945, 600, 0.5], [920, 760],
  [830, 900, 0.5], [690, 935], [570, 890, 0.5], [450, 790], [330, 700, 0.5], [190, 560], [100, 400, 0.5],
]

let roca
layer("roca", () => {
  roca = blob(SIL, { fill: P.base })

  clip(roca, () => {
    // sombra general hacia abajo a la derecha (formas planas)
    blob([[1000, 250], [1000, 1000], [150, 1000], [300, 700], [520, 730], [700, 680], [810, 540], [830, 400]], { fill: P.shadow })
    blob([[1000, 420], [1000, 1000], [400, 1000], [580, 860], [790, 780], [890, 620]], { fill: P.deep })
    // luz arriba a la izquierda
    blob([[40, 40], [600, 40], [540, 190], [400, 230], [270, 300], [190, 420], [140, 520], [40, 560]], { fill: P.light })
    blob([[40, 40], [420, 40], [340, 140], [230, 200], [160, 300], [120, 390], [40, 420]], { fill: P.top })
  })
})

layer("crateres", () => {
  const crater = (cx, cy, r, dark) => {
    const c = ellipse(cx, cy, r, r * 0.82, { fill: dark ? P.deep : P.shadow, rotate: -30 })
    clip(c, () => {
      // borde inferior derecho recibe luz (interior cóncavo)
      ellipse(cx + r * 0.35, cy + r * 0.3, r * 0.85, r * 0.7, { fill: dark ? P.shadow : P.base, rotate: -30 })
    })
    // labio iluminado arriba a la izquierda, muy sutil
    brush(arc(cx, cy, r * 1.06, 215, 320, 8), { color: dark ? P.base : P.light, width: r * 0.14, opacity: 0.6 })
  }
  crater(260, 250, 58, false)
  crater(420, 220, 30, false)
  crater(190, 420, 34, false)
  crater(470, 420, 78, false)
  crater(380, 600, 40, false)
  crater(640, 610, 56, true)
  crater(720, 830, 34, true)
  crater(600, 330, 30, false)
})

layer("grietas", () => {
  clip(roca, () => {
    brush([[230, 520], [280, 560], [270, 620], [320, 680]], { width: 7, color: P.deep, taper: [0.2, 0.5] })
    brush([[280, 560], [340, 575]], { width: 4, color: P.deep })
    brush([[500, 180], [540, 250], [610, 270], [660, 340]], { width: 6, color: P.shadow, taper: [0.3, 0.4] })
    brush([[520, 720], [570, 770], [560, 850]], { width: 5, color: P.deep })
  })
})

layer("celdas", () => {
  // un par de celdas hexagonales abandonadas en el lado derecho
  const hex = (cx, cy, r, lit) => {
    const pts = range(6).map(i => polar(cx, cy, r, i * 60 + 30))
    const h = poly(pts, { fill: P.cellA })
    clip(h, () => {
      poly(range(6).map(i => polar(cx + r * 0.08, cy + r * 0.1, r * 0.72, i * 60 + 30)), { fill: P.cellB })
      // pared hundida: la de arriba a la izquierda queda en sombra
      poly(range(6).map(i => polar(cx - r * 0.12, cy - r * 0.14, r * 0.66, i * 60 + 30)), { fill: P.deep, opacity: 0.55 })
      poly(range(6).map(i => polar(cx + r * 0.06, cy + r * 0.06, r * 0.62, i * 60 + 30)), { fill: P.cellB })
      if (lit) poly([[cx - r, cy - r], [cx + r, cy - r], [cx + r, cy - r * 0.62], [cx - r, cy - r * 0.62]], { fill: P.cellC, opacity: 0.5 })
    })
  }
  clip(roca, () => {
    // panal: vecinos a r·√3 en horizontal y 1,5·r en vertical, con junta
    const R = 50, W = R * Math.sqrt(3) + 8, H = R * 1.5 + 7
    const c0 = [735, 420]
    hex(c0[0], c0[1], R, true)
    hex(c0[0] + W, c0[1], R, true)
    hex(c0[0] + W / 2, c0[1] + H, R, false)
    // celda rota en el borde
    hex(c0[0] + W * 1.5, c0[1] + H, R, false)
  })
})

layer("textura", () => {
  clip(roca, () => {
    for (let i = 0; i < 22; i++) {
      const x = rand(140, 880), y = rand(150, 860)
      circle(x, y, rand(3, 7), { fill: P.deep, opacity: 0.35 })
    }
    grain({ opacity: 0.06 })
  })
})
