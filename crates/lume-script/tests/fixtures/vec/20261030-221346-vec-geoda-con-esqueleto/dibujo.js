// dibujo.js: geoda enemiga vista desde arriba, montada con esqueleto.
canvas({ width: 1024, height: 1024, background: null })

const P = {
  tinta: "#1e0f33",
  borde: "#ffffff",
  cian: { luz: "#30e0ff", medio: "#1a88c8", oscuro: "#0f559c", brillo: "#f2fdff" },
  violeta: { luz: "#c9a2ff", medio: "#7a50d8", oscuro: "#4329b2", brillo: "#fbf6ff" },
  anilloLuz: "#7148ab",
  anilloSombra: "#2a1650",
  gema: "#ff2fa6",
  gemaSombra: "#c0187f",
}
const C = [512, 510]
setLight(-1, -1)

// Cristales: ángulo (grados, horario desde arriba), largo hasta la punta, distancia a los hombros y medio ancho
const CRISTALES = [
  { th: 0, tipo: "cian", L: 455, D: 262, W: 112 },
  { th: 60, tipo: "violeta", L: 425, D: 255, W: 114 },
  { th: 120, tipo: "cian", L: 455, D: 262, W: 112 },
  { th: 180, tipo: "violeta", L: 425, D: 255, W: 114 },
  { th: 240, tipo: "cian", L: 458, D: 264, W: 112 },
  { th: 300, tipo: "violeta", L: 425, D: 255, W: 114 },
]
// Base de cada cristal (escondida bajo el anillo)
const BV = 125, BW = 62

// Marco local de un cristal: v hacia fuera, u lateral; u < 0 siempre es el lado iluminado (arriba-izquierda)
function marco(th) {
  const t = (th * Math.PI) / 180
  const a = [Math.sin(t), -Math.cos(t)]
  const r = [Math.cos(t), Math.sin(t)]
  const s = r[0] * -1 + r[1] * -1 < 0 ? 1 : -1
  const f = (u, v) => [C[0] + a[0] * v + r[0] * u * s, C[1] + a[1] * v + r[1] * u * s]
  f.a = a
  return f
}
const silueta = (c) => {
  const f = marco(c.th)
  return [f(0, c.L), f(c.W, c.D), f(BW, BV), f(-BW, BV), f(-c.W, c.D)]
}

// Anillo de 12 facetas
const NF = 12, RO = 190, RI = 110, ROT = -4
const vert = (R, i) => {
  const t = ((ROT + 15 + i * 30) * Math.PI) / 180
  return [C[0] + Math.sin(t) * R, C[1] - Math.cos(t) * R]
}
const anilloPts = range(NF).map((i) => vert(RO, i))

layer("geoda", () => {
  rig({ origin: C })
  bone("nucleo", { at: C })
  bone("gema", { parent: "nucleo", at: C })
  CRISTALES.forEach((c, i) => {
    const f = marco(c.th)
    bone("cristal_" + (i + 1), { parent: "nucleo", at: f(0, 270) })
  })

  // 1) Borde blanco de pegatina (todo detrás)
  part("nucleo", () => {
    poly(anilloPts, { fill: P.borde, stroke: P.borde, width: 70, join: "round" })
  })
  CRISTALES.forEach((c, i) => {
    part("cristal_" + (i + 1), () => {
      poly(silueta(c), { fill: P.borde, stroke: P.borde, width: 66, join: "round" })
    })
  })

  // 2) Cristales
  CRISTALES.forEach((c, i) => {
    part("cristal_" + (i + 1), () => {
      const f = marco(c.th)
      const K = P[c.tipo]
      const pts = silueta(c)
      const base = poly(pts, { fill: K.medio })
      clip(base, () => {
        // mitad iluminada
        poly([f(0, c.L), f(-c.W, c.D), f(-BW, BV), f(0, BV)], { fill: K.luz })
        // astilla de brillo blanco
        poly([f(-10, c.L - 48), f(-74, c.D + 2), f(-16, c.D - 72), f(-12, c.D + 20)], { fill: K.brillo })
        // faceta oscura de la mitad en sombra
        poly([f(30, c.D + 24), f(c.W - 6, c.D + 4), f(88, 160), f(18, 160)], { fill: K.oscuro })
        // arista central
        brush([f(0, c.L), f(0, (c.L + BV) / 2), f(0, BV)], { width: [3, 7, 8], taper: [0.25, 0], color: P.tinta })
      })
      ink(base, { width: 12, weight: 1.3, color: P.tinta })
    })
  })

  // 3) Anillo facetado
  part("nucleo", () => {
    const anillo = poly(anilloPts, { fill: P.anilloSombra })
    clip(anillo, () => {
      range(NF).forEach((i) => {
        const j = (i + 1) % NF
        // normal de la faceta frente a la luz (arriba-izquierda)
        const t = ((ROT + 30 + i * 30) * Math.PI) / 180
        const k = clamp((Math.sin(t) * -0.7 + -Math.cos(t) * -0.7 + 0.35) / 1.3, 0, 1)
        poly([vert(RO + 4, i), vert(RO + 4, j), vert(RI, j), vert(RI, i)], {
          fill: mix(P.anilloSombra, P.anilloLuz, smoothstep(0, 1, k)),
        })
        brush([vert(RI, i), vert(RO, i)], { width: 5, taper: [0, 0], color: P.tinta })
      })
      // banda de luz en el borde iluminado
      outline(anillo, -8, alpha("#b48cff", 0.35), { blend: "screen" })
    })
    // borde interior oscuro donde asienta la gema
    circle(C[0], C[1], RI - 4, { fill: P.tinta })
    circle(C[0], C[1], RI - 4, { fill: "none", stroke: shade(P.anilloSombra, -0.2), width: 6 })
    ink(anillo, { width: 14, weight: 1.3, color: P.tinta })
  })

  // 4) Gema
  part("gema", () => {
    const g = circle(C[0], C[1], 95, { fill: P.gemaSombra })
    clip(g, () => {
      circle(C[0] - 14, C[1] - 16, 90, { fill: P.gema })
      circle(C[0] - 30, C[1] - 30, 50, { fill: "#ff6cc3", blur: 22, opacity: 0.8 })
    })
    ellipse(C[0] - 36, C[1] - 42, 32, 18, { fill: "#ffffff", rotate: -32 })
    circle(C[0] + 28, C[1] - 58, 9, { fill: "#ffffff" })
    ink(g, { width: 12, weight: 1.3, color: P.tinta })
  })
})

// ---------- Acciones ----------
// Desplaza cada cristal d unidades a lo largo de su eje (d > 0 hacia fuera) y lo gira rot grados
const abrir =(fn) => {
  const pose = {}
  CRISTALES.forEach((c, i) => {
    const a = marco(c.th).a
    const [d, rot] = fn(c, i)
    pose["cristal_" + (i + 1)] = { rot: rot || 0, x: a[0] * d, y: a[1] * d }
  })
  return pose
}
const violeta = (c) => c.tipo === "violeta"

// latido: los violetas se abren primero y los cian les siguen; la gema late
action("latido", { duration: 1.2, loop: true }, [
  [0, { ...abrir(() => [0]), gema: { rot: 0, x: 0, y: 0 } }],
  [0.3, { ...abrir((c) => [violeta(c) ? 14 : 4]), gema: { rot: 0, x: 0, y: -3 } }],
  [0.5, { ...abrir((c) => [violeta(c) ? 12 : 15]), gema: { rot: 0, x: 0, y: 1 } }],
  [0.75, { ...abrir((c) => [violeta(c) ? 3 : 8]), gema: { rot: 0, x: 0, y: -1 } }],
  [1, { ...abrir(() => [0]), gema: { rot: 0, x: 0, y: 0 } }],
])

// golpe: sacudida hacia dentro y rebote
action("golpe", { duration: 0.25, loop: false }, [
  [0, { ...abrir(() => [0]), gema: { rot: 0, x: 0, y: 0 } }],
  [0.3, { ...abrir((c, i) => [-20, i % 2 ? 5 : -5]), gema: { rot: 0, x: 3, y: -2 } }],
  [0.65, { ...abrir((c, i) => [6, i % 2 ? -2 : 2]), gema: { rot: 0, x: -2, y: 1 } }],
  [1, { ...abrir(() => [0]), gema: { rot: 0, x: 0, y: 0 } }],
])

// romper: un tirón hacia dentro y los cristales salen disparados girando; la gema tiembla
action("romper", { duration: 0.5, loop: false }, [
  [0, { ...abrir(() => [0]), gema: { rot: 0, x: 0, y: 0 } }],
  [0.15, { ...abrir(() => [-12]), gema: { rot: 0, x: 4, y: -2 } }],
  [0.3, { ...abrir((c, i) => [150, i % 2 ? 25 : -25]), gema: { rot: 0, x: -4, y: 3 } }],
  [0.45, { gema: { rot: 0, x: 3, y: 2 } }],
  [0.6, { ...abrir((c, i) => [290, i % 2 ? 160 : -160]), gema: { rot: 0, x: -3, y: -3 } }],
  [0.75, { gema: { rot: 0, x: 4, y: 1 } }],
  [0.9, { gema: { rot: 0, x: -2, y: -2 } }],
  [1, { ...abrir((c, i) => [420, i % 2 ? 300 : -300]), gema: { rot: 0, x: 0, y: 0 } }],
])
