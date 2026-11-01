// dibujo.js: escolta giratoria vista desde arriba, mirando hacia arriba (cañones arriba).
canvas({ width: 1024, height: 1024, background: null })

const P = {
  tinta: "#1c1a2e",
  rojo: "#e8212f",
  rojoOsc: "#8e1222",
  rojoLuz: "#ff7a7a",
  aro: "#f2f2f8",
  aroSom: "#aeb6d4",
  engr: "#2c2d40",
  engrLuz: "#3e4159",
  canon: "#2a2b3c",
  canonMed: "#3d3f55",
  franja: "#6d739a",
  punta: "#ff3b45",
  puntaLuz: "#ffb4b4",
  nucleo: "#f4f4fa",
  nucleoSom: "#b0b6d4",
}
const BORDE = "#ffffff"
const CX = 512, CY = 586
setLight(-1, -1)

rig({ origin: [CX, CY] })
bone("cuerpo", { at: [CX, CY] })
bone("anillo", { parent: "cuerpo", at: [CX, CY] })
bone("canones", { parent: "cuerpo", at: [CX, 300] })
bone("nucleo", { parent: "cuerpo", at: [CX, 580] })

// Cañones: silueta de pegatina detrás de todo (se mueve con ellos)
const tubos = (st) => {
  rect(412, 72, 96, 400, { r: 26, ...st })
  rect(516, 72, 96, 400, { r: 26, ...st })
}
part("canones", () => {
  tubos({ fill: BORDE, stroke: BORDE, width: 44 })
})

// Cuerpo: borde blanco de pegatina y contorno oscuro de la base
part("cuerpo", () => {
  circle(CX, CY, 352, { fill: BORDE, stroke: BORDE, width: 44 })
})

// Anillo: aro blanco, anillo rojo y corona dentada (gira entero)
const pa = (r, a) => [CX + r * Math.sin(a * Math.PI / 180), CY - r * Math.cos(a * Math.PI / 180)]
part("anillo", () => {
  circle(CX, CY, 352, { fill: P.tinta })
  circle(CX, CY, 337, { fill: P.aro })
  circle(CX, CY, 308, { fill: P.tinta })
  circle(CX, CY, 296, { fill: P.rojo })
  // muescas del anillo rojo: marcan el giro
  for (const a of [0, 90, 180, 270]) {
    brush([pa(300, a), pa(268, a)], { width: 16, taper: [0, 0.5], color: P.tinta })
  }
  for (const a of [45, 135, 225, 315]) {
    circle(...pa(276, a), 7, { fill: P.rojoOsc })
    circle(...pa(276, a) .map((v, i) => v - (i ? 2 : 2)), 3, { fill: P.rojoLuz, opacity: 0.8 })
  }
  // corona dentada
  circle(CX, CY, 206, { fill: P.tinta })
  circle(CX, CY, 194, { fill: P.engr })
  for (let i = 0; i < 12; i++) {
    const a0 = i * 30 + 3, a1 = i * 30 + 27
    const pts = [...arcPts(184, a0, a1), ...arcPts(156, a1, a0)]
    poly(pts, { fill: i % 2 ? P.engrLuz : mix(P.engrLuz, P.engr, 0.35) })
  }
  for (let i = 0; i < 12; i++) {
    brush([pa(206, i * 30), pa(176, i * 30)], { width: 15, taper: [0, 0], color: P.tinta })
  }
  // muesca de referencia en la corona (se ve el giro aunque sea simétrica)
  brush([pa(184, 180), pa(160, 180)], { width: 10, taper: [0, 0], color: P.punta })
  circle(CX, CY, 150, { fill: P.tinta })
})
function arcPts(r, a0, a1) {
  const n = 8, out = []
  for (let k = 0; k <= n; k++) out.push(pa(r, lerp(a0, a1, k / n)))
  return out
}

// Cuerpo: sombras y brillos fijos sobre el anillo (no giran)
part("cuerpo", () => {
  // sombra del aro blanco (abajo a la derecha)
  const sa = circle(CX, CY, 337, { fill: P.aroSom, blend: "multiply" })
  cut(sa, [circle(CX - 22, CY - 26, 334, { hidden: true }), circle(CX, CY, 308, { hidden: true })])
  // sombra del anillo rojo
  const sr = circle(CX, CY, 296, { fill: P.rojoOsc, opacity: 0.92 })
  cut(sr, [circle(CX - 26, CY - 30, 290, { hidden: true }), circle(CX, CY, 206, { hidden: true })])
  // brillos del rojo
  brush(arcPts(268, -62, -28), { width: 20, taper: [0.5, 0.5], color: P.rojoLuz, opacity: 0.85 })
  brush(arcPts(268, 26, 52), { width: 16, taper: [0.5, 0.5], color: P.rojoLuz, opacity: 0.7 })
  circle(...pa(318, -62), 7, { fill: "#ffffff" })
  brush(arcPts(322, -50, -30), { width: 6, taper: [0.5, 0.5], color: "#ffffff", opacity: 0.9 })
})

// Cañones
part("canones", () => {
  rect(470, 140, 84, 340, { fill: P.tinta })
  for (const x0 of [412, 516]) {
    const t = rect(x0, 72, 96, 400, { r: 26, fill: P.canon, stroke: P.tinta, width: 14 })
    clip(t, () => {
      rect(x0 + 48, 72, 48, 400, { fill: P.canonMed })
      rect(x0, 72, 96, 66, { fill: P.tinta })
      rect(x0, 196, 96, 14, { fill: P.tinta })
      rect(x0 + 29, 148, 10, 290, { r: 5, fill: P.franja })
    })
    rect(x0 + 18, 90, 60, 42, { r: 10, fill: P.punta, glow: { blur: 12, color: alpha(P.punta, 0.7) } })
    rect(x0 + 18, 116, 60, 16, { r: 6, fill: shade(P.punta, -0.35), opacity: 0.6 })
    rect(x0 + 30, 98, 36, 10, { r: 5, fill: P.puntaLuz })
    rect(x0 + 14, 130, 68, 10, { r: 5, fill: P.tinta })
  }
})

// Núcleo blanco (cuerpo) y bolita roja (nucleo)
part("cuerpo", () => {
  circle(CX, CY, 146, { fill: P.tinta })
  const n = circle(CX, CY, 136, { fill: P.nucleo })
  clip(n, () => {
    const s = circle(CX, CY, 136, { fill: P.nucleoSom })
    cut(s, [circle(CX - 26, CY - 30, 140, { hidden: true })])
    circle(CX, CY - 6, 110, { fill: radial(CX, CY - 6, 110, [alpha("#ff6a7a", 0.45), alpha("#ff6a7a", 0)]) })
  })
  brush(arcPts(118, -40, 10), { width: 9, taper: [0.5, 0.5], color: "#ffffff" })
})
part("nucleo", () => {
  circle(CX, 580, 72, { fill: "#4a1a2a" })
  circle(CX, 580, 60, { fill: radial(CX, 580, 62, ["#ff8a8a", P.rojo, P.rojoOsc], { fx: 495, fy: 562 }) })
  const s = circle(CX, 580, 60, { fill: P.rojoOsc, opacity: 0.5, blend: "multiply" })
  cut(s, [circle(CX - 10, 570, 60, { hidden: true })])
  circle(492, 564, 13, { fill: "#ffffff" })
  circle(530, 604, 4, { fill: "#ffffff", opacity: 0.5 })
})

action("flotar", { duration: 1.2, loop: true }, [
  // latido doble del núcleo (las poses no admiten escala: late con un golpe corto en y)
  [0, { anillo: 0, nucleo: { y: 0 } }],
  [0.1, { nucleo: { y: 6 } }],
  [0.2, { nucleo: { y: 0 } }],
  [0.25, { anillo: 90 }],
  [0.3, { nucleo: { y: 4 } }],
  [0.42, { nucleo: { y: 0 } }],
  [0.5, { anillo: 180 }],
  [0.75, { anillo: 270 }],
  [1, { anillo: 360, nucleo: { y: 0 } }],
])
action("disparo", { duration: 0.15, loop: false }, [
  [0, { canones: { y: 0 } }],
  [0.2, { canones: { y: 40 } }],
  [1, { canones: { y: 0 } }],
])
action("lanzar", { duration: 0.3, loop: true }, [
  [0, { anillo: 0 }],
  [0.25, { anillo: 90 }],
  [0.5, { anillo: 180 }],
  [0.75, { anillo: 270 }],
  [1, { anillo: 360 }],
])
