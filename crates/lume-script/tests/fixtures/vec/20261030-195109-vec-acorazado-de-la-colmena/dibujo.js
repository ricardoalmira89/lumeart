// Acorazado de la Colmena — vista cenital, proa hacia abajo. Estilo pegatina.
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

const P = {
  ink: "#140a1f",        // contorno
  greenD: "#173d22",     // sombra de quitina
  green: "#2c7a37",      // quitina base
  greenL: "#62c24a",     // luz de quitina
  purpleD: "#3b1754",    // sombra de placas
  purple: "#6d2f96",     // placas blindadas
  purpleL: "#a865d6",    // luz de placas
  plat: "#22252d",       // plataformas de torreta
  platL: "#363b47",
  red: "#ff3b2b",        // núcleo
  white: "#ffffff",
}

// Utilidades de simetría: M refleja puntos respecto a x = 512
const M = (pts, s = 1) => pts.map(p => { const q = [512 + s * (p[0] - 512), p[1]]; if (p.length > 2) q.push(p[2]); return q })
const sym = (half) => [...half, ...M(half.slice(1, -1).reverse(), -1)]
const pol = (cx, cy, r, deg) => [cx + r * Math.cos(deg * Math.PI / 180), cy + r * Math.sin(deg * Math.PI / 180)]
const arcPts = (cx, cy, r, a0, a1, n = 8) => range(n + 1).map(i => pol(cx, cy, r, lerp(a0, a1, i / n)))

// Siluetas
const HULL = sym([[512, 36], [656, 50], [776, 108], [850, 218], [866, 340], [838, 470], [762, 588], [640, 668], [512, 700, 0.5]])
const LEG_R = [[790, 205], [900, 150], [958, 124, 0], [932, 186], [835, 300]]
const LEG_F = [[830, 418], [938, 436], [974, 462, 0], [930, 490], [800, 528]]
const MAND = [[540, 650], [636, 655], [654, 700], [578, 742, 0], [596, 706], [540, 690]]
const PIECES = [LEG_R, M(LEG_R, -1), LEG_F, M(LEG_F, -1), MAND, M(MAND, -1), HULL]

// Posiciones de las seis plataformas
const PLATS = [[382, 214], [642, 214], [250, 378], [774, 378], [392, 572], [632, 572]]

layer("borde blanco", () => {
  for (const pts of PIECES) blob(pts, { fill: P.white, stroke: P.white, width: 46, join: "round" })
})

let HULL_SHAPE
layer("patas y mandíbulas", () => {
  const limb = (pts, base, s) => {
    const f = blob(pts, { fill: base, stroke: P.ink, width: 20, behind: true, join: "round" })
    cel(f, { depth: 0.45, color: shade(base, -0.45) })
    return f
  }
  for (const s of [1, -1]) {
    const r = limb(M(LEG_R, s), P.purple, s)
    clip(r, () => {
      brush(M([[850, 205], [905, 170], [940, 148]], s), { width: 7, color: P.purpleL, taper: [0.2, 0.6] })
      brush(M([[860, 240], [880, 228]], s), { width: 6, color: P.ink, taper: [0, 0] })
    })
    const f = limb(M(LEG_F, s), P.purple, s)
    clip(f, () => {
      brush(M([[870, 438], [930, 446], [960, 458]], s), { width: 7, color: P.purpleL, taper: [0.2, 0.6] })
      brush(M([[878, 455], [872, 500]], s), { width: 6, color: P.ink, taper: [0, 0] })
    })
    const m = limb(M(MAND, s), P.greenL, s)
    clip(m, () => brush(M([[600, 672], [636, 680], [630, 708], [600, 728]], s), { width: 7, color: "#c9f58c", opacity: s > 0 ? 0.5 : 1, taper: [0.2, 0.6] }))
  }
})

layer("casco", () => {
  HULL_SHAPE = blob(HULL, { fill: P.green, stroke: P.ink, width: 22, behind: true, join: "round" })
  cel(HULL_SHAPE, { depth: 0.18, color: P.greenD })
})

layer("placas", () => {
  clip(HULL_SHAPE, () => {
    // costura dorsal (élitros)
    brush([[512, 40], [512, 300]], { width: 10, color: P.ink, taper: [0, 0.3] })
    brush([[512, 470], [512, 700]], { width: 10, color: P.ink, taper: [0.3, 0] })

    // nervaduras de la quitina
    for (const s of [1, -1]) {
      const veins = [
        [[600, 320], [680, 290], [760, 230], [820, 170]],
        [[612, 400], [700, 440], [790, 470]],
        [[590, 470], [640, 540], [700, 610]],
        [[560, 140], [640, 180], [720, 160], [790, 120]],
        [[470 + 40, 600], [470 + 40, 600]],
      ]
      for (const v of veins.slice(0, 4)) {
        brush(M(v, s), { width: [3, 9, 3], color: P.greenD, taper: [0.3, 0.5] })
        brush(M(shift(v, -2, -4), s), { width: [1, 3, 1], color: P.greenL, opacity: 0.7 })
      }
    }

    // bandas traseras superpuestas (placas moradas)
    const plate = (pts, base) => {
      const f = blob(pts, { fill: base, stroke: P.ink, width: 12, behind: true, join: "round" })
      cel(f, { depth: 0.3, color: P.purpleD })
      return f
    }
    const b2 = plate(sym([[512, 100], [600, 108], [690, 140], [660, 180], [512, 196, 0.5]]), P.purple)
    clip(b2, () => brush([[420, 122], [520, 112], [590, 116]], { width: 7, color: P.purpleL }))
    const b1 = plate(sym([[512, 30], [640, 40], [740, 92], [640, 122], [512, 134, 0.5]]), P.purple)
    clip(b1, () => {
      brush([[370, 72], [470, 52], [560, 52]], { width: 8, color: P.purpleL })
      // juntas de las placas traseras
      for (const s of [1, -1]) brush(M([[600, 36], [612, 80], [604, 126]], s), { width: [4, 7, 4], color: P.ink, taper: [0, 0] })
    })
    clip(b2, () => { for (const s of [1, -1]) brush(M([[590, 104], [596, 150], [588, 196]], s), { width: [4, 6, 4], color: P.ink, taper: [0, 0] }) })

    // placas de hombro y de proa (alrededor de las plataformas)
    for (const s of [1, -1]) {
      const sh = plate(M([[700, 300], [790, 262], [880, 300], [880, 460], [800, 500], [716, 462], [690, 380]], s), P.purple)
      clip(sh, () => brush(M(s > 0 ? [[712, 330], [780, 290], [840, 292]] : [[712, 330], [780, 290], [840, 292]], s), { width: 7, color: P.purpleL, opacity: s > 0 ? 0.5 : 1 }))
      const fr = plate(M([[560, 512], [660, 500], [720, 560], [700, 630], [612, 668], [552, 620]], s), P.purple)
      clip(fr, () => brush(M([[566, 540], [640, 516], [690, 530]], s), { width: 6, color: P.purpleL, opacity: s > 0 ? 0.5 : 1 }))
    }
  })

  // panal hexagonal entre las plataformas traseras y la escotilla
  const hex = (cx, cy, r) => {
    const pts = range(6).map(i => pol(cx, cy, r, 30 + i * 60))
    poly(pts, { fill: "#1f5a2c", stroke: P.ink, width: 5, join: "round" })
    poly(shift(grow(pts, 0.6), -1.5, -1.5), { fill: "#7cc44e", opacity: 0.85 })
  }
  const r = 17, dx = r * Math.sqrt(3)
  for (const [cx, cy] of [[512, 232], [512 - dx, 232], [512 + dx, 232], [512 - dx / 2, 258], [512 + dx / 2, 258], [512 - dx / 2, 206], [512 + dx / 2, 206]]) hex(cx, cy, r - 1)
  for (const s of [1, -1]) for (const [cx, cy] of [[668, 470], [668 + dx / 2, 496], [668 - dx / 2, 496]]) hex(M([[cx, cy]], s)[0][0], cy, 14)
})

layer("plataformas", () => {
  for (const [x, y] of PLATS) {
    circle(x, y, 54, { fill: P.purpleD, stroke: P.ink, width: 8 })
    circle(x, y, 43, { fill: P.plat, stroke: P.ink, width: 5 })
    circle(x + 3, y + 3, 36, { fill: "#191b21" })
    brush(arcPts(x, y, 47, 195, 255), { width: 5, color: P.purpleL, taper: [0.3, 0.3] })
    brush(arcPts(x, y, 37, 200, 245), { width: 4, color: P.platL, taper: [0.3, 0.3] })
  }
})

layer("escotilla", () => {
  const cx = 512, cy = 384
  circle(cx, cy, 102, { fill: P.purpleD, stroke: P.ink, width: 10 })
  circle(cx, cy, 92, { fill: P.purple, stroke: P.ink, width: 5 })
  brush(arcPts(cx, cy, 97, 190, 260), { width: 5, color: P.purpleL })
  // núcleo rojo
  circle(cx, cy, 82, { fill: "#2a0710", stroke: P.ink, width: 4 })
  // brillo rojo solo en las juntas (cruz en X)
  for (const a of [0, 90, 180, 270]) brush([pol(cx, cy, 4, a), pol(cx, cy, 78, a)], { width: [16, 12, 6], color: P.red, taper: [0, 0.5], glow: { blur: 6, color: alpha(P.red, 0.7) } })
  circle(cx, cy, 14, { fill: "#ffb07a", glow: { blur: 8, color: P.red } })
  // cuatro pétalos blindados en X
  for (const a of [45, 135, 225, 315]) {
    const pts = [pol(cx, cy, 9, a), pol(cx, cy, 70, a - 37), pol(cx, cy, 86, a - 21), pol(cx, cy, 88, a), pol(cx, cy, 86, a + 21), pol(cx, cy, 70, a + 37)]
    const lit = a === 225
    const base = lit ? "#7e3cab" : a === 45 ? "#55247a" : P.purple
    const pe = blob(pts.map((p, i) => i === 0 ? [...p, 0] : p), { fill: base, stroke: P.ink, width: 5, join: "round" })
    clip(pe, () => {
      brush([pol(cx, cy, 22, a), pol(cx, cy, 70, a)], { width: 6, color: shade(base, -0.4), taper: [0.4, 0.2] })
      brush(arcPts(cx, cy, 72, a - 25, a + 10, 4), { width: 5, color: P.purpleL, opacity: lit ? 1 : 0.55 })
    })
  }
  circle(cx, cy, 16, { fill: P.purpleD, stroke: P.ink, width: 5 })
  circle(cx - 4, cy - 4, 5, { fill: P.purpleL })
})

layer("brillos y tinta", () => {
  // contorno de tinta de grosor variable sobre el casco
  ink(HULL_SHAPE, { width: 5, weight: 3, color: P.ink })
  // brillos blancos marcados (luz arriba a la izquierda)
  clip(HULL_SHAPE, () => {
    brush([[178, 330], [196, 230], [250, 150], [330, 92]], { width: 12, color: P.white, taper: [0.4, 0.5], opacity: 0.9 })
    circle(196, 372, 6, { fill: P.white, opacity: 0.9 })
  })
  brush([[400, 66], [470, 48], [540, 46]], { width: 9, color: P.white, taper: [0.4, 0.5], opacity: 0.85 })
  for (const [x, y] of PLATS) circle(x - 32, y - 30, 4, { fill: P.white, opacity: 0.8 })
  circle(456, 322, 5, { fill: P.white, opacity: 0.9 })
})
