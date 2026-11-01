// Nave con módulo Burbuja: vista cenital, morro hacia arriba, estilo pegatina.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  ol: "#1c2244",      // contorno oscuro
  white: "#f3f6fb",   // casco
  lav: "#a9b5d8",     // sombra del casco
  teal: "#1bb3a6",    // alas y franjas
  tealD: "#11727e",   // banda oscura del ala
  tealL: "#8eebd9",   // brillos del ala
  cyan: "#5fe1fb",    // cabina
  blue: "#2a8be0",    // sombra de la cabina
  orange: "#ff8a1f",  // tomas de aire y detalles
  orangeD: "#a3401c",
  yellow: "#ffc531",  // núcleo
  pink: "#ff5fae",    // arma Burbuja
  pinkD: "#d6307f",
  pinkL: "#ffc2e2",
  nozzle: "#2b2f5a",
  nozzleL: "#4c5799",
}
const LW = 13   // grosor del contorno
const WB = 62   // grosor del borde blanco de pegatina

// Utilidades de simetría (todas las piezas se definen en el lado izquierdo)
const mx = (pts) => pts.map((p) => [1024 - p[0], ...p.slice(1)])
const sh = (pts, dx) => pts.map((p) => [p[0] + dx, ...p.slice(1)])
const side = (s) => (x) => (s < 0 ? x : 1024 - x)

// Pieza: relleno, detalles recortados dentro y contorno encima
function piece(make, details) {
  const s = make({ fill: P.white })
  if (details) clip(s, details)
  make({ fill: "none", stroke: P.ol, width: LW, join: "round" })
  return s
}
const WHITE = { fill: "#ffffff", stroke: "#ffffff", width: WB, join: "round" }

// ---------- Geometría ----------
const FUS = [[512, 170], [550, 176, 0.5], [574, 222], [592, 320], [612, 470], [621, 620], [616, 760], [596, 815], [556, 838], [512, 845], [468, 838], [428, 815], [408, 760], [403, 620], [412, 470], [432, 320], [450, 222], [474, 176, 0.5]]
const WING_L = [[492, 328, 0], [214, 522, 0], [214, 728, 0], [492, 728, 0]]
const GOND_L = [[196, 436], [244, 448], [262, 500], [266, 620], [258, 780], [248, 820, 0.5], [198, 826], [148, 820, 0.5], [136, 780], [128, 620], [130, 500], [148, 448]]
const PLATE_L = [[446, 418, 0.5], [326, 500, 0.5], [312, 612, 0.5], [446, 628, 0.5]]
const PETAL_L = [[496, 200, 0.5], [458, 190], [438, 150], [450, 92, 0], [480, 110], [496, 140]]
const DEP = { x: 392, y: 300, r: 44 }
const EMI = { x: 512, y: 146, r: 58 }
const BUB = { x: 512, y: 96, r: 40 }
const FLAMES = [
  { name: "llama_izq", cx: 436, w: 17, tip: 960 },
  { name: "llama_centro", cx: 512, w: 25, tip: 994 },
  { name: "llama_der", cx: 588, w: 17, tip: 960 },
]

// ---------- Esqueleto ----------
rig({ origin: [512, 540] })
bone("fuselaje", { at: [512, 540] })
bone("ala_izq", { parent: "fuselaje", at: [470, 470] })
bone("ala_der", { parent: "fuselaje", at: [554, 470] })
bone("gondola_izq", { parent: "ala_izq", at: [197, 630] })
bone("gondola_der", { parent: "ala_der", at: [827, 630] })
for (const f of FLAMES) bone(f.name, { parent: "fuselaje", at: [f.cx, 900] })
bone("placa_izq", { parent: "fuselaje", at: [470, 470] })
bone("placa_der", { parent: "fuselaje", at: [554, 470] })
bone("deposito_izq", { parent: "fuselaje", at: [DEP.x, DEP.y] })
bone("deposito_der", { parent: "fuselaje", at: [1024 - DEP.x, DEP.y] })
bone("emisor", { parent: "fuselaje", at: [EMI.x, EMI.y] })
bone("burbuja", { parent: "fuselaje", at: [512, 136] })

// ---------- Formas de cada pieza ----------
const flamePts = (f, k = 1) => [
  [f.cx - f.w, 860], [f.cx + f.w, 860], [f.cx + f.w * 0.95, 905],
  [f.cx + f.w * 0.55 * k, f.tip - (f.tip - 905) * 0.45], [f.cx, f.tip, 0],
  [f.cx - f.w * 0.55 * k, f.tip - (f.tip - 905) * 0.45], [f.cx - f.w * 0.95, 905],
]
const tubePts = (s) => {
  const X = side(s)
  return [[X(DEP.x), DEP.y - 40], [X(DEP.x + 6), 214], [X(438), 168], [X(470), 152]]
}
const bracket = (s, st) => {
  const X = side(s)
  return rect(s < 0 ? DEP.x + 14 : X(DEP.x + 14) - 48, DEP.y - 15, 48, 30, { r: 9, ...st })
}

function drawFlame(f) {
  blob(flamePts(f), { fill: P.orange, glow: { blur: 18, color: alpha("#ff7a1a", 0.8) } })
  blob(flamePts(f, 0.7).map((p, i) => i === 4 ? [p[0], p[1] - (f.tip - 905) * 0.35, 0] : p),
    { fill: "#ffd23a" })
  ellipse(f.cx, 912, f.w * 0.45, 14, { fill: "#fff6c8" })
}

function drawWing(s) {
  const X = side(s)
  const pts = s < 0 ? WING_L : mx(WING_L)
  const mk = (st) => poly(pts, st)
  const w = mk({ fill: P.teal })
  clip(w, () => {
    poly([[X(150), 644], [X(520), 630], [X(520), 760], [X(150), 760]], { fill: P.tealD })
    // franja de luz del borde de ataque
    poly([[X(492), 328], [X(214), 522], [X(214), 545], [X(492), 360]], { fill: alpha(P.tealL, 0.55) })
    brush([[X(272), 528], [X(416), 424]], { width: 9, color: alpha(P.tealL, 0.9), taper: [0.2, 0.6] })
    brush([[X(300), 520], [X(378), 470]], { width: 6, color: alpha(P.tealL, 0.8), taper: [0.3, 0.6] })
    // panel
    line(X(300), 560, X(300), 640, { stroke: alpha(P.tealD, 0.5), width: 3 })
  })
  mk({ fill: "none", stroke: P.ol, width: LW, join: "round" })
}

function drawGondola(s) {
  const dx = s < 0 ? 0 : 631
  const G = sh(GOND_L, dx)
  const mk = (st) => blob(G, st)
  piece(mk, () => {
    blob(sh([[208, 430, 0], [300, 430, 0], [300, 850, 0], [100, 850, 0], [100, 748, 0], [196, 744, 0.5], [218, 700], [220, 560], [210, 470]], dx), { fill: P.lav })
    brush(sh([[150, 520], [146, 610], [152, 690]], dx), { width: 6, color: "#ffffff", taper: [0.4, 0.4] })
    rect(100 + dx, 690, 200, 26, { fill: P.teal })
    rect(216 + dx, 690, 80, 26, { fill: "#14968f" })
  })
  // toma de aire
  const cx = 196 + dx
  ellipse(cx, 487, 37, 27, { fill: P.ol })
  const io = ellipse(cx, 487, 26, 17, { fill: P.orange })
  clip(io, () => {
    ellipse(cx, 500, 30, 12, { fill: P.orangeD })
    brush([[cx - 16, 480], [cx, 474], [cx + 14, 478]], { width: 4, color: "#ffd9a8", taper: [0.3, 0.3] })
  })
}

function drawPlate(s) {
  const X = side(s)
  const pts = s < 0 ? PLATE_L : mx(PLATE_L)
  const mk = (st) => blob(pts, st)
  piece(mk, () => {
    poly([[X(280), 592], [X(460), 600], [X(460), 650], [X(280), 650]], { fill: P.lav })
    blob(s < 0 ? grow(PLATE_L, 0.82) : mx(grow(PLATE_L, 0.82)), { fill: "none", stroke: alpha(P.lav, 0.9), width: 3 })
    // líneas de energía
    for (const o of [0, 30, 60]) {
      const pts = [[X(446), 448 + o], [X(372), 498 + o], [X(372), 520 + o]]
      polyline(pts, { stroke: P.pink, width: 9, cap: "round", join: "round", glow: { blur: 6, color: alpha(P.pink, 0.7) } })
      polyline(pts, { stroke: P.pinkL, width: 3, cap: "round", join: "round" })
    }
  })
  for (const [x, y] of [[420, 446], [334, 514], [330, 596], [424, 608]]) {
    circle(X(x), y, 6, { fill: P.orange, stroke: P.ol, width: 3 })
  }
}

function drawFuselage() {
  // toberas
  for (const [x, y, w, h] of [[405, 812, 64, 98], [555, 812, 64, 98]]) {
    const n = rect(x, y, w, h, { r: 12, fill: P.nozzle, stroke: P.ol, width: 10 })
    clip(n, () => rect(x + 10, y, 9, h, { fill: P.nozzleL }))
  }
  const c = rect(461, 828, 102, 96, { r: 14, fill: P.nozzle, stroke: P.ol, width: 10 })
  clip(c, () => {
    rect(474, 828, 10, 96, { fill: P.nozzleL })
    rect(461, 900, 102, 30, { fill: "#20234a" })
  })
  const mk = (st) => blob(FUS, st)
  piece(mk, () => {
    blob([[588, 290, 0], [720, 290, 0], [720, 900, 0], [580, 900, 0], [584, 770], [600, 650], [596, 470]], { fill: P.lav })
    poly([[380, 792], [470, 778], [560, 772], [610, 760], [720, 760], [720, 900], [380, 900]], { fill: P.lav })
    // collar verde azulado bajo el emisor
    poly([[380, 214], [512, 194], [650, 214], [650, 240], [512, 222], [380, 240]], { fill: P.teal })
    poly([[560, 200], [650, 214], [650, 240], [560, 226]], { fill: "#14968f" })
    brush([[424, 450], [416, 600], [428, 730]], { width: 7, color: "#ffffff", taper: [0.4, 0.4] })
  })
  // cabina
  const cab = ellipse(513, 316, 50, 96, { fill: P.cyan })
  clip(cab, () => {
    ellipse(560, 330, 26, 110, { fill: P.blue })
    brush([[492, 268], [484, 300], [486, 315]], { width: 11, color: "#ffffff", taper: [0.2, 0.5] })
    circle(498, 346, 5, { fill: "#ffffff" })
  })
  ellipse(513, 316, 50, 96, { fill: "none", stroke: P.ol, width: LW })
  // núcleo amarillo
  circle(513, 540, 56, { fill: P.ol })
  circle(513, 540, 45, { fill: "#1e8a8a" })
  circle(513, 540, 40, { fill: "#fff1b0" })
  const core = circle(513, 540, 36, { fill: radial(500, 525, 50, ["#ffd84a", "#f08a00"]) })
  clip(core, () => {
    circle(500, 527, 10, { fill: "#ffffff" })
    circle(528, 556, 4, { fill: "#ffffff" })
  })
}

function drawDepot(s) {
  const X = side(s)
  const cx = X(DEP.x)
  // tubo blanco hacia el emisor
  curve(tubePts(s), { stroke: P.ol, width: 26, cap: "round" })
  curve(tubePts(s), { stroke: P.white, width: 12, cap: "round" })
  curve(tubePts(s).map((p) => [p[0] + 2, p[1] + 3]), { stroke: alpha(P.lav, 0.8), width: 3, cap: "round" })
  // soporte al fuselaje
  const b = bracket(s, { fill: P.white, stroke: P.ol, width: 9 })
  clip(b, () => rect(s < 0 ? DEP.x + 14 : X(DEP.x + 14) - 48, DEP.y + 4, 48, 14, { fill: P.lav }))
  // tapa
  rect(cx - 16, DEP.y - DEP.r - 12, 32, 20, { r: 6, fill: P.white, stroke: P.ol, width: 8 })
  // cristal
  const g = circle(cx, DEP.y, DEP.r, { fill: "#dff3ff" })
  clip(g, () => {
    path(`M ${cx - 60} ${DEP.y - 2} Q ${cx - 30} ${DEP.y - 14} ${cx} ${DEP.y - 2} T ${cx + 60} ${DEP.y - 2} L ${cx + 60} ${DEP.y + 60} L ${cx - 60} ${DEP.y + 60} Z`,
      { fill: linear(0, DEP.y - 10, 0, DEP.y + 44, [P.pink, P.pinkD]) })
    ellipse(cx + 26, DEP.y + 18, 18, 30, { fill: alpha("#7a1f5a", 0.35) })
    circle(cx - 12, DEP.y + 18, 6, { fill: alpha("#ffffff", 0.6) })
    circle(cx + 6, DEP.y + 30, 4, { fill: alpha("#ffffff", 0.5) })
    circle(cx - 2, DEP.y + 8, 3, { fill: alpha("#ffffff", 0.6) })
    brush(arc(cx, DEP.y, DEP.r - 12, 220, 300, 6), { width: 7, color: "#ffffff", taper: [0.3, 0.3] })
  })
  circle(cx, DEP.y, DEP.r, { fill: "none", stroke: P.ol, width: LW })
}

function drawEmitter() {
  // mitades del morro abiertas
  for (const s of [-1, 1]) {
    const pts = s < 0 ? PETAL_L : mx(PETAL_L)
    const X = side(s)
    piece((st) => blob(pts, st), () => {
      poly([[X(400), 116], [X(500), 136], [X(500), 168], [X(400), 148]], { fill: P.teal })
      if (s > 0) poly([[X(400), 116], [X(500), 136], [X(500), 168], [X(400), 148]], { fill: "#14968f" })
      blob(s < 0 ? [[420, 200], [446, 150], [446, 90], [430, 90], [420, 150]] : mx([[420, 200], [446, 150], [446, 90], [430, 90], [420, 150]]), { fill: s < 0 ? "#ffffff" : alpha(P.lav, 0.9) })
    })
  }
  // carcasa redonda
  piece((st) => circle(EMI.x, EMI.y, EMI.r, st), () => {
    circle(EMI.x + 22, EMI.y + 18, EMI.r, { fill: P.lav })
    circle(EMI.x - 4, EMI.y - 4, EMI.r - 4, { fill: P.white })
  })
  // anillo rosa
  circle(EMI.x, EMI.y, 44, { fill: P.pink, stroke: P.ol, width: 9 })
  const ring = circle(EMI.x, EMI.y, 40, { fill: "none" })
  brush(arc(EMI.x, EMI.y, 38, 230, 330, 6), { width: 5, color: P.pinkL, taper: [0.3, 0.3] })
  // boca
  circle(EMI.x, EMI.y, 29, { fill: radial(EMI.x, EMI.y, 30, ["#ff9ad0", "#7a1f5a", "#2a1d4a"]), stroke: P.ol, width: 7 })
  // luces naranjas
  for (const s of [-1, 1]) circle(EMI.x + s * 48, EMI.y + 26, 6, { fill: P.orange, stroke: P.ol, width: 3 })
}

function drawBubble() {
  const b = circle(BUB.x, BUB.y, BUB.r, {
    fill: radial(BUB.x - 12, BUB.y - 12, BUB.r * 1.3, ["#fff0f8", "#ffb3dc", "#ff6fb8", "#c46bff"]),
    opacity: 0.95,
  })
  clip(b, () => {
    curve(arc(BUB.x, BUB.y, BUB.r - 4, 30, 150, 8), { stroke: alpha("#7fe8ff", 0.8), width: 6 })
    curve(arc(BUB.x, BUB.y, BUB.r - 6, 160, 220, 6), { stroke: alpha("#fff07a", 0.8), width: 5 })
  })
  circle(BUB.x, BUB.y, BUB.r, { fill: "none", stroke: P.ol, width: 9 })
  brush(arc(BUB.x, BUB.y, BUB.r - 11, 285, 345, 6), { width: 7, color: "#ffffff", taper: [0.3, 0.3] })
  circle(BUB.x + 16, BUB.y - 20, 4, { fill: "#ffffff" })
}

// ---------- Pasada 1: borde blanco de pegatina (detrás de todo) ----------
for (const f of FLAMES) part(f.name, () => blob(flamePts(f), { ...WHITE, width: WB + 20 }))
part("ala_izq", () => poly(WING_L, WHITE))
part("ala_der", () => poly(mx(WING_L), WHITE))
part("gondola_izq", () => blob(GOND_L, WHITE))
part("gondola_der", () => blob(sh(GOND_L, 631), WHITE))
part("fuselaje", () => {
  blob(FUS, WHITE)
  rect(405, 812, 214, 112, { r: 14, ...WHITE })
})
part("placa_izq", () => blob(PLATE_L, WHITE))
part("placa_der", () => blob(mx(PLATE_L), WHITE))
for (const s of [-1, 1]) {
  part(s < 0 ? "deposito_izq" : "deposito_der", () => {
    curve(tubePts(s), { stroke: "#ffffff", width: 26 + WB, cap: "round" })
    circle(side(s)(DEP.x), DEP.y, DEP.r, WHITE)
    bracket(s, WHITE)
  })
}
part("emisor", () => {
  blob(PETAL_L, WHITE)
  blob(mx(PETAL_L), WHITE)
  circle(EMI.x, EMI.y, EMI.r, WHITE)
})
part("burbuja", () => circle(BUB.x, BUB.y, BUB.r, WHITE))

// ---------- Pasada 2: piezas, de atrás hacia delante ----------
for (const f of FLAMES) part(f.name, () => drawFlame(f))
part("ala_izq", () => drawWing(-1))
part("gondola_izq", () => drawGondola(-1))
part("ala_der", () => drawWing(1))
part("gondola_der", () => drawGondola(1))
part("placa_izq", () => drawPlate(-1))
part("placa_der", () => drawPlate(1))
part("fuselaje", () => drawFuselage())
part("deposito_izq", () => drawDepot(-1))
part("deposito_der", () => drawDepot(1))
part("emisor", () => drawEmitter())
part("burbuja", () => drawBubble())

// ---------- Acciones ----------
const FOLD = 26
action("plegar", { duration: 1, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, placa_izq: 0, placa_der: 0 }],
  [0.5, { ala_izq: -FOLD * 0.5, ala_der: FOLD * 0.5, placa_izq: -FOLD * 0.5, placa_der: FOLD * 0.5 }],
  [1, { ala_izq: -FOLD, ala_der: FOLD, placa_izq: -FOLD, placa_der: FOLD }],
])

action("motor", { duration: 0.4, loop: true }, [
  [0, { llama_izq: { y: 0 }, llama_centro: { y: 14 }, llama_der: { y: 26 }, burbuja: -3 }],
  [0.33, { llama_izq: { y: 26 }, llama_centro: { y: 0 }, llama_der: { y: 14 } }],
  [0.5, { burbuja: 3 }],
  [0.67, { llama_izq: { y: 14 }, llama_centro: { y: 26 }, llama_der: { y: 0 } }],
  [1, { llama_izq: { y: 0 }, llama_centro: { y: 14 }, llama_der: { y: 26 }, burbuja: -3 }],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ala_izq: 0, ala_der: 0, placa_izq: 0, placa_der: 0, llama_izq: { y: 0 }, llama_centro: { y: 0 }, llama_der: { y: 0 } }],
  [0.35, { ala_izq: -16, ala_der: 16, placa_izq: -16, placa_der: 16, llama_izq: { y: 55 }, llama_centro: { y: 70 }, llama_der: { y: 55 } }],
  [1, { ala_izq: -11, ala_der: 11, placa_izq: -11, placa_der: 11, llama_izq: { y: 48 }, llama_centro: { y: 62 }, llama_der: { y: 48 } }],
])

action("acoplar", { duration: 0.7, loop: false }, [
  [0, {
    placa_izq: { x: -150, y: 20 }, placa_der: { x: 150, y: 20 },
    deposito_izq: { x: -150, y: -10 }, deposito_der: { x: 150, y: -10 },
    emisor: { y: -140 }, burbuja: { y: -140 },
  }],
  [0.22, { placa_izq: { x: 8, y: 0 }, placa_der: { x: -8, y: 0 } }],
  [0.3, { placa_izq: { x: 0, y: 0 }, placa_der: { x: 0, y: 0 }, deposito_izq: { x: -150, y: -10 }, deposito_der: { x: 150, y: -10 } }],
  [0.5, { deposito_izq: { x: 7, y: 0 }, deposito_der: { x: -7, y: 0 }, emisor: { y: -140 }, burbuja: { y: -140 } }],
  [0.58, { deposito_izq: { x: 0, y: 0 }, deposito_der: { x: 0, y: 0 } }],
  [0.82, { emisor: { y: 9 }, burbuja: { y: 9 } }],
  [0.91, { emisor: { y: -3 }, burbuja: { y: -5 } }],
  [1, { emisor: { y: 0 }, burbuja: { y: 0 } }],
])
