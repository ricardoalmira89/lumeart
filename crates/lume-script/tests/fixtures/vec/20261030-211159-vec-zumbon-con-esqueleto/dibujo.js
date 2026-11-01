// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Zumbón: dron insecto visto desde arriba, cabeza (visor) hacia abajo.
const P = {
  ink: "#173017",      // contorno del cuerpo
  lime: "#98e22c",     // abdomen y tórax
  limeSh: "#5ea422",   // sombra del lima
  stripe: "#2f6e18",   // franjas del abdomen
  head: "#3f8a22",     // cabeza
  headSh: "#1e5e14",   // sombra de la cabeza
  jaw: "#d6ef70",      // mandíbulas
  jawIn: "#9ee32a",
  wing: "#b8dffc",     // ala
  wingSh: "#85b9ec",
  wingHi: "#e2f3ff",
  wingInk: "#1f3a6b",
  red: "#ff2a3a",
  redSh: "#cc0030",
  white: "#ffffff",
}
setLight(-1, -1)
const BORDER = 46 // borde blanco de pegatina

// ---------- geometría (pose de reposo) ----------
const WING_L = [[452, 500], [415, 440], [330, 345], [220, 268], [120, 222], [70, 240], [62, 300], [110, 370], [210, 440], [330, 495], [420, 515]]
const WING_R = WING_L.map(([x, y]) => [1024 - x, y])
const ABD = [[512, 60, 0.4], [578, 82], [626, 145], [646, 240], [634, 330], [596, 398], [540, 432], [484, 432], [428, 398], [390, 330], [378, 240], [398, 145], [446, 82]]
const JAW_L = [[436, 718], [480, 728], [490, 770], [490, 812], [500, 850, 0], [462, 832], [428, 802], [416, 760]]
const JAW_R = JAW_L.map(([x, y, s]) => s === undefined ? [1024 - x, y] : [1024 - x, y, s])
const TH = { cx: 515, cy: 475, rx: 118, ry: 82 }
const HD = { cx: 515, cy: 655, rx: 136, ry: 104 }

rig({ origin: [512, 512] })
bone("torax", { at: [512, 475] })
bone("abdomen", { parent: "torax", at: [512, 405] })
bone("cabeza", { parent: "torax", at: [512, 560] })
bone("ala_izq", { parent: "torax", at: [452, 458] })
bone("ala_der", { parent: "torax", at: [572, 458] })

// ---------- borde blanco de pegatina (todas las siluetas, detrás de todo) ----------
const sticker = { fill: P.white, stroke: P.white, width: BORDER, join: "round" }
part("ala_izq", () => { blob(WING_L, sticker) })
part("ala_der", () => { blob(WING_R, sticker) })
part("abdomen", () => { blob(ABD, sticker) })
part("torax", () => { ellipse(TH.cx, TH.cy, TH.rx, TH.ry, sticker) })
part("cabeza", () => {
  blob(JAW_L, sticker); blob(JAW_R, sticker)
  ellipse(HD.cx, HD.cy, HD.rx, HD.ry, sticker)
})

// ---------- alas ----------
function wing(pts, side) {
  const s = blob(pts, { fill: P.wing })
  // m(x) refleja la coordenada para el ala derecha
  const m = (q) => q.map(([x, y]) => [side < 0 ? x : 1024 - x, y])
  clip(s, () => {
    // banda inferior más densa, con borde ondulado
    blob(m([[40, 330], [95, 325], [140, 370], [200, 410], [320, 418], [440, 425], [480, 560], [40, 560]]), { fill: P.wingSh })
    // reflejo claro arriba
    blob(m([[150, 240], [260, 262], [370, 320], [430, 380], [360, 352], [240, 300], [160, 262]]), { fill: P.wingHi, opacity: 0.85 })
    // nervios
    brush(m([[110, 268], [210, 300], [310, 350], [400, 410], [445, 445]]), { width: [3, 9, 11], taper: [0.5, 0], color: P.white })
    brush(m([[300, 352], [262, 392], [215, 410], [170, 414]]), { width: [7, 5, 2], taper: [0, 0.5], color: P.white })
  })
  blob(pts, { fill: "none", stroke: P.wingInk, width: 11, join: "round" })
  ink(s, { width: 4, weight: 2.4, color: P.wingInk })
  return s
}
part("ala_izq", () => { wing(WING_L, -1) })
part("ala_der", () => { wing(WING_R, 1) })

// ---------- abdomen ----------
part("abdomen", () => {
  const a = blob(ABD, { fill: P.lime })
  clip(a, () => {
    // franjas onduladas
    const bands = [[140, 192], [218, 272], [300, 355]]
    for (const [y0, y1] of bands) {
      blob([[360, y0 + 4], [440, y0 + 16], [520, y0 + 20], [600, y0 + 6], [680, y0 - 8],
            [680, y1 - 10], [600, y1 - 4], [520, y1 + 6], [440, y1 + 2], [360, y1 - 6]], { fill: P.stripe })
    }
    // sombra del lado derecho (un solo tono)
    blob([[560, 60], [640, 120], [680, 260], [640, 420], [520, 460], [570, 380], [604, 260], [596, 150]], { fill: P.limeSh, blend: "multiply", opacity: 0.55 })
    // brillo vertical
    ellipse(457, 175, 22, 60, { fill: P.white, opacity: 0.82, rotate: 10 })
  })
  blob(ABD, { fill: "none", stroke: P.ink, width: 12, join: "round" })
  ink(a, { width: 4, weight: 2.6, color: P.ink })
})

// ---------- tórax ----------
part("torax", () => {
  const t = ellipse(TH.cx, TH.cy, TH.rx, TH.ry, { fill: P.lime })
  clip(t, () => {
    // media luna de sombra: sombra entera y el color base desplazado hacia la luz
    ellipse(TH.cx, TH.cy, TH.rx + 10, TH.ry + 10, { fill: P.limeSh })
    ellipse(TH.cx - 18, TH.cy - 12, TH.rx - 2, TH.ry - 6, { fill: P.lime })
    ellipse(470, 444, 40, 19, { fill: P.white, opacity: 0.85, rotate: -10 })
  })
  ellipse(TH.cx, TH.cy, TH.rx, TH.ry, { fill: "none", stroke: P.ink, width: 12 })
  ink(t, { width: 4, weight: 2.6, color: P.ink })
})

// ---------- cabeza (mandíbulas detrás) ----------
function jaw(pts, side) {
  const j = blob(pts, { fill: P.jaw })
  const m = (q) => q.map(([x, y]) => [side < 0 ? x : 1024 - x, y])
  clip(j, () => {
    brush(m([[474, 730], [482, 780], [484, 815], [496, 846]]), { width: [16, 12, 3], taper: [0, 0.6], color: P.jawIn })
  })
  blob(pts, { fill: "none", stroke: P.ink, width: 10, join: "round" })
  ink(j, { width: 3, weight: 2, color: P.ink })
}
part("cabeza", () => {
  jaw(JAW_L, -1)
  jaw(JAW_R, 1)
  const h = ellipse(HD.cx, HD.cy, HD.rx, HD.ry, { fill: P.head })
  clip(h, () => {
    // sombra inferior derecha
    ellipse(HD.cx, HD.cy, HD.rx + 10, HD.ry + 10, { fill: P.headSh })
    ellipse(HD.cx - 22, HD.cy - 20, HD.rx - 6, HD.ry - 10, { fill: P.head })
    ellipse(455, 590, 34, 14, { fill: P.white, opacity: 0.72, rotate: -10 })
  })
  ellipse(HD.cx, HD.cy, HD.rx, HD.ry, { fill: "none", stroke: P.ink, width: 12 })
  ink(h, { width: 4, weight: 2.6, color: P.ink })
  // visor
  rect(398, 616, 230, 72, { r: 36, fill: P.ink })
  const v = rect(408, 626, 210, 54, { r: 27, fill: P.red })
  clip(v, () => {
    rect(400, 655, 230, 40, { fill: P.redSh })
    rect(432, 633, 84, 13, { r: 6.5, fill: "#ffc4c8" })
    circle(588, 640, 7, { fill: P.white })
  })
})

// ---------- acciones ----------
// ala_izq: positivo = punta hacia atrás (arriba, hacia el abdomen); ala_der al revés.
action("volar", { duration: 0.16, loop: true }, [
  [0,    { ala_izq: 18,  ala_der: -18, abdomen: -3, cabeza: 0.5 }],
  [0.5,  { ala_izq: -17, ala_der: 17,  abdomen: 3,  cabeza: -0.5 }],
  [1,    { ala_izq: 18,  ala_der: -18, abdomen: -3, cabeza: 0.5 }],
])

action("picado", { duration: 0.5, loop: true }, [
  [0,    { ala_izq: 52, ala_der: -52, abdomen: 0, cabeza: 0, torax: { rot: 0, y: 0 } }],
  [0.125,{ ala_izq: 47, ala_der: -47 }],
  [0.25, { ala_izq: 53, ala_der: -53, torax: { rot: 0, y: 3 } }],
  [0.375,{ ala_izq: 46, ala_der: -46 }],
  [0.5,  { ala_izq: 52, ala_der: -52, torax: { rot: 0, y: 0 } }],
  [0.625,{ ala_izq: 47, ala_der: -47 }],
  [0.75, { ala_izq: 53, ala_der: -53, torax: { rot: 0, y: 3 } }],
  [0.875,{ ala_izq: 46, ala_der: -46 }],
  [1,    { ala_izq: 52, ala_der: -52, abdomen: 0, cabeza: 0, torax: { rot: 0, y: 0 } }],
])

action("golpe", { duration: 0.25, loop: false }, [
  [0,    { torax: { rot: 0, x: 0, y: 0 }, cabeza: 0, abdomen: 0, ala_izq: 0, ala_der: 0 }],
  [0.15, { torax: { rot: -9, x: -14, y: -6 }, cabeza: 6, abdomen: 8, ala_izq: 25, ala_der: -10 }],
  [0.35, { torax: { rot: 7, x: 11, y: 2 }, cabeza: -5, abdomen: -9, ala_izq: -12, ala_der: 22 }],
  [0.55, { torax: { rot: -4, x: -6, y: 0 }, cabeza: 3, abdomen: 5, ala_izq: 10, ala_der: -6 }],
  [0.78, { torax: { rot: 1.5, x: 2, y: 0 }, cabeza: -1, abdomen: -2, ala_izq: -3, ala_der: 3 }],
  [1,    { torax: { rot: 0, x: 0, y: 0 }, cabeza: 0, abdomen: 0, ala_izq: 0, ala_der: 0 }],
])
