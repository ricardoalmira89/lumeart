// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  ink: "#1b2240",      // contorno
  white: "#f4f7fb",    // fuselaje y góndolas
  whiteSh: "#a9b6d6",  // sombra del blanco
  teal: "#16b3a6",     // alas y galón
  tealSh: "#0b6f7e",
  cyan: "#5fe8ff",     // cabina
  cyanSh: "#1f86c9",
  orange: "#ff7a1a",   // tomas y llamas
  yellow: "#ffd83a",   // núcleo y llamas
  dark: "#2a3150",     // toberas
}
const CX = 512
setLight(-1, -1)

// Construye una forma simétrica a partir de la mitad derecha (de arriba abajo)
function sym(half) {
  const left = half.slice().reverse().filter(p => p[0] !== CX).map(p => [2 * CX - p[0], p[1], p[2]].filter(v => v !== undefined))
  return half.concat(left)
}

// Siluetas
const FUS = sym([[CX, 62, 0], [556, 170], [592, 320], [616, 500], [624, 680], [614, 800], [592, 846, 0.5], [CX, 852]])
const WING_R = [[596, 380, 0], [800, 520, 0], [842, 560, 0], [842, 720, 0], [596, 720, 0]]
const WING_L = WING_R.map(p => [2 * CX - p[0], p[1], 0])
const POD = (s) => {
  const x = CX + s * 318
  return [[x, 440], [x + 52, 480], [x + 60, 600], [x + 56, 760], [x + 40, 812, 0.5], [x - 40, 812, 0.5], [x - 56, 760], [x - 60, 600], [x - 52, 480]]
}
const NOZ = [[CX - 68, 870, 30], [CX, 880, 38], [CX + 68, 870, 30]] // x, y, medio ancho

layer("pegatina", () => {
  // Borde blanco grueso alrededor de toda la figura
  const st = { fill: "#ffffff", stroke: "#ffffff", width: 56, join: "round" }
  blob(FUS, st); blob(WING_R, st); blob(WING_L, st); blob(POD(1), st); blob(POD(-1), st)
  for (const [x, y, w] of NOZ) blob([[x - w, y - 30, 0], [x + w, y - 30, 0], [x + w * 1.15, y + 34, 0.5], [x - w * 1.15, y + 34, 0.5]], st)
  blob([[CX - 100, 900], [CX, 990], [CX + 100, 900]], st)
})

layer("llamas", () => {
  // Llamas cortas bajo las toberas
  for (const [x, y, w] of NOZ) {
    const len = w > 30 ? 78 : 58
    const top = y + 30
    blob([[x - w * 1.05, top], [x, top + len, 0], [x + w * 1.05, top]], { fill: P.orange, glow: { blur: 18, color: alpha(P.orange, 0.7) } })
    blob([[x - w * 0.6, top], [x, top + len * 0.62, 0], [x + w * 0.6, top]], { fill: P.yellow })
    blob([[x - w * 0.25, top], [x, top + len * 0.3, 0], [x + w * 0.25, top]], { fill: "#fffbe0" })
  }
})

layer("toberas", () => {
  // Tres toberas oscuras
  for (const [x, y, w] of NOZ) {
    const n = blob([[x - w, y - 30, 0], [x + w, y - 30, 0], [x + w * 1.15, y + 34, 0.5], [x - w * 1.15, y + 34, 0.5]], { fill: P.dark })
    clip(n, () => {
      rect(x + w * 0.2, y - 40, w * 1.2, 90, { fill: shade(P.dark, -0.4) })
      rect(x - w * 0.8, y - 30, w * 0.35, 64, { fill: "#4a5680" })
    })
    ellipse(x, y + 32, w * 0.85, 9, { fill: "#11152a" })
    ink(n, { width: 7, weight: 1.8, color: P.ink })
  }
})

layer("alas", () => {
  // Alas en flecha azul verdoso
  for (const pts of [WING_L, WING_R]) {
    const w = blob(pts, { fill: P.teal })
    clip(w, () => {
      // sombra de un tono en la mitad trasera
      poly([[0, 650], [1024, 610], [1024, 760], [0, 760]], { fill: P.tealSh })
      // brillo en el borde de ataque
      const s = pts === WING_R ? 1 : -1
      brush([[CX + s * 100, 418], [CX + s * 250, 520]], { width: 18, color: "#8ff2e2", taper: [0.15, 0.4] })
    })
    ink(w, { width: 9, weight: 1.8, color: P.ink })
  }
})

layer("gondolas", () => {
  // Góndolas blancas con tomas naranjas
  for (const s of [-1, 1]) {
    const x = CX + s * 318
    const g = blob(POD(s), { fill: P.white })
    clip(g, () => {
      blob([[x + 14, 430], [x + 100, 430], [x + 100, 840], [x - 80, 840], [x - 70, 770], [x + 10, 740], [x + 20, 600]], { fill: P.whiteSh })
      brush([[x - 34, 520], [x - 38, 640], [x - 34, 720]], { width: 14, color: "#ffffff", taper: [0.3, 0.3] })
      // banda azul verdoso
      rect(x - 70, 690, 140, 26, { fill: P.teal })
    })
    ink(g, { width: 9, weight: 1.8, color: P.ink })
    // toma de aire naranja delante
    const t = ellipse(x, 486, 34, 26, { fill: P.orange })
    clip(t, () => { ellipse(x, 500, 26, 16, { fill: "#a33a0c" }) })
    ink(t, { width: 6, weight: 1.6, color: P.ink })
  }
})

layer("fuselaje", () => {
  // Fuselaje blanco, galón, cabina y núcleo
  const f = blob(FUS, { fill: P.white })
  clip(f, () => {
    // sombra de un tono a la derecha
    blob([[CX + 30, 40], [700, 300], [700, 900], [380, 900], [400, 800], [CX + 50, 760], [CX + 66, 600], [CX + 52, 330]], { fill: P.whiteSh })
    // galón azul verdoso en el morro
    poly([[CX, 110], [700, 250], [700, 320], [CX, 184], [324, 320], [324, 250]], { fill: P.teal })
    clip(poly([[CX, 110], [700, 250], [700, 320], [CX, 184], [324, 320], [324, 250]], { hidden: true }), () => {
      rect(CX + 30, 100, 200, 240, { fill: P.tealSh })
    })
    // brillo lateral
    brush([[470, 250], [452, 420], [450, 600], [458, 720]], { width: 18, color: "#ffffff", taper: [0.3, 0.3] })
  })
  ink(f, { width: 10, weight: 1.8, color: P.ink })

  // cabina de cristal
  const c = blob([[CX, 218], [550, 270], [554, 350], [538, 400], [486, 400], [470, 350], [474, 270]], { fill: P.cyan })
  clip(c, () => {
    blob([[CX + 8, 210], [580, 300], [580, 420], [CX + 14, 420], [CX + 22, 320]], { fill: P.cyanSh })
    brush([[494, 262], [484, 320]], { width: 14, color: "#ffffff", taper: [0.2, 0.3] })
    circle(498, 346, 6, { fill: "#ffffff" })
  })
  ink(c, { width: 8, weight: 1.6, color: P.ink })

  // núcleo amarillo brillante (punto vulnerable)
  const ringC = circle(CX, 540, 52, { fill: P.tealSh })
  clip(ringC, () => { circle(CX + 14, 554, 52, { fill: shade(P.tealSh, -0.4) }) })
  ink(ringC, { width: 8, weight: 1.6, color: P.ink })
  circle(CX, 540, 41, { fill: "#fff3a0" })
  const core = circle(CX, 540, 36, { fill: radial(CX - 10, 528, 44, ["#fffbe0", P.yellow, "#ffa01a"]) })
  circle(CX - 13, 527, 10, { fill: "#ffffff" })
  circle(CX + 14, 556, 4, { fill: "#ffffff", opacity: 0.8 })
})
