// dibujo.js: nave con módulo Rastreador (vista cenital, mirando hacia arriba)
canvas({ width: 1024, height: 1024, background: null })

const P = {
  ink: "#1c2340",      // contorno
  white: "#f3f6fb",    // casco
  lav: "#a8b4d6",      // sombra del casco
  teal: "#1bb5a7",     // alas, franjas
  tealD: "#0f7480",    // banda oscura del ala
  tealL: "#8ff0de",    // brillos del ala
  cyan: "#5ee4fb",     // cabina
  blue: "#2c8fe0",     // sombra de la cabina
  orange: "#f57c1f",   // tomas de aire, llamas
  yellow: "#ffc21a",   // núcleo, llamas
  noz: "#2e3150",      // toberas
  vio: "#8a3ffc",      // acento violeta
  vioD: "#5a22b0",
  vioL: "#c79bff",
  glow: "#b56cff",
}
const SW = 12 // grosor del contorno
const BW = 58 // grosor del stroke blanco de pegatina
const M = (pts) => pts.map((p) => (p.length > 2 ? [1024 - p[0], p[1], p[2]] : [1024 - p[0], p[1]]))
const S = (fill, extra = {}) => ({ fill, stroke: P.ink, width: SW, join: "round", ...extra })
const W = { fill: "#ffffff", stroke: "#ffffff", width: BW, join: "round" }

// ---------- Siluetas ----------
const FUS = [[512, 50, 0], [548, 130], [580, 240], [604, 380], [616, 540], [616, 700], [606, 790], [572, 836], [512, 850], [452, 836], [418, 790], [408, 700], [408, 540], [420, 380], [444, 240], [476, 130]]
const WING = [[470, 337, 0], [250, 490, 0], [200, 525, 0], [200, 728, 0], [470, 728, 0]]
const GON = [[197, 436], [246, 450], [262, 520], [262, 690], [250, 815, 0.4], [146, 818, 0.4], [134, 690], [132, 520], [148, 450]]
// contenedor de cohetes (izquierdo): x, y, ancho, alto; cohetes: centros, media anchura, base de la punta
const PX = 300, PY = 508, PW = 88, PH = 226
const ROCKX = [322, 344, 366]
const RH = 10, RY = 446

function nozzles(st) {
  rect(404, 830, 64, 82, { r: 10, ...st })
  rect(556, 830, 64, 82, { r: 10, ...st })
  rect(461, 838, 102, 88, { r: 12, ...st })
}
function podSil(side, st) {
  // side 0 = izquierda, 1 = derecha (reflejada respecto a x = 512)
  const X = (x) => (side ? 1024 - x : x)
  for (const x of ROCKX) blob([[X(x), RY - 30, 0], [X(x + RH), RY - 4], [X(x + RH), PY + 10, 0], [X(x - RH), PY + 10, 0], [X(x - RH), RY - 4]], st)
  rect(side ? 1024 - PX - PW : PX, PY, PW, PH, { r: 28, ...st })
}

rig({ origin: [512, 540] })
bone("fuselaje", { at: [512, 540] })
bone("ala_izq", { parent: "fuselaje", at: [440, 430] })
bone("ala_der", { parent: "fuselaje", at: [584, 430] })
bone("gondola_izq", { parent: "ala_izq", at: [235, 600] })
bone("gondola_der", { parent: "ala_der", at: [789, 600] })
bone("contenedor_izq", { parent: "ala_izq", at: [345, 600] })
bone("contenedor_der", { parent: "ala_der", at: [679, 600] })
bone("radar", { parent: "fuselaje", at: [512, 449] })
bone("llama_izq", { parent: "fuselaje", at: [436, 905] })
bone("llama_centro", { parent: "fuselaje", at: [512, 915] })
bone("llama_der", { parent: "fuselaje", at: [588, 905] })

// ---------- Pasada 1: borde blanco de pegatina (cada uno con su hueso) ----------
layer("borde", () => {
  part("ala_izq", () => poly(WING, W))
  part("ala_der", () => poly(M(WING), W))
  part("contenedor_izq", () => podSil(0, W))
  part("contenedor_der", () => podSil(1, W))
  part("gondola_izq", () => blob(GON, W))
  part("gondola_der", () => blob(M(GON), W))
  part("fuselaje", () => { nozzles(W); blob(FUS, W) })
  part("radar", () => circle(512, 449, 40, W))
})

// ---------- Pasada 2: piezas de color, de atrás hacia delante ----------
// Llama: columna larga escondida bajo la tobera; al bajar el hueso se ve más larga
function flame(cx, w, tip) {
  ellipse(cx, tip - 40, w * 1.1, 70, { fill: alpha(P.orange, 0.35), blur: 18 })
  blob([[cx - w, 820], [cx + w, 820], [cx + w, 915], [cx + w * 0.55, tip - 30], [cx, tip, 0], [cx - w * 0.55, tip - 30], [cx - w, 915]], { fill: P.orange })
  blob([[cx - w * 0.5, 820], [cx + w * 0.5, 820], [cx + w * 0.5, 915], [cx, tip - 28, 0], [cx - w * 0.5, 915]], { fill: P.yellow })
  ellipse(cx, 905, w * 0.25, 30, { fill: "#fff6cf", opacity: 0.9 })
}
layer("llamas", () => {
  part("llama_izq", () => flame(436, 22, 962))
  part("llama_der", () => flame(588, 22, 962))
  part("llama_centro", () => flame(512, 30, 990))
})

function wing(side) {
  const pts = side ? M(WING) : WING
  const X = (x) => (side ? 1024 - x : x)
  const w = poly(pts, { fill: P.teal })
  clip(w, () => {
    poly([[X(180), 642], [X(480), 628], [X(480), 740], [X(180), 740]], { fill: P.tealD })
    brush([[X(405), 402], [X(330), 456], [X(262), 520]], { width: 14, color: P.tealL, taper: [0.15, 0.6] })
    brush([[X(378), 466], [X(300), 520]], { width: 7, color: alpha(P.tealL, 0.8), taper: [0.3, 0.5] })
  })
  poly(pts, { fill: "none", stroke: P.ink, width: SW, join: "round" })
}
layer("alas", () => {
  part("ala_izq", () => wing(0))
  part("ala_der", () => wing(1))
})
function pod(side) {
  const X = (x) => (side ? 1024 - x : x)
  const x0 = side ? 1024 - PX - PW : PX
  const cx = x0 + PW / 2
  const yB = PY + PH
  // pestillo violeta hacia la góndola (encaja en el ala)
  rect(side ? x0 + PW - 14 : x0 - 26, 600, 40, 28, S(P.vioD, { r: 8, width: 9 }))
  // cohetes: cuerpo blanco y punta morada
  for (const x of ROCKX) {
    const rx = X(x)
    const body = rect(rx - RH, RY - 6, RH * 2, PY - RY + 30, S(P.white, { width: 8 }))
    clip(body, () => rect(rx + 2, RY - 10, RH, 100, { fill: P.lav }))
    const tip = blob([[rx, RY - 30, 0], [rx + RH, RY - 6, 0.6], [rx + RH, RY + 2, 0], [rx - RH, RY + 2, 0], [rx - RH, RY - 6, 0.6]], S(P.vio, { width: 8 }))
    clip(tip, () => blob([[rx - 1, RY - 26, 0], [rx - 8, RY - 2], [rx - 3, RY - 2]], { fill: P.vioL }))
    rect(rx - RH - 1, RY + 14, RH * 2 + 2, 6, { fill: P.vio })
  }
  // cuerpo del contenedor
  const b = rect(x0, PY, PW, PH, { r: 28, fill: P.white })
  clip(b, () => {
    rect(cx + 18, PY - 10, 60, PH + 20, { fill: P.lav })
    rect(x0 - 10, PY, PW + 20, 30, { fill: P.noz })
    for (const x of ROCKX) circle(X(x), PY + 15, 9, { fill: "#141830" })
    rect(x0 - 10, yB - 34, PW + 20, 40, { fill: P.vioD })
    rect(x0 - 10, yB - 40, PW + 20, 8, { fill: P.vio })
    brush([[x0 + 12, PY + 50], [x0 + 10, PY + 120], [x0 + 12, PY + 170]], { width: 6, color: "#ffffff", taper: [0.3, 0.3] })
  })
  rect(x0, PY, PW, PH, { r: 28, fill: "none", stroke: P.ink, width: SW })
  // panel violeta con rejilla
  const pn = rect(x0 + 18, PY + 50, PW - 36, 106, S(P.vio, { r: 12, width: 7 }))
  clip(pn, () => {
    rect(cx + 6, PY + 40, 40, 130, { fill: P.vioD })
    rect(x0 + 25, PY + 58, 7, 40, { r: 3.5, fill: P.vioL })
  })
  for (let k = 0; k < 3; k++) line(x0 + 30, PY + 110 + k * 13, x0 + PW - 30, PY + 110 + k * 13, { stroke: P.ink, width: 4.5, cap: "round" })
  // remaches
  for (const y of [PY + 40, yB - 52]) for (const x of [x0 + 14, x0 + PW - 14]) circle(x, y, 3.5, { fill: P.ink })
}
layer("contenedores", () => {
  part("contenedor_izq", () => pod(0))
  part("contenedor_der", () => pod(1))
})
function gondola(side) {
  const X = (x) => (side ? 1024 - x : x)
  const off = side ? 1024 - 394 : 0 // las sombras van a la derecha en las dos
  const pts = side ? M(GON) : GON
  const g = blob(pts, { fill: P.white })
  const cx = side ? 827 : 197
  clip(g, () => {
    // sombra a la derecha (luz arriba a la izquierda)
    blob([[cx + 18, 440, 0], [cx + 80, 440, 0], [cx + 80, 830, 0], [cx - 70, 830, 0], [cx - 66, 748], [cx + 4, 744], [cx + 22, 700], [cx + 22, 560]], { fill: P.lav })
    rect(cx - 70, 690, 140, 26, { fill: P.teal })
    rect(cx + 20, 690, 60, 26, { fill: shade(P.teal, -0.25) })
    brush([[cx - 46, 520], [cx - 50, 620], [cx - 46, 700]], { width: 7, color: "#ffffff", taper: [0.3, 0.3] })
  })
  blob(pts, { fill: "none", stroke: P.ink, width: SW, join: "round" })
  // toma de aire
  ellipse(cx - 1, 486, 37, 27, { fill: P.ink })
  const t = ellipse(cx - 1, 488, 27, 18, { fill: P.orange })
  clip(t, () => {
    ellipse(cx - 1, 500, 30, 12, { fill: "#9a3a12" })
    ellipse(cx + 3, 478, 16, 5, { fill: "#ffd08a" })
  })
}
layer("gondolas", () => {
  part("gondola_izq", () => gondola(0))
  part("gondola_der", () => gondola(1))
})
layer("fuselaje", () => {
  part("fuselaje", () => {
    // toberas
    nozzles(S(P.noz))
    rect(414, 842, 12, 60, { r: 5, fill: "#4a4f80" })
    rect(566, 842, 12, 60, { r: 5, fill: "#4a4f80" })
    rect(474, 852, 14, 64, { r: 6, fill: "#4a4f80" })
    // casco
    const f = blob(FUS, { fill: P.white })
    clip(f, () => {
      poly([[556, 90], [640, 90], [640, 860], [580, 860], [592, 760], [596, 560], [586, 380], [566, 240]], { fill: P.lav })
      blob([[380, 790, 0], [470, 776], [560, 770], [650, 750, 0], [650, 900, 0], [380, 900, 0]], { fill: P.lav })
      // franja verde del morro
      poly([[420, 140], [512, 106], [604, 140], [604, 214], [512, 184], [420, 214]], { fill: P.teal })
      poly([[543, 110], [604, 130], [604, 214], [543, 194]], { fill: shade(P.teal, -0.2) })
      brush([[428, 420], [420, 560], [426, 700], [440, 760]], { width: 8, color: "#ffffff", taper: [0.3, 0.4] })
      // juntas de panel
      curve([[430, 236], [512, 226], [594, 236]], { stroke: alpha(P.ink, 0.25), width: 4 })
    })
    blob(FUS, { fill: "none", stroke: P.ink, width: SW + 1, join: "round" })
    // cabina
    const cp = ellipse(512, 312, 47, 97, S(P.cyan, { width: 11 }))
    clip(cp, () => {
      ellipse(560, 320, 30, 110, { fill: P.blue })
      brush([[492, 262], [484, 300], [486, 318]], { width: 12, color: "#ffffff", taper: [0.2, 0.4] })
      circle(498, 346, 6, { fill: "#ffffff" })
    })
    // núcleo amarillo
    circle(512, 540, 54, { fill: P.ink })
    circle(512, 540, 41, { fill: radial(495, 520, 55, ["#ffd84a", P.yellow, P.orange]), stroke: "#fff1b0", width: 4 })
    circle(498, 526, 10, { fill: "#ffffff" })
    circle(527, 554, 4, { fill: "#ffffff" })
  })
})
layer("radar", () => {
  part("radar", () => {
    circle(512, 449, 37, S(P.vioD, { width: 9 }))
    for (let a = 0; a < 360; a += 45) circle(...polar(512, 449, 30, a), 2.6, { fill: P.vioL })
    const d = circle(512, 449, 24, S(radial(503, 440, 30, ["#ffffff", P.lav]), { width: 6 }))
    clip(d, () => ellipse(528, 460, 20, 17, { fill: alpha(P.lav, 0.9) }))
    // ojo violeta brillante, mirando hacia delante
    circle(512, 437, 20, { fill: alpha(P.glow, 0.45), blur: 7 })
    ellipse(512, 438, 15, 12, { fill: P.ink })
    ellipse(512, 438, 11.5, 8.5, { fill: radial(509, 435, 12, ["#f6eaff", P.glow, P.vio]), glow: { blur: 14, color: P.glow } })
    circle(508, 435.5, 2.8, { fill: "#ffffff" })
  })
})

// ---------- Acciones ----------
// Alas: negativo en la izquierda y positivo en la derecha = plegar hacia atrás.
// Llamas: bajar el hueso (y +) saca más columna de debajo de la tobera.
action("plegar", { duration: 1, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, gondola_izq: 0, gondola_der: 0 }],
  [0.5, { ala_izq: -10, ala_der: 10, gondola_izq: 4, gondola_der: -4 }],
  [1, { ala_izq: -22, ala_der: 22, gondola_izq: 10, gondola_der: -10 }],
])

action("motor", { duration: 0.4, loop: true }, [
  [0, { llama_izq: { y: 0 }, llama_centro: { y: 22 }, llama_der: { y: 11 }, radar: -9 }],
  [0.25, { llama_izq: { y: 11 }, llama_centro: { y: 11 }, llama_der: { y: 22 } }],
  [0.5, { llama_izq: { y: 22 }, llama_centro: { y: 0 }, llama_der: { y: 11 }, radar: 9 }],
  [0.75, { llama_izq: { y: 11 }, llama_centro: { y: 11 }, llama_der: { y: 0 } }],
  [1, { llama_izq: { y: 0 }, llama_centro: { y: 22 }, llama_der: { y: 11 }, radar: -9 }],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, llama_izq: { y: 0 }, llama_centro: { y: 0 }, llama_der: { y: 0 } }],
  [0.4, { ala_izq: -13, ala_der: 13, llama_izq: { y: 40 }, llama_centro: { y: 52 }, llama_der: { y: 40 } }],
  [1, { ala_izq: -9, ala_der: 9, llama_izq: { y: 36 }, llama_centro: { y: 48 }, llama_der: { y: 36 } }],
])

// Acoplar: contenedor izquierdo, luego el derecho y por último el radar, cada uno con rebote
const LEJOS_I = { x: -55, y: 270, rot: -14 }
const LEJOS_D = { x: 55, y: 270, rot: 14 }
const RADAR_ARRIBA = { x: 0, y: 150, rot: 0 }
const MONTADO = { x: 0, y: 0, rot: 0 }
action("acoplar", { duration: 0.7, loop: false }, [
  [0, { contenedor_izq: LEJOS_I, contenedor_der: LEJOS_D, radar: RADAR_ARRIBA }],
  [0.28, { contenedor_izq: { x: 5, y: -10, rot: 3 }, contenedor_der: LEJOS_D }],
  [0.38, { contenedor_izq: MONTADO }],
  [0.55, { contenedor_der: { x: -5, y: -10, rot: -3 }, radar: RADAR_ARRIBA }],
  [0.65, { contenedor_der: MONTADO }],
  [0.88, { radar: { x: 0, y: -10, rot: 0 } }],
  [1, { contenedor_izq: MONTADO, contenedor_der: MONTADO, radar: MONTADO }],
])
