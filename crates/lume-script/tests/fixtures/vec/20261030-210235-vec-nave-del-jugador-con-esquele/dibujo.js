// dibujo.js: nave del jugador vista desde arriba, mirando hacia arriba, con esqueleto.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  ink: "#1b2340",      // contorno oscuro
  white: "#f4f7fc",    // fuselaje y góndolas
  shadeW: "#a9b5d6",   // sombra lavanda de lo blanco
  teal: "#19b3a7",     // alas y galones
  tealD: "#0e7180",    // banda oscura de las alas
  mint: "#93f0de",     // brillo de las alas
  cyan: "#5fe5fc",     // cabina
  blue: "#2189d0",     // lado en sombra de la cabina
  gold: "#ffc21c", goldL: "#ffe76e", goldD: "#f08a0a",
  orange: "#ff7a1a", orangeD: "#9a3a0e", yellow: "#ffd93a",
  noz: "#2a2f52", nozL: "#4f5989", nozD: "#151932",
  sticker: "#ffffff",
}
const LINE = 12   // grosor base del contorno
const BORDER = 64 // grosor del borde blanco (la mitad sobresale)
setLight(-1, -1)

// ---------- Siluetas (se usan para el color y para el borde blanco) ----------
const FUS = [[512, 56, 0.35], [548, 140], [582, 270], [607, 420], [619, 600], [615, 760], [594, 828],
  [512, 852], [430, 828], [409, 760], [405, 600], [417, 420], [442, 270], [476, 140]]

// ala izquierda: la raíz entra bien por debajo del fuselaje y la punta por debajo de la góndola
const WING_L = [[470, 344], [176, 548], [176, 724], [432, 724], [430, 430]]
const mx = (pts) => pts.map(([x, y, s]) => s === undefined ? [1024 - x, y] : [1024 - x, y, s])
const WING_R = mx(WING_L)

const gondolaPts = (cx) => [[cx, 437], [cx + 44, 455], [cx + 62, 520], [cx + 64, 640], [cx + 59, 762],
  [cx + 46, 816, 0.5], [cx, 823], [cx - 46, 816, 0.5], [cx - 60, 762], [cx - 64, 640], [cx - 62, 520], [cx - 44, 455]]
const GL = 196, GR = 828

// llama: columna oculta bajo la tobera y punta afilada abajo
const flamePts = (cx, top, mouth, w, tip) => [[cx - w, top], [cx + w, top], [cx + w, mouth], [cx + w * 0.7, mouth + (tip - mouth) * 0.45],
  [cx, tip, 0], [cx - w * 0.7, mouth + (tip - mouth) * 0.45], [cx - w, mouth]]
// la columna empieza dentro de la tobera (por debajo de su borde superior) y solo asoma por la boca;
// su largo oculto (boca de la tobera − top) es el máximo que puede bajar la llama sin abrir hueco
const FLAMES = { izq: [442, 844, 900, 24, 958], centro: [512, 846, 912, 34, 990], der: [582, 844, 900, 24, 958] }

// ---------- Esqueleto ----------
rig({ origin: [512, 540] })
bone("fuselaje", { at: [512, 540] })
bone("ala_izq", { parent: "fuselaje", at: [450, 378] })
bone("ala_der", { parent: "fuselaje", at: [574, 378] })
bone("gondola_izq", { parent: "ala_izq", at: [232, 600] })
bone("gondola_der", { parent: "ala_der", at: [792, 600] })
bone("llama_izq", { parent: "fuselaje", at: [442, 902] })
bone("llama_centro", { parent: "fuselaje", at: [512, 914] })
bone("llama_der", { parent: "fuselaje", at: [582, 902] })

// ---------- Borde blanco de pegatina (detrás de todo) ----------
const stick = { fill: P.sticker, stroke: P.sticker, width: BORDER, join: "round" }
for (const [k, f] of Object.entries(FLAMES)) part("llama_" + k, () => {
  const [cx, top, mouth, w, tip] = f
  blob(flamePts(cx, mouth - 10, mouth, w + 4, tip + 4), stick)
})
part("ala_izq", () => poly(WING_L, stick))
part("ala_der", () => poly(WING_R, stick))
part("gondola_izq", () => blob(gondolaPts(GL), stick))
part("gondola_der", () => blob(gondolaPts(GR), stick))
part("fuselaje", () => {
  blob(FUS, stick)
  rect(402, 820, 220, 104, { ...stick, r: 20 })
})

// ---------- Llamas ----------
function drawFlame(k) {
  const [cx, top, mouth, w, tip] = FLAMES[k]
  part("llama_" + k, () => {
    blob(flamePts(cx, top, mouth, w, tip), { fill: P.orange, glow: { blur: 18, color: alpha(P.orange, 0.7) } })
    blob(flamePts(cx, top, mouth, w * 0.55, mouth + (tip - mouth) * 0.62), { fill: P.yellow })
    blob(flamePts(cx, top, mouth, w * 0.22, mouth + (tip - mouth) * 0.3), { fill: "#fff6c8" })
  })
}
drawFlame("izq"); drawFlame("der"); drawFlame("centro")

// ---------- Alas ----------
function drawWing(side) {
  const pts = side === "izq" ? WING_L : WING_R
  const s = side === "izq" ? 1 : -1
  const X = (x) => side === "izq" ? x : 1024 - x
  part("ala_" + side, () => {
    const w = poly(pts, { fill: P.teal, stroke: P.ink, width: LINE, join: "round" })
    clip(w, () => {
      // banda oscura de la parte trasera
      poly([[X(150), 642], [X(300), 638], [X(470), 630], [X(470), 760], [X(150), 760]], { fill: P.tealD })
      // brillo menta a lo largo del borde de ataque
      brush([[X(262), 528], [X(340), 474], [X(420), 420]], { width: [6, 16, 26], taper: [0.5, 0], color: P.mint })
      brush([[X(300), 520], [X(380), 466]], { width: 5, taper: [0.5, 0.5], color: alpha("#ffffff", 0.6) })
    })
  })
}
drawWing("izq"); drawWing("der")

// ---------- Góndolas ----------
function drawGondola(side) {
  const cx = side === "izq" ? GL : GR
  part("gondola_" + side, () => {
    const g = blob(gondolaPts(cx), { fill: P.white })
    clip(g, () => {
      // sombra lavanda: lado derecho y culata
      blob([[cx + 14, 440], [cx + 80, 440], [cx + 80, 840], [cx - 80, 840], [cx - 80, 752], [cx + 4, 742], [cx + 22, 690], [cx + 18, 560]], { fill: P.shadeW })
      // banda verde azulada
      rect(cx - 80, 690, 160, 26, { fill: P.teal })
      rect(cx + 20, 690, 60, 26, { fill: P.tealD, opacity: 0.55 })
      // brillo vertical
      brush([[cx - 40, 520], [cx - 44, 610], [cx - 42, 690]], { width: 10, taper: [0.4, 0.4], color: "#ffffff" })
    })
    blob(gondolaPts(cx), { fill: null, stroke: P.ink, width: LINE, join: "round" })
    // toma de aire naranja
    ellipse(cx, 487, 38, 29, { fill: P.ink })
    ellipse(cx, 485, 29, 20, { fill: P.orange })
    const hole = ellipse(cx, 485, 29, 20, { hidden: true })
    clip(hole, () => {
      ellipse(cx + 2, 500, 27, 14, { fill: P.orangeD })
      brush([[cx - 18, 476], [cx, 471], [cx + 16, 474]], { width: 4, color: alpha("#ffffff", 0.7) })
    })
  })
}
drawGondola("izq"); drawGondola("der")
// ---------- Fuselaje: toberas, casco, galones, cabina y núcleo ----------
part("fuselaje", () => {
  // toberas (laterales detrás de la central)
  const nozzle = (cx, w, y1, y2) => {
    const n = blob([[cx - w * 0.86, y1, 0.3], [cx + w * 0.86, y1, 0.3], [cx + w, y2, 0.3], [cx - w, y2, 0.3]],
      { fill: P.noz, stroke: P.ink, width: LINE * 0.8, join: "round" })
    clip(n, () => {
      rect(cx + w * 0.35, y1 - 10, w, y2 - y1 + 20, { fill: P.nozD, opacity: 0.8 })
      brush([[cx - w * 0.55, y1 + 6], [cx - w * 0.6, y2 - 10]], { width: 9, taper: [0, 0], color: P.nozL })
      ellipse(cx, y2 - 2, w * 0.95, 9, { fill: P.nozD })
    })
  }
  nozzle(442, 32, 838, 906)
  nozzle(582, 32, 838, 906)
  nozzle(512, 44, 842, 918)

  // casco
  const body = blob(FUS, { fill: P.white })
  clip(body, () => {
    // galón del morro
    poly([[512, 108], [640, 185], [640, 240], [512, 185], [384, 240], [384, 185]], { fill: P.teal })
    poly([[542, 120], [640, 185], [640, 240], [542, 200]], { fill: P.tealD, opacity: 0.45 })
    // sombra del lado derecho y de la cola
    blob([[548, 150, 0], [580, 300], [590, 460], [588, 620], [578, 760], [470, 778], [400, 800], [400, 880], [640, 880], [640, 140]], { fill: P.shadeW })
    // brillo largo a la izquierda
    brush([[438, 330], [428, 470], [426, 620], [440, 760]], { width: [4, 11, 8, 3], taper: [0.3, 0.3], color: "#ffffff" })
  })
  ink(body, { width: LINE, weight: 1.6, color: P.ink })

  // cabina
  const CAB = [[512, 216, 0.6], [541, 240], [557, 300], [556, 372], [540, 404], [512, 411], [484, 404], [468, 372], [467, 300], [483, 240]]
  const cab = blob(CAB, { fill: P.cyan })
  clip(cab, () => {
    blob([[522, 210], [600, 210], [600, 430], [522, 430], [536, 390], [544, 320], [536, 250]], { fill: P.blue })
    blob([[486, 272, 0.5], [496, 262], [497, 290], [488, 318, 0.5], [480, 312]], { fill: "#ffffff" })
    circle(498, 346, 5.5, { fill: "#ffffff" })
  })
  ink(cab, { width: LINE * 0.9, weight: 1.4, color: P.ink })

  // núcleo amarillo
  circle(513, 541, 56, { fill: P.ink })
  circle(513, 541, 46, { fill: P.tealD })
  circle(513, 541, 41, { fill: "#fff3b0" })
  circle(513, 541, 37, { fill: radial(500, 525, 46, [P.goldL, P.gold, P.goldD], { fx: 498, fy: 522 }) })
  circle(499, 527, 10, { fill: "#ffffff" })
  circle(527, 555, 4, { fill: "#ffffff", opacity: 0.85 })
})

// ---------- Acciones ----------
// alas: ángulo negativo en la izquierda (y positivo en la derecha) las lleva hacia atrás;
// las góndolas giran lo contrario para seguir mirando hacia arriba.
const wings = (a) => ({ ala_izq: -a, ala_der: a, gondola_izq: a, gondola_der: -a })

const FOLD = 31
action("plegar", { duration: 1, loop: false }, [
  [0, wings(0)],
  [0.25, wings(FOLD * 0.25)],
  [0.5, wings(FOLD * 0.5)],
  [0.75, wings(FOLD * 0.75)],
  [1, wings(FOLD)],
])

const fl = (y, rot = 0, x = 0) => ({ rot, x, y })
action("motor", { duration: 0.4, loop: true }, [
  [0, { llama_izq: fl(4, 0), llama_centro: fl(14, 1), llama_der: fl(20, -1) }],
  [0.25, { llama_izq: fl(18, 2, 1), llama_centro: fl(4, -1.5), llama_der: fl(8, 1, -1) }],
  [0.5, { llama_izq: fl(8, -1.5), llama_centro: fl(22, 1, 1), llama_der: fl(2, -2) }],
  [0.75, { llama_izq: fl(22, 1, -1), llama_centro: fl(8, -1, -1), llama_der: fl(16, 2, 1) }],
  [1, { llama_izq: fl(4, 0), llama_centro: fl(14, 1), llama_der: fl(20, -1) }],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ...wings(0), llama_izq: fl(0), llama_centro: fl(0), llama_der: fl(0) }],
  [0.3, { ...wings(6), llama_izq: fl(62, 1), llama_centro: fl(68, -1), llama_der: fl(62, -1) }],
  [0.65, { ...wings(-1.5), llama_izq: fl(48, -1), llama_centro: fl(60, 1), llama_der: fl(48, 1) }],
  // termina en la clave 0 de "motor" para enlazar con ella sin salto
  [1, { ...wings(0), llama_izq: fl(4, 0), llama_centro: fl(14, 1), llama_der: fl(20, -1) }],
])
