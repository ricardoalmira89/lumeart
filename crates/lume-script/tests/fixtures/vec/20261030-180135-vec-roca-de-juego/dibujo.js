// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)
seed(7)
setLight(-1, -1)

const P = {
  base: "#8f7663",     // marrón grisáceo cálido
  sombra: "#5e4a4f",   // sombra de un solo tono (hacia violeta)
  luz: "#b89c7f",      // cara iluminada
  veta: "#6b5446",     // vetas oscuras
  crater: "#4f3d41",   // fondo de cráter
  tinta: "#2a1c1c",    // contorno
  brillo: "#fff6e6",
  blanco: "#ffffff",
}

// Silueta: redonda pero con bultos y algunas aristas
const S0 = [
  [500, 122], [600, 140, 0.5], [690, 158], [775, 226, 0], [838, 320], [880, 420, 0.5],
  [898, 530], [868, 640], [815, 760, 0], [720, 828], [640, 880, 0.5], [520, 898],
  [410, 880], [318, 838, 0], [222, 770], [160, 668, 0.5], [128, 545], [145, 420],
  [188, 312, 0], [272, 222], [380, 150, 0.5],
]
const S = roughen(S0, 8, { detail: 1, sharp: 0.2 })

layer("asteroide", () => {
  element("roca", { pivot: [512, 510] }, () => {
    // Borde blanco de pegatina
    blob(S, { fill: P.blanco, stroke: P.blanco, width: 64, behind: true, join: "round" })
    // Cuerpo
    const roca = blob(S, { fill: P.base })
    clip(roca, () => {
      // Luz: casquete arriba a la izquierda
      blob([[90, 90], [600, 60], [560, 260], [430, 330], [300, 430], [200, 600], [60, 640]],
        { fill: P.luz, opacity: 0.9 })
      // Sombra de un tono (media luna abajo a la derecha)
      cut(blob(S, { fill: P.sombra }),
        [circle(440, 430, 395, { hidden: true })])
      // Vetas oscuras
      const vetas = [
        [[300, 300], [380, 340], [440, 330], [520, 370]],
        [[600, 230], [660, 300], [720, 310]],
        [[230, 560], [300, 600], [330, 680], [400, 720]],
        [[560, 620], [640, 600], [700, 650], [780, 640]],
        [[470, 460], [520, 520], [500, 580]],
      ]
      for (const v of vetas)
        brush(jitter(v, 8), { width: [3, rand(12, 18), 4], color: P.veta })
      // Franja de luz en el borde iluminado
      brush(arc(512, 505, 362, 165, 262, 10), { width: [2, 16, 2], color: shade(P.luz, 0.35), opacity: 0.8 })
    })

    // Cráteres: borde claro arriba-izq, fondo oscuro, sombra interna arriba-izq
    const crateres = [[380, 300, 78], [660, 430, 96], [420, 640, 64], [680, 720, 44], [250, 470, 36]]
    for (const [cx, cy, r] of crateres) {
      // Sombra proyectada del labio sobre la roca (abajo a la derecha)
      circle(cx + r * 0.14, cy + r * 0.14, r * 1.2, { fill: P.sombra, opacity: 0.55 })
      const ext = circle(cx, cy, r * 1.16, { fill: shade(P.base, 0.2) })
      clip(ext, () => circle(cx + r * 0.22, cy + r * 0.22, r * 1.08, { fill: P.base }))
      const fondo = circle(cx, cy, r, { fill: P.crater })
      clip(fondo, () => {
        circle(cx + r * 0.32, cy + r * 0.32, r * 0.96, { fill: P.veta })
        // Luz en la pared interior opuesta
        brush(arc(cx, cy, r * 0.88, 95, 175, 8), { width: [1, r * 0.14, 1], color: shade(P.base, 0.1) })
      })
      ink(fondo, { width: Math.max(5, r * 0.07), weight: 2, color: P.tinta })
    }

    // Brillos pequeños
    brush([[250, 330], [300, 255], [370, 205]], { width: [6, 22, 6], color: P.brillo })
    circle(232, 385, 13, { fill: P.brillo })
    ellipse(345, 275, 18, 10, { fill: P.brillo, rotate: -30, opacity: 0.9 })

    // Contorno de tinta con grosor variable
    ink(roca, { width: 14, weight: 2.4, color: P.tinta })
  })
})
