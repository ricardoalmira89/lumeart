// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  void: "#0b1012",    // oscuro máximo (grietas, oclusión)
  deep: "#131c15",    // sombra de quitina
  chitin: "#1f2d1c",  // quitina base
  olive: "#384323",   // oliva medio
  oliveL: "#57632f",  // oliva iluminado
  rock: "#3b3631",    // roca del planeta
  rockD: "#211d1b",   // roca en sombra
  lime: "#b6ff3c",    // resplandor principal
  limeL: "#f0ffb0",   // núcleo de luz
  green: "#4cc82a",   // celdas brillantes
  amber: "#e2a032",   // celdas ámbar
  rim: "#6e9cc6",     // contraluz frío
}

const C = [512, 470], RX = 350, RY = 380   // bulbo
const O = [512, 545]                        // abertura (punto focal)

// proyección esférica del bulbo: a = longitud, b = latitud (radianes)
const sph = (a, b) => [C[0] + RX * Math.sin(a) * Math.cos(b), C[1] + RY * Math.sin(b)]
const depth = (a, b) => Math.cos(a) * Math.cos(b)

const bulbPts = [
  [512, 80], [680, 112], [800, 215], [864, 375], [858, 545], [796, 690], [692, 792],
  [512, 852], [332, 792], [228, 690], [166, 545], [160, 375], [224, 215], [344, 112],
]

// Raíces: detrás de todo, hacia los lados, perdiéndose en la oscuridad
layer("raices", () => {
  seed(7)
  const root = (x0, y0, dir, len, w, lvl) => {
    const pts = [[x0, y0]]
    let x = x0, y = y0, ang = rand(-0.25, 0.35)
    const n = 6
    for (let i = 1; i <= n; i++) {
      ang += rand(-0.35, 0.35)
      x += dir * Math.cos(ang) * len / n
      y += Math.sin(ang) * len / n + rand(-6, 10)
      pts.push([x, y])
    }
    const fade = linear(x0, 0, x0 + dir * len, 0, [mix(P.chitin, P.deep, lvl * 0.4), alpha(P.deep, 0)])
    brush(pts, { width: (u) => w * (1 - u * 0.85), taper: [0, 0.5], color: fade })
    // luz de la abertura en el lomo de la raíz, cerca del bulbo
    if (lvl === 0) brush(shift(pts.slice(0, 4), 0, -w * 0.25), { width: (u) => w * 0.25 * (1 - u), taper: [0.1, 0.8], color: linear(x0, 0, x0 + dir * len * 0.5, 0, [alpha(P.oliveL, 0.6), alpha(P.oliveL, 0)]) })
    if (lvl < 2) {
      for (let k = 1; k < pts.length - 2; k += 2) {
        if (rand() < 0.7) root(pts[k][0], pts[k][1], dir, len * rand(0.35, 0.55), w * 0.45, lvl + 1)
      }
    }
  }
  for (const dir of [-1, 1]) {
    const ys = [640, 700, 760, 820, 870]
    ys.forEach((y, i) => {
      const x = 512 + dir * (lerp(300, 200, i / 4))
      root(x, y, dir, rand(380, 480), rand(34, 50), 0)
    })
  }
  // raíces colgantes hacia abajo
  for (let i = 0; i < 7; i++) {
    const x = 330 + i * 60 + rand(-15, 15)
    const pts = [[x, 840]]
    let px = x, py = 840
    for (let k = 0; k < 4; k++) { px += rand(-25, 25); py += rand(30, 45); pts.push([px, py]) }
    brush(pts, { width: (u) => 22 * (1 - u * 0.8), taper: [0, 0.6], color: linear(0, 840, 0, py, [P.deep, alpha(P.deep, 0)]) })
  }
})

// Restos del planeta: corteza rota bajo el bulbo
layer("planeta", () => {
  seed(11)
  // casquete roto del planeta: borde superior dentado, panza redonda
  const crust = roughen([[150, 770, 0], [260, 735, 0], [380, 760, 0], [512, 728, 0], [640, 758, 0], [770, 732, 0], [880, 772, 0], [820, 880], [680, 950], [512, 972], [344, 950], [204, 880]], 16, { detail: 3, sharp: 0.7 })
  const cr = blob(crust, { fill: P.rockD })
  freeform(cr, [
    [512, 760, P.rock, 0.2],
    [300, 770, mix(P.rock, P.rockD, 0.4), 0.1],
    [740, 770, mix(P.rock, P.rockD, 0.4), 0.1],
    [512, 950, P.void, 0.2],
    [220, 860, P.void, 0.1],
    [810, 860, P.void, 0.1],
  ])
  clip(cr, () => {
    // estratos curvos de la corteza
    for (let i = 0; i < 6; i++) {
      const y = 790 + i * 26
      const k = 1 - i * 0.12
      brush([[512 - 360 * k, y - 30], [512 - 200 * k, y + rand(-6, 10)], [512, y + 20 + rand(-6, 6)], [512 + 200 * k, y + rand(-6, 10)], [512 + 360 * k, y - 30]], { width: rand(2, 5), color: alpha(P.void, 0.55) })
    }
    // grietas con brillo verde (la colmena se alimenta del núcleo)
    for (let i = 0; i < 5; i++) {
      const x = rand(300, 720)
      const pts = [[x, 760]]
      for (let k = 0; k < 4; k++) pts.push([pts[k][0] + rand(-25, 25), pts[k][1] + rand(25, 40)])
      brush(pts, { width: [3, 2, 1], color: alpha(P.green, 0.55), glow: { blur: 6, color: alpha(P.green, 0.5) } })
    }
    ellipse(512, 900, 420, 120, { fill: P.void, opacity: 0.6, blur: 40 })
    ellipse(512, 975, 300, 30, { fill: alpha(P.rim, 0.12), blur: 16 })
  })
  // fragmentos sueltos flotando
  for (let i = 0; i < 9; i++) {
    const side = i % 2 ? 1 : -1
    const x = 512 + side * rand(330, 470), y = rand(700, 900)
    const r = rand(10, 30)
    const f = blob(roughen(ring(x, y, r, r * 0.7, 7, rand(0, 60)), r * 0.3, { sharp: 0.8 }), { fill: mix(P.rockD, P.rock, 0.4), opacity: 0.85 })
    clip(f, () => ellipse(x + side * r * 0.4, y - r * 0.4, r * 0.6, r * 0.4, { fill: alpha(P.rim, 0.18), blur: 4 }))
  }
})

// Bulbo: masa principal con volumen
layer("bulbo", () => {
  seed(31)
  // crestas y cuernos de quitina que rompen la silueta (detrás del bulbo)
  const horns = [[-150, 1.0], [-125, 1.25], [-100, 0.9], [-62, 1.1], [-28, 0.85], [0, 1.2], [28, 0.85], [62, 1.1], [100, 0.9], [125, 1.25], [150, 1.0]]
  for (const [deg, k] of horns) {
    const t = (deg - 90) * Math.PI / 180
    const bx = C[0] + Math.cos(t) * RX * 0.9, by = C[1] + Math.sin(t) * RY * 0.9
    const len = 150 * k + rand(-10, 10)
    const curl = deg < 0 ? -0.4 : deg > 0 ? 0.4 : 0
    const pts = range(5).map((i) => {
      const u = i / 4
      const a = t + curl * u * u
      return [bx + Math.cos(a) * len * u, by + Math.sin(a) * len * u]
    })
    brush(pts, { width: (u) => 70 * k * (1 - u * 0.9), taper: [0, 0.75], color: mix(P.deep, P.void, 0.3) })
    // contraluz frío en el borde exterior del cuerno
    const o = deg < 0 ? -6 : 6
    brush(shift(pts.slice(1), Math.cos(t + 1.57) * o, Math.sin(t + 1.57) * o), { width: (u) => 5 * (1 - u * 0.8), taper: [0.2, 0.8], color: alpha(P.rim, 0.3) })
  }
  // contrafuertes: raíces gruesas que bajan del bulbo y se clavan en la corteza
  for (const dir of [-1, 1]) {
    for (const [y0, x1, y1, w] of [[520, 330, 800, 60], [640, 250, 770, 50], [700, 150, 800, 34]]) {
      const x0 = 512 + dir * 320
      const pts = [[x0, y0], [512 + dir * (x1 * 0.4 + 320 * 0.6) + dir * 30, lerp(y0, y1, 0.5)], [512 + dir * x1, y1]]
      brush(pts, { width: (u) => w * (1 - u * 0.5), taper: [0, 0.35], color: mix(P.deep, P.void, 0.2) })
      brush(shift(pts, dir * w * 0.3, -4), { width: (u) => w * 0.12 * (1 - u * 0.6), taper: [0.2, 0.5], color: alpha(P.rim, 0.22) })
    }
  }

  const b = blob(bulbPts, { fill: P.chitin })
  freeform(b, [
    [512, 560, P.oliveL, 0.3],
    [512, 300, P.olive, 0.2],
    [512, 120, P.chitin, 0.1],
    [200, 450, P.deep, 0.2],
    [830, 450, P.deep, 0.2],
    [512, 830, P.deep, 0.2],
    [300, 720, P.void, 0.1],
    [720, 720, P.void, 0.1],
  ])
})

// Panal hexagonal, nervaduras y rocas incrustadas
layer("panal", () => {
  seed(21)
  const bulb = blob(bulbPts, { hidden: true })
  const s = 0.15
  const lightAt = (x, y, z) => {
    const d = Math.hypot(x - O[0], (y - O[1]) * 1.15)
    return Math.pow(clamp(1 - d / 420, 0, 1), 1.6) * (0.4 + 0.6 * Math.sqrt(Math.max(z, 0)))
  }
  clip(bulb, () => {
    // ranuras entre celdas (oscuro de oclusión)
    blob(bulbPts, { fill: alpha(P.void, 0.55) })
    for (let j = -10; j <= 10; j++) {
      const b = j * s * 0.87
      for (let i = -12; i <= 12; i++) {
        const a = (i + (j & 1) * 0.5) * s
        const z = depth(a, b)
        if (z < 0.03 || Math.abs(a) > 1.6) continue
        const [x, y] = sph(a, b)
        if (Math.hypot(x - O[0], y - O[1]) < 120) continue
        const hex = range(6).map((k) => {
          const t = (k * 60 + 30) * Math.PI / 180
          return sph(a + Math.cos(t) * s * 0.53, b + Math.sin(t) * s * 0.53)
        })
        const L = lightAt(x, y, z)
        const n = rand(-0.06, 0.06)
        const wall = mix(mix(P.deep, P.chitin, 0.5 + n * 4), P.oliveL, clamp(L * 1.1 + n, 0, 1))
        poly(hex, { fill: wall })
        // cavidad de la celda: más oscura, desplazada hacia la luz (la pared del lado lejano recibe luz)
        const dx = (O[0] - x), dy = (O[1] - y), dl = Math.hypot(dx, dy) || 1
        const cav = shift(grow(hex, 0.72), dx / dl * 2.5, dy / dl * 2.5)
        const r = rand()
        if (r < 0.05) {
          const g = r < 0.03 ? P.green : P.amber
          poly(cav, { fill: shade(g, -0.15), glow: { blur: 10, color: alpha(g, 0.7) } })
          poly(grow(cav, 0.55), { fill: shade(g, 0.35), opacity: 0.9 })
        } else {
          poly(cav, { fill: mix(P.void, wall, 0.45 + L * 0.2) })
          // brillo húmedo en el borde de la pared que mira a la abertura
          if (L > 0.12) poly(shift(grow(hex, 0.78), -dx / dl * 2, -dy / dl * 2), { fill: "none", stroke: alpha(P.lime, L * 0.35), width: 1.2 })
        }
      }
    }
    // nervaduras orgánicas que irradian desde la abertura hacia el borde
    for (let i = 0; i < 11; i++) {
      const deg = i * 360 / 11 + rand(-8, 8) - 90
      const t = deg * Math.PI / 180
      const pts = range(7).map((k) => {
        const u = k / 6
        const w = Math.sin(u * 5 + i) * 0.12 * u
        const r0 = lerp(200, 1, u), ex = Math.cos(t + w), ey = Math.sin(t + w)
        return [lerp(O[0] + ex * 200, C[0] + ex * RX * 1.05, u), lerp(O[1] + ey * 175, C[1] + ey * RY * 1.05, u)]
      })
      brush(pts, { width: (u) => 26 * (1 - u * 0.6), taper: [0, 0.4], color: P.deep, shadow: { x: 0, y: 4, blur: 6, color: alpha(P.void, 0.7) } })
      brush(pts, { width: (u) => 8 * (1 - u * 0.7), taper: [0, 0.6], color: alpha(P.oliveL, 0.5) })
      // ramificación
      const k = 2 + (i % 3)
      const p = pts[k], q = pts[k + 1]
      const side = i % 2 ? 1 : -1
      brush([p, [lerp(p[0], q[0], 0.5) + side * 40, lerp(p[1], q[1], 0.5)], [q[0] + side * 90, q[1] + 40]], { width: [12, 7, 2], taper: [0, 0.6], color: P.deep })
    }
    // sombreado de volumen global: borde inferior y laterales en sombra
    ellipse(512, 860, 360, 140, { fill: P.void, opacity: 0.6, blur: 50, blend: "multiply" })
    ellipse(512, 470, 350, 380, { fill: "none", stroke: P.void, width: 120, opacity: 0.55, blur: 50 })
    // bañado verde desde la abertura
    circle(O[0], O[1], 300, { fill: radial(O[0], O[1], 300, [alpha(P.lime, 0.35), alpha(P.green, 0.12), alpha(P.green, 0)]), blend: "screen" })
  })
  // rocas del planeta incrustadas
  const rocks = [[330, 260, 46], [700, 300, 38], [250, 520, 34], [770, 560, 44], [400, 160, 26], [610, 720, 40], [380, 700, 30], [560, 190, 22]]
  for (const [x, y, r] of rocks) {
    const pts = roughen(ring(x, y, r, r * 0.75, 8, rand(0, 90)), r * 0.25, { sharp: 0.8, detail: 2 })
    blob(grow(pts, 1.18), { fill: P.void, opacity: 0.7, blur: 4 })
    const rk = blob(pts, { fill: mix(P.rockD, P.rock, 0.6) })
    clip(rk, () => {
      const L = lightAt(x, y, 1)
      const dx = O[0] - x, dy = O[1] - y, dl = Math.hypot(dx, dy)
      ellipse(x + dx / dl * r * 0.6, y + dy / dl * r * 0.6, r * 0.7, r * 0.5, { fill: alpha(P.lime, 0.15 + L * 0.5), blur: 8, blend: "screen" })
      ellipse(x - dx / dl * r * 0.5, y - dy / dl * r * 0.5, r * 0.8, r * 0.6, { fill: P.void, opacity: 0.5, blur: 8 })
      cracks([x - r, y - r, r * 2, r * 2], { count: 2, length: r })
    })
  }
})

// Abertura en flor con el resplandor
layer("abertura", () => {
  seed(41)
  const [ox, oy] = O
  const sq = 0.86 // la abertura mira un poco hacia abajo
  const pp = (r, deg) => { const t = (deg - 90) * Math.PI / 180; return [ox + Math.cos(t) * r, oy + Math.sin(t) * r * sq] }
  // halo sobre la quitina
  ellipse(ox, oy, 300, 270, { fill: radial(ox, oy, 300, [alpha(P.lime, 0.4), alpha(P.green, 0.15), alpha(P.green, 0)]), blend: "screen" })
  // labio exterior: anillo carnoso oscuro
  const lip = blob(ring(ox, oy, 205, 205 * sq, 12, 8).map((p, i) => [p[0] + rand(-6, 6), p[1] + rand(-6, 6)]), { fill: P.deep, blur: 5, shadow: { x: 0, y: 10, blur: 20, color: alpha(P.void, 0.8) } })
  clip(lip, () => {
    ellipse(ox, oy + 30, 200, 160, { fill: alpha(P.lime, 0.25), blur: 30, blend: "screen" })
    ellipse(ox, oy - 150, 200, 60, { fill: P.void, opacity: 0.5, blur: 20 })
  })
  // pozo de luz
  ellipse(ox, oy, 150, 150 * sq, { fill: radial(ox, oy, 150, [P.limeL, P.lime, alpha(P.green, 0.8), alpha(P.green, 0)]), glow: { blur: 40, color: alpha(P.lime, 0.8) } })
  ellipse(ox, oy, 60, 60 * sq, { fill: "#fbffe8", blur: 14 })
  const N = 9
  // rendijas de luz entre las placas
  for (let i = 0; i < N; i++) {
    const d = (i + 0.5) * 360 / N + 10
    brush([pp(70, d), pp(140, d + 1), pp(205, d)], { width: [10, 6, 1], taper: [0, 0.6], color: P.lime, glow: { blur: 12, color: alpha(P.lime, 0.7) } })
  }
  // placas en flor, entreabiertas
  for (let i = 0; i < N; i++) {
    const d = i * 360 / N + 10 + rand(-2, 2)
    const rin = rand(55, 72), span = 15
    const pts = [pp(rin, d).concat(0), pp(110, d - span), pp(186, d - span + 1).concat(0), pp(210, d).concat(0.5), pp(186, d + span - 1).concat(0), pp(110, d + span)]
    const pl = blob(pts, { fill: P.chitin, shadow: { x: 0, y: 0, blur: 8, color: alpha(P.void, 0.7) } })
    clip(pl, () => {
      // luz del pozo en la punta interior, sombra hacia fuera
      circle(ox, oy, 200, { fill: radial(ox, oy, 200, [[0, alpha(P.limeL, 0.95)], [0.4, alpha(P.lime, 0.6)], [0.7, alpha(P.olive, 0.2)], [1, alpha(P.void, 0.6)]]) })
      // nervio central de la placa
      brush([pp(rin + 10, d), pp(130, d + 1), pp(190, d)], { width: [2, 6, 3], color: alpha(P.void, 0.5) })
      brush([pp(rin + 12, d + 3), pp(130, d + 4)], { width: [1, 3, 1], color: alpha(P.limeL, 0.5) })
      // segmentos transversales
      for (const r of [125, 160]) brush(range(5).map((k) => pp(r, d - span + k * span / 2)), { width: [1, 3, 1], taper: [0.2, 0.2], color: alpha(P.void, 0.5) })
      // borde exterior en sombra, hacia donde la placa se hunde en el labio
      circle(ox, oy, 230, { fill: "none", stroke: P.void, width: 50, opacity: 0.45, blur: 10 })
    })
  }
  circle(ox, oy, 90, { fill: alpha(P.lime, 0.35), blur: 30, blend: "screen" })
})

// Retoques: contraluz, partículas, grano
layer("retoques", () => {
  seed(51)
  const bulb = blob(bulbPts, { hidden: true })
  clip(bulb, () => {
    // contraluz frío solo en los hombros superiores
    for (const dir of [-1, 1]) {
      brush(arc(C[0], C[1] + 10, 372, dir * 25, dir * 105, 8).map(([x, y]) => [x, C[1] + (y - C[1]) * 1.03]), { width: [4, 22, 4], color: alpha(P.rim, 0.28), blur: 10 })
    }
    blob(shift(grow(bulbPts, 0.97), 0, 18), { fill: "none", stroke: P.void, width: 40, blur: 20, opacity: 0.5 })
    grain({ opacity: 0.08, size: 1.2 })
  })
  // esporas brillantes que flotan desde la abertura
  for (let i = 0; i < 40; i++) {
    const a = rand(0, Math.PI * 2), d = rand(160, 470)
    const x = O[0] + Math.cos(a) * d, y = O[1] + Math.sin(a) * d * 0.9 - rand(0, 60)
    const r = rand(1, 3.2) * (1 - d / 600)
    circle(x, y, r, { fill: pick([P.lime, P.lime, P.amber]), opacity: rand(0.4, 0.9), glow: { blur: 6, color: alpha(P.lime, 0.6) } })
  }
})
