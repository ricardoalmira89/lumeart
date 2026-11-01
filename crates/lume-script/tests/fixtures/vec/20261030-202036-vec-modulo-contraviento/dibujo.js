// Módulo de morro CONTRAVIENTO: vista cenital, apunta hacia arriba. Luz arriba-izquierda.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#24182e",      // contorno
  white: "#f4f1ea",    // metal blanco
  whiteSh: "#b9b4c8",  // sombra del metal
  clamp: "#3a3550",    // abrazadera oscura
  clampHi: "#5d5878",
  ring: "#f2522a",     // anillo rojo anaranjado
  ringSh: "#b02a2a",
  ringHi: "#ff9a5a",
  glow: "#ffb02e",     // bocas encendidas
  glowCore: "#fff2b0",
}

const RC = [512, 445]       // centro del anillo
const R_OUT = 222, R_IN = 122
const ANG = 20              // giro de la veleta (grados, horario)

// ---------- Siluetas (se usan para el borde de pegatina y para clip) ----------
const clampPts = [[340, 690], [684, 690], [712, 730, 0.5], [712, 950, 0.5], [690, 975, 0.5], [610, 975, 0.5], [596, 950, 0.5], [596, 820], [512, 772], [428, 820], [428, 950, 0.5], [414, 975, 0.5], [334, 975, 0.5], [312, 950, 0.5], [312, 730, 0.5]]
const bodyPts = [[512, 330], [724, 390], [790, 570], [724, 730], [512, 790], [300, 730], [234, 570], [300, 390]]

function emitterShapes(a, st, mouthSt) {
  // emisor en coordenadas locales apuntando hacia arriba desde el borde del anillo
  const top = RC[1] - R_OUT - 115
  group({ rotate: a, around: RC }, () => {
    rect(RC[0] - 58, top + 20, 116, 150, { ...st, r: 26 })
    rect(RC[0] - 70, top, 140, 52, { ...(mouthSt || st), r: 20 })
  })
}

function silhouette(st) {
  blob(clampPts, st)
  blob(bodyPts.map(p => [...p, 0.6]), st)
  circle(RC[0], RC[1], R_OUT, st)
  emitterShapes(ANG, st)
  emitterShapes(ANG + 180, st)
}

layer("borde-pegatina", () => {
  silhouette({ fill: "#ffffff", stroke: "#ffffff", width: 90, join: "round" })
  silhouette({ fill: P.ink, stroke: P.ink, width: 30, join: "round" })
})

layer("abrazadera", () => {
  const c = blob(clampPts, { fill: P.clamp })
  clip(c, () => {
    rect(300, 640, 120, 340, { fill: P.clampHi })
    rect(560, 640, 200, 340, { fill: "#2a253c" })
  })
  // tornillos
  for (const x of [370, 654]) {
    circle(x, 915, 18, { fill: P.clampHi, stroke: P.ink, width: 8 })
  }
})

layer("cuerpo", () => {
  const b = blob(bodyPts.map(p => [...p, 0.6]), { fill: P.white })
  clip(b, () => {
    rect(200, 300, 700, 600, { fill: P.whiteSh })
    blob(shift(bodyPts, -26, -30).map(p => [...p, 0.6]), { fill: P.white })
    // junta del panel
    curve([[300, 690], [512, 750], [724, 690]], { stroke: alpha(P.ink, 0.5), width: 10 })
  })
  brush([[262, 600], [270, 500], [320, 410]], { width: 18, color: "#ffffff", taper: [0.3, 0.3] })
})

layer("emisor-trasero", () => {
  emitterShapes(ANG + 180, { fill: P.white, stroke: P.ink, width: 22 }, { fill: P.glow, stroke: P.ink, width: 22, glow: { blur: 18, color: P.glow } })
})

layer("anillo", () => {
  const ring = circle(RC[0], RC[1], R_OUT, { fill: P.ring, stroke: P.ink, width: 24 })
  clip(ring, () => {
    // sombra de un solo tono abajo-derecha
    circle(RC[0], RC[1], R_OUT, { fill: P.ringSh })
    circle(RC[0] - 26, RC[1] - 30, R_OUT, { fill: P.ring })
  })
  // luz arriba-izquierda
  path(`M ${RC[0] - 168} ${RC[1] - 10} A 172 172 0 0 1 ${RC[0] + 10} ${RC[1] - 168}`, { stroke: P.ringHi, width: 30, cap: "round", fill: "none" })
  // muescas de giro
  for (let i = 0; i < 4; i++) {
    const a = ANG + 45 + i * 90
    const p1 = polar(RC[0], RC[1], R_IN + 30, a)
    const p2 = polar(RC[0], RC[1], R_OUT - 30, a)
    line(p1[0], p1[1], p2[0], p2[1], { stroke: P.ink, width: 16, cap: "round" })
  }
  // núcleo blanco
  const hub = circle(RC[0], RC[1], R_IN, { fill: P.white, stroke: P.ink, width: 22 })
  clip(hub, () => {
    circle(RC[0], RC[1], R_IN, { fill: P.whiteSh })
    circle(RC[0] - 20, RC[1] - 22, R_IN, { fill: P.white })
  })
  circle(RC[0], RC[1], 48, { fill: P.clamp, stroke: P.ink, width: 16 })
  circle(RC[0] - 14, RC[1] - 14, 13, { fill: P.clampHi })
  brush([[RC[0] - 90, RC[1] + 10], [RC[0] - 74, RC[1] - 60], [RC[0] - 14, RC[1] - 92]], { width: 16, color: "#ffffff" })
})

layer("emisor-delantero", () => {
  emitterShapes(ANG, { fill: P.white, stroke: P.ink, width: 22 }, { fill: P.glow, stroke: P.ink, width: 22, glow: { blur: 18, color: P.glow } })
})

layer("detalles-emisores", () => {
  for (const a of [ANG, ANG + 180]) {
    const top = RC[1] - R_OUT - 115
    group({ rotate: a, around: RC }, () => {
      // sombra lateral del cañón
      // en el emisor trasero (girado 180°) la luz cae en el otro lado
      const s = a === ANG ? 1 : -1
      rect(RC[0] + s * 29 - 15, top + 62, 30, 56, { fill: P.whiteSh, r: 10 })
      rect(RC[0] - s * 34 - 6, top + 64, 12, 46, { fill: "#ffffff", r: 6 })
      // boca encendida
      rect(RC[0] - 40, top + 12, 80, 22, { fill: P.glowCore, r: 11 })
    })
  }
})
