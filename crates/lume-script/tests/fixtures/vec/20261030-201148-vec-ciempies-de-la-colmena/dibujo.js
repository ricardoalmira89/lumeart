// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1280x720: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1280, height: 720, background: null }) // null = fondo transparente (objeto suelto)
setLight(-1, -1)

const P = {
  ink: "#12261a",      // contorno
  dark: "#1f6e2c",     // quitina oscura
  darkS: "#123f22",    // sombra de la quitina oscura
  lime: "#8fe23a",     // quitina lima
  limeS: "#3f9a2e",    // sombra lima
  limeL: "#e0ff8a",    // brillo lima
  purple: "#7a3cc8",   // placa
  purpleS: "#4a1f86",
  purpleL: "#c9a0ff",
  amber: "#f5ab1e",    // mandíbulas
  amberS: "#b55e0c",
  red: "#ff2a2a",      // ojos (punto débil)
  magenta: "#ff2fd0",  // poro
  white: "#ffffff",
}
const WHITE = 34, INK = 12

// pieza con sombra de un tono: base iluminada desplazada hacia la luz sobre el tono de sombra
function shaded(pts, base, sh, d = 22) {
  const s = blob(pts, { fill: sh })
  clip(s, () => blob(shift(grow(pts, 1.02), -d, -d), { fill: base }))
  return s
}

// ---------- geometría: cabeza (centro 320, 345) ----------
const HX = 320
const head = ring(HX, 335, 205, 170, 12)
const headPlate = [[HX, 205], [HX + 105, 225], [HX + 135, 295, 0.5], [HX + 85, 360], [HX, 378], [HX - 85, 360], [HX - 135, 295, 0.5], [HX - 105, 225]]
const mandL = [[HX - 62, 478], [HX - 108, 515], [HX - 108, 570], [HX - 72, 610], [HX - 16, 626, 0], [HX - 58, 584], [HX - 70, 540], [HX - 40, 500]]
const mandR = mandL.map(([x, y, s]) => s === undefined ? [2 * HX - x, y] : [2 * HX - x, y, s])
const antL = [[HX - 150, 440], [HX - 205, 490], [HX - 225, 545]]
const antR = antL.map(([x, y]) => [2 * HX - x, y])
const eyes = [[HX - 92, 432], [HX + 92, 432]]

// ---------- geometría: segmento (centro 960, 360) ----------
const SX = 960, SY = 360
const seg = ring(SX, SY, 172, 160, 12)
const segPlate = [[SX, 262], [SX + 82, 300], [SX + 96, 380, 0.5], [SX + 50, 445], [SX, 458], [SX - 50, 445], [SX - 96, 380, 0.5], [SX - 82, 300]]
function leg(y, dy) {
  // pata derecha: muslo hacia fuera, garra hacia abajo (atrás del avance)
  return [[SX + 140, y - 26], [SX + 215, y - 30 + dy], [SX + 262, y + 6 + dy], [SX + 270, y + 62 + dy, 0], [SX + 238, y + 22 + dy], [SX + 200, y + 12 + dy], [SX + 140, y + 28]]
}
const legsR = [leg(300, 0), leg(398, 8)]
const legsL = legsR.map((l) => l.map(([x, y, s]) => s === undefined ? [2 * SX - x, y] : [2 * SX - x, y, s]))

// ---------- capa de pegatina (borde blanco + contorno oscuro) ----------
layer("borde-pegatina", () => {
  const w = { fill: P.white, stroke: P.white, width: WHITE, join: "round" }
  for (const a of [antL, antR]) curve(a, { stroke: P.white, width: 26 + WHITE, cap: "round" })
  blob(head, w); blob(mandL, w); blob(mandR, w)
  blob(seg, w); for (const l of [...legsR, ...legsL]) blob(l, w)
  const k = { fill: P.ink, stroke: P.ink, width: INK * 2, join: "round" }
  for (const a of [antL, antR]) curve(a, { stroke: P.ink, width: 26 + INK * 2, cap: "round" })
  for (const a of [antL, antR]) circle(a[2][0], a[2][1], 22 + INK, { fill: P.ink })
  blob(mandL, k); blob(mandR, k); blob(head, k)
  for (const l of [...legsR, ...legsL]) blob(l, k)
  blob(seg, k)
})

// ---------- sprite 1: cabeza ----------
element("cabeza", { pivot: [HX, 360] }, () => {
  // antenas
  for (const a of [antL, antR]) {
    curve(a, { stroke: P.limeS, width: 26, cap: "round" })
    curve(a.slice(0, 2).concat([[lerp(a[1][0], a[2][0], 0.6), lerp(a[1][1], a[2][1], 0.6)]]), { stroke: P.lime, width: 12, cap: "round" })
    circle(a[2][0], a[2][1], 22, { fill: P.dark })
    circle(a[2][0] - 6, a[2][1] - 7, 7, { fill: P.limeL })
  }
  // mandíbulas
  for (const m of [mandL, mandR]) {
    const s = shaded(m, P.amber, P.amberS, 14)
    ink(s, { width: 4, weight: 2.5, color: P.ink })
  }
  brush([[HX - 100, 528], [HX - 94, 570], [HX - 68, 600]], { width: 8, color: "#ffe9a8" })
  brush([[HX + 100, 528], [HX + 96, 556]], { width: 5, color: "#ffe9a8", })
  // cráneo
  const h = shaded(head, P.lime, P.limeS, 30)
  clip(h, () => {
    // banda trasera de quitina oscura
    blob([[HX - 230, 150], [HX + 230, 150], [HX + 230, 230], [HX, 205], [HX - 230, 230]], { fill: P.dark })
    // brillo
    blob([[HX - 175, 290], [HX - 150, 235], [HX - 105, 212], [HX - 120, 250], [HX - 152, 300]], { fill: P.limeL })
    circle(HX - 170, 325, 10, { fill: P.limeL })
  })
  ink(h, { width: 5, weight: 2.5, color: P.ink })
  // placa morada
  const pl = shaded(headPlate, P.purple, P.purpleS, 18)
  clip(pl, () => {
    line(HX, 210, HX, 375, { stroke: P.purpleS, width: 8 })
    brush([[HX - 110, 280], [HX - 85, 238], [HX - 30, 220]], { width: 14, color: P.purpleL, taper: [0.3, 0.3] })
  })
  ink(pl, { width: 5, weight: 2, color: P.ink })
  // ojos rojos brillantes (punto débil)
  clip(h, () => {
    // placas de quitina oscura que enmarcan los ojos
    for (const [ex, ey] of eyes) blob(ring(ex, ey + 6, 68, 60, 9), { fill: P.dark, stroke: P.ink, width: 8 })
  })
  for (const [ex, ey] of eyes) {
    circle(ex, ey, 46, { fill: P.ink })
    const e = circle(ex, ey, 38, { fill: radial(ex - 8, ey - 8, 40, ["#ffb3a0", P.red, "#b3001a"]), glow: { blur: 14, color: alpha(P.red, 0.8) } })
    circle(ex - 13, ey - 14, 11, { fill: P.white })
    circle(ex + 12, ey + 10, 4.5, { fill: P.white, opacity: 0.8 })
  }
})

// ---------- sprite 2: segmento ----------
element("segmento", { pivot: [SX, SY] }, () => {
  // patas
  for (const l of [...legsR, ...legsL]) {
    const s = shaded(l, P.dark, P.darkS, 12)
    ink(s, { width: 4, weight: 2.5, color: P.ink })
  }
  for (const sx of [1, -1]) for (const [y, dy] of [[300, 0], [398, 8]]) {
    brush([[SX + sx * 180, y - 18 + dy * 0.5], [SX + sx * 228, y - 16 + dy]], { width: 7, color: P.lime })
  }
  // caparazón
  const b = shaded(seg, P.dark, P.darkS, 30)
  clip(b, () => {
    // borde lima de la quitina (luz)
    blob([[SX - 160, 330], [SX - 125, 255], [SX - 60, 212], [SX, 205], [SX - 55, 232], [SX - 115, 280], [SX - 145, 345]], { fill: P.lime })
    circle(SX - 150, 380, 9, { fill: P.limeL })
    brush([[SX - 140, 300], [SX - 105, 252], [SX - 60, 228]], { width: 7, color: P.limeL })
  })
  ink(b, { width: 5, weight: 2.5, color: P.ink })
  // placa morada
  const pl = shaded(segPlate, P.purple, P.purpleS, 16)
  clip(pl, () => {
    line(SX, 265, SX, 455, { stroke: P.purpleS, width: 7 })
    brush([[SX - 78, 360], [SX - 70, 310], [SX - 25, 280]], { width: 12, color: P.purpleL, taper: [0.3, 0.3] })
  })
  ink(pl, { width: 5, weight: 2, color: P.ink })
  // poro magenta encendido
  circle(SX, 232, 25, { fill: P.ink })
  circle(SX, 232, 18, { fill: radial(SX - 4, 228, 20, ["#ffd0f4", P.magenta, "#a0108a"]), glow: { blur: 12, color: alpha(P.magenta, 0.8) } })
  circle(SX - 6, 226, 5, { fill: P.white })
})
