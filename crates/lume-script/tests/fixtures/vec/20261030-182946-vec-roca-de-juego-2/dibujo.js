// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  ink: "#221b26",     // contorno
  shadow: "#3b3442",  // sombra de un solo tono
  base: "#5e5660",    // pizarra cálida
  light: "#7f7580",   // plano iluminado
  hi: "#b7aeb3",      // brillos
  brown: "#6d5848",   // toque marrón
  crys: "#d8bcff",    // cristal violeta
  crysHi: "#f6eeff",
  crysDk: "#9a72d6",
}
setLight(-1, -1)

// Silueta de patata: elipse girada con radios irregulares y algunas aristas
const CX = 512, CY = 512, RX = 410, RY = 268, ROT = -30
const R = [1.0, 0.95, 1.07, 1.02, 0.9, 1.03, 0.98, 1.05, 0.95, 0.88, 1.05, 1.0]
const S = [1, 1, 1, 0.3, 1, 1, 1, 0.5, 1, 0.3, 1, 1]
const body = range(12).map(i => {
  const a = (i / 12) * Math.PI * 2
  const x = Math.cos(a) * RX * R[i], y = Math.sin(a) * RY * R[i]
  const c = Math.cos(ROT * Math.PI / 180), s = Math.sin(ROT * Math.PI / 180)
  return [CX + x * c - y * s, CY + x * s + y * c, S[i]]
})
// punto en coordenadas locales de la roca (u a lo largo, v a lo ancho, en -1..1)
function L(u, v) {
  const x = u * RX, y = v * RY
  const c = Math.cos(ROT * Math.PI / 180), s = Math.sin(ROT * Math.PI / 180)
  return [CX + x * c - y * s, CY + x * s + y * c]
}

element("roca", { pivot: [CX, CY] }, () => {
  layer("pegatina", () => {
    // borde blanco detrás de toda la silueta
    blob(body, { fill: P.ink, stroke: "#ffffff", width: 64, behind: true, join: "round" })
  })

  let rock
  layer("cuerpo", () => {
    rock = blob(body, { fill: P.shadow })
    clip(rock, () => {
      // masa en medio tono (deja la sombra abajo a la derecha)
      blob(shift(grow(body, 0.9), -50, -58), { fill: P.base })
      // toque marrón cálido en el centro
      blob(shift(grow(body, 0.6), 30, -10), { fill: P.brown, opacity: 0.35, blur: 40 })
      // plano de luz arriba a la izquierda
      blob([L(-0.95, -0.15), L(-0.7, -0.75), L(-0.1, -0.98), L(0.45, -0.9), L(0.2, -0.55), L(-0.35, -0.35), L(-0.75, 0.05)], { fill: P.light })
    })
  })

  layer("aristas", () => {
    clip(rock, () => {
      // arista principal: separa plano de luz del medio tono
      brush([L(-0.8, 0.08), L(-0.35, -0.32), L(0.2, -0.52), L(0.5, -0.92)], { width: [4, 9, 6, 3], color: P.ink, taper: [0.2, 0.3] })
      brush([L(-0.82, 0.02), L(-0.37, -0.38), L(0.18, -0.58)], { width: [2, 5, 2], color: P.hi, opacity: 0.7 })
      // arista inferior hacia la sombra
      brush([L(0.35, 0.95), L(0.5, 0.45), L(0.95, 0.15)], { width: [3, 8, 3], color: P.ink })
      brush([L(-0.3, 0.95), L(-0.15, 0.6), L(0.05, 0.5)], { width: [2, 6, 2], color: P.ink, opacity: 0.85 })
    })
  })

  layer("crateres", () => {
    clip(rock, () => {
      // cráter: borde oscuro, sombra interior arriba-izq, luz en el borde abajo-der
      function crater(u, v, rx, ry) {
        const [x, y] = L(u, v)
        blob(turn(ring(x + rx * 0.06, y + ry * 0.08, rx * 1.2, ry * 1.2, 9), ROT), { fill: P.light, opacity: 0.5 })
        const hole = blob(turn(jitter(ring(x, y, rx, ry, 9), rx * 0.04), ROT), { fill: P.shadow })
        clip(hole, () => {
          ellipse(x + rx * 0.22, y + ry * 0.3, rx * 0.95, ry * 0.95, { fill: P.base, rotate: ROT })
        })
        ink(hole, { width: Math.max(3, rx * 0.09), weight: 1.8, color: P.ink })
        brush(arc(x, y, rx * 1.12, 100, 190, 8), { width: [1, Math.max(2, rx * 0.08), 1], color: P.hi, opacity: 0.8 })
      }
      crater(-0.42, 0.28, 108, 80)
      crater(0.45, -0.12, 86, 68)
      crater(-0.05, -0.6, 34, 24)
      crater(0.1, 0.55, 40, 28)
      crater(0.78, 0.28, 30, 22)
      crater(-0.78, -0.3, 26, 19)
      crater(0.25, 0.2, 22, 16)
      crater(0.62, 0.62, 20, 14)
    })
  })

  layer("vetas", () => {
    clip(rock, () => {
      function vein(pts, w) {
        polyline(pts, { stroke: P.crys, width: w * 3.4, opacity: 0.4, blur: 8, join: "round", cap: "round" })
        polyline(pts, { stroke: P.ink, width: w * 1.7, join: "round", cap: "round" })
        polyline(pts, { stroke: P.crysDk, width: w * 1.3, join: "round", cap: "round" })
        polyline(pts, { stroke: P.crys, width: w, join: "round", cap: "round", glow: { blur: 10, color: alpha(P.crys, 0.8) } })
        polyline(pts.slice(0, 3), { stroke: P.crysHi, width: w * 0.4, cap: "round" })
      }
      vein([L(-0.18, -0.28), L(-0.1, -0.18), L(0.0, -0.12), L(-0.03, 0.02), L(0.06, 0.12), L(0.12, 0.3), L(0.08, 0.42)], 11)
      vein([L(0.6, -0.62), L(0.68, -0.48), L(0.74, -0.42), L(0.71, -0.28), L(0.82, -0.15)], 9)
      vein([L(-0.5, -0.8), L(-0.42, -0.68), L(-0.46, -0.58), L(-0.36, -0.46)], 8)
    })
  })

  layer("brillos", () => {
    clip(rock, () => {
      const [a, b] = L(-0.55, -0.55)
      ellipse(a, b, 34, 14, { fill: P.hi, rotate: ROT })
      circle(a + 44, b - 22, 7, { fill: P.hi })
      const [c, d] = L(0.1, -0.82)
      ellipse(c, d, 20, 8, { fill: P.hi, rotate: ROT, opacity: 0.85 })
    })
  })

  layer("tinta", () => {
    ink(rock, { width: 14, weight: 2.4, color: P.ink })
  })

  layer("textura", () => {
    clip(rock, () => {
      seed(7)
      for (let i = 0; i < 40; i++) {
        const [x, y] = L(rand(-0.9, 0.9), rand(-0.8, 0.8))
        circle(x, y, rand(2, 5), { fill: rand() < 0.5 ? P.ink : P.hi, opacity: 0.35 })
      }
      grain({ opacity: 0.06 })
    })
  })
})
