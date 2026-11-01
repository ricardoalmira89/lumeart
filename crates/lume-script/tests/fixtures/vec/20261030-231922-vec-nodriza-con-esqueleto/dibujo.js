// dibujo.js: nodriza vista desde arriba, mirando hacia abajo, con esqueleto.
canvas({ width: 1024, height: 768, background: null })

const P = {
  ink: "#0f2b17",        // contorno oscuro
  green: "#46a93a",      // cuerpo
  greenL: "#78d250",     // luces del cuerpo
  greenD: "#2b6b2c",     // sombra de un tono
  purple: "#7a3fb4",     // columna y compuertas
  purpleD: "#4a2172",
  purpleL: "#a978e6",
  orange: "#f39a1e",     // acentos
  orangeD: "#b5600f",
  lime: "#d9ff3c",       // cruz y ojo brillantes
  white: "#ffffff",
}
const OUT = 8      // grosor del contorno oscuro
const BORDER = 26  // borde blanco de pegatina

// x reflejada respecto al centro (512) para las piezas del lado derecho
const X = (s, x) => (s < 0 ? x : 1024 - x)
const pts = (s, l) => l.map(p => [X(s, p[0]), p[1], ...(p.length > 2 ? [p[2]] : [])])

rig({ origin: [512, 384] })
bone("cuerpo", { at: [512, 340] })
bone("ala_izq", { parent: "cuerpo", at: [272, 285] })
bone("ala_der", { parent: "cuerpo", at: [752, 285] })
bone("columna", { parent: "cuerpo", at: [512, 310] })
bone("cabeza", { parent: "cuerpo", at: [512, 590] })
bone("mandibula_izq", { parent: "cabeza", at: [395, 668] })
bone("mandibula_der", { parent: "cabeza", at: [629, 668] })

// ---------- alas membranosas ----------
function wing(s) {
  const W = pts(s, [[340, 172], [292, 150], [122, 104, 0.3], [74, 162, 0.3], [114, 238, 0], [88, 322, 0.3], [250, 405], [340, 400]])
  blob(W, { fill: P.white, stroke: P.white, width: BORDER })
  const w = blob(W, { fill: P.green, stroke: P.ink, width: OUT })
  clip(w, () => {
    // banda de sombra abajo y banda clara arriba
    blob(pts(s, [[60, 300], [250, 380], [360, 360], [360, 460], [60, 420]]), { fill: P.greenD })
    blob(pts(s, [[70, 150], [130, 128], [300, 170], [320, 200], [130, 160]]), { fill: P.greenL, opacity: 0.6 })
    // tira naranja del borde superior
    brush(pts(s, [[128, 124], [200, 136], [270, 155]]), { width: 12, color: P.orange, taper: [0.2, 0.6] })
    // nervios violetas desde la unión
    const base = [X(s, 262), 300]
    for (const [tx, ty, wd] of [[112, 124, 8], [108, 228, 9], [96, 312, 9], [200, 368, 8]]) {
      const tip = [X(s, tx), ty]
      const mid = [lerp(base[0], tip[0], 0.5) + (ty < 300 ? 0 : 0), lerp(base[1], tip[1], 0.5) + 8]
      brush([base, mid, tip], { width: [wd, wd * 0.7, 1], color: P.purpleD, taper: [0, 0.8] })
    }
    // brillo blanco
    brush(pts(s, [[125, 145], [180, 140], [240, 148]]), { width: 6, color: P.white, taper: [0.5, 0.5], opacity: 0.85 })
  })
}

// ---------- mandíbulas ----------
function jaw(s) {
  const J = pts(s, [[376, 640], [414, 650], [400, 688], [428, 718], [490, 734, 0], [488, 748, 0.3], [416, 746], [372, 722], [356, 678]])
  blob(J, { fill: P.white, stroke: P.white, width: BORDER })
  const j = blob(J, { fill: P.orange, stroke: P.ink, width: OUT })
  clip(j, () => {
    blob(pts(s, [[350, 640], [380, 650], [372, 700], [420, 730], [500, 740], [500, 760], [350, 760]]), { fill: P.orangeD })
    brush(pts(s, [[392, 702], [420, 726], [460, 736]]), { width: 6, color: P.white, taper: [0.3, 0.5], opacity: 0.9 })
  })
}

// ---------- cuerpo ----------
const BODY = [[512, 46], [618, 70], [708, 128], [762, 230], [776, 350], [758, 455], [712, 525], [640, 572], [512, 592], [384, 572], [312, 525], [266, 455], [248, 350], [262, 230], [316, 128], [406, 70]]
const SPIKE = [[478, 52], [512, 16, 0], [546, 52]]
const HATCHES = [
  { cx: 352, cy: 210, rx: 64, ry: 82, rot: -6 },
  { cx: 672, cy: 210, rx: 64, ry: 82, rot: 6 },
  { cx: 344, cy: 425, rx: 67, ry: 84, rot: -4 },
  { cx: 680, cy: 425, rx: 67, ry: 84, rot: 4 },
]

function sideSpike(s, y, len) {
  // púa naranja asomando por el lado del cuerpo
  const p = pts(s, [[300, y - 26], [300 - len, y, 0], [300, y + 26]])
  blob(p, { fill: P.orange, stroke: P.ink, width: OUT - 2 })
  blob(pts(s, [[300, y + 2], [300 - len + 6, y + 1, 0], [300, y + 24]]), { fill: P.orangeD, opacity: 0.7 })
}

function hatch(h) {
  group({ rotate: h.rot, around: [h.cx, h.cy] }, () => {
    ellipse(h.cx, h.cy, h.rx + 6, h.ry + 6, { fill: P.ink })
    const e = ellipse(h.cx, h.cy, h.rx - 2, h.ry - 2, { fill: radial(h.cx - 18, h.cy - 26, h.ry * 1.1, [P.purpleL, P.purple, [1, P.purpleD]]) })
    clip(e, () => {
      ellipse(h.cx + 20, h.cy + 26, h.rx, h.ry, { fill: P.purpleD, opacity: 0.55, blend: "multiply" })
      // cruz verde brillante
      for (const a of [-12, 78]) {
        const d = polar(0, 0, 100, a)
        line(h.cx - d[0], h.cy - d[1], h.cx + d[0], h.cy + d[1], { stroke: P.lime, width: 26, opacity: 0.35, blur: 8 })
        line(h.cx - d[0], h.cy - d[1], h.cx + d[0], h.cy + d[1], { stroke: P.greenD, width: 13 })
        line(h.cx - d[0], h.cy - d[1], h.cx + d[0], h.cy + d[1], { stroke: P.lime, width: 7 })
      }
      // remaches naranjas en los cuadrantes
      for (const a of [-57, 33, 123, 213]) {
        const q = [h.cx + Math.cos((a + 0) * Math.PI / 180) * h.rx * 0.55, h.cy + Math.sin(a * Math.PI / 180) * h.ry * 0.55]
        circle(q[0], q[1], 8, { fill: P.ink })
        circle(q[0], q[1], 5, { fill: P.orange })
      }
      // brillo blanco arriba a la izquierda
      brush(arc(h.cx, h.cy, h.rx * 0.72, -95, -150, 6), { width: 9, color: P.white, taper: [0.4, 0.5], opacity: 0.9 })
    })
    circle(h.cx, h.cy, 26, { fill: P.lime, opacity: 0.55, blur: 9 })
    circle(h.cx, h.cy, 9, { fill: P.lime, stroke: P.ink, width: 3 })
    circle(h.cx, h.cy, 4, { fill: P.ink })
  })
}

function body() {
  // borde blanco solo donde no tapa a las alas ni a la cabeza
  clip([rect(0, 0, 1024, 140, { hidden: true }), rect(0, 405, 1024, 160, { hidden: true })], () => {
    blob(BODY, { fill: P.white, stroke: P.white, width: BORDER })
    blob(SPIKE, { fill: P.white, stroke: P.white, width: BORDER })
  })
  for (const s of [-1, 1]) {
    blob(pts(s, [[300, 452], [236, 480, 0], [300, 504]]), { fill: P.white, stroke: P.white, width: BORDER })
    sideSpike(s, 300, 92); sideSpike(s, 478, 64)
  }
  blob(SPIKE, { fill: P.greenD, stroke: P.ink, width: OUT })
  const b = blob(BODY, { fill: P.green, stroke: P.ink, width: OUT })
  clip(b, () => {
    // sombra de un tono a la derecha y abajo
    blob([[640, 40], [800, 100], [820, 600], [560, 640], [690, 470], [735, 300], [700, 140]], { fill: P.greenD })
    // franjas: surco oscuro y luz debajo
    for (const y of [100, 166, 234, 292, 352, 425, 495, 552]) {
      brush([[230, y - 16], [512, y + 6], [800, y - 16]], { width: 18, color: P.greenL, taper: [0.1, 0.1], opacity: 0.55 })
      brush([[230, y - 26], [512, y - 4], [800, y - 26]], { width: [5, 9, 9, 5], color: P.ink, taper: [0.05, 0.05] })
    }
    // brillos blancos
    brush([[340, 100], [390, 80], [440, 74]], { width: 10, color: P.white, taper: [0.3, 0.6], opacity: 0.85 })
    brush([[270, 320], [262, 360], [266, 395]], { width: 7, color: P.white, taper: [0.4, 0.4], opacity: 0.8 })
  })
  for (const h of HATCHES) hatch(h)
}

// ---------- columna ----------
function column() {
  const C = [[512, 56], [548, 120], [566, 250], [566, 400], [552, 520], [512, 562], [472, 520], [458, 400], [458, 250], [476, 120]]
  const c = blob(C, { fill: P.purple, stroke: P.ink, width: OUT })
  clip(c, () => {
    rect(512, 0, 80, 600, { fill: P.purpleD })
    for (const y of [166, 236, 296, 366, 436, 502]) {
      brush([[450, y - 4], [512, y + 6], [574, y - 4]], { width: [4, 7, 4], color: P.ink, taper: [0, 0] })
    }
    brush([[490, 110], [480, 250], [480, 400], [488, 510]], { width: [2, 7, 7, 2], color: P.purpleL, taper: [0.3, 0.3] })
  })
  for (const y of [190, 330, 470]) {
    circle(512, y, 19, { fill: P.orange, stroke: P.ink, width: 6 })
    clip(circle(512, y, 16, { hidden: true }), () => circle(518, y + 6, 15, { fill: P.orangeD }))
    circle(506, y - 5, 5, { fill: P.white })
  }
}

// ---------- cabeza ----------
const HEAD = [[340, 608], [390, 566], [512, 548], [634, 566], [686, 608], [670, 664], [620, 702], [512, 716], [404, 702], [354, 664]]
function head() {
  clip([rect(0, 585, 1024, 70, { hidden: true }), rect(478, 585, 68, 200, { hidden: true })], () => blob(HEAD, { fill: P.white, stroke: P.white, width: BORDER }))
  const h = blob(HEAD, { fill: P.green, stroke: P.ink, width: OUT })
  clip(h, () => {
    blob([[600, 640], [700, 600], [700, 720], [512, 730], [600, 690]], { fill: P.greenD })
    brush([[360, 640], [380, 680], [430, 705]], { width: 10, color: P.greenL, opacity: 0.6, taper: [0.3, 0.3] })
  })
  // casco violeta
  const cap = blob([[356, 604, 0.5], [400, 574], [512, 558], [624, 574], [668, 604, 0.5], [590, 612], [512, 640], [434, 612]], { fill: P.purple, stroke: P.ink, width: OUT - 2 })
  clip(cap, () => {
    blob([[350, 600], [440, 596], [512, 610], [584, 596], [680, 600], [680, 660], [340, 660]], { fill: P.purpleD })
    brush([[400, 584], [470, 570], [540, 568]], { width: 6, color: P.purpleL, taper: [0.3, 0.6] })
  })
  // ojillos naranjas
  for (const s of [-1, 1]) {
    const [ex, ey] = [X(s, 394), 630]
    ellipse(ex, ey, 18, 12, { fill: P.orange, stroke: P.ink, width: 5, rotate: s * -20 })
    circle(ex + 4, ey - 3, 4, { fill: P.white })
  }
  // ojo central con dientes naranjas
  circle(512, 680, 42, { fill: P.greenD, stroke: P.ink, width: 6 })
  repeat_radial(512, 680, 8, (i) => {
    poly([[503, 640], [512, 650], [521, 640]], { fill: P.orange, stroke: P.ink, width: 2.5 })
  })
  circle(512, 680, 30, { fill: "#3d8a34", stroke: P.ink, width: 3 })
  circle(512, 680, 26, { fill: P.lime, opacity: 0.5, blur: 6 })
  circle(512, 680, 19, { fill: P.lime })
  circle(508, 676, 8, { fill: "#f6ffd0" })
}

layer("nodriza", () => {
  part("ala_izq", () => wing(-1))
  part("ala_der", () => wing(1))
  part("mandibula_izq", () => jaw(-1))
  part("mandibula_der", () => jaw(1))
  part("cuerpo", () => body())
  part("columna", () => column())
  part("cabeza", () => head())
})

// El cuerpo (y con él las cuatro compuertas) nunca se mueve en ninguna acción.
action("latir", { duration: 1.4, loop: true }, [
  [0, { ala_izq: 0, ala_der: 0, mandibula_izq: 0, mandibula_der: 0, columna: { y: 0 }, cabeza: { y: 0 } }],
  [0.12, { columna: { y: -4 } }],
  [0.24, { columna: { y: 1 } }],
  [0.36, { columna: { y: -2.5 } }],
  [0.25, { mandibula_izq: 9, mandibula_der: -9 }],
  [0.5, { ala_izq: 7, ala_der: -7, mandibula_izq: -2, mandibula_der: 2, columna: { y: 0 }, cabeza: { y: 2 } }],
  [0.75, { mandibula_izq: 9, mandibula_der: -9 }],
  [1, { ala_izq: 0, ala_der: 0, mandibula_izq: 0, mandibula_der: 0, columna: { y: 0 }, cabeza: { y: 0 } }],
].sort((a, b) => a[0] - b[0]))

// Sacudida: alas, cabeza, mandíbulas y columna tiemblan; el cuerpo queda fijo.
action("golpe", { duration: 0.25, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, cabeza: { rot: 0, x: 0, y: 0 }, columna: { x: 0, y: 0 }, mandibula_izq: 0, mandibula_der: 0 }],
  [0.2, { ala_izq: -8, ala_der: 8, cabeza: { rot: 4, x: 7, y: -5 }, columna: { x: -4, y: -3 }, mandibula_izq: 14, mandibula_der: -14 }],
  [0.45, { ala_izq: 5, ala_der: -5, cabeza: { rot: -3, x: -6, y: 2 }, columna: { x: 3, y: 1 }, mandibula_izq: -4, mandibula_der: 4 }],
  [0.7, { ala_izq: -2, ala_der: 2, cabeza: { rot: 1, x: 2, y: 0 }, columna: { x: -1, y: 0 } }],
  [1, { ala_izq: 0, ala_der: 0, cabeza: { rot: 0, x: 0, y: 0 }, columna: { x: 0, y: 0 }, mandibula_izq: 0, mandibula_der: 0 }],
])
