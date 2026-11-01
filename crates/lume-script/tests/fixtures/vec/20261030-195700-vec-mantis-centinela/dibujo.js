// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Mantis Centinela: vista cenital, cabeza abajo (hacia el jugador), abdomen arriba.
setLight(-1, -1)
const P = {
  ink: "#0f2a16",      // contorno de pegatina (verde casi negro)
  lime: "#9be52b",     // quitina clara
  green: "#3f8f24",    // quitina media
  dark: "#24561a",     // quitina oscura
  purple: "#8a45c8",   // placas
  purpleD: "#4e2380",  // placas en sombra
  amber: "#ffb320",    // detalles
  red: "#ff2b3d",      // ojos (punto débil)
  wing: "#c4f7a8",     // alas translúcidas
  white: "#ffffff",
}
const BORDER = 30 // borde blanco extra (por lado)
const LINE = 9    // contorno oscuro (por lado)
const M = (pts) => pts.map(([x, y, ...r]) => [1024 - x, y, ...r])

// ---------- Geometría (lado izquierdo; el derecho con M) ----------
const LEGS = [
  // pata media
  { segs: [
    { pts: [[470, 548], [380, 515], [292, 482]], w: 34 },
    { pts: [[292, 482], [220, 560], [152, 650]], w: 24 },
    { pts: [[152, 650], [126, 700], [118, 752]], w: 14 },
  ], knee: [292, 482, 24], ankle: [152, 650, 15] },
  // pata trasera
  { segs: [
    { pts: [[478, 480], [400, 395], [322, 300]], w: 34 },
    { pts: [[322, 300], [235, 225], [155, 140]], w: 24 },
    { pts: [[155, 140], [128, 104], [112, 58]], w: 14 },
  ], knee: [322, 300, 24], ankle: [155, 140, 15] },
]
const ABDOMEN = [[512, 62], [572, 92], [630, 190], [646, 300], [620, 420], [575, 495], [512, 515], [449, 495], [404, 420], [378, 300], [394, 190], [452, 92]]
const THORAX = [[512, 430], [600, 455], [628, 530], [600, 600], [512, 620], [424, 600], [396, 530], [424, 455]]
const PRO = [[512, 585], [566, 598], [596, 650], [620, 705], [588, 755], [556, 805], [512, 820], [468, 805], [436, 755], [404, 705], [428, 650], [458, 598]]
const HEAD = [[512, 790], [585, 790, 0.5], [640, 818], [604, 885], [550, 945], [512, 960, 0.5], [474, 945], [420, 885], [384, 818], [439, 790, 0.5]]
const EYE = [[420, 838, 62, 56]] // cx, cy, rx, ry (izq)
const SOCKET = [392, 700, 50]
const WING = [[512, 500], [474, 488], [430, 430], [412, 300], [424, 185], [460, 118, 0.5], [500, 130], [520, 220], [528, 340], [522, 450]]

function legShapes(mirrorSide, extra, color) {
  for (const L of LEGS) {
    for (const s of L.segs) {
      const pts = mirrorSide ? M(s.pts) : s.pts
      brush(pts, { width: s.w + extra * 2, taper: [0, 0], color })
    }
    for (const j of [L.knee, L.ankle]) {
      const x = mirrorSide ? 1024 - j[0] : j[0]
      circle(x, j[1], j[2] + extra, { fill: color })
    }
  }
}
function bodySil(extra, color) {
  const st = { fill: color, stroke: color, width: extra * 2, join: "round" }
  for (const side of [false, true]) legShapes(side, extra, color)
  blob(ABDOMEN, st)
  blob(WING, st); blob(M(WING), st)
  blob(THORAX, st)
  blob(PRO, st)
  circle(SOCKET[0], SOCKET[1], SOCKET[2] + extra, { fill: color })
  circle(1024 - SOCKET[0], SOCKET[1], SOCKET[2] + extra, { fill: color })
  blob(HEAD, st)
  for (const [x, y, rx, ry] of EYE) {
    ellipse(x, y, rx + extra, ry + extra, { fill: color })
    ellipse(1024 - x, y, rx + extra, ry + extra, { fill: color })
  }
}

layer("borde", () => {
  // borde blanco de pegatina y contorno oscuro de la silueta completa
  bodySil(LINE + BORDER, P.white)
  bodySil(LINE, P.ink)
})

layer("patas", () => {
  // patas finas plegadas: base, sombra de un tono abajo-derecha, luz arriba-izquierda
  for (const side of [false, true]) {
    for (const L of LEGS) {
      for (const s of L.segs) {
        const pts = side ? M(s.pts) : s.pts
        brush(pts, { width: s.w, taper: [0, 0], color: P.green })
        brush(shift(pts, s.w * 0.18, s.w * 0.18), { width: s.w * 0.5, taper: [0.15, 0.15], color: P.dark })
        brush(shift(pts, -s.w * 0.18, -s.w * 0.2), { width: s.w * 0.32, taper: [0.2, 0.2], color: P.lime })
      }
      const k = L.knee, a = L.ankle
      const kx = side ? 1024 - k[0] : k[0], ax = side ? 1024 - a[0] : a[0]
      circle(kx, k[1], k[2], { fill: P.purple })
      circle(kx + 5, k[1] + 5, k[2] * 0.7, { fill: P.purpleD })
      circle(kx - 6, k[1] - 7, k[2] * 0.32, { fill: P.amber })
      circle(ax, a[1], a[2], { fill: P.purple })
      circle(ax - 4, a[1] - 4, a[2] * 0.4, { fill: P.amber })
    }
  }
})

layer("abdomen", () => {
  // abdomen segmentado con placas moradas
  const ab = blob(ABDOMEN, { fill: P.lime })
  cel(ab, { depth: 0.45, color: P.green })
  clip(ab, () => {
    for (let i = 0; i < 6; i++) {
      const y = 130 + i * 66
      // placas laterales moradas por segmento
      for (const sx of [1, -1]) {
        const x0 = 512 + sx * 95
        blob([[x0, y - 24], [x0 + sx * 70, y - 18], [x0 + sx * 70, y + 22], [x0, y + 24]], { fill: sx < 0 ? P.purple : P.purpleD, stroke: P.ink, width: 5 })
        brush([[x0 + sx * 12, y - 12], [x0 + sx * 50, y - 10]], { width: 6, color: shade(P.purple, 0.5) })
      }
      curve(arc(512, y - 110, 180, 140, 220, 9), { stroke: P.ink, width: 7 })
    }
  })
  // remate de la punta en ámbar
  circle(512, 92, 18, { fill: P.amber, stroke: P.ink, width: 5 })
})

layer("alas", () => {
  // alas cerradas translúcidas, la izquierda encima
  for (const side of [true, false]) {
    const pts = side ? M(WING) : WING
    const w = blob(pts, { fill: P.wing, opacity: 0.62, stroke: P.ink, width: 7 })
    clip(w, () => {
      const c = center(pts)
      const sx = side ? -1 : 1
      // nervaduras
      brush([[512 - sx * 20, 490], [c[0] - sx * 10, 330], [c[0] - sx * 5, 140]], { width: 7, color: P.dark, taper: [0.1, 0.5] })
      for (let i = 0; i < 4; i++) {
        const y = 420 - i * 75
        brush([[c[0] - sx * 6, y], [c[0] - sx * 50, y - 45], [c[0] - sx * 85, y - 60]], { width: 4, color: alpha(P.dark, 0.8), taper: [0.1, 0.6] })
        brush([[c[0] - sx * 6, y + 10], [c[0] + sx * 40, y - 20]], { width: 4, color: alpha(P.dark, 0.8), taper: [0.1, 0.6] })
      }
      // brillo del ala izquierda (luz arriba-izquierda)
      if (!side) brush([[440, 380], [432, 260], [455, 160]], { width: 16, color: alpha(P.white, 0.75), taper: [0.3, 0.4] })
    })
  }
})

layer("torax", () => {
  // mesotórax con placa morada
  const th = blob(THORAX, { fill: P.green })
  cel(th, { depth: 0.4, color: P.dark })
  const pl = blob([[512, 452], [578, 472], [596, 530], [568, 585], [512, 598], [456, 585], [428, 530], [446, 472]], { fill: P.purple, stroke: P.ink, width: 6 })
  cel(pl, { depth: 0.45, color: P.purpleD })
  brush([[455, 520], [470, 482], [512, 468]], { width: 10, color: shade(P.purple, 0.5) })

  // protórax blindado
  const pr = blob(PRO, { fill: P.lime })
  cel(pr, { depth: 0.45, color: P.green })
  const pp = blob([[512, 612], [552, 628], [568, 690], [548, 760], [512, 790], [476, 760], [456, 690], [472, 628]], { fill: P.purple, stroke: P.ink, width: 6 })
  cel(pp, { depth: 0.45, color: P.purpleD })
  brush([[512, 625], [512, 700], [512, 775]], { width: 9, color: P.amber, taper: [0.2, 0.2] })
  brush([[478, 700], [484, 650], [505, 630]], { width: 8, color: shade(P.purple, 0.5) })
  ink(pr, { width: 4, weight: 2, color: P.ink })

  // hombros: articulaciones redondas y vacías, encima del protórax
  for (const sx of [-1, 1]) {
    const x = 512 + sx * (512 - SOCKET[0]), y = SOCKET[1], r = SOCKET[2]
    circle(x, y, r, { fill: P.purple })
    circle(x + 6, y + 6, r - 6, { fill: P.purpleD })
    circle(x, y, r * 0.62, { fill: P.amber, stroke: P.ink, width: 6 })
    circle(x, y, r * 0.45, { fill: "#08150c" })
    circle(x + 4, y + 4, r * 0.3, { fill: "#030a05" })
    brush(arc(x, y, r - 9, 250, 330, 6), { width: 7, color: shade(P.purple, 0.55), taper: [0.3, 0.3] })
    circle(x, y, r, { fill: "none", stroke: P.ink, width: 8 })
  }
})

layer("cabeza", () => {
  // cabeza triangular
  const hd = blob(HEAD, { fill: P.lime })
  cel(hd, { depth: 0.4, color: P.green })
  // placa frontal morada en forma de flecha
  const fp = blob([[512, 800], [552, 808], [540, 860], [512, 905, 0], [484, 860], [472, 808]], { fill: P.purple, stroke: P.ink, width: 6 })
  cel(fp, { depth: 0.45, color: P.purpleD })
  // mandíbulas ámbar
  for (const sx of [-1, 1]) {
    const md = blob([[512 + sx * 12, 902], [512 + sx * 42, 902], [512 + sx * 34, 930], [512 + sx * 6, 946, 0]], { fill: P.amber, stroke: P.ink, width: 6 })
    cel(md, { depth: 0.4, color: shade(P.amber, -0.35) })
  }
  ink(hd, { width: 4, weight: 2, color: P.ink })

  // ojos compuestos rojos (punto débil)
  for (const sx of [-1, 1]) {
    const [ex, ey, rx, ry] = EYE[0]
    const x = sx < 0 ? ex : 1024 - ex
    const e = ellipse(x, ey, rx, ry, { fill: radial(x - 15, ey - 15, rx * 1.1, ["#ff7a6a", P.red, "#a8001c"]), glow: { blur: 16, color: alpha(P.red, 0.5) } })
    clip(e, () => {
      ellipse(x + 16, ey + 18, rx * 0.9, ry * 0.8, { fill: "#b0001f", opacity: 0.6 })
      // facetas
      for (let i = -3; i <= 3; i++)
        for (let j = -3; j <= 3; j++)
          circle(x + i * 18 + (j % 2) * 9, ey + j * 16, 3.2, { fill: "#ffd0c0", opacity: 0.35 })
    })
    ellipse(x, ey, rx, ry, { fill: "none", stroke: P.ink, width: 8 })
    // brillo blanco marcado arriba-izquierda
    ellipse(x - rx * 0.35, ey - ry * 0.4, rx * 0.28, ry * 0.18, { fill: P.white, rotate: -25 })
    circle(x - rx * 0.05, ey - ry * 0.55, 6, { fill: P.white })
  }
})

layer("retoques", () => {
  // brillos y grano
})
