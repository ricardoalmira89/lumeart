// dibujo.js: nave con módulo HORQUILLA, vista desde arriba mirando hacia arriba.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#1c2244",      // contorno
  white: "#f3f6fc",    // casco
  lav: "#a9b4d6",      // sombra del casco
  teal: "#1bb3a6",     // alas y franjas
  tealD: "#10717f",    // banda oscura del ala
  tealL: "#8fe9da",    // brillo del ala
  cyan: "#5ce1f8",     // cabina
  blue: "#2a8bd8",     // sombra de cabina
  orange: "#ff7f1a",   // tomas de aire y llamas
  rust: "#9b3a14",
  yellow: "#ffc21a",
  noz: "#2b2d52",      // toberas
  lime: "#a8f22a",     // acento HORQUILLA
  limeL: "#e6ff9a",
}
const OUT = 10        // grosor del contorno
const STK = 62        // borde blanco de pegatina

// Reflejo horizontal respecto al eje de la nave
const mx = (pts) => pts.map(p => [1024 - p[0], p[1], ...p.slice(2)])
const sx = (s, x) => s < 0 ? x : 1024 - x
const side = (s, pts) => s < 0 ? pts : mx(pts)

// ---------- Geometría (lado izquierdo; el derecho se refleja) ----------
const FUS = [[512, 52, 0], [560, 175], [594, 330], [614, 500], [620, 650], [613, 778], [582, 834], [512, 850], [442, 834], [411, 778], [404, 650], [410, 500], [430, 330], [464, 175]]
const WING = [[478, 318, 0], [214, 505, 0], [214, 728, 0], [478, 728, 0]]
const POD = [[196, 434], [246, 448], [262, 520], [259, 700], [250, 812, 0.5], [196, 826], [142, 812, 0.5], [133, 700], [130, 520], [146, 448]]
const FAIR = [[406, 466], [410, 560], [406, 660], [372, 704], [314, 700], [282, 656], [286, 566], [310, 504], [356, 462]]
// cañón: recámara ancha, caña y boca; se inclina hacia fuera (CAN_ROT) sobre su hueso
const CAN = [[333, 320, 0.5], [331, 492], [316, 514, 0.5], [316, 628], [336, 652, 0.5], [376, 652, 0.5], [396, 628], [396, 514, 0.5], [381, 492], [379, 320, 0.5]]
const MUZ = [[322, 292, 0.5], [356, 270], [390, 292, 0.5], [392, 340, 0.5], [356, 350], [320, 340, 0.5]]
const CREST_PTS = [[512, 598, 0], [542, 632, 0], [546, 742, 0.4], [512, 782, 0], [478, 742, 0.4], [482, 632, 0]]
const CAN_ROT = 8
const CREST = [[512, 596, 0], [542, 650], [548, 738], [512, 784, 0.5], [476, 738], [482, 650]]
const NOZ_L = [[406, 826, 0], [466, 826, 0], [466, 908, 0.5], [436, 912], [406, 904, 0.5]]
const NOZ_C = [[462, 840, 0], [562, 840, 0], [560, 916, 0.5], [512, 924], [464, 916, 0.5]]
const FLAME_S = [[440, 840], [462, 880], [458, 918], [440, 962, 0], [422, 918], [418, 880]]
const FLAME_C = [[512, 850], [548, 890], [545, 930], [512, 994, 0], [479, 930], [476, 890]]

// ---------- Esqueleto ----------
rig({ origin: [512, 560] })
bone("fuselaje", { at: [512, 540] })
bone("ala_izq", { parent: "fuselaje", at: [420, 420] })
bone("ala_der", { parent: "fuselaje", at: [604, 420] })
bone("gondola_izq", { parent: "ala_izq", at: [214, 600] })
bone("gondola_der", { parent: "ala_der", at: [810, 600] })
bone("canon_izq", { parent: "ala_izq", at: [356, 600] })
bone("canon_der", { parent: "ala_der", at: [668, 600] })
bone("cresta", { parent: "fuselaje", at: [512, 770] })
bone("llama_izq", { parent: "fuselaje", at: [440, 905] })
bone("llama_centro", { parent: "fuselaje", at: [512, 918] })
bone("llama_der", { parent: "fuselaje", at: [584, 905] })

// ---------- 1. Borde blanco de pegatina (todas las siluetas, debajo de todo) ----------
const W = { fill: "#ffffff", stroke: "#ffffff", width: STK, join: "round" }
part("llama_izq", () => blob(shift(FLAME_S, 0, 0), W))
part("llama_der", () => blob(shift(FLAME_S, 144, 0), W))
part("llama_centro", () => blob(FLAME_C, W))
for (const s of [-1, 1]) {
  const k = s < 0 ? "izq" : "der"
  part("ala_" + k, () => { poly(side(s, WING), W); blob(side(s, FAIR), W) })
  part("gondola_" + k, () => blob(side(s, POD), W))
  part("canon_" + k, () => group({ rotate: s * CAN_ROT, around: [sx(s, 356), 600] }, () => {
    blob(side(s, CAN), W)
    blob(side(s, MUZ), W)
  }))
}
part("fuselaje", () => { blob(FUS, W); blob(NOZ_L, W); blob(mx(NOZ_L), W); blob(NOZ_C, W) })
part("cresta", () => blob(CREST_PTS, W))

// ---------- 2. Llamas ----------
function flame(pts, cx, top) {
  blob(pts, { fill: P.orange, glow: { blur: 22, color: alpha("#ff8a2a", 0.8) } })
  blob(grow(shift(pts, 0, -8), 0.55).map((p, i) => [cx + (p[0] - cx) * 0.9, p[1] - 4, ...p.slice(2)]), { fill: "#ffd23a" })
  brush([[cx - 4, top + 50], [cx - 2, top + 85]], { width: 5, color: "#fff6c8", opacity: 0.9 })
}
part("llama_izq", () => flame(FLAME_S, 440, 840))
part("llama_der", () => flame(shift(FLAME_S, 144, 0), 584, 840))
part("llama_centro", () => flame(FLAME_C, 512, 850))

// ---------- 3. Alas y carenados ----------
function wing(s) {
  const w = poly(side(s, WING), { fill: P.teal })
  clip(w, () => {
    poly(side(s, [[490, 628], [200, 640], [200, 740], [490, 740]]), { fill: P.tealD })
    brush(side(s, [[430, 392], [340, 452], [262, 512]]), { width: [10, 16, 6], color: P.tealL, taper: [0.1, 0.5] })
    brush(side(s, [[390, 440], [345, 470], [300, 505]]), { width: [3, 8, 2], color: P.tealL, opacity: 0.85 })
  })
  ink(w, { width: OUT, weight: 1.5, color: P.ink })
  // carenado del cañón: blanco con franjas lima
  const f = blob(side(s, FAIR), { fill: P.white })
  clip(f, () => {
    blob(side(s, [[300, 640], [420, 600], [420, 720], [300, 720]]), { fill: P.lav })
    for (const y of [520, 560]) poly(side(s, [[270, y + 22], [350, y - 6], [420, y + 14], [420, y + 30], [350, y + 10], [270, y + 38]]), { fill: P.lime })
    brush(side(s, [[300, 560], [298, 620], [310, 670]]), { width: [3, 7, 3], color: "#ffffff" })
    // zócalo donde encaja la recámara
    blob(side(s, [[312, 640], [400, 640], [396, 690], [356, 702], [316, 690]]), { fill: mix(P.lav, P.ink, 0.3) })
  })
  ink(f, { width: OUT * 0.8, weight: 1.5, color: P.ink })
}

// ---------- 4. Góndolas ----------
function pod(s) {
  const cx = sx(s, 196)
  const p = blob(side(s, POD), { fill: P.lav })
  clip(p, () => {
    blob([[cx - 80, 430], [cx + 14, 430], [cx + 22, 560], [cx + 24, 690], [cx + 14, 742], [cx - 30, 750], [cx - 80, 752]], { fill: P.white })
    rect(cx - 80, 690, 160, 26, { fill: P.teal })
    rect(cx + 22, 690, 60, 26, { fill: shade(P.teal, -0.2) })
    brush([[cx - 46, 530], [cx - 48, 640]], { width: 7, color: "#ffffff" })
  })
  ink(p, { width: OUT, weight: 1.5, color: P.ink })
  const t = ellipse(cx, 487, 36, 27, { fill: P.orange })
  clip(t, () => {
    ellipse(cx, 506, 40, 20, { fill: P.rust })
    curve([[cx - 22, 478], [cx - 4, 470], [cx + 18, 474]], { stroke: "#ffd6a8", width: 5, cap: "round" })
  })
  ellipse(cx, 487, 36, 27, { fill: "none", stroke: P.ink, width: OUT })
}
for (const s of [-1, 1]) {
  const k = s < 0 ? "izq" : "der"
  part("ala_" + k, () => wing(s))
  part("gondola_" + k, () => pod(s))
  part("canon_" + k, () => group({ rotate: s * CAN_ROT, around: [sx(s, 356), 600] }, () => cannon(s)))
}
// ---------- 5. Fuselaje, toberas, cabina y núcleo ----------
function nozzle(pts, hx) {
  const n = blob(pts, { fill: P.noz })
  clip(n, () => {
    brush([[hx, pts[0][1] + 20], [hx, pts[2][1] - 12]], { width: 7, color: "#5a5f96", taper: [0.1, 0.2] })
    ellipse((pts[0][0] + pts[1][0]) / 2, pts[2][1] + 4, 80, 12, { fill: shade(P.noz, -0.4) })
  })
  ink(n, { width: OUT * 0.8, weight: 1.4, color: P.ink })
}
part("fuselaje", () => {
  nozzle(NOZ_L, 420)
  nozzle(mx(NOZ_L), 1024 - 448)
  nozzle(NOZ_C, 482)
  const b = blob(FUS, { fill: P.lav })
  clip(b, () => {
    blob([[380, 60], [540, 60], [578, 300], [592, 450], [596, 650], [586, 742], [540, 772], [440, 778], [380, 790]], { fill: P.white })
    // franja del morro
    poly([[380, 160], [512, 106], [660, 160], [660, 210], [512, 184], [380, 210]], { fill: P.teal })
    poly([[546, 120], [660, 120], [660, 214], [546, 196]], { fill: shade(P.teal, -0.18) })
    brush([[424, 420], [418, 560], [426, 720]], { width: [3, 7, 3], color: "#ffffff" })
  })
  ink(b, { width: OUT * 1.1, weight: 1.5, color: P.ink })
  // cabina
  const c = blob([[512, 214], [548, 240], [562, 320], [556, 392], [512, 412], [468, 392], [462, 320], [476, 240]], { fill: P.blue })
  clip(c, () => {
    blob([[506, 214], [536, 240], [548, 320], [540, 392], [500, 412], [458, 392], [452, 320], [466, 240]], { fill: P.cyan })
    brush([[492, 262], [484, 312]], { width: [9, 6], color: "#ffffff", taper: [0.3, 0.3] })
    circle(498, 346, 5.5, { fill: "#ffffff" })
  })
  blob([[512, 214], [548, 240], [562, 320], [556, 392], [512, 412], [468, 392], [462, 320], [476, 240]], { fill: "none", stroke: P.ink, width: OUT })
  // núcleo amarillo
  circle(512, 540, 52, { fill: P.ink })
  circle(512, 540, 41, { fill: radial(498, 522, 50, ["#ffd84a", "#ffb40f", "#ef860a"]), stroke: "#fff1a6", width: 3 })
  circle(499, 526, 9.5, { fill: "#ffffff" })
  circle(527, 555, 4, { fill: "#fff8d8" })
})

// ---------- 6. Cresta ----------
part("cresta", () => {
  const c = blob(CREST_PTS, { fill: P.lav })
  clip(c, () => {
    poly([[460, 590], [512, 590], [512, 790], [460, 790]], { fill: P.white })
    // placas de refuerzo en chevron
    for (const y of [690, 726]) poly([[460, y], [512, y + 14], [560, y], [560, y + 13], [512, y + 27], [460, y + 13]], { fill: P.teal })
    poly([[512, 690], [560, 676], [560, 760], [512, 760]], { fill: alpha(P.ink, 0.18) })
    brush([[490, 640], [488, 740]], { width: [2, 5, 2], color: "#ffffff" })
  })
  ink(c, { width: OUT * 0.9, weight: 1.5, color: P.ink })
  // luz verde en el nervio
  rect(502, 626, 20, 46, { r: 10, fill: P.lime, glow: { blur: 18, color: alpha(P.lime, 0.95) } })
  rect(502, 626, 20, 46, { r: 10, fill: "none", stroke: shade(P.lime, -0.55), width: 3 })
  rect(507, 632, 6, 20, { r: 3, fill: "#ffffff" })
})

// ---------- 7. Cañones HORQUILLA ----------
function cannon(s) {
  // la luz viene de la izquierda: el lado derecho de cada pieza va en sombra
  const c = blob(side(s, CAN), { fill: P.lav })
  const cx = sx(s, 356)
  clip(c, () => {
    poly([[cx - 60, 300], [cx + 8, 300], [cx + 8, 506], [cx + 18, 524], [cx + 18, 660], [cx - 60, 660]], { fill: P.white })
    // anillos teal en la caña
    for (const y of [392, 432]) rect(cx - 40, y, 80, 13, { fill: P.teal })
    rect(cx + 8, 392, 30, 13, { fill: shade(P.teal, -0.2) })
    rect(cx + 8, 432, 30, 13, { fill: shade(P.teal, -0.2) })
    // recámara: franja lima y rejilla
    rect(cx - 40, 556, 80, 14, { fill: P.lime })
    rect(cx + 18, 556, 30, 14, { fill: shade(P.lime, -0.25) })
    for (const y of [588, 604, 620]) line(cx - 18, y, cx + 10, y, { stroke: shade(P.lav, -0.35), width: 4, cap: "round" })
    brush([[cx - 9, 334], [cx - 10, 420], [cx - 9, 490]], { width: [3, 6, 3], color: "#ffffff" })
    brush([[cx - 26, 534], [cx - 27, 630]], { width: [3, 6, 3], color: "#ffffff" })
  })
  ink(c, { width: OUT * 0.9, weight: 1.5, color: P.ink })
  // boca con emisor verde
  const m = blob(side(s, MUZ), { fill: P.tealD })
  clip(m, () => {
    blob([[cx - 50, 260], [cx + 6, 260], [cx + 4, 360], [cx - 50, 360]], { fill: P.teal })
    brush([[cx - 26, 304], [cx - 24, 332]], { width: 5, color: P.tealL })
  })
  ink(m, { width: OUT * 0.85, weight: 1.4, color: P.ink })
  ellipse(cx, 296, 20, 14, { fill: P.lime, glow: { blur: 16, color: alpha(P.lime, 0.9) } })
  ellipse(cx, 296, 20, 14, { fill: "none", stroke: shade(P.lime, -0.55), width: 3.5 })
  ellipse(cx - 4, 293, 8, 5, { fill: "#ffffff" })
}
// (los part de los cañones se escriben antes del fuselaje: al plegar se esconden debajo)
// ---------- Acciones ----------
// Ala izquierda hacia atrás = giro antihorario (negativo); la derecha, al revés.
action("plegar", { duration: 1, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, gondola_izq: 0, gondola_der: 0 }],
  [0.5, { ala_izq: -11, ala_der: 11, gondola_izq: 5, gondola_der: -5 }],
  [1, { ala_izq: -24, ala_der: 24, gondola_izq: 12, gondola_der: -12 }],
])

// Llamas: bajan (se alargan) y suben (se acortan) bajo las toberas, desfasadas
action("motor", { duration: 0.4, loop: true }, [
  [0, { llama_izq: { y: 0 }, llama_centro: { y: 22 }, llama_der: { y: 12 } }],
  [0.25, { llama_izq: { y: 12 }, llama_centro: { y: 10 }, llama_der: { y: 24 } }],
  [0.5, { llama_izq: { y: 24 }, llama_centro: { y: -4 }, llama_der: { y: 12 } }],
  [0.75, { llama_izq: { y: 12 }, llama_centro: { y: 10 }, llama_der: { y: 0 } }],
  [1, { llama_izq: { y: 0 }, llama_centro: { y: 22 }, llama_der: { y: 12 } }],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, llama_izq: { y: 0 }, llama_centro: { y: 0 }, llama_der: { y: 0 }, fuselaje: { y: 0 } }],
  [0.35, { ala_izq: -12, ala_der: 12, llama_izq: { y: 50 }, llama_centro: { y: 62 }, llama_der: { y: 50 }, fuselaje: { y: 6 } }],
  [1, { ala_izq: -9, ala_der: 9, llama_izq: { y: 58 }, llama_centro: { y: 70 }, llama_der: { y: 58 }, fuselaje: { y: 0 } }],
])

// Acoplar: cresta, cañón izquierdo y cañón derecho encajan uno a uno con rebote
const SEP_I = { rot: -22, x: -150, y: 150 }
const SEP_D = { rot: 22, x: 150, y: 150 }
const SEP_C = { rot: 0, x: 0, y: 130 }
action("acoplar", { duration: 0.7, loop: false }, [
  [0, { canon_izq: SEP_I, canon_der: SEP_D, cresta: SEP_C }],
  [0.22, { cresta: { rot: 0, x: 0, y: -10 } }],
  [0.3, { cresta: { rot: 0, x: 0, y: 0 }, canon_izq: SEP_I }],
  [0.5, { canon_izq: { rot: 3, x: 6, y: -12 }, canon_der: SEP_D }],
  [0.6, { canon_izq: { rot: 0, x: 0, y: 0 } }],
  [0.8, { canon_der: { rot: -3, x: -6, y: -12 } }],
  [0.9, { canon_der: { rot: 0, x: 0, y: 0 }, fuselaje: { y: 0 } }],
  [0.95, { fuselaje: { y: 4 } }],
  [1, { canon_izq: { rot: 0, x: 0, y: 0 }, canon_der: { rot: 0, x: 0, y: 0 }, cresta: { rot: 0, x: 0, y: 0 }, fuselaje: { y: 0 } }],
])
