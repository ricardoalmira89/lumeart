// dibujo.js: escolta frontal vista desde arriba, mirando hacia arriba, con esqueleto.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  tinta: "#161a35",
  blanco: "#ffffff",
  azul: "#2058f8",
  azulOsc: "#1534a8",
  azulLuz: "#6c98ff",
  gris: "#4b5265",
  grisOsc: "#2c303e",
  grisLuz: "#7d8598",
  banda: "#5e6678",
  ojoBlanco: "#f2f6fb",
  ojoSombra: "#b9c4d8",
  anillo: "#2b3242",
  cian: "#3ef0ff",
  cianOsc: "#1198c0",
  cianLuz: "#d4fbff",
  lente: "#a3d9ff",
}
const T = 18      // grosor del contorno oscuro
const B = 64      // ancho del trazo blanco de pegatina (por detrás)
const C = [512, 585] // centro del ojo

rig({ origin: C })
bone("cuerpo", { at: C })
bone("canon_izq", { parent: "cuerpo", at: [420, 330] })
bone("canon_der", { parent: "cuerpo", at: [604, 330] })
bone("aleta_izq", { parent: "cuerpo", at: [255, 610] })
bone("aleta_der", { parent: "cuerpo", at: [769, 610] })
bone("ojo", { parent: "cuerpo", at: C })

// --- Siluetas -------------------------------------------------------------
const canonX = { izq: 345, der: 529 }
const aletaPts = (s) => {
  // s = 1 lado izquierdo, -1 lado derecho (reflejo sobre x = 512)
  const X = (x) => (s === 1 ? x : 1024 - x)
  return [[X(100), 590, 0], [X(170), 535], [X(238), 497, 0.5], [X(300), 500], [X(300), 712], [X(240), 714], [X(116), 710, 0.5], [X(106), 698, 0]]
}
const propPts = [[422, 830], [602, 830], [584, 932], [440, 932]]

// --- Borde blanco de pegatina (primero todo, por detrás de cada pieza) ----
part("canon_izq", () => rect(canonX.izq, 50, 150, 440, { r: 44, fill: P.blanco, stroke: P.blanco, width: B, join: "round" }))
part("canon_der", () => rect(canonX.der, 50, 150, 440, { r: 44, fill: P.blanco, stroke: P.blanco, width: B, join: "round" }))
part("aleta_izq", () => blob(aletaPts(1), { fill: P.blanco, stroke: P.blanco, width: B, join: "round" }))
part("aleta_der", () => blob(aletaPts(-1), { fill: P.blanco, stroke: P.blanco, width: B, join: "round" }))
part("cuerpo", () => {
  poly(propPts, { fill: P.blanco, stroke: P.blanco, width: B, join: "round" })
  circle(C[0], C[1], 300, { fill: P.blanco, stroke: P.blanco, width: B - T })
})

// --- Cañones ----------------------------------------------------------------
function canon(lado) {
  const x0 = canonX[lado]
  part("canon_" + lado, () => {
    rect(x0, 50, 150, 440, { r: 44, fill: P.tinta })
    const cuerpo = rect(x0 + 18, 68, 114, 440, { r: 28, fill: P.gris })
    clip(cuerpo, () => {
      rect(x0 + 88, 60, 60, 440, { fill: P.grisOsc })
      // franja de luz
      rect(x0 + 35, 140, 20, 360, { r: 10, fill: P.grisLuz, opacity: 0.85 })
      // abrazadera
      rect(x0, 220, 150, 11, { fill: P.banda })
      rect(x0, 231, 150, 22, { fill: P.grisOsc })
    })
    // boca con luz cian
    rect(x0 + 27, 76, 96, 58, { r: 20, fill: P.tinta })
    rect(x0 + 39, 88, 73, 34, { r: 9, fill: P.cian, glow: { blur: 6, color: alpha(P.cian, 0.5) } })
    rect(x0 + 45, 91, 26, 10, { r: 5, fill: P.blanco })
  })
}
canon("izq")
canon("der")

// --- Aletas -----------------------------------------------------------------
function aleta(lado) {
  const s = lado === "izq" ? 1 : -1
  const X = (x) => (s === 1 ? x : 1024 - x)
  part("aleta_" + lado, () => {
    const f = blob(aletaPts(s), { fill: P.azul, stroke: P.tinta, width: T, join: "round" })
    clip(f, () => {
      rect(0, 645, 1024, 90, { fill: P.azulOsc })
      // brillo de la punta
      blob([[X(132), 576, 0], [X(162), 556, 0.5], [X(188), 555, 0], [X(178), 576], [X(160), 590], [X(138), 592, 0.5]], { fill: P.azulLuz })
    })
    blob(aletaPts(s), { fill: "none", stroke: P.tinta, width: T, join: "round" })
  })
}
aleta("izq")
aleta("der")

// --- Cuerpo: propulsor, cápsula y blanco del ojo -----------------------------
part("cuerpo", () => {
  // propulsor trasero
  const pr = poly(propPts, { fill: P.gris, stroke: P.tinta, width: T, join: "round" })
  clip(pr, () => rect(512, 800, 120, 150, { fill: P.grisOsc }))
  poly(propPts, { fill: "none", stroke: P.tinta, width: T, join: "round" })
  ellipse(512, 918, 62, 22, { fill: P.lente, stroke: P.tinta, width: 12 })
  ellipse(500, 915, 28, 7, { fill: P.blanco })

  // cápsula
  circle(C[0], C[1], 300, { fill: P.tinta })
  const caps = circle(C[0], C[1], 281, { fill: P.azulOsc })
  clip(caps, () => {
    circle(495, 567, 263, { fill: P.azul })
    // brillo en media luna arriba a la izquierda
    blob([[268, 545, 0], [280, 470], [325, 400], [395, 355], [470, 340, 0], [405, 375], [345, 425], [300, 485]], { fill: P.azulLuz, opacity: 0.9 })
  })

  // blanco del ojo
  circle(C[0], C[1], 207, { fill: P.tinta })
  const blanco = circle(C[0], C[1], 197, { fill: P.ojoSombra })
  clip(blanco, () => {
    circle(490, 560, 188, { fill: P.ojoBlanco })
    blob([[385, 492], [420, 455], [470, 437, 0.5], [472, 448], [440, 470], [400, 505], [385, 508]], { fill: P.blanco })
    circle(367, 559, 14, { fill: P.blanco })
  })
})

// --- Ojo cian (iris que late) ----------------------------------------------
part("ojo", () => {
  circle(C[0], C[1], 114, { fill: P.tinta })
  circle(C[0], C[1], 96, { fill: P.anillo })
  const iris = circle(C[0], C[1], 80, { fill: P.cianOsc })
  clip(iris, () => {
    circle(498, 572, 80, { fill: P.cian })
  })
  circle(C[0], C[1], 36, { fill: P.cianLuz })
  ellipse(478, 548, 22, 13, { fill: P.blanco, rotate: -32 })
  circle(546, 617, 8, { fill: P.blanco })
})

// --- Acciones ----------------------------------------------------------------
action("flotar", { duration: 1.2, loop: true }, [
  [0, { aleta_izq: 0, aleta_der: 0, ojo: { rot: 0, x: 0, y: 0 } }],
  [0.25, { aleta_izq: 6, aleta_der: -6, ojo: { rot: 0, x: 0, y: -3 } }],
  [0.5, { aleta_izq: 0, aleta_der: 0, ojo: { rot: 0, x: 0, y: 0 } }],
  [0.75, { aleta_izq: -6, aleta_der: 6, ojo: { rot: 0, x: 0, y: 3 } }],
  [1, { aleta_izq: 0, aleta_der: 0, ojo: { rot: 0, x: 0, y: 0 } }],
])

action("disparo", { duration: 0.15, loop: false }, [
  [0, { canon_izq: { rot: 0, y: 0 }, canon_der: { rot: 0, y: 0 } }],
  [0.2, { canon_izq: { rot: 0, y: 42 }, canon_der: { rot: 0, y: 42 } }],
  [1, { canon_izq: { rot: 0, y: 0 }, canon_der: { rot: 0, y: 0 } }],
])

const plegadaIzq = (a) => ({ rot: a, x: 28, y: 0 })
const plegadaDer = (a) => ({ rot: -a, x: -28, y: 0 })
action("lanzar", { duration: 0.3, loop: true }, [
  [0, { aleta_izq: plegadaIzq(-38), aleta_der: plegadaDer(-38) }],
  [0.25, { aleta_izq: plegadaIzq(-43), aleta_der: plegadaDer(-41) }],
  [0.5, { aleta_izq: plegadaIzq(-38), aleta_der: plegadaDer(-38) }],
  [0.75, { aleta_izq: plegadaIzq(-41), aleta_der: plegadaDer(-43) }],
  [1, { aleta_izq: plegadaIzq(-38), aleta_der: plegadaDer(-38) }],
])
