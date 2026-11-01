// dibujo.js: planeta partido (fondo para shooter vertical)
canvas({ width: 1024, height: 1024, background: null })
seed(7)

const P = {
  space: "#0b1024",     // tinte azul marino del polvo frío
  shadow: "#17162a",    // sombra profunda
  violet: "#3a3050",    // corteza violeta oscura
  crust: "#5f5762",     // corteza media gris parda
  light: "#7c7276",     // corteza iluminada (cálida, contenida)
  rim: "#4d5f8a",       // contraluz frío
  red: "#8e2a24",       // magma rojo
  orange: "#d0662c",    // magma naranja
  hot: "#e89a52",       // centro del núcleo (sin deslumbrar)
}

const C = [512, 520]   // centro del planeta
const R = 290
const rad = (d) => d * Math.PI / 180
const at = (a, r, c = C) => [c[0] + Math.cos(rad(a)) * r, c[1] + Math.sin(rad(a)) * r]

// grietas compartidas entre pedazos: del centro al borde, en zigzag
function crack(a) {
  const pts = []
  const n = 9
  for (let i = 0; i <= n; i++) {
    const t = i / n
    const r = t * R
    const j = (i === 0 || i === n) ? 0 : rand(-22, 22) * Math.sin(t * Math.PI)
    const p = at(a, r)
    pts.push([p[0] + Math.cos(rad(a + 90)) * j, p[1] + Math.sin(rad(a + 90)) * j])
  }
  return pts
}
const grietas = { a: crack(-62), b: crack(78), c: crack(196) }

// un pedazo = arco exterior + dos grietas
function piecePts(a0, a1, k0, k1) {
  const outer = []
  const n = Math.round((a1 - a0) / 3)
  for (let i = 0; i <= n; i++) {
    const a = lerp(a0, a1, i / n)
    outer.push(at(a, R + rand(-2.5, 2.5)))
  }
  const back = grietas[k1].slice().reverse().slice(1, -1)
  const fwd = grietas[k0].slice(0, -1)
  return [...outer, ...back, ...fwd]
}
const pieces = [
  { name: "pedazo_arriba", pts: piecePts(196, 298, "c", "a"), mid: 247, d: 78, rot: -7 },
  { name: "pedazo_derecha", pts: piecePts(-62, 78, "a", "b"), mid: 8, d: 92, rot: 9 },
  { name: "pedazo_abajo", pts: piecePts(78, 196, "b", "c"), mid: 137, d: 96, rot: -10 },
]

layer("resplandor", () => {
  // halo cálido muy contenido detrás del núcleo
  circle(C[0], C[1], 230, { fill: P.red, opacity: 0.35, blur: 70 })
  circle(C[0], C[1], 150, { fill: P.orange, opacity: 0.25, blur: 60 })
  // atmósfera fría alrededor del planeta
})

layer("nucleo", () => {
  // núcleo fundido
  element("nucleo", { pivot: C }, () => {
    const core = circle(C[0], C[1], 215, { fill: radial(C[0], C[1], 215, [[0, P.hot], [0.35, P.orange], [0.75, P.red], [1, shade(P.red, -0.5)]], { fx: C[0] - 40, fy: C[1] - 50 }) })
    clip(core, () => {
      // remolinos de magma
      for (let i = 0; i < 22; i++) {
        const a = rand(0, 360), r0 = rand(30, 220)
        const pts = arc(C[0], C[1], r0, a, a + rand(30, 80), 6).map(([x, y]) => [x + rand(-6, 6), y + rand(-6, 6)])
        brush(pts, { width: rand(4, 12), color: pick([alpha(P.hot, 0.25), alpha(P.orange, 0.3), alpha(P.red, 0.3)]), blur: 5 })
      }
      // sombra del núcleo abajo a la derecha
      circle(C[0] + 120, C[1] + 130, 230, { fill: shade(P.red, -0.6), opacity: 0.55, blur: 60, blend: "multiply" })
    })
  })
})

// gira y desplaza un punto según su pedazo
function moveFn(pc) {
  const dir = [Math.cos(rad(pc.mid)), Math.sin(rad(pc.mid))]
  const off = [dir[0] * pc.d, dir[1] * pc.d]
  const cr = Math.cos(rad(pc.rot)), sr = Math.sin(rad(pc.rot))
  return ([x, y]) => {
    const dx = x - C[0], dy = y - C[1]
    return [C[0] + dx * cr - dy * sr + off[0], C[1] + dx * sr + dy * cr + off[1]]
  }
}

layer("pedazos", () => {
  for (const pc of pieces) {
    const mv = moveFn(pc)
    const pts = pc.pts.map(mv)
    const pcC = mv(C)
    const dir = [Math.cos(rad(pc.mid)), Math.sin(rad(pc.mid))]
    element(pc.name, { pivot: pcC }, () => {
      // cara de fractura: copia desplazada hacia el núcleo (el grosor del pedazo)
      const wall = shift(pts, -dir[0] * 20, -dir[1] * 20)
      const w = poly(wall, { fill: radial(pcC[0] - dir[0] * 20, pcC[1] - dir[1] * 20, R, [[0, P.orange], [0.45, P.red], [0.75, "#3b2236"], [1, P.shadow]]) })
      clip(w, () => {
        // estratos de la corteza en la cara de fractura
        for (let k = 1; k < 6; k++) {
          circle(pcC[0] - dir[0] * 20, pcC[1] - dir[1] * 20, R * (0.45 + k * 0.1), { fill: "none", stroke: shade(P.shadow, -0.2), width: rand(2, 5), opacity: 0.4, blur: 1 })
        }
      })
      const s = poly(pts, { fill: P.crust })
      clip(s, () => {
        // volumen esférico: luz arriba-izquierda
        circle(pcC[0], pcC[1], R + 4, { fill: radial(pcC[0] - 150, pcC[1] - 170, 560, [[0, P.light], [0.35, P.crust], [0.6, P.violet], [0.85, P.shadow]]) })
        // manchas violeta y pardas
        for (let i = 0; i < 24; i++) {
          const p = at(rand(0, 360), rand(20, R), pcC)
          blob(roughen(ring(p[0], p[1], rand(25, 70), rand(12, 35), 8, rand(0, 180)), 10),
            { fill: pick([P.violet, "#6a5e58", "#4a3e4c", "#574b5c"]), opacity: 0.3, blur: rand(4, 12) })
        }
        // crestas y vetas de la corteza
        for (let i = 0; i < 14; i++) {
          const p = at(rand(0, 360), rand(40, R - 30), pcC)
          const a = rand(0, 180)
          const pts = [at(a, -rand(30, 60), p), at(a + rand(-20, 20), 0, p), at(a, rand(30, 60), p)]
          brush(pts, { width: rand(2, 5), color: alpha(shade(P.violet, -0.3), 0.35), blur: 1.5 })
          brush(shift(pts, -2.5, -2.5), { width: rand(1, 3), color: alpha(P.light, 0.18), blur: 1 })
        }
        // cráteres
        for (let i = 0; i < 16; i++) {
          const p = at(rand(0, 360), rand(30, R - 20), pcC)
          const r = rand(8, 30)
          const dx = p[0] - pcC[0], dy = p[1] - pcC[1]
          const dd = Math.hypot(dx, dy) / R
          const sq = 1 - dd * 0.55   // achatado hacia el borde
          const ang = Math.atan2(dy, dx) * 180 / Math.PI + 90
          ellipse(p[0], p[1], r, r * sq, { fill: shade(P.violet, -0.3), opacity: 0.45, rotate: ang, blend: "multiply", blur: 1.5 })
          ellipse(p[0] + r * 0.18, p[1] + r * 0.18, r * 0.8, r * sq * 0.8, { fill: shade(P.light, 0.1), opacity: 0.12, rotate: ang, blur: 2 })
        }
        // sombra profunda abajo a la derecha (terminador suave)
        circle(pcC[0] + 210, pcC[1] + 230, 300, { fill: P.shadow, opacity: 0.75, blur: 70, blend: "multiply" })
        // contraluz frío en el borde opuesto
        clip(circle(pcC[0] + 150, pcC[1] + 170, 300, { hidden: true }), () => {
          circle(pcC[0], pcC[1], R - 6, { fill: "none", stroke: P.rim, width: 20, blur: 12, opacity: 0.4, blend: "screen" })
        })
        // resplandor del magma en los bordes de las grietas
        const inner = pts.filter(p => Math.hypot(p[0] - pcC[0], p[1] - pcC[1]) < R - 10)
        brush(inner, { width: 26, color: alpha(P.orange, 0.5), blur: 12, taper: [0.1, 0.1] })
        brush(inner, { width: 6, color: alpha(P.hot, 0.35), blur: 3, taper: [0.15, 0.15] })
        grain({ opacity: 0.08, size: 1.2 })
      })
    })
  }
})

layer("escombros", () => {
  // rocas pequeñas que se alejan, sobre todo por las grietas
  const gaps = [-62, 78, 196]
  function rock(name, x, y, r, i) {
    const pts = roughen(ring(x, y, r, r * rand(0.6, 0.9), 7, rand(0, 180)), r * 0.25, { detail: 2 })
    const s = blob(pts.map(p => [p[0], p[1], 0.5]), { fill: P.crust })
    clip(s, () => {
      const far = clamp((Math.hypot(x - C[0], y - C[1]) - R) / (R * 0.6), 0, 1)
      const lit = radial(x - r * 0.5, y - r * 0.6, r * 2, [[0, mix(P.light, P.violet, 0.3 + far * 0.3)], [0.4, mix(P.crust, P.violet, 0.5)], [0.75, P.violet], [1, P.shadow]])
      rect(x - r * 2, y - r * 2, r * 4, r * 4, { fill: lit })
      // reflejo cálido del magma en la cara que mira al núcleo
      const a = Math.atan2(C[1] - y, C[0] - x)
      circle(x + Math.cos(a) * r, y + Math.sin(a) * r, r * 0.8, { fill: P.orange, opacity: 0.35, blur: r * 0.4, blend: "screen" })
    })
  }
  let n = 0
  // unos pocos trozos medianos lejos, que dan escala
  for (const [a, d, r] of [[-40, 1.45, 26], [100, 1.5, 20], [215, 1.42, 22], [160, 1.55, 14], [20, 1.55, 16], [-120, 1.38, 15]]) {
    const p = at(a, R * d)
    element("trozo_" + (n++), { pivot: p }, () => rock("t", p[0], p[1], r, 0))
  }
  for (let i = 0; i < 60; i++) {
    const g = i < 36 ? pick(gaps) + rand(-16, 16) : rand(0, 360)
    const dist = i < 36 ? rand(R * 0.85, R * 1.6) : rand(R * 1.08, R * 1.6)
    const p = at(g, dist)
    const r = lerp(11, 3, (dist - R * 0.85) / (R * 0.75)) * rand(0.6, 1.2)
    if (p[0] < 40 || p[0] > 984 || p[1] < 40 || p[1] > 984) continue
    element("roca_" + (n++), { pivot: p }, () => rock("r", p[0], p[1], r, i))
  }
})

layer("polvo", () => {
  // estelas de polvo que salen por las grietas
  for (const g of [-62, 78, 196]) {
    for (let k = 0; k < 4; k++) {
      const a = g + rand(-10, 10)
      const pts = [at(a, R * 0.7), at(a + rand(-4, 4), R * 1.05), at(a + rand(-8, 8), R * rand(1.3, 1.55))]
      brush(pts, { width: [4, rand(18, 30), 6], color: alpha(mix(P.orange, P.rim, 0.5), 0.12), blur: 10 })
    }
  }
  // chispas y polvo brillante
  for (let i = 0; i < 90; i++) {
    const g = rand() < 0.6 ? pick([-62, 78, 196]) + rand(-20, 20) : rand(0, 360)
    const p = at(g, rand(R * 0.8, R * 1.6))
    if (p[0] < 20 || p[0] > 1004 || p[1] < 20 || p[1] > 1004) continue
    const hot = rand() < 0.5
    circle(p[0], p[1], rand(0.8, 2.2), { fill: hot ? P.hot : "#9fb0d6", opacity: rand(0.25, 0.6), glow: { blur: 4, color: alpha(hot ? P.orange : P.rim, 0.5) } })
  }
})
