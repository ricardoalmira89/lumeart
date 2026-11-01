// dibujo.js: nave con módulo Centella (vista cenital, mirando hacia arriba)
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#1c2346",     // contorno
  white: "#f3f6fb",   // casco
  lav: "#a9b5d8",     // sombra del casco
  metal: "#d6dcec",   // piezas metálicas del arma
  teal: "#1bb5a8",    // alas
  tealD: "#10757f",   // banda baja de las alas
  tealL: "#8fe9d9",   // brillos de las alas
  cyan: "#5ee3f8",    // cabina
  blue: "#2a8be0",    // sombra de la cabina
  orange: "#f68a1e",  // tomas y remaches
  orangeD: "#9a3d12",
  yellow: "#ffd23a",
  navy: "#2b2f55",    // toberas
  elec: "#2f6bff",    // azul eléctrico del arma
  elecL: "#8fdcff",
}

// La nave se dibuja en coordenadas de la referencia y se encoge para dejar sitio a las antenas
const K = 0.8
const X = (x) => 512 + (x - 512) * K
const Y = (y) => 148 + y * K
const p = (x, y) => [X(x), Y(y)]
const s = (v) => v * K
const T = (pts) => pts.map((q) => (q.length > 2 ? [X(q[0]), Y(q[1]), q[2]] : [X(q[0]), Y(q[1])]))
const M = (pts) => pts.map((q) => [1024 - q[0], ...q.slice(1)])
const side = (pts, sd) => T(sd < 0 ? M(pts) : pts) // geometría del lado derecho, reflejada para el izquierdo
const sx = (x, sd) => (sd < 0 ? 1024 - x : x)

const WB = 46 // grosor del borde blanco de pegatina
const WH = (w = WB) => ({ fill: "#ffffff", stroke: "#ffffff", width: w, join: "round", cap: "round" })
const INK = (sh, w = 8) => ink(sh, { width: w, weight: 1.7, color: P.ink })

// ---------- Geometría (coordenadas de referencia, lado derecho) ----------
const G = {
  fuselaje: [[512, 55, 0], [545, 120], [580, 250], [605, 400], [616, 560], [614, 700], [605, 800], [575, 838], [512, 850], [449, 838], [419, 800], [410, 700], [408, 560], [419, 400], [444, 250], [479, 120]],
  ala: [[512, 420], [598, 370], [772, 485], [815, 500], [815, 725], [600, 725], [512, 700]],
  gondola: [[829, 435], [880, 450], [896, 520], [893, 700], [884, 815], [829, 826], [774, 815], [765, 700], [762, 520], [778, 450]],
  placa: [[606, 582], [704, 596], [728, 704], [606, 716]],
  llamaC: [[478, 860], [546, 860], [542, 935], [512, 995, 0], [482, 935]],
  llamaL: [[418, 850], [462, 850], [459, 912], [440, 960, 0], [421, 912]],
}
const ANT = { root: [528, 205], tip: [600, -58] }
const antAt = (t) => [lerp(ANT.root[0], ANT.tip[0], t), lerp(ANT.root[1], ANT.tip[1], t)]

// ---------- Esqueleto ----------
rig({ origin: p(512, 540) })
bone("fuselaje", { at: p(512, 540) })
bone("ala_izq", { parent: "fuselaje", at: p(470, 560) })
bone("ala_der", { parent: "fuselaje", at: p(554, 560) })
bone("gondola_izq", { parent: "ala_izq", at: p(195, 630) })
bone("gondola_der", { parent: "ala_der", at: p(829, 630) })
bone("llama_izq", { parent: "fuselaje", at: p(440, 905) })
bone("llama_centro", { parent: "fuselaje", at: p(512, 915) })
bone("llama_der", { parent: "fuselaje", at: p(584, 905) })
bone("placa_izq", { parent: "fuselaje", at: p(470, 560) })
bone("placa_der", { parent: "fuselaje", at: p(554, 560) })
bone("antena_izq", { parent: "fuselaje", at: p(1024 - 528, 205) })
bone("antena_der", { parent: "fuselaje", at: p(528, 205) })
bone("chispa", { parent: "fuselaje", at: p(512, -58) })
bone("bobina_izq", { parent: "fuselaje", at: p(1024 - 643, 480) })
bone("bobina_der", { parent: "fuselaje", at: p(643, 480) })

// ---------- 1. Borde blanco de pegatina (todas las siluetas, debajo de todo) ----------
layer("borde", () => {
  for (const sd of [-1, 1]) {
    const n = sd < 0 ? "izq" : "der"
    part("llama_" + n, () => blob(side(G.llamaL, sd), WH(40)))
    part("ala_" + n, () => poly(side(G.ala, sd), WH()))
    part("gondola_" + n, () => blob(side(G.gondola, sd), WH()))
    part("placa_" + n, () => poly(side(G.placa, sd), WH()))
    part("antena_" + n, () => {
      curve(side([ANT.root, ANT.tip], sd), { stroke: "#ffffff", width: s(16) + WB, cap: "round" })
      circle(X(sx(ANT.tip[0], sd)), Y(ANT.tip[1]), s(24) + WB / 2, { fill: "#ffffff" })
    })
    part("bobina_" + n, () => {
      rect(X(sx(sd < 0 ? 668 : 618, sd)), Y(372), s(50), s(196), { ...WH(), r: s(22) })
    })
  }
  part("llama_centro", () => blob(T(G.llamaC), WH(40)))
  part("fuselaje", () => {
    blob(T(G.fuselaje), WH())
    rect(X(405), Y(835), s(214), s(88), { ...WH(), r: s(12) })
  })
})

// ---------- 2. Llamas ----------
function llama(pts, sc) {
  blob(grow(pts, 1.25), { fill: alpha("#ff8a2a", 0.45), blur: 9 })
  const sh = blob(pts, { fill: P.orange })
  blob(shift(grow(pts, 0.62), 0, -s(8) * sc), { fill: P.yellow })
  blob(shift(grow(pts, 0.28), 0, -s(14) * sc), { fill: "#fff6c8" })
  return sh
}
layer("llamas", () => {
  part("llama_izq", () => llama(side(G.llamaL, -1), 1))
  part("llama_der", () => llama(side(G.llamaL, 1), 1))
  part("llama_centro", () => llama(T(G.llamaC), 1.3))
})

// ---------- 3. Alas y góndolas ----------
function ala(sd) {
  const sh = poly(side(G.ala, sd), { fill: P.teal, join: "round" })
  clip(sh, () => {
    poly(side([[490, 632], [830, 642], [830, 740], [490, 740]], sd), { fill: P.tealD })
    poly(side([[600, 388], [768, 500], [764, 530], [640, 446], [600, 428]], sd), { fill: P.tealL, opacity: 0.75 })
    brush(side([[652, 468], [726, 522]], sd), { width: s(5), color: P.tealL })
  })
  INK(sh)
}
function gondola(sd) {
  const sh = blob(side(G.gondola, sd), { fill: P.white })
  // la luz viene de arriba a la izquierda en las dos góndolas: la sombra se traslada, no se refleja
  const L = (pts) => T(sd < 0 ? shift(pts, -634, 0) : pts)
  clip(sh, () => {
    blob(L([[852, 440], [905, 450], [905, 840], [755, 840], [755, 745], [805, 750], [840, 732], [852, 690], [850, 560], [842, 470]]), { fill: P.lav })
    poly(L([[750, 690], [905, 690], [905, 716], [750, 716]]), { fill: P.teal })
    poly(L([[848, 690], [905, 690], [905, 716], [850, 716]]), { fill: "#14928f" })
    brush(L([[782, 540], [779, 650], [784, 720]]), { width: s(7), color: "#ffffff", opacity: 0.9 })
  })
  INK(sh)
  const cx = X(sx(829, sd)), cy = Y(487)
  ellipse(cx, cy, s(38), s(28), { fill: P.ink })
  const toma = ellipse(cx, cy - s(2), s(29), s(18), { fill: P.orange })
  clip(toma, () => ellipse(cx, cy + s(10), s(30), s(13), { fill: P.orangeD }))
  brush([[cx - s(14), cy - s(10)], [cx + s(6), cy - s(13)]], { width: s(5), color: "#ffe2b8" })
}
layer("alas", () => {
  for (const sd of [-1, 1]) {
    const n = sd < 0 ? "izq" : "der"
    part("ala_" + n, () => ala(sd))
    part("gondola_" + n, () => gondola(sd))
  }
})
// ---------- 4. Placas de blindaje ----------
const energia = (pts, w = 5) => {
  brush(pts, { width: s(w + 6), color: P.ink, taper: [0, 0] })
  brush(pts, { width: s(w), color: P.elec, taper: [0, 0] })
  brush(pts, { width: s(w * 0.35), color: P.elecL, taper: [0.2, 0.2] })
}
function placa(sd) {
  const sh = poly(side(G.placa, sd), { fill: P.metal, join: "round" })
  clip(sh, () => {
    poly(side([[600, 600], [740, 610], [740, 720], [600, 720], [600, 690], [716, 680], [700, 612]], sd), { fill: P.lav })
    poly(side([[600, 576], [710, 590], [706, 604], [600, 594]], sd), { fill: "#ffffff", opacity: 0.85 })
  })
  INK(sh, 7)
  energia(side([[620, 628], [688, 636], [702, 690]], sd))
  energia(side([[620, 660], [672, 664]], sd), 4)
  for (const [x, y] of [[702, 690], [672, 664]]) {
    const q = side([[x, y]], sd)[0]
    circle(q[0], q[1], s(8), { fill: P.elecL, stroke: P.ink, width: 3, glow: { blur: 6, color: P.elec } })
  }
  for (const [x, y] of [[619, 602], [692, 610], [712, 694], [619, 700]]) {
    const q = side([[x, y]], sd)[0]
    circle(q[0], q[1], s(6), { fill: P.orange, stroke: P.ink, width: 2.5 })
  }
}
layer("placas", () => {
  part("placa_izq", () => placa(-1))
  part("placa_der", () => placa(1))
})

// ---------- 5. Antenas y chispa ----------
function antena(sd) {
  const pts = side([ANT.root, ANT.tip], sd)
  const seg = (t0, t1) => side([antAt(t0), antAt(t1)], sd)
  curve(pts, { stroke: P.ink, width: s(18), cap: "round" })
  curve(pts, { stroke: P.metal, width: s(9), cap: "round" })
  curve(seg(0.3, 0.95), { stroke: "#ffffff", width: s(3), cap: "round", opacity: 0.9 })
  // collarín verde azulado y anillo naranja
  curve(seg(0.38, 0.5), { stroke: P.ink, width: s(28), cap: "round" })
  curve(seg(0.38, 0.5), { stroke: P.teal, width: s(18), cap: "round" })
  curve(seg(0.4, 0.44), { stroke: P.tealL, width: s(8), cap: "round" })
  curve(seg(0.78, 0.82), { stroke: P.ink, width: s(22), cap: "round" })
  curve(seg(0.78, 0.82), { stroke: P.orange, width: s(13), cap: "round" })
  // esfera
  const [tx, ty] = pts[1]
  circle(tx, ty, s(40), { fill: alpha(P.elecL, 0.55), blur: 10 })
  circle(tx, ty, s(24), {
    fill: radial(tx - s(8), ty - s(8), s(30), ["#e6f7ff", P.elecL, P.elec, "#1734a8"]),
    stroke: P.ink, width: 7,
  })
  circle(tx - s(8), ty - s(9), s(6), { fill: "#ffffff" })
  circle(tx + s(9), ty + s(7), s(2.5), { fill: "#ffffff", opacity: 0.8 })
}
layer("antenas", () => {
  part("antena_izq", () => antena(-1))
  part("antena_der", () => antena(1))
  part("chispa", () => {
    const zz = [[450, -58]]
    for (let i = 1; i < 9; i++) zz.push([450 + i * 15.5, -58 + (i % 2 ? -1 : 1) * rand(9, 16)])
    zz.push([574, -58])
    const q = T(zz)
    polyline(q, { stroke: alpha(P.elecL, 0.6), width: s(34), join: "round", blur: 8 })
    polyline(q, { stroke: P.elec, width: s(14), join: "miter", opacity: 0.8, blur: 1.5 })
    polyline(q, { stroke: P.elecL, width: s(7), join: "miter" })
    polyline(q, { stroke: "#ffffff", width: s(2.5), join: "miter" })
    for (const [x, y, a] of [[470, -76, -40], [552, -40, 30], [512, -82, -10]]) {
      const c = p(x, y)
      brush([polar(c[0], c[1], s(8), a), polar(c[0], c[1], s(-8), a)], { width: s(4), color: P.elecL })
    }
  })
})

// ---------- 6. Fuselaje ----------
layer("fuselaje", () => {
  part("fuselaje", () => {
    // toberas
    const tob = (x, y, w, h) => {
      const r = rect(X(x), Y(y), s(w), s(h), { fill: P.navy, r: s(10) })
      clip(r, () => {
        rect(X(x + w * 0.62), Y(y), s(w), s(h), { fill: "#1d2042" })
        rect(X(x + 9), Y(y + 8), s(6), s(h - 22), { fill: "#5a6199", r: 3 })
      })
      INK(r, 7)
    }
    tob(404, 832, 62, 80)
    tob(558, 832, 62, 80)
    tob(461, 846, 102, 76)

    const sh = blob(T(G.fuselaje), { fill: P.white })
    clip(sh, () => {
      poly(T([[380, 100], [512, 108], [640, 140], [640, 208], [512, 185], [380, 208]]), { fill: P.teal })
      poly(T([[540, 110], [640, 120], [640, 215], [544, 200]]), { fill: "#169a90" })
      blob(T([[586, 240], [614, 400], [628, 600], [624, 790], [512, 890], [395, 860], [412, 788], [480, 772], [560, 768], [592, 712], [598, 560], [592, 400], [575, 270]]), { fill: P.lav })
      brush(T([[428, 430], [420, 560], [424, 700]]), { width: s(8), color: "#ffffff" })
    })
    INK(sh, 9)
    // cabina
    const cab = blob(T([[512, 215], [545, 230], [562, 300], [560, 380], [540, 410], [512, 414], [484, 410], [464, 380], [462, 300], [479, 230]]), { fill: P.cyan })
    clip(cab, () => {
      blob(T([[538, 225], [570, 300], [565, 400], [520, 420], [540, 380], [548, 300]]), { fill: P.blue })
      brush(T([[492, 262], [484, 310]]), { width: s(11), color: "#ffffff", taper: [0.4, 0.4] })
      circle(X(498), Y(345), s(5), { fill: "#ffffff" })
    })
    INK(cab, 8)
    // núcleo amarillo
    circle(X(512), Y(540), s(56), { fill: P.ink })
    circle(X(512), Y(540), s(42), {
      fill: radial(X(500), Y(525), s(50), ["#ffe066", P.yellow, "#f59b12", "#e07a0c"]),
      stroke: "#fff1b0", width: 3,
    })
    circle(X(499), Y(526), s(10), { fill: "#ffffff" })
    circle(X(527), Y(555), s(4), { fill: "#ffffff" })
  })
})

// ---------- 7. Bobinas ----------
function bobina(sd) {
  const cx = X(sx(643, sd))
  const x0 = cx - s(25)
  // abrazadera al fuselaje
  const ab = rect(X(sx(sd < 0 ? 625 : 598, sd)), Y(446), s(27), s(64), { fill: P.navy, r: 4 })
  INK(ab, 5)
  // punta emisora
  const dome = ellipse(cx, Y(398), s(22), s(26), { fill: P.metal })
  INK(dome, 6)
  circle(cx, Y(386), s(7), { fill: P.elecL, stroke: P.ink, width: 2.5, glow: { blur: 8, color: P.elec } })
  // cuerpo
  const body = rect(x0, Y(398), s(50), s(160), {
    fill: linear(x0, 0, x0 + s(50), 0, sd < 0 ? ["#9aa6cc", "#e9eef8", "#c9d1e6"] : ["#f1f4fb", "#c9d1e6", "#8f9cc6"]),
    r: s(16),
  })
  clip(body, () => {
    for (const y of [432, 464, 496, 528]) {
      const pts = [[x0 - 4, Y(y - 4)], [cx, Y(y + 4)], [x0 + s(50) + 4, Y(y - 4)]]
      curve(pts, { stroke: P.ink, width: s(15) })
      curve(pts, { stroke: P.elec, width: s(9), glow: { blur: 6, color: P.elecL } })
      curve(shift(pts, 0, -s(2)), { stroke: P.elecL, width: s(3) })
    }
    rect(cx + sd * s(-14) - s(3), Y(405), s(6), s(140), { fill: "#ffffff", opacity: 0.7, r: 3 })
  })
  INK(body, 7)
  // tapa trasera naranja
  const tapa = ellipse(cx, Y(558), s(21), s(10), { fill: P.orange })
  INK(tapa, 5)
}
layer("bobinas", () => {
  part("bobina_izq", () => bobina(-1))
  part("bobina_der", () => bobina(1))
})
// ---------- Acciones ----------
// Alas: negativo lleva el ala izquierda hacia atrás, positivo la derecha. Las placas comparten pivote con su ala.
const flecha = (a) => ({ ala_izq: -a, ala_der: a, placa_izq: -a, placa_der: a, gondola_izq: a, gondola_der: -a })

action("plegar", { duration: 1, loop: false }, [
  [0, flecha(0)],
  [0.5, flecha(14)],
  [1, flecha(30)],
])

action("motor", { duration: 0.4, loop: true }, [
  [0, { llama_centro: { y: 0 }, llama_izq: { y: 12 }, llama_der: { y: 6 }, chispa: { rot: 0, x: 0, y: 0 } }],
  [0.17, { chispa: { rot: 3, x: 2, y: -2 } }],
  [0.33, { llama_centro: { y: 16 }, llama_izq: { y: 0 }, llama_der: { y: 14 }, chispa: { rot: -2, x: -2, y: 1 } }],
  [0.5, { chispa: { rot: 2, x: 1, y: 2 } }],
  [0.67, { llama_centro: { y: 6 }, llama_izq: { y: 8 }, llama_der: { y: 0 }, chispa: { rot: -3, x: -1, y: -1 } }],
  [0.83, { chispa: { rot: 1, x: 2, y: 1 } }],
  [1, { llama_centro: { y: 0 }, llama_izq: { y: 12 }, llama_der: { y: 6 }, chispa: { rot: 0, x: 0, y: 0 } }],
])

action("acelerar", { duration: 0.3, loop: false }, [
  [0, { ...flecha(0), llama_centro: { y: 0 }, llama_izq: { y: 0 }, llama_der: { y: 0 } }],
  [0.4, { ...flecha(16), llama_centro: { y: 40 }, llama_izq: { y: 34 }, llama_der: { y: 34 } }],
  [1, { ...flecha(11), llama_centro: { y: 44 }, llama_izq: { y: 38 }, llama_der: { y: 38 } }],
])

// Acoplar: placas, luego bobinas y por último antenas y chispa, cada una con un rebote
const placasFuera = { placa_izq: { rot: 18, x: -95, y: 20 }, placa_der: { rot: -18, x: 95, y: 20 } }
const placasRebote = { placa_izq: { rot: -3, x: 6, y: 0 }, placa_der: { rot: 3, x: -6, y: 0 } }
const bobinasFuera = { bobina_izq: { rot: -12, x: -110, y: -20 }, bobina_der: { rot: 12, x: 110, y: -20 } }
const bobinasRebote = { bobina_izq: { rot: 2, x: 7, y: 0 }, bobina_der: { rot: -2, x: -7, y: 0 } }
const antenasFuera = { antena_izq: { rot: -22, x: -25, y: -110 }, antena_der: { rot: 22, x: 25, y: -110 }, chispa: { rot: 0, x: 0, y: -150 } }
const antenasRebote = { antena_izq: { rot: 3, x: 0, y: 8 }, antena_der: { rot: -3, x: 0, y: 8 }, chispa: { rot: 0, x: 0, y: 10 } }
const montado = (names) => Object.fromEntries(names.map((n) => [n, { rot: 0, x: 0, y: 0 }]))
const P_ = ["placa_izq", "placa_der"], B_ = ["bobina_izq", "bobina_der"], A_ = ["antena_izq", "antena_der", "chispa"]

action("acoplar", { duration: 0.7, loop: false }, [
  [0, { ...placasFuera, ...bobinasFuera, ...antenasFuera }],
  [0.28, { ...placasRebote }],
  [0.38, { ...montado(P_), ...bobinasFuera }],
  [0.58, { ...bobinasRebote, ...antenasFuera }],
  [0.68, { ...montado(B_) }],
  [0.88, { ...antenasRebote }],
  [1, { ...montado(A_) }],
])
