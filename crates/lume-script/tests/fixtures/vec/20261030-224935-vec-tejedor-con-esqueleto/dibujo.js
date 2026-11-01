// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Paleta (de la referencia)
const P = {
  tinta: "#1c0d2b",      // contorno oscuro
  blanco: "#ffffff",     // borde de pegatina
  verde: "#3fbe3a",      // patas y cefalotórax
  verdeOsc: "#1e7a2c",
  verdeLuz: "#a6f25e",
  lila: "#6b2db3",       // puntas de las patas
  lilaOsc: "#3e1672",
  lilaLuz: "#a874e6",
  abd: "#7a36ba",        // abdomen
  abdOsc: "#43197a",
  hex: "#a3909f",
  naranja: "#ffab1c",    // articulaciones y lunares
  naranjaOsc: "#c86f00",
  rojo: "#e3121c",       // ojos
  rojoOsc: "#6e0010",
  oliva: "#45631f",
  colmillo: "#ffb31c",
  colmilloOsc: "#b4560c",
  seda: "#ecd9ff",
}

const OUT = 6     // grosor del contorno oscuro
const WH = 13     // grosor del borde blanco
const LUZ = [-0.7071, -0.7071]
setLight(-1, -1)

// ---------- utilidades ----------
function unit(A, B) {
  const dx = B[0] - A[0], dy = B[1] - A[1], L = Math.hypot(dx, dy)
  return { dx: dx / L, dy: dy / L, L, nx: -dy / L, ny: dx / L }
}
// Cápsula de A a B con radio r (extremos redondeados centrados en A y B)
function capD(A, B, r) {
  const u = unit(A, B), nx = u.nx * r, ny = u.ny * r
  return `M ${A[0] + nx} ${A[1] + ny} L ${B[0] + nx} ${B[1] + ny} A ${r} ${r} 0 0 0 ${B[0] - nx} ${B[1] - ny} L ${A[0] - nx} ${A[1] - ny} A ${r} ${r} 0 0 0 ${A[0] + nx} ${A[1] + ny} Z`
}
// Punta de lápiz: redonda en J, afilada en T
function tipD(J, T, r, k = 0) {
  const u = unit(J, T), R = r + k
  const nx = u.nx * R, ny = u.ny * R
  const m = [J[0] + u.dx * u.L * 0.8, J[1] + u.dy * u.L * 0.8]
  const tp = [T[0] + u.dx * k * 1.6, T[1] + u.dy * k * 1.6]
  return `M ${J[0] + nx} ${J[1] + ny} L ${m[0] + nx} ${m[1] + ny} L ${tp[0]} ${tp[1]} L ${m[0] - nx} ${m[1] - ny} L ${J[0] - nx} ${J[1] - ny} A ${R} ${R} 0 0 0 ${J[0] + nx} ${J[1] + ny} Z`
}
// Lado en sombra (normal unitaria que se aparta de la luz)
function sombraN(A, B) {
  const u = unit(A, B), s = u.nx * LUZ[0] + u.ny * LUZ[1]
  return s > 0 ? [-u.nx, -u.ny] : [u.nx, u.ny]
}
function along(A, B, t, off = [0, 0], d = 0) {
  return [A[0] + (B[0] - A[0]) * t + off[0] * d, A[1] + (B[1] - A[1]) * t + off[1] * d]
}

// Segmento de pata con volumen: tinta, base, sombra de un tono y brillo
function segmento(A, B, r, base, osc, luz) {
  path(capD(A, B, r + OUT), { fill: P.tinta })
  const f = path(capD(A, B, r), { fill: base })
  const n = sombraN(A, B)
  clip(f, () => {
    path(capD([A[0] + n[0] * r * 0.95, A[1] + n[1] * r * 0.95], [B[0] + n[0] * r * 0.95, B[1] + n[1] * r * 0.95], r), { fill: osc })
    brush([along(A, B, 0.12, n, -r * 0.42), along(A, B, 0.82, n, -r * 0.42)], { width: r * 0.36, color: luz, taper: [0.2, 0.3] })
  })
}
function puntaLila(J, T, r) {
  path(tipD(J, T, r, OUT), { fill: P.tinta })
  const f = path(tipD(J, T, r), { fill: P.lila })
  const n = sombraN(J, T)
  clip(f, () => {
    path(tipD([J[0] + n[0] * r, J[1] + n[1] * r], [T[0] + n[0] * r, T[1] + n[1] * r], r), { fill: P.lilaOsc })
    brush([along(J, T, 0.1, n, -r * 0.4), along(J, T, 0.72, n, -r * 0.4)], { width: r * 0.34, color: P.lilaLuz, taper: [0.2, 0.3] })
  })
  // collar oscuro antes de la punta
  const u = unit(J, T), m = along(J, T, 0.8)
  line(m[0] + u.nx * r, m[1] + u.ny * r, m[0] - u.nx * r, m[1] - u.ny * r, { stroke: P.tinta, width: 4, cap: "round" })
}
function bola(c, r) {
  circle(c[0], c[1], r + OUT - 1, { fill: P.tinta })
  const b = circle(c[0], c[1], r, { fill: P.naranja })
  clip(b, () => {
    circle(c[0] + r * 0.45, c[1] + r * 0.45, r, { fill: P.naranjaOsc })
    circle(c[0] - r * 0.12, c[1] - r * 0.12, r * 0.82, { fill: P.naranja })
  })
  circle(c[0] - r * 0.32, c[1] - r * 0.32, r * 0.3, { fill: "#fff6d8" })
}

// ---------- geometría ----------
const C = { abd: [512, 338], abdR: [186, 198], cef: [512, 578], cefR: 118, ojos: [512, 716] }
// pata_1 = delantera (junto a la cabeza, abajo) ... pata_4 = trasera (arriba)
// H cadera, K rodilla, J segunda articulación, T punta (lado izquierdo de la pantalla)
const PATAS = {
  1: { H: [452, 648], K: [322, 688], J: [248, 790], T: [212, 952] },
  2: { H: [425, 612], K: [260, 594], J: [158, 680], T: [60, 812] },
  3: { H: [420, 565], K: [260, 457], J: [143, 411], T: [36, 457] },
  4: { H: [432, 530], K: [280, 348], J: [197, 226], T: [126, 68] },
}
const mx = (p, s) => (s === "izq" ? p : [1024 - p[0], p[1]])
const LADOS = ["izq", "der"]
const pata = (i, s) => {
  const d = PATAS[i]
  return { H: mx(d.H, s), K: mx(d.K, s), J: mx(d.J, s), T: mx(d.T, s) }
}
const R_MUSLO = 17, R_TIBIA = 13, R_PUNTA = 11

// ---------- esqueleto ----------
rig({ origin: [512, 512] })
bone("cuerpo", { at: C.abd })
bone("cabeza", { parent: "cuerpo", at: [512, 500] })
for (const s of LADOS) for (const i of [1, 2, 3, 4]) {
  const p = pata(i, s)
  bone(`pata_${i}_${s}`, { parent: "cabeza", at: p.H })
  bone(`pata_${i}_punta_${s}`, { parent: `pata_${i}_${s}`, at: p.K })
}

// ---------- 1. borde blanco de pegatina (todas las piezas, detrás de todo) ----------
const W = OUT + WH
for (const s of LADOS) for (const i of [4, 3, 2, 1]) {
  const p = pata(i, s)
  part(`pata_${i}_${s}`, () => {
    path(capD(p.H, p.K, R_MUSLO + W), { fill: P.blanco })
  })
  part(`pata_${i}_punta_${s}`, () => {
    circle(p.K[0], p.K[1], 18 + W, { fill: P.blanco })
    path(capD(p.K, p.J, R_TIBIA + W), { fill: P.blanco })
    circle(p.J[0], p.J[1], 12 + W, { fill: P.blanco })
    path(tipD(p.J, p.T, R_PUNTA, W), { fill: P.blanco })
  })
}
part("cuerpo", () => {
  // hilo de seda
  line(512, -20, 512, 170, { stroke: P.blanco, width: 10 + 2 * W, cap: "round" })
  ellipse(C.abd[0], C.abd[1], C.abdR[0] + W, C.abdR[1] + W, { fill: P.blanco })
  ellipse(512, 172, 46 + W, 30 + W, { fill: P.blanco })
})
part("cabeza", () => {
  circle(C.cef[0], C.cef[1], C.cefR + W, { fill: P.blanco })
  ellipse(C.ojos[0], C.ojos[1], 92 + W, 72 + W, { fill: P.blanco })
  mirror(512, () => {
    blob([[488, 772], [452, 786, 0.5], [448, 812], [462, 846], [482, 870, 0], [492, 840], [500, 806], [500, 778]], { fill: P.blanco, stroke: P.blanco, width: 2 * W, join: "round" })
  })
})

// ---------- 2. patas ----------
for (const i of [4, 3, 2, 1]) for (const s of LADOS) {
  const p = pata(i, s)
  part(`pata_${i}_${s}`, () => {
    // muslo
    segmento(p.H, p.K, R_MUSLO, P.verde, P.verdeOsc, P.verdeLuz)
  })
  part(`pata_${i}_punta_${s}`, () => {
    // punta lila, tibia y articulaciones
    puntaLila(p.J, p.T, R_PUNTA)
    segmento(p.K, p.J, R_TIBIA, P.verde, P.verdeOsc, P.verdeLuz)
    bola(p.J, 10)
    bola(p.K, 15)
  })
}

// ---------- 3. abdomen (cuerpo) ----------
part("cuerpo", () => {
  // hilo de seda
  line(512, -20, 512, 170, { stroke: P.tinta, width: 12, cap: "round" })
  line(512, -20, 512, 170, { stroke: P.seda, width: 5, cap: "round" })
  // abdomen
  const [ax, ay] = C.abd, [rx, ry] = C.abdR
  ellipse(ax, ay, rx + OUT + 2, ry + OUT + 2, { fill: P.tinta })
  const abd = ellipse(ax, ay, rx, ry, { fill: P.abdOsc })
  clip(abd, () => {
    // volumen: luz desde arriba a la izquierda, media luna de sombra a la derecha
    ellipse(ax - 26, ay - 14, rx - 22, ry - 12, { fill: P.abd })
    // franja central más oscura
    ellipse(ax, ay + 10, 46, ry - 20, { fill: P.abdOsc, opacity: 0.35 })
    // panal hexagonal
    const hs = 27, hw = hs * Math.sqrt(3)
    for (let row = -9; row <= 9; row++) for (let col = -6; col <= 6; col++) {
      const cx = ax + col * hw + (row % 2 ? hw / 2 : 0), cy = ay + row * hs * 1.5
      poly(ring(cx, cy, hs, hs, 6, 0), { fill: "none", stroke: P.hex, width: 3, opacity: 0.75, join: "round" })
    }
    // la sombra vuelve a oscurecer el panal en el borde derecho
    cut(ellipse(ax, ay, rx, ry, { fill: P.abdOsc, opacity: 0.75 }), [ellipse(ax - 30, ay - 16, rx - 18, ry - 8, { hidden: true })])
    // brillo de borde a la izquierda
    brush([[352, 380], [338, 300], [358, 228], [400, 182]], { width: 22, color: P.lilaLuz, taper: [0.5, 0.5], opacity: 0.8 })
  })
  // cheurones
  for (const [y, w, h, t] of [[266, 44, 30, 12], [318, 86, 44, 18], [390, 72, 46, 18]]) {
    poly([[ax - w, y], [ax, y + h], [ax + w, y], [ax + w - 6, y + t * 0.7], [ax, y + h + t], [ax - w + 6, y + t * 0.7]], { fill: P.abdOsc, opacity: 0.9, join: "round" })
  }
  // lunares naranjas
  for (const [x, y, r] of [[456, 258, 13], [428, 348, 11], [446, 437, 9]]) {
    for (const xx of [x, 1024 - x]) {
      circle(xx, y, r + 4, { fill: P.tinta })
      circle(xx, y, r, { fill: P.naranja })
      circle(xx - r * 0.3, y - r * 0.3, r * 0.3, { fill: "#fff3cc" })
    }
  }
  // brillo blanco
  blob([[392, 236], [420, 206, 0.5], [452, 192], [446, 202], [416, 222], [396, 246, 0.5]], { fill: P.blanco })
  brush([[376, 296], [372, 260], [384, 228]], { width: 9, color: P.blanco, opacity: 0.45, taper: [0.4, 0.4] })
  circle(367, 321, 8, { fill: P.blanco })
  // hileras (de donde sale el hilo)
  ellipse(512, 172, 50, 28, { fill: P.seda, opacity: 0.35, blur: 12 })
  for (const [x, y, rx2, ry2] of [[488, 180, 14, 19], [536, 180, 14, 19], [512, 166, 15, 23]]) {
    ellipse(x, y, rx2 + 5, ry2 + 5, { fill: P.tinta })
    ellipse(x, y, rx2, ry2, { fill: P.seda })
    ellipse(x - 3, y - 5, rx2 * 0.4, ry2 * 0.45, { fill: P.blanco })
  }
})

// ---------- 4. cefalotórax, ojos y quelíceros (cabeza) ----------
part("cabeza", () => {
  const [cx, cy] = C.cef, R = C.cefR
  // colmillos (quelíceros), detrás del grupo de ojos
  mirror(512, () => {
    const fang = [[488, 772], [452, 786, 0.5], [448, 812], [462, 846], [482, 870, 0], [492, 840], [500, 806], [500, 778]]
    blob(fang, { fill: P.colmillo, stroke: P.tinta, width: 2 * OUT, behind: true, join: "round" })
    const f = blob(fang, { fill: P.colmillo })
    clip(f, () => {
      blob([[478, 790], [472, 812], [480, 845], [486, 866, 0], [496, 830], [500, 800]], { fill: P.colmilloOsc })
      brush([[460, 800], [458, 822], [470, 846]], { width: 6, color: "#fff2c0", taper: [0.3, 0.5] })
    })
  })
  // cefalotórax
  circle(cx, cy, R + OUT + 1, { fill: P.tinta })
  const cef = circle(cx, cy, R, { fill: P.verde })
  clip(cef, () => {
    // la parte alta queda en la sombra del abdomen; sombra de un tono abajo a la derecha
    cut(circle(cx, cy, R, { fill: P.verdeOsc }), [circle(cx - 18, cy + 36, R - 6, { hidden: true })])
    // dibujo central: surcos radiales
    for (const a of [-50, -28, -10, 10, 28, 50]) {
      const p0 = polar(512, 528, 18, 90 + a * 0.4), p1 = polar(512, 528, 62, 90 + a)
      brush([p0, p1], { width: 7, color: P.verdeOsc, taper: [0.1, 0.8] })
    }
    brush([[512, 530], [512, 584]], { width: 8, color: P.verdeOsc, taper: [0.2, 0.2] })
    circle(512, 592, 10, { fill: P.verdeOsc })
  })
  // manchas lilas
  mirror(512, () => {
    const m = blob([[450, 520], [468, 548], [462, 580], [432, 592], [414, 572], [424, 534]], { fill: P.lila, stroke: P.tinta, width: OUT, join: "round" })
    clip(m, () => ellipse(424, 572, 18, 26, { fill: P.lilaOsc, rotate: 30 }))
  })
  // brillo
  blob([[440, 552], [446, 522], [476, 504], [500, 502], [480, 516], [456, 536]], { fill: P.verdeLuz })
  blob([[462, 516], [482, 506], [494, 508], [476, 518]], { fill: P.blanco })
  // grupo de ojos
  const [ex, ey] = C.ojos
  ellipse(ex, ey, 92 + OUT, 72 + OUT, { fill: P.tinta })
  const og = ellipse(ex, ey, 92, 72, { fill: radial(ex, ey, 92, ["#8a2a1a", P.oliva, "#2e4a17"]) })
  clip(og, () => {
    ellipse(ex - 16, ey - 46, 60, 22, { fill: P.verdeLuz, opacity: 0.35 })
    ellipse(ex + 20, ey + 40, 90, 36, { fill: "#1a2a0c", opacity: 0.4 })
  })
  const ojo = (x, y, r) => {
    circle(x, y, r + 4, { fill: P.tinta })
    circle(x, y, r, { fill: radial(x - r * 0.3, y - r * 0.3, r * 1.3, ["#ff5a52", P.rojo, P.rojoOsc]) })
    circle(x - r * 0.35, y - r * 0.35, Math.max(2.5, r * 0.28), { fill: P.blanco })
  }
  for (const [x, y, r] of [[457, 686, 12], [567, 686, 12], [512, 680, 10], [447, 727, 11], [577, 727, 11], [474, 756, 9], [550, 756, 9]]) ojo(x, y, r)
  ojo(488, 718, 24)
  ojo(536, 718, 24)
})

// ---------- acciones ----------
// Signo de giro que lleva un tramo hacia el centro del cuerpo (+ = horario en pantalla)
const CENTRO = [512, 520]
function haciaDentro(A, B, O = CENTRO) {
  const v = [B[0] - A[0], B[1] - A[1]], w = [O[0] - A[0], O[1] - A[1]]
  return Math.sign(v[0] * w[1] - v[1] * w[0])
}
const signo = {}
for (const s of LADOS) for (const i of [1, 2, 3, 4]) {
  const p = pata(i, s)
  // el muslo gira hacia el eje del cuerpo; la punta se dobla por la rodilla hacia la cadera
  signo[`${i}_${s}`] = { m: haciaDentro(p.H, p.K), p: haciaDentro(p.K, p.T, p.H) }
}

// colgar: las ocho patas "nadan" en el aire, desfasadas, y el abdomen respira
const N = 8
action("colgar", { duration: 1.4, loop: true }, range(N + 1).map((k) => {
  const t = k / N, pose = {}
  for (const s of LADOS) for (const i of [1, 2, 3, 4]) {
    const fase = 2 * Math.PI * (t + (i - 1) * 0.27 + (s === "der" ? 0.5 : 0))
    const sg = signo[`${i}_${s}`]
    pose[`pata_${i}_${s}`] = sg.m * 7 * Math.sin(fase)
    pose[`pata_${i}_punta_${s}`] = sg.p * 11 * Math.sin(fase - 0.9)
  }
  const resp = Math.sin(2 * Math.PI * t)
  pose.cuerpo = { rot: 1.2 * resp, y: -5 * resp }
  pose.cabeza = { rot: -1.2 * resp, y: 5 * resp }
  return [t, pose]
}))

// tejer: las dos patas delanteras se alternan rápido; el resto, quietas
const tejerPose = (dentro, fuera) => {
  const a = signo["1_izq"], b = signo["1_der"]
  return {
    pata_1_izq: a.m * (dentro ? 24 : -8), pata_1_punta_izq: a.p * (dentro ? 40 : -10),
    pata_1_der: b.m * (fuera ? 24 : -8), pata_1_punta_der: b.p * (fuera ? 40 : -10),
  }
}
action("tejer", { duration: 0.5, loop: true }, [
  [0, tejerPose(true, false)],
  [0.5, tejerPose(false, true)],
  [1, tejerPose(true, false)],
])

// golpe: las patas se encogen de golpe hacia el cuerpo y se vuelven a abrir
const golpePose = (km, kp, y) => {
  const pose = { cuerpo: { rot: 0, y } }
  for (const s of LADOS) for (const i of [1, 2, 3, 4]) {
    const sg = signo[`${i}_${s}`]
    pose[`pata_${i}_${s}`] = sg.m * km
    pose[`pata_${i}_punta_${s}`] = sg.p * kp
  }
  return pose
}
action("golpe", { duration: 0.25, loop: false }, [
  [0, golpePose(0, 0, 0)],
  [0.3, golpePose(16, 42, -8)],
  [0.7, golpePose(-4, -6, 2)],
  [1, golpePose(0, 0, 0)],
])
