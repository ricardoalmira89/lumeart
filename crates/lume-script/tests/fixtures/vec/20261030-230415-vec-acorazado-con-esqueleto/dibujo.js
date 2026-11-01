// dibujo.js: acorazado orgánico visto desde arriba, montado con esqueleto.
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

const P = {
  tinta: "#1a0a24",
  blanco: "#ffffff",
  verde: "#2e7a3a",
  verdeOsc: "#1b4a26",
  verdeLuz: "#4fa04a",
  verdeHex: "#7cc04e",
  lila: "#6c2f9c",
  lilaOsc: "#3e1858",
  lilaLuz: "#a76ad8",
  hueco: "#1c1c20",
  aro: "#3a1650",
  rojo: "#ff3a26",
  rojoOsc: "#7a0c12",
}
const MX = (pts) => pts.map(([x, y, s]) => (s === undefined ? [1024 - x, y] : [1024 - x, y, s]))

// ---------- formas base ----------
const CASCO = [[512, 30], [700, 44], [818, 118], [872, 255], [872, 420], [822, 560], [720, 660], [600, 704], [512, 708], [424, 704], [304, 660], [202, 560], [152, 420], [152, 255], [206, 118], [324, 44]]
const PUA_A = [[205, 150], [125, 138], [60, 118, 0], [86, 182], [126, 250], [166, 292], [210, 250]]
const PUA_B = [[185, 398], [112, 420], [42, 462, 0], [100, 500], [168, 534], [218, 530], [205, 450]]
const PATA = [[362, 655], [366, 708], [394, 734], [452, 748, 0], [436, 720], [458, 702], [446, 655]]
const CUPULA = [[288, 88], [348, 54], [512, 40], [676, 54], [738, 88], [700, 118], [600, 132], [512, 136], [424, 132], [324, 118]]
const VISERA = [[330, 132], [400, 114], [512, 108], [624, 114], [694, 132], [690, 172], [640, 194], [512, 200], [384, 194], [334, 172]]
const PLACA_M = [[138, 270], [220, 256], [300, 262], [338, 292], [342, 400], [326, 470], [282, 496], [205, 502], [150, 470]]
const PLACA_B = [[302, 524], [338, 494], [440, 488], [480, 512], [486, 600], [470, 656], [420, 674], [348, 666], [304, 622], [296, 566]]
const HUECOS = [[381, 215], [643, 215], [250, 378], [774, 378], [392, 572], [632, 572]]

// gajo del núcleo: cuarto de disco redondeado (sx, sy = ±1)
function gajo(sx, sy, R = 86) {
  const cx = 512 + sx * 10, cy = 383 + sy * 10
  const pts = [[cx, cy, 0.5]]
  for (let i = 0; i <= 4; i++) {
    const a = (i / 4) * Math.PI / 2
    pts.push([cx + sx * Math.cos(a) * (R - 6) * (i === 0 || i === 4 ? 0.98 : 1), cy + sy * Math.sin(a) * (R - 6)])
  }
  return pts
}

// ---------- esqueleto ----------
rig({ origin: [512, 384] })
bone("casco", { at: [512, 384] })
bone("cupula", { parent: "casco", at: [512, 90] })
bone("pua_izq", { parent: "casco", at: [180, 330] })
bone("pua_der", { parent: "casco", at: [844, 330] })
bone("pata_izq", { parent: "casco", at: [405, 672] })
bone("pata_der", { parent: "casco", at: [619, 672] })
bone("nucleo", { parent: "casco", at: [512, 383] })
bone("placa_nucleo_izq", { parent: "casco", at: [470, 383] })
bone("placa_nucleo_der", { parent: "casco", at: [554, 383] })

// ---------- borde blanco de pegatina (detrás de todo) ----------
const BW = 34 // grosor del borde blanco (stroke completo; la mitad queda fuera)
part("pua_izq", () => { for (const s of [PUA_A, PUA_B]) blob(s, { fill: P.blanco, stroke: P.blanco, width: BW, join: "round" }) })
part("pua_der", () => { for (const s of [PUA_A, PUA_B]) blob(MX(s), { fill: P.blanco, stroke: P.blanco, width: BW, join: "round" }) })
part("pata_izq", () => blob(PATA, { fill: P.blanco, stroke: P.blanco, width: BW, join: "round" }))
part("pata_der", () => blob(MX(PATA), { fill: P.blanco, stroke: P.blanco, width: BW, join: "round" }))
part("casco", () => blob(CASCO, { fill: P.blanco, stroke: P.blanco, width: BW }))

// ---------- púas ----------
function pua(pts, lado) {
  const f = blob(pts, { fill: P.lila })
  clip(f, () => {
    // sombra de un tono en la mitad inferior de la púa
    const c = center(pts)
    blob(shift(grow(pts, 0.9), 0, 26), { fill: P.lilaOsc, opacity: 0.85 })
    blob(shift(grow(pts, 0.55), lado * 10, -12), { fill: shade(P.lila, 0.15) })
  })
  ink(f, { width: 7, weight: 1.8, color: P.tinta })
  return f
}
function puasLado(lado) {
  const M = lado < 0 ? (p) => p : MX
  pua(M(PUA_A), lado)
  pua(M(PUA_B), lado)
  // brillos y muescas
  brush(M([[86, 150], [130, 168], [168, 190]]), { width: 6, color: P.lilaLuz, taper: [0.5, 0.5] })
  brush(M([[72, 448], [110, 440], [150, 436]]), { width: 7, color: P.lilaLuz, taper: [0.5, 0.5] })
  brush(M([[142, 228], [160, 240]]), { width: 6, color: P.tinta, taper: [0, 0.3] })
  brush(M([[146, 454], [152, 500]]), { width: 6, color: P.tinta, taper: [0.2, 0.3] })
}
part("pua_izq", () => puasLado(-1))
part("pua_der", () => puasLado(1))

// ---------- patas ----------
function pata(M) {
  const f = blob(M(PATA), { fill: P.verde })
  clip(f, () => {
    blob(M([[400, 640], [450, 650], [452, 748], [420, 740], [430, 700]]), { fill: P.verdeOsc, opacity: 0.8 })
    brush(M([[378, 680], [386, 712], [412, 730], [440, 740]]), { width: 9, color: "#9be06a", taper: [0.3, 0.5] })
  })
  ink(f, { width: 7, weight: 1.8, color: P.tinta })
}
part("pata_izq", () => pata((p) => p))
part("pata_der", () => pata(MX))

// ---------- casco ----------
function placa(pts, casco) {
  const f = blob(pts, { fill: P.lila, hidden: true })
  clip(casco, () => {
    const g = blob(pts, { fill: P.lila })
    clip(g, () => {
      blob(shift(grow(pts, 0.95), 30, 34), { fill: P.lilaOsc, opacity: 0.9 })
      const b = bbox(g)
      brush([[b.x + 40, b.y + 30], [b.cx, b.y + 22], [b.x + b.w - 30, b.y + 36]], { width: 8, color: P.lilaLuz, taper: [0.4, 0.5] })
    })
    ink(g, { width: 7, weight: 1.7, color: P.tinta })
  })
}
function hueco(cx, cy) {
  circle(cx, cy, 58, { fill: P.tinta })
  circle(cx, cy, 51, { fill: radial(cx - 15, cy - 15, 60, [P.aro, "#1f0b2a"]) })
  circle(cx, cy, 43, { fill: radial(cx + 6, cy + 6, 46, ["#2c2c32", P.hueco]) })
  curve(arc(cx, cy, 47, 200, 290, 8), { stroke: P.lilaLuz, width: 4, cap: "round" })
  circle(cx - 31, cy - 31, 4.5, { fill: "#f4eefa" })
}
function hexa(cx, cy, r = 14) {
  const pts = range(6).map((i) => polar(cx, cy, r, 30 + i * 60))
  poly(pts, { fill: P.verdeHex, stroke: P.tinta, width: 5, join: "round" })
  poly(grow(pts, 0.55), { fill: "#a8e070", opacity: 0.6 })
}

part("casco", () => {
  const c = blob(CASCO, { fill: P.verde })
  clip(c, () => {
    // sombra de un tono: lado derecho y parte baja
    blob([[700, 40], [880, 200], [890, 560], [780, 640], [760, 420], [770, 200]], { fill: P.verdeOsc, opacity: 0.9 })
    blob([[200, 560], [512, 610], [820, 560], [720, 720], [300, 720]], { fill: P.verdeOsc, opacity: 0.9 })
    // surcos que salen del núcleo
    const sur = [
      [[420, 318], [330, 250], [240, 210], [196, 160]],
      [[604, 318], [694, 250], [784, 210], [830, 160]],
      [[440, 300], [400, 230], [300, 160], [260, 120]],
      [[584, 300], [624, 230], [724, 160], [764, 120]],
      [[408, 420], [300, 470]],
      [[616, 420], [724, 470]],
    ]
    for (const s of sur) {
      brush(s, { width: 7, color: P.verdeOsc, taper: [0.2, 0.6] })
      brush(shift(s, 0, 6), { width: 3, color: P.verdeLuz, taper: [0.3, 0.6] })
    }
    // costura central de abajo
    brush([[512, 488], [512, 600], [514, 708]], { width: 9, color: P.tinta, taper: [0.1, 0] })
    // visera morada bajo la cúpula
    const v = blob(VISERA, { fill: "#401a5c" })
    clip(v, () => blob(shift(VISERA, 0, 26), { fill: "#2c1040" }))
    ink(v, { width: 6, weight: 1.6, color: P.tinta })
    brush([[432, 108], [426, 150], [432, 196]], { width: 6, color: P.tinta, taper: [0, 0] })
    brush([[594, 108], [598, 150], [592, 196]], { width: 6, color: P.tinta, taper: [0, 0] })
    // placas moradas
    placa(PLACA_M, c)
    placa(MX(PLACA_M), c)
    placa(PLACA_B, c)
    placa(MX(PLACA_B), c)
    // brillo grande del casco
    brush([[180, 330], [190, 250], [222, 180], [270, 128], [330, 92]], { width: [3, 13, 7, 2], color: "#f2f6ee", taper: [0.25, 0.4] })
  })
  ink(c, { width: 11, weight: 1.6, color: P.tinta })
  // hexágonos
  for (const [x, y] of [[497, 205], [527, 205], [482, 231], [512, 231], [542, 231], [497, 257], [527, 257]]) hexa(x, y)
  for (const [x, y] of [[355, 468], [340, 494], [370, 494], [669, 468], [654, 494], [684, 494]]) hexa(x, y)
  // marco del núcleo
  circle(512, 383, 108, { fill: P.tinta })
  circle(512, 383, 100, { fill: P.lila })
  curve(arc(512, 383, 95, 220, 320, 10), { stroke: P.lilaLuz, width: 5, cap: "round" })
  circle(512, 383, 91, { fill: P.tinta })
  // los seis huecos de las torretas
  for (const [x, y] of HUECOS) hueco(x, y)
})

// ---------- núcleo y placas ----------
part("nucleo", () => {
  circle(512, 383, 90, { fill: radial(512, 383, 90, [[0, "#ff8a5a"], [0.35, P.rojo], [1, P.rojoOsc]]) })
  circle(512, 383, 90, { fill: "none", stroke: P.tinta, width: 4 })
  // cruz encendida
  brush([[512, 300], [512, 383], [512, 466]], { width: [2, 12, 2], color: P.rojo, taper: [0.45, 0.45], glow: { blur: 8, color: "#ff5030" } })
  brush([[430, 383], [512, 383], [594, 383]], { width: [2, 12, 2], color: P.rojo, taper: [0.45, 0.45], glow: { blur: 8, color: "#ff5030" } })
  brush([[512, 320], [512, 446]], { width: 3, color: "#ffd0a0", taper: [0.4, 0.4] })
  brush([[450, 383], [574, 383]], { width: 3, color: "#ffd0a0", taper: [0.4, 0.4] })
  // botón central
  circle(512, 383, 17, { fill: P.tinta })
  circle(512, 383, 12, { fill: radial(508, 379, 12, ["#c070e8", P.lila]) })
  circle(508, 379, 3.5, { fill: "#f4eefa" })
})

function gajos(sx) {
  for (const sy of [-1, 1]) {
    const pts = gajo(sx, sy)
    const g = blob(pts, { fill: P.lila })
    clip(g, () => {
      blob(shift(pts, 10, 12), { fill: "#4a1f6c", opacity: 0.85, hidden: false, stroke: "none" })
      blob(shift(grow(pts, 0.92), -3, -3), { fill: P.lila })
      brush(sy < 0 ? [[512 + sx * 18, 383 - 70], [512 + sx * 40, 383 - 62]] : [[512 + sx * 20, 383 + 22], [512 + sx * 34, 383 + 24]], { width: 4, color: "#9a5cd0", taper: [0.4, 0.4] })
      // marca diagonal
      const a = [512 + sx * 22, 383 + sy * 22], b = [512 + sx * 52, 383 + sy * 52]
      brush([a, b], { width: 5, color: "#2c0f40", taper: [0.3, 0.3] })
      curve(arc(512, 383, 72, sx < 0 ? (sy < 0 ? 235 : 145) : (sy < 0 ? 325 : 55), sx < 0 ? (sy < 0 ? 260 : 165) : (sy < 0 ? 345 : 75), 5), { stroke: P.lilaLuz, width: 4, cap: "round" })
    })
    ink(g, { width: 5, weight: 1.6, color: P.tinta })
  }
}
part("placa_nucleo_izq", () => gajos(-1))
part("placa_nucleo_der", () => gajos(1))

// ---------- cúpula ----------
part("cupula", () => {
  const c = blob(CUPULA, { fill: "#441c62" })
  clip(c, () => {
    blob(shift(CUPULA, 0, 30), { fill: "#2e1044" })
    brush([[372, 70], [450, 52], [550, 48]], { width: [2, 9, 2], color: P.lilaLuz, taper: [0.4, 0.4] })
    brush([[420, 54], [500, 46]], { width: 4, color: "#f2eafa", taper: [0.4, 0.4] })
  })
  ink(c, { width: 7, weight: 1.6, color: P.tinta })
  brush([[418, 42], [412, 90], [434, 134]], { width: 6, color: P.tinta, taper: [0, 0] })
  brush([[606, 42], [614, 90], [598, 134]], { width: 6, color: P.tinta, taper: [0, 0] })
})

// ---------- acciones ----------
// El casco (raíz) nunca se mueve: así los seis huecos de las torretas quedan fijos.
// La respiración y la sacudida se hacen con las piezas que cuelgan de él.
action("motor", { duration: 1, loop: true }, [
  [0, { pata_izq: -12, pata_der: 12, pua_izq: -3, pua_der: 3, cupula: { rot: 0, y: 0 }, nucleo: 0 }],
  [0.25, { pua_izq: 2, pua_der: -2, cupula: { rot: 0, y: -2 } }],
  [0.5, { pata_izq: 14, pata_der: -14, pua_izq: 4, pua_der: -4, cupula: { rot: 0, y: -1 }, nucleo: 2 }],
  [0.75, { pua_izq: -1, pua_der: 1, cupula: { rot: 0, y: 1 } }],
  [1, { pata_izq: -12, pata_der: 12, pua_izq: -3, pua_der: 3, cupula: { rot: 0, y: 0 }, nucleo: 0 }],
])

action("abrir", { duration: 0.8, loop: false }, [
  [0, { placa_nucleo_izq: { rot: 0, x: 0, y: 0 }, placa_nucleo_der: { rot: 0, x: 0, y: 0 }, nucleo: { rot: 0, x: 0, y: 0 } }],
  [0.1, { placa_nucleo_izq: { rot: 0, x: -3, y: 1 }, placa_nucleo_der: { rot: 0, x: 3, y: -1 } }],
  [0.18, { placa_nucleo_izq: { rot: 0, x: 1, y: -1 }, placa_nucleo_der: { rot: 0, x: -1, y: 1 } }],
  [0.26, { placa_nucleo_izq: { rot: 0, x: -5, y: 1 }, placa_nucleo_der: { rot: 0, x: 5, y: -1 }, nucleo: { rot: 0, x: 0, y: 0 } }],
  [0.34, { placa_nucleo_izq: { rot: 0, x: -2, y: -1 }, placa_nucleo_der: { rot: 0, x: 2, y: 1 } }],
  [0.6, { placa_nucleo_izq: { rot: -6, x: -82, y: 0 }, placa_nucleo_der: { rot: 6, x: 82, y: 0 }, nucleo: { rot: 6, x: 0, y: -2 } }],
  [0.7, { placa_nucleo_izq: { rot: -3, x: -72, y: 0 }, placa_nucleo_der: { rot: 3, x: 72, y: 0 }, nucleo: { rot: -5, x: 0, y: 2 } }],
  [0.8, { placa_nucleo_izq: { rot: -4, x: -76, y: 0 }, placa_nucleo_der: { rot: 4, x: 76, y: 0 }, nucleo: { rot: 0, x: 0, y: 0 } }],
])

action("golpe", { duration: 0.25, loop: false }, [
  [0, { cupula: { rot: 0, x: 0, y: 0 }, pua_izq: 0, pua_der: 0, pata_izq: 0, pata_der: 0, placa_nucleo_izq: { x: 0, y: 0 }, placa_nucleo_der: { x: 0, y: 0 }, nucleo: { rot: 0 } }],
  [0.2, { cupula: { rot: 3, x: 6, y: -6 }, pua_izq: -10, pua_der: 10, pata_izq: 14, pata_der: -14, placa_nucleo_izq: { x: -3, y: -3 }, placa_nucleo_der: { x: 3, y: -3 }, nucleo: { rot: 8 } }],
  [0.45, { cupula: { rot: -2, x: -4, y: 3 }, pua_izq: 6, pua_der: -6, pata_izq: -8, pata_der: 8, placa_nucleo_izq: { x: 2, y: 2 }, placa_nucleo_der: { x: -2, y: 2 }, nucleo: { rot: -5 } }],
  [0.7, { cupula: { rot: 1, x: 2, y: -1 }, pua_izq: -2, pua_der: 2, pata_izq: 3, pata_der: -3, nucleo: { rot: 2 } }],
  [1, { cupula: { rot: 0, x: 0, y: 0 }, pua_izq: 0, pua_der: 0, pata_izq: 0, pata_der: 0, placa_nucleo_izq: { x: 0, y: 0 }, placa_nucleo_der: { x: 0, y: 0 }, nucleo: { rot: 0 } }],
])
