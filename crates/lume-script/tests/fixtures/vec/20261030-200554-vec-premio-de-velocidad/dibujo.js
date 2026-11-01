// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)
setLight(-1, -1)

const P = {
  tinta: "#141b3a",     // contorno oscuro
  metal: "#f4f6fb",     // metal blanco
  metalM: "#c9d1e4",    // medio
  metalS: "#8d97b8",    // sombra fría
  tobera: "#3b4468",    // boca de la tobera
  azul: "#1f6bff",      // fuego azul
  azulC: "#7fe9ff",     // azul claro de los galones
  azulS: "#2aa6e8",     // sombra de los galones
  blanco: "#ffffff",
}

// Geometría
const C = [512, 470], R = 318
const nozzle = [[372, 730], [652, 730], [690, 862, 0.3], [334, 862, 0.3]]
const flame = [[400, 850], [624, 850], [584, 915], [512, 966, 0], [440, 915]]
function chevron(ty) {
  const hw = 246, h = 200, t = 128
  return [[512, ty, 0], [512 + hw, ty + h, 0], [512 + hw - 30, ty + h + t, 0], [512, ty + t + 40, 0],
          [512 - hw + 30, ty + h + t, 0], [512 - hw, ty + h, 0]]
}
const ch1 = chevron(150), ch2 = chevron(380)

// Silueta: dibuja todas las piezas con un estilo dado
function silueta(st) {
  blob(flame, st)
  blob(nozzle, st)
  circle(C[0], C[1], R, st)
  blob(ch1, st)
  blob(ch2, st)
}

layer("borde de pegatina", () => {
  silueta({ fill: P.blanco, stroke: P.blanco, width: 84, join: "round" })
  silueta({ fill: P.tinta, stroke: P.tinta, width: 34, join: "round" })
})

layer("llama", () => {
  const f = blob(flame, { fill: P.azul })
  clip(f, () => {
    blob([[440, 850], [584, 850], [552, 900], [512, 934, 0], [472, 900]], { fill: "#5fb8ff" })
    blob([[472, 850], [552, 850], [532, 884], [512, 906, 0], [492, 884]], { fill: P.blanco })
  })
  ink(f, { width: 12, weight: 1.8, color: P.tinta })
})

layer("tobera", () => {
  const n = blob(nozzle, { fill: P.metalM })
  clip(n, () => {
    poly([[560, 700], [700, 700], [720, 880], [600, 880]], { fill: P.metalS })
    poly([[380, 700], [440, 700], [420, 880], [350, 880]], { fill: P.metal })
    // boca encendida
    ellipse(512, 818, 150, 30, { fill: P.tobera })
    ellipse(512, 824, 122, 19, { fill: P.azul })
    ellipse(512, 827, 74, 9, { fill: "#bff3ff" })
  })
  ink(n, { width: 14, weight: 1.8, color: P.tinta })
})

layer("cuerpo", () => {
  const body = circle(C[0], C[1], R, { fill: P.metal })
  clip(body, () => {
    // sombra de un tono abajo a la derecha
    circle(C[0], C[1], R, { fill: P.metalM })
    circle(C[0] - 46, C[1] - 52, R - 10, { fill: P.metal })
    // anillo interior (placa central)
    circle(C[0], C[1], R - 62, { fill: "none", stroke: P.metalS, width: 14, opacity: 0.8 })
    circle(C[0] - 4, C[1] - 4, R - 62, { fill: "none", stroke: P.blanco, width: 6, opacity: 0.9 })
    // remaches
    for (let i = 0; i < 12; i++) {
      const [x, y] = polar(C[0], C[1], R - 31, i * 30 + 15)
      circle(x, y, 13, { fill: P.metalS })
      circle(x - 3, y - 3, 7, { fill: P.blanco })
    }
  })
  ink(body, { width: 16, weight: 1.8, color: P.tinta })
})

layer("galones", () => {
  for (const pts of [ch1, ch2]) {
    // sombra proyectada sobre el metal
    blob(shift(pts, 14, 18), { fill: P.tinta, opacity: 0.35 })
    const g = blob(pts, { fill: P.azulS })
    clip(g, () => {
      // luz: todo menos una franja de sombra abajo
      blob(shift(pts, -6, -38), { fill: P.azulC })
      // brazo derecho, algo más apartado de la luz
      poly([[512, pts[0][1] - 20], [800, pts[0][1] + 180], [800, pts[0][1] + 400], [512, pts[0][1] + 400]], { fill: "#3fb8f0", opacity: 0.35 })
    })
    ink(g, { width: 20, weight: 1.5, color: P.tinta })
  }
})

layer("brillos", () => {
  // brillo del metal
  brush(arc(C[0], C[1], R - 22, 150, 205, 10), { width: 20, color: P.blanco, taper: [0.3, 0.3] })
  circle(polar(C[0], C[1], R - 22, 216)[0], polar(C[0], C[1], R - 22, 216)[1], 9, { fill: P.blanco })
  // brillos de los galones (borde superior izquierdo)
  for (const pts of [ch1, ch2]) {
    const ty = pts[0][1]
    brush([[300, ty + 196], [400, ty + 106], [496, ty + 26]], { width: 20, color: P.blanco, taper: [0.25, 0.35] })
  }
})
