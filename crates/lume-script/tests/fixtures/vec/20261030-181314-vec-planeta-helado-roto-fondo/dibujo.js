// Planeta helado roto: pieza de fondo lejana, poco contraste, sin contornos.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)
seed(7)

const P = {
  hielo: "#b4c1d0",     // luz del hielo (claro contenido)
  hieloMed: "#8393ad",  // medio
  hieloSom: "#46537a",  // sombra azul
  hieloOsc: "#232c4e",  // sombra profunda
  calido: "#cfcac0",    // luz cálida tenue
  roca: "#5a4875",      // roca violeta (estrato alto)
  rocaOsc: "#33284d",   // roca profunda
  rocaNeg: "#1d1733",
  turq: "#46b8b4",      // resplandor de la grieta
  turqClaro: "#93d9d2",
}

const C = [478, 524]  // centro del planeta
const R = 352
// Anillo inclinado
const RING = { rx: 480, ry: 112, rot: -16 }
function ringPt(t, k = 1, dy = 0) {
  const x = RING.rx * k * Math.cos(t), y = RING.ry * k * Math.sin(t) + dy
  const a = RING.rot * Math.PI / 180
  return [C[0] + x * Math.cos(a) - y * Math.sin(a), C[1] + x * Math.sin(a) + y * Math.cos(a)]
}
// Mordisco: borde de la herida (hacia el planeta) y borde exterior (silueta del hueco)
const BA = -24 * Math.PI / 180
// círculo del impacto a distancia d del centro, con radio r
const biteCircle = (d, r) => [[C[0] + Math.cos(BA) * d, C[1] + Math.sin(BA) * d], r]
const [BITE_C, RB] = biteCircle(R * 0.9, 262)  // herida en el hielo
const [OUT_C, RO] = biteCircle(R * 1.02, 238)  // silueta del hueco (roca al fondo)
// Radio de la silueta en el ángulo a: la esfera recortada por el borde cercano de un círculo (c, r)
function nearEdge(a, c, r) {
  const dx = c[0] - C[0], dy = c[1] - C[1]
  const d = Math.hypot(dx, dy), dl = Math.atan2(dy, dx) - a
  const s = d * Math.sin(dl)
  if (Math.abs(s) >= r || Math.cos(dl) <= 0) return Infinity
  return Math.max(0, d * Math.cos(dl) - Math.sqrt(r * r - s * s))
}
// Mellas irregulares del borde roto
const jag = (a, k) => k * (Math.sin(a * 11 + 1) * 0.55 + Math.sin(a * 19 + 3) * 0.3 + Math.sin(a * 37) * 0.15)
function cutShape(cIn, rIn, rough, step = 1) {
  return range(Math.round(360 / step)).map(i => {
    const a = i * step * Math.PI / 180
    let rho = R
    const e = nearEdge(a, cIn, rIn)
    if (e < R) rho = Math.max(0, e + jag(a, rough))
    return [C[0] + Math.cos(a) * rho, C[1] + Math.sin(a) * rho]
  })
}
const silPts = cutShape(OUT_C, RO, 7)      // silueta del planeta roto
// Pared de roca: banda alrededor del hueco, medida desde su centro; más gruesa en el fondo
const BACK = BA + Math.PI
function wallW(phi) {
  const dl = Math.atan2(Math.sin(phi - BACK), Math.cos(phi - BACK)) * 180 / Math.PI
  return 14 + 78 * Math.max(0, 1 - (dl / 74) ** 2)
}
// estrato: t = 0 en la superficie del hielo, t = 1 en el borde del hueco
function strata(t, rough) {
  return range(240).map(i => {
    const phi = i / 240 * Math.PI * 2
    const rho = RO + wallW(phi) * (1 - t) + (t < 1 ? Math.sin(phi * 14 + t * 7) * rough + jag(phi * 1.3 + t * 9, rough * 0.8) : 0)
    return [OUT_C[0] + Math.cos(phi) * rho, OUT_C[1] + Math.sin(phi) * rho]
  })
}

// Esquirla de hielo o roca con su cara iluminada (arriba-izquierda)
function chunk(x, y, s, rock, dim) {
  const n = Math.floor(rand(5, 8))
  const pts = jitter(ring(x, y, s, s * rand(0.55, 0.9), n, rand(0, 360)), s * 0.25).map(p => [p[0], p[1], pick([0, 0.5, 0.5])])
  const base = mix(rock ? P.rocaOsc : P.hieloSom, P.hieloOsc, dim)
  const sh = blob(pts, { fill: base })
  if (s > 3) clip(sh, () => {
    blob(shift(grow(pts, 0.85), -s * 0.3, -s * 0.3), { fill: mix(rock ? P.roca : P.hieloMed, P.hieloSom, 0.2 + dim * 0.8), opacity: 0.8 })
    blob(shift(pts, s * 0.45, s * 0.45), { fill: alpha(P.hieloOsc, 0.5) })
  })
}
// Recorrido del anillo: s = 0 nace en el hueco, s = 1 da la vuelta entera (pasa primero por detrás)
const debris = range(260).map(() => {
  const s = Math.pow(rand(), 1.7)
  const t = -s * Math.PI * 2 - 0.08
  const spread = lerp(0.05, 0.13, s)
  const k = 1 + rand(-spread, spread), dy = rand(-20, 20) * (0.5 + s)
  const [x, y] = ringPt(t, k, dy)
  return { x, y, s, t, size: lerp(20, 4, Math.sqrt(s)) * rand(0.4, 1.1), rock: rand() < 0.3, front: Math.sin(t) > 0 }
})
// Esquirlas que salen del hueco hacia el anillo
const burst = range(34).map(() => {
  const u = rand()
  const from = [OUT_C[0] - Math.cos(BA) * 210, OUT_C[1] - Math.sin(BA) * 210]
  const to = ringPt(-0.05, 1, 0)
  return { x: lerp(from[0], to[0], u) + rand(-50, 50) * (1 - u * 0.5), y: lerp(from[1], to[1], u) + rand(-60, 60) * (1 - u * 0.5), size: rand(5, 22), rock: rand() < 0.55, u }
})

// Banda de polvo: tramos sueltos de hebras anchas y tenues, más densa cerca del hueco
function dust(t0, t1, a) {
  for (let i = 0; i < 26; i++) {
    const k = 1 + rand(-0.1, 0.1), dy = rand(-16, 16)
    const ta = lerp(t0, t1, rand(0, 0.6)), tb = Math.min(t1, ta + rand(0.6, 1.8))
    const arc = range(24).map(j => ringPt(lerp(ta, tb, j / 23), k + Math.sin(j * 0.7 + i) * 0.01, dy))
    brush(arc, { width: rand(4, 16), color: alpha(pick([P.hieloMed, P.hieloSom, P.hieloSom]), a * rand(0.5, 1)), taper: [0.4, 0.4] })
  }
}

layer("planeta_helado_roto", { parallax: 0.15 }, () => {
  element("anillo_trasero", { pivot: C }, () => {
    // banda de polvo trasera
    // banda de polvo: muchas hebras finas en lugar de un tubo
    dust(Math.PI, Math.PI * 2, 0.05)
    // esquirlas traseras: más apagadas y frías
    for (const d of debris.filter(d => !d.front)) chunk(d.x, d.y, d.size, d.rock, 0.45)
  })

  element("planeta", { pivot: C }, () => {
    // silueta del planeta roto con la superficie de hielo
    const sil = poly(silPts, { fill: radial(C[0] - 150, C[1] - 160, R * 1.75, [[0, P.calido], [0.22, P.hielo], [0.5, P.hieloMed], [0.78, P.hieloSom], [1, P.hieloOsc]]) })

    clip(sil, () => {
      // volumen: capas suaves
      ellipse(C[0] - 120, C[1] - 130, 170, 140, { fill: alpha(P.hielo, 0.35), blur: 60 })
      ellipse(C[0] - 260, C[1] + 160, 110, 160, { fill: alpha(P.hieloMed, 0.3), blur: 50 })
      // bandas de hielo siguiendo la curvatura
      for (let i = 0; i < 14; i++) {
        const y = C[1] - R + 30 + i * 48 + rand(-10, 10)
        const half = Math.sqrt(Math.max(0, R * R - (y - C[1]) ** 2))
        const pts = range(6).map(j => {
          const u = j / 5
          return [C[0] - half + u * half * 2, y + Math.sin(u * Math.PI) * 20 + rand(-5, 5)]
        })
        brush(pts, { width: rand(8, 22), color: alpha(i % 2 ? P.hielo : P.hieloSom, 0.12), taper: [0.4, 0.4], blur: 3 })
      }
      // casquete polar
      ellipse(C[0] - 80, C[1] - R + 50, 200, 60, { fill: alpha(P.hielo, 0.3), blur: 22, rotate: -14 })
      // cráteres viejos, suaves
      for (let i = 0; i < 14; i++) {
        const a = rand(0, Math.PI * 2), d = Math.sqrt(rand()) * (R - 40)
        const x = C[0] + Math.cos(a) * d, y = C[1] + Math.sin(a) * d
        const r = rand(7, 24)
        ellipse(x + r * 0.15, y + r * 0.15, r, r * 0.8, { fill: alpha(P.hieloSom, 0.16), blur: 3 })
        ellipse(x - r * 0.15, y - r * 0.15, r * 0.7, r * 0.5, { fill: alpha(P.hielo, 0.1), blur: 3 })
      }
      // grietas de hielo que irradian desde el impacto
      for (let i = 0; i < 10; i++) {
        const a = BACK + rand(-60, 60) * Math.PI / 180
        const len = rand(110, 280), r0 = RO + wallW(a) - 6
        const pts = range(5).map(j => {
          const u = j / 4
          return [OUT_C[0] + Math.cos(a) * (r0 + u * len) + rand(-12, 12), OUT_C[1] + Math.sin(a) * (r0 + u * len) + rand(-12, 12)]
        })
        brush(pts, { width: [3, 1.8, 0.5], color: alpha(P.hieloOsc, 0.28), taper: [0, 0.8] })
      }
      // terminador: sombra azul hacia abajo a la derecha
      circle(C[0] + 170, C[1] + 150, R * 0.95, { fill: alpha(P.hieloOsc, 0.5), blend: "multiply", blur: 80 })
      // contraluz frío y tenue en el borde en sombra (abajo a la derecha)
      circle(C[0] - 16, C[1] - 16, R, { fill: "none", stroke: alpha(mix(P.hieloMed, P.turq, 0.3), 0.22), width: 16, blur: 8 })
      grain({ opacity: 0.06, size: 1.4 })

      // sombra del anillo delantero proyectada sobre el hielo (la luz viene de arriba)
      const shadowArc = range(30).map(j => ringPt(lerp(0.2, Math.PI - 0.2, j / 29), 0.97, 34))
      brush(shadowArc, { width: 34, color: alpha(P.hieloOsc, 0.32), blur: 10, blend: "multiply", taper: [0.2, 0.2] })

      // labio de la herida: oclusión azul y reflejo turquesa tenue de la grieta
      const lip = strata(0, 0)
      poly(lip, { fill: "none", stroke: alpha(P.hieloOsc, 0.5), width: 36, blur: 14 })
      poly(lip, { fill: "none", stroke: alpha(P.turq, 0.1), width: 80, blur: 34, blend: "screen" })

      // ---- pared de roca del impacto: estratos de la superficie al fondo ----
      const N = 7
      for (let k = 0; k < N; k++) {
        const t = k / N
        poly(strata(t, k === 0 ? 7 : 3 + k * 0.8), { fill: mix(mix(P.roca, P.hieloSom, 0.15), P.rocaNeg, Math.pow(t, 0.8)) })
      }
      // vetas finas entre estratos
      for (let k = 1; k < 12; k++) {
        poly(strata(k / 12 + rand(-0.03, 0.03), 3), { fill: "none", stroke: alpha(k % 2 ? shade(P.roca, 0.35) : P.rocaNeg, 0.18), width: rand(1, 2.5) })
      }
      // luz en la parte alta de la pared, sombra en la baja
      ellipse(OUT_C[0] - 150, OUT_C[1] - 190, 140, 90, { fill: alpha(shade(P.roca, 0.35), 0.28), blur: 40 })
      ellipse(OUT_C[0] - 120, OUT_C[1] + 220, 170, 110, { fill: alpha(P.rocaNeg, 0.55), blur: 40, blend: "multiply" })

      // grieta turquesa en la pared
      const g = (deg, t) => {
        const phi = BACK + deg * Math.PI / 180, rho = RO + wallW(phi) * (1 - t)
        return [OUT_C[0] + Math.cos(phi) * rho, OUT_C[1] + Math.sin(phi) * rho]
      }
      const crack = [g(-40, 0.25), g(-28, 0.45), g(-18, 0.38), g(-6, 0.6), g(5, 0.52), g(16, 0.72), g(30, 0.66), g(42, 0.85)]
      ellipse(...g(0, 0.5), 110, 90, { fill: alpha(P.turq, 0.14), blur: 40, blend: "screen" })
      brush(crack, { width: 26, color: alpha(P.turq, 0.16), blur: 18 })
      brush(crack, { width: [2, 6, 4, 9, 6, 7, 4, 1.5], color: alpha(P.turq, 0.7), taper: [0.2, 0.3], glow: { blur: 12, color: alpha(P.turq, 0.5) } })
      brush(crack, { width: [0.5, 2.5, 1.5, 3.5, 2, 2.5, 1.5, 0.5], color: alpha(P.turqClaro, 0.6), taper: [0.3, 0.4] })
      brush([g(-6, 0.6), g(-12, 0.8), g(-10, 0.97)], { width: [4, 1.5], color: alpha(P.turq, 0.5), taper: [0, 1] })
      brush([g(16, 0.72), g(22, 0.45), g(26, 0.25)], { width: [3.5, 1.2], color: alpha(P.turq, 0.45), taper: [0, 1] })
      brush([g(-28, 0.45), g(-36, 0.7), g(-34, 0.9)], { width: [3, 1], color: alpha(P.turq, 0.4), taper: [0, 1] })
      // segunda grieta más fina y profunda
      const crack2 = [g(-58, 0.55), g(-50, 0.75), g(-44, 0.68), g(-36, 0.88)]
      brush(crack2, { width: [1, 3, 2, 1], color: alpha(P.turq, 0.45), glow: { blur: 8, color: alpha(P.turq, 0.35) } })

      // textura de la roca: cascotes y motas dentro de la pared
      clip(poly(strata(0, 0), { hidden: true }), () => {
        for (let i = 0; i < 70; i++) {
          const p = g(rand(-70, 70), rand(0.05, 1))
          const s = rand(2, 7)
          const dark = rand() < 0.6
          blob(jitter(ring(p[0], p[1], s, s * 0.7, 5, rand(0, 90)), s * 0.3).map(q => [q[0], q[1], 0]), { fill: alpha(dark ? P.rocaNeg : shade(P.roca, 0.25), dark ? 0.45 : 0.3) })
        }
      })
    })
  })

  element("anillo_delantero", { pivot: C }, () => {
    // banda de polvo delantera
    dust(0, Math.PI, 0.08)
    // esquirlas delanteras
    for (const d of debris.filter(d => d.front)) chunk(d.x, d.y, d.size, d.rock, 0.15)
    // chorro de fragmentos que sale del hueco; los cercanos con un toque turquesa de la grieta
    for (const b of burst) {
      chunk(b.x, b.y, b.size, b.rock, 0.25)
      if (b.u < 0.35) brush([[b.x - b.size * 0.6, b.y + b.size * 0.3], [b.x - b.size * 0.2, b.y + b.size * 0.6]], { width: b.size * 0.25, color: alpha(P.turq, 0.35), taper: [0.4, 0.4] })
    }
  })
})
