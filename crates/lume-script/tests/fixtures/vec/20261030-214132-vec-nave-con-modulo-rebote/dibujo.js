// dibujo.js: nave con módulo REBOTE integrado (vista cenital, mirando hacia arriba).
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  navy: "#1c2240",      // contorno
  white: "#f3f6fb",     // casco
  gray: "#a9b4d6",      // sombra del casco
  teal: "#1bb3a6",      // alas y franjas
  tealDark: "#117482",  // banda trasera del ala
  tealShade: "#14918c", // franja en sombra
  tealLight: "#8ee8d8", // brillos del ala
  cyan: "#5ee3f7",      // cabina
  blue: "#2a86dc",      // cabina en sombra
  orange: "#f28a1a",    // tomas de aire
  orangeDark: "#9a3a10",
  nozzle: "#2c2f58",    // toberas
  nozzleLight: "#555c99",
  gold: "#f6bd2e",      // acento dorado (arma)
  goldLight: "#ffe589",
  goldDark: "#c47812",
  amber: "#ffab1f",     // energía ámbar
  flame: "#ff7a1a",
  flameCore: "#ffd23a",
}
const BORDE = 58 // grosor del borde blanco de pegatina
const LW = 11    // grosor base de la tinta

// --- utilidades de lado: s = -1 izquierda, s = 1 derecha (espejo sobre x = 512)
const MX = (x, s) => (s < 0 ? x : 1024 - x)
const M = (pts, s) => (s < 0 ? pts : pts.map((p) => [1024 - p[0], ...p.slice(1)]))

// --- geometría de cada pieza (pose de reposo)
const FUS = [[512, 50, 0], [545, 130], [580, 250], [606, 400], [622, 580], [624, 740], [598, 825], [512, 852], [426, 825], [402, 740], [404, 580], [420, 400], [446, 250], [480, 130]]
const ALA = (s) => M([[455, 356], [230, 512], [230, 728], [455, 728]], s)
const GOND = (s) => M([[196, 433], [240, 442], [262, 500], [266, 700], [256, 814], [196, 828], [136, 814], [126, 700], [130, 500], [152, 442]], s)
const TOBERAS = [[405, 832, 62, 80], [557, 832, 62, 80], [462, 842, 100, 82]]

// placa del borde de ataque: a lo largo del borde (455,356)->(230,512)
function placaPts(s) {
  const u = [-0.822, 0.57], n = [-0.57, -0.822]
  const S = [430, 373], L = 165
  const at = (a, b) => [S[0] + u[0] * a + n[0] * b, S[1] + u[1] * a + n[1] * b]
  return {
    shape: M([[...at(0, 18), 0.5], [...at(L, 20), 0.5], [...at(L + 8, -4), 0.5], [...at(L - 4, -28), 0.5], [...at(4, -28), 0.5], [...at(-8, -4), 0.5]], s),
    inner: M([at(-20, -10), at(L + 20, -10), at(L + 20, -40), at(-20, -40)], s),
    energia: M([at(14, -14), at(L / 2 - 10, -14)], s),
    energia2: M([at(L / 2 + 10, -14), at(L - 14, -14)], s),
    junta: M([at(L / 2, 22), at(L / 2, -30)], s),
    filo: M([at(10, 14), at(L - 6, 16)], s),
    nodos: M([at(L / 2 - 26, -16), at(L / 2 + 26, -16)], s),
  }
}

// prisma emisor sobre la punta de la góndola, apuntando adelante y un poco afuera
function prismaPts(s) {
  const B = [196, 452], d = [-0.208, -0.978], p = [0.978, -0.208]
  const L = 128, W = 46
  const tip = [B[0] + d[0] * L, B[1] + d[1] * L]
  const bl = [B[0] - p[0] * W, B[1] - p[1] * W]
  const br = [B[0] + p[0] * W, B[1] + p[1] * W]
  const back = [B[0] - d[0] * 20, B[1] - d[1] * 20]
  const outer = M([tip, bl, back], s), inner = M([tip, br, back], s)
  return {
    shape: M([[...tip, 0], [...br, 0.15], [...back, 0.5], [...bl, 0.15]], s),
    dark: s < 0 ? inner : outer, // la luz viene de la izquierda
    tip: M([tip], s)[0], back: M([back], s)[0],
    mid: M([[B[0] + d[0] * 60, B[1] + d[1] * 60]], s)[0],
  }
}
const COLLAR = (s) => M([[132, 482], [136, 455], [158, 434], [196, 425], [234, 434], [256, 455], [260, 482], [196, 490]], s)
const LLAMA = (cx, w, tip) => [[cx - w / 2, 790], [cx + w / 2, 790], [cx + w / 2, 905], [cx + w * 0.35, tip - 40], [cx, tip, 0], [cx - w * 0.35, tip - 40], [cx - w / 2, 905]]

// --- esqueleto
rig({ origin: [512, 540] })
bone("fuselaje", { at: [512, 540] })
bone("ala_izq", { parent: "fuselaje", at: [430, 400] })
bone("ala_der", { parent: "fuselaje", at: [594, 400] })
bone("gondola_izq", { parent: "ala_izq", at: [242, 600] })
bone("gondola_der", { parent: "ala_der", at: [782, 600] })
bone("placa_izq", { parent: "ala_izq", at: [362, 420] })
bone("placa_der", { parent: "ala_der", at: [662, 420] })
bone("prisma_izq", { parent: "gondola_izq", at: [196, 452] })
bone("prisma_der", { parent: "gondola_der", at: [828, 452] })
bone("llama_izq", { parent: "fuselaje", at: [436, 905] })
bone("llama_centro", { parent: "fuselaje", at: [512, 915] })
bone("llama_der", { parent: "fuselaje", at: [588, 905] })

// ============ PASADA 1: borde blanco de pegatina (detrás de todo) ============
const WB = { fill: "#ffffff", stroke: "#ffffff", width: BORDE, join: "round" }
for (const s of [-1, 1]) {
  const L = s < 0 ? "izq" : "der"
  part("ala_" + L, () => poly(ALA(s), WB))
  part("gondola_" + L, () => blob(GOND(s), WB))
  part("placa_" + L, () => blob(placaPts(s).shape, WB))
  part("prisma_" + L, () => { blob(COLLAR(s), WB); blob(prismaPts(s).shape, WB) })
}
part("fuselaje", () => {
  blob(FUS, WB)
  for (const [x, y, w, h] of TOBERAS) rect(x, y, w, h, { ...WB, r: 12 })
})

// ============ PASADA 2: piezas a color, de atrás hacia delante ============
// llamas (detrás de las toberas)
for (const [name, cx, w, tip] of [["llama_izq", 436, 40, 958], ["llama_der", 588, 40, 958], ["llama_centro", 512, 56, 992]]) {
  part(name, () => {
    ellipse(cx, tip - 50, w * 0.9, 60, { fill: alpha(P.flame, 0.55), blur: 18 })
    blob(LLAMA(cx, w, tip), { fill: P.flame })
    blob(LLAMA(cx, w * 0.5, tip - 32), { fill: P.flameCore })
    blob(LLAMA(cx, w * 0.18, tip - 52), { fill: "#fff6c8" })
  })
}

// alas
for (const s of [-1, 1]) {
  part(s < 0 ? "ala_izq" : "ala_der", () => {
    const w = poly(ALA(s), { fill: P.teal, join: "round" })
    clip(w, () => {
      poly(M([[200, 636], [470, 628], [470, 760], [200, 760]], s), { fill: P.tealDark })
      brush(M([[418, 410], [270, 520]], s), { width: 9, color: P.tealLight, taper: [0.2, 0.6] })
      brush(M([[380, 462], [300, 520]], s), { width: 5, color: P.tealLight, taper: [0.3, 0.6], opacity: 0.8 })
    })
    poly(ALA(s), { fill: "none", stroke: P.navy, width: LW, join: "round" })
  })
}

// placas doradas del borde de ataque
for (const s of [-1, 1]) {
  part(s < 0 ? "placa_izq" : "placa_der", () => {
    const g = placaPts(s)
    const pl = blob(g.shape, { fill: P.gold })
    clip(pl, () => {
      poly(g.inner, { fill: P.goldDark })
      brush(g.filo, { width: 7, color: P.goldLight, taper: [0.2, 0.2] })
      for (const e of [g.energia, g.energia2]) brush(e, { width: 6, color: P.amber, taper: [0.15, 0.15], glow: { blur: 6, color: P.amber } })
      for (const e of [g.energia, g.energia2]) brush(e, { width: 2.2, color: "#fff4c0", taper: [0.2, 0.2] })
      line(...g.junta[0], ...g.junta[1], { stroke: P.navy, width: 5 })
    })
    for (const [x, y] of g.nodos) {
      circle(x, y, 6, { fill: P.amber, stroke: P.navy, width: 3, glow: { blur: 6, color: P.amber } })
    }
    ink(pl, { width: 8, weight: 1.6, color: P.navy })
  })
}

// góndolas
for (const s of [-1, 1]) {
  part(s < 0 ? "gondola_izq" : "gondola_der", () => {
    const cx = MX(196, s), dx = cx - 196
    const g = blob(GOND(s), { fill: P.white })
    clip(g, () => {
      blob(shift([[214, 440], [290, 440, 0], [290, 850, 0], [100, 850, 0], [100, 750], [150, 744], [205, 742], [220, 700], [214, 560]], dx, 0), { fill: P.gray })
      rect(cx - 80, 688, 160, 26, { fill: P.teal })
      rect(cx + 20, 688, 70, 26, { fill: P.tealShade })
      brush([[cx - 48, 530], [cx - 52, 610], [cx - 50, 690]], { width: 6, color: "#ffffff", taper: [0.3, 0.4] })
      // toma de aire
      ellipse(cx, 512, 37, 25, { fill: P.navy })
      const t = ellipse(cx, 512, 27, 16, { fill: P.orange })
      clip(t, () => ellipse(cx, 524, 32, 13, { fill: P.orangeDark }))
      brush([[cx - 14, 503], [cx + 6, 501]], { width: 4, color: "#ffe2b0", taper: [0.3, 0.3] })
    })
    ink(g, { width: LW, weight: 1.5, color: P.navy })
  })
}

// collares y prismas emisores
for (const s of [-1, 1]) {
  part(s < 0 ? "prisma_izq" : "prisma_der", () => {
    const cx = MX(196, s)
    const c = blob(COLLAR(s), { fill: P.gold })
    clip(c, () => {
      rect(cx + 18, 400, 80, 100, { fill: P.goldDark })
      ellipse(cx, 492, 70, 14, { fill: P.goldDark, opacity: 0.8 })
      brush([[cx - 50, 470], [cx - 40, 446], [cx - 14, 434]], { width: 5, color: P.goldLight })
    })
    ink(c, { width: 9, weight: 1.5, color: P.navy })
    for (const x of [cx - 46, cx + 46]) circle(x, 474, 4.5, { fill: P.goldLight, stroke: P.navy, width: 2.5 })

    const g = prismaPts(s)
    const pr = blob(g.shape, { fill: P.gold })
    clip(pr, () => {
      poly(g.dark, { fill: P.goldDark })
      brush([g.tip, g.back], { width: 4, color: P.goldLight, taper: [0.1, 0.3] })
      brush([g.mid, g.back], { width: 5, color: P.amber, taper: [0.3, 0.1], glow: { blur: 6, color: P.amber } })
    })
    ink(pr, { width: 9, weight: 1.6, color: P.navy })
    // brillo ámbar del emisor
    circle(...g.tip, 22, { fill: alpha(P.amber, 0.35), glow: { blur: 16, color: P.amber } })
    circle(...g.tip, 11, { fill: P.amber, glow: { blur: 10, color: P.amber } })
    circle(...g.tip, 5, { fill: "#fff8d8" })
  })
}

// fuselaje con toberas, cabina y núcleo
part("fuselaje", () => {
  for (const [x, y, w, h] of TOBERAS) {
    const tb = rect(x, y, w, h, { fill: P.nozzle, r: 12 })
    clip(tb, () => {
      rect(x + 8, y, 10, h, { fill: P.nozzleLight })
      rect(x, y + h - 14, w, 14, { fill: shade(P.nozzle, -0.4) })
    })
    rect(x, y, w, h, { fill: "none", stroke: P.navy, width: 8, r: 12 })
  }
  const f = blob(FUS, { fill: P.white })
  clip(f, () => {
    // franja del morro
    const band = poly([[380, 168], [512, 106], [650, 168], [650, 222], [512, 184], [380, 222]], { fill: P.teal })
    clip(band, () => rect(546, 100, 120, 130, { fill: P.tealShade }))
    // sombra del lado derecho y del vientre
    blob([[574, 230], [660, 230, 0], [660, 900, 0], [360, 900, 0], [360, 792], [460, 776], [560, 768], [592, 720], [598, 560], [588, 380]], { fill: P.gray })
    brush([[430, 440], [420, 600], [426, 740]], { width: 7, color: "#ffffff", taper: [0.3, 0.4] })
    brush([[488, 210], [470, 150]], { width: 5, color: "#ffffff", taper: [0.3, 0.5], opacity: 0.8 })
  })
  ink(f, { width: LW + 1, weight: 1.5, color: P.navy })

  // cabina
  ellipse(512, 312, 52, 102, { fill: P.navy })
  const cab = ellipse(512, 312, 40, 89, { fill: P.blue })
  clip(cab, () => ellipse(500, 306, 38, 92, { fill: P.cyan }))
  brush([[494, 262], [486, 300], [488, 310]], { width: 11, color: "#ffffff", taper: [0.2, 0.5] })
  circle(497, 345, 5, { fill: "#ffffff" })

  // núcleo
  circle(512, 540, 55, { fill: P.navy })
  circle(512, 540, 44, { fill: "#fff1c0" })
  circle(512, 540, 40, { fill: radial(498, 524, 56, ["#ffe066", "#ffb300", [1, "#e47400"]]) })
  circle(498, 526, 11, { fill: "#ffffff" })
  circle(527, 555, 4, { fill: "#ffffff" })
})

// ============ ACCIONES ============
const R = { ala_izq: 0, ala_der: 0, gondola_izq: 0, gondola_der: 0 }
const L0 = { llama_izq: { y: 0 }, llama_centro: { y: 0 }, llama_der: { y: 0 } }

action("plegar", { duration: 1, loop: false }, [
  [0, { ...R }],
  [0.5, { ala_izq: -13, ala_der: 13, gondola_izq: 9, gondola_der: -9 }],
  [1, { ala_izq: -23, ala_der: 23, gondola_izq: 16, gondola_der: -16 }],
])

const fl = (a, b, c) => ({ llama_izq: { y: a }, llama_centro: { y: b }, llama_der: { y: c } })
action("motor", { duration: 0.4, loop: true }, [
  [0, fl(0, 26, 12)],
  [0.25, fl(16, 10, 28)],
  [0.5, fl(28, 0, 10)],
  [0.75, fl(12, 18, 0)],
  [1, fl(0, 26, 12)],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ...R, ...L0 }],
  [0.35, { ala_izq: -15, ala_der: 15, gondola_izq: 7, gondola_der: -7, ...fl(78, 92, 78) }],
  [1, { ala_izq: -10, ala_der: 10, gondola_izq: 5, gondola_der: -5, ...fl(66, 84, 66) }],
])

// acoplar: cada pieza llega desde delante y afuera, encaja con un pequeño rebote
const pieza = (b, s, lejos, t0, t1) => {
  const ini = { x: s * lejos[0], y: lejos[1], rot: s * lejos[2] }
  return [
    [0, { [b]: ini }], [t0, { [b]: ini }],
    [t1, { [b]: { x: -s * 5, y: 9, rot: -s * 4 } }],
    [t1 + 0.07, { [b]: { x: 0, y: 0, rot: 0 } }],
    [1, { [b]: { x: 0, y: 0, rot: 0 } }],
  ]
}
const claves = [
  ...pieza("placa_izq", -1, [170, -190, 55], 0.0, 0.3),
  ...pieza("placa_der", 1, [170, -190, 55], 0.12, 0.45),
  ...pieza("prisma_izq", -1, [140, -260, 80], 0.27, 0.62),
  ...pieza("prisma_der", 1, [140, -260, 80], 0.42, 0.86),
]
const porT = {}
for (const [t, p] of claves) { const k = t.toFixed(3); porT[k] = { ...(porT[k] || {}), ...p } }
action("acoplar", { duration: 0.7, loop: false },
  Object.keys(porT).map(Number).sort((a, b) => a - b).map((t) => [t, porT[t.toFixed(3)]]))
