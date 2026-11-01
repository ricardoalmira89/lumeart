// Pared de la Colmena: textura enlazable por los cuatro lados (vista cenital).
// Todo lo que toca un borde se dibuja también desplazado ±1024 (función W).
const S = 1024
const P = {
  base: "#1d2113",     // fondo uniforme en todos los bordes
  oscuro: "#11140b",   // grietas y fondos de celda
  hondo: "#0b0d07",
  oliva: "#2a3018",    // placas de resina
  marron: "#2c2516",   // quitina marrón verdosa
  medio: "#363b1f",    // relieves
  luz: "#4a4a28",      // luz cálida (arriba-izquierda)
  frio: "#2f4a48",     // contraluz frío (abajo-derecha)
  lima: "#9be03a",     // brillo tenue en el fondo de algunas celdas
}
canvas({ width: S, height: S, background: P.base })
seed(7)
const huecos = []   // centros de los alvéolos, para que los bultos los eviten

// Dibuja fn(dx, dy) en cada copia desplazada que toque el lienzo
function W(x0, y0, x1, y1, fn) {
  for (const dx of [-S, 0, S]) for (const dy of [-S, 0, S]) {
    if (x1 + dx < 0 || x0 + dx > S || y1 + dy < 0 || y0 + dy > S) continue
    fn(dx, dy)
  }
}
function Wpts(pts, m, fn) {
  const xs = pts.map(p => p[0]), ys = pts.map(p => p[1])
  W(Math.min(...xs) - m, Math.min(...ys) - m, Math.max(...xs) + m, Math.max(...ys) + m,
    (dx, dy) => fn(pts.map(p => [p[0] + dx, p[1] + dy, ...p.slice(2)])))
}

layer("masa", () => {
  // manchas grandes de resina y quitina
  for (let i = 0; i < 46; i++) {
    const x = rand(0, S), y = rand(0, S), r = rand(70, 190)
    const c = pick([P.oliva, P.marron, P.oscuro, P.marron, P.medio])
    const pts = jitter(ring(x, y, r, r * rand(0.6, 1), 8, rand(0, 180)), r * 0.15)
    const op = rand(0.35, 0.7)
    Wpts(pts, 60, q => blob(q, { fill: c, opacity: op, blur: 38 }))
  }
})

layer("placas", () => {
  // placas de resina endurecida con relieve suave
  for (let i = 0; i < 36; i++) {
    const x = ((i % 6) + rand(0.1, 0.9)) * S / 6, y = (Math.floor(i / 6) + rand(0.1, 0.9)) * S / 6
    const r = rand(70, 140)
    const c = pick([P.oliva, P.marron, mix(P.oliva, P.marron, 0.5)])
    const pts = jitter(ring(x, y, r, r * rand(0.5, 0.9), 11, rand(0, 180)), r * 0.22)
    Wpts(pts, 40, q => {
      blob(shift(q, 7, 10), { fill: P.hondo, opacity: 0.4, blur: 16 })
      const b = blob(q, { fill: mix(c, P.luz, 0.22), opacity: 0.5, blur: 2 })
      clip(b, () => {
        blob(shift(q, 9, 12), { fill: c, blur: 10 })
        blob(shift(grow(q, 0.6), 18, 22), { fill: shade(c, -0.35), opacity: 0.5, blur: 26 })
        // contraluz fría en el borde inferior derecho
        const f = blob(q, { fill: P.frio, opacity: 0.13, blur: 3 })
        cut(f, [blob(shift(q, -6, -7), { hidden: true })])
      })
    })
  }
})

layer("vetas", () => {
  // vetas finas de resina solidificada (textura de fondo)
  for (let i = 0; i < 90; i++) {
    const x = rand(0, S), y = rand(0, S), a = rand(0, 360), L = rand(30, 90)
    const m = polar(x, y, L / 2, a + 90)
    const pts = [polar(x, y, L / 2, a + 180), [m[0] * 0.15 + x * 0.85, m[1] * 0.15 + y * 0.85], polar(x, y, L / 2, a)]
    const c = pick(["#3e3a20", P.hondo, "#3b2c1a"]), w = rand(1, 2.5), op = rand(0.25, 0.5)
    Wpts(pts, 10, q => brush(q, { width: w, color: c, opacity: op }))
  }
})

layer("venas", () => {
  // venas orgánicas finas: paseos aleatorios con ramas
  function vena(x, y, ang, n, w) {
    const pts = [[x, y]]
    for (let k = 0; k < n; k++) {
      ang += rand(-22, 22)
      x += Math.cos(ang * Math.PI / 180) * rand(30, 50)
      y += Math.sin(ang * Math.PI / 180) * rand(30, 50)
      pts.push([x, y])
    }
    const wd = [w * 0.6, w, w * 0.8, w * 0.4]
    Wpts(pts, 20, q => {
      brush(shift(q, 2, 3), { width: wd.map(v => v * 2), color: P.hondo, taper: [0.2, 0.5], opacity: 0.6 })
      brush(q, { width: wd, color: "#33291a", taper: [0.2, 0.5] })
      brush(shift(q, -0.8, -1), { width: wd.map(v => v * 0.3), color: "#4f4428", taper: [0.3, 0.6] })
    })
    return pts
  }
  for (let i = 0; i < 9; i++) {
    const main = vena(rand(0, S), rand(0, S), rand(0, 360), Math.floor(rand(8, 14)), rand(4, 6.5))
    for (let j = 0; j < 2; j++) {
      const k = Math.floor(rand(1, main.length - 2)), p = main[k], n = main[k + 1]
      const a = Math.atan2(n[1] - p[1], n[0] - p[0]) * 180 / Math.PI + pick([-1, 1]) * rand(30, 55)
      vena(p[0], p[1], a, Math.floor(rand(3, 5)), rand(2, 3))
    }
  }
})

layer("celdas", () => {
  // celdas hexagonales hundidas en racimos irregulares
  const hex = (x, y, r, rot) => range(6).map(k => polar(x, y, r, rot + k * 60))
  const centros = []
  for (let i = 0; i < 16; i++) centros.push([((i % 4) + rand(0.15, 0.85)) * S / 4, (Math.floor(i / 4) + rand(0.15, 0.85)) * S / 4])
  for (const [cx, cy] of centros) {
    if (rand() < 0.2) continue
    const R = rand(17, 28), rot = rand(0, 60), lado = R * Math.sqrt(3)
    const ext = rand(1.2, 2.8)
    const celdas = []
    for (let a = -3; a <= 3; a++) for (let b = -3; b <= 3; b++) {
      const ox = lado * (a + b / 2), oy = lado * b * Math.sqrt(3) / 2
      const d = Math.hypot(ox, oy) / lado
      if (d > ext || (d > 0.6 && rand() < 0.35)) continue
      const t = rot * Math.PI / 180
      const x = cx + ox * Math.cos(t) - oy * Math.sin(t), y = cy + ox * Math.sin(t) + oy * Math.cos(t)
      celdas.push({ x, y, r: R * rand(0.72, 0.85), glow: rand() < 0.3, g: rand(0.12, 0.3) })
      huecos.push([x, y, R])
    }
    const m = lado * (ext + 1.5)
    W(cx - m, cy - m, cx + m, cy + m, (dx, dy) => {
      // masa de cera endurecida que sostiene el racimo
      for (const c of celdas) circle(c.x + dx + 4, c.y + dy + 6, R * 1.15, { fill: P.hondo, opacity: 0.35, blur: 10 })
      for (const c of celdas) circle(c.x + dx, c.y + dy, R * 1.5, { fill: mix(P.oliva, P.marron, 0.4), opacity: 0.5, blur: 14 })
      for (const c of celdas) blob(hex(c.x + dx, c.y + dy, R * 1.08, rot + 30), { fill: mix(P.oliva, P.luz, 0.2), blur: 1.5 })
      for (const c of celdas) {
        const x = c.x + dx, y = c.y + dy, h = hex(x, y, c.r, rot + 30)
        // borde iluminado del alvéolo
        poly(hex(x - 1.5, y - 2, c.r * 1.12, rot + 30), { fill: mix(P.medio, P.luz, 0.4), opacity: 0.55, blur: 2 })
        const p = poly(h, { fill: "#2c2a17" })
        clip(p, () => {
          poly(shift(grow(h, 0.95), -4, -5), { fill: P.oscuro, blur: 3 })
          circle(x + 1, y + 1, c.r * 0.55, { fill: P.hondo, blur: 6 })
          if (c.glow) circle(x + 2, y + 2, c.r * 0.6, { fill: radial(x + 2, y + 2, c.r * 0.6, [P.lima, alpha(P.lima, 0)]), opacity: c.g, blend: "screen" })
        })
      }
    })
  }
})

layer("protuberancias", () => {
  // pequeños bultos con luz cálida y contraluz frío
  for (let i = 0; i < 70; i++) {
    const x = rand(0, S), y = rand(0, S), r = rand(4, 11)
    const pts = jitter(ring(x, y, r, r * rand(0.7, 1), 7, rand(0, 90)), r * 0.15)
    // distancia con vuelta por los bordes: los bultos no caen sobre los alvéolos
    const wd = (a, b) => { const d = Math.abs(a - b) % S; return Math.min(d, S - d) }
    if (huecos.some(([hx, hy, R]) => Math.hypot(wd(x, hx), wd(y, hy)) < R * 1.3 + r)) continue
    W(x - 20, y - 20, x + 20, y + 20, (dx, dy) => {
      const q = shift(pts, dx, dy), X = x + dx, Y = y + dy
      circle(X + r * 0.4, Y + r * 0.55, r * 1.25, { fill: P.hondo, opacity: 0.5, blur: r * 0.5 })
      const b = blob(q, { fill: radial(X - r * 0.35, Y - r * 0.4, r * 1.4, ["#4c4628", "#2b2c17", "#1a1c0f"]) })
      clip(b, () => {
        const f = blob(q, { fill: P.frio, opacity: 0.35 })
        cut(f, [blob(shift(q, -r * 0.25, -r * 0.3), { hidden: true })])
        circle(X - r * 0.35, Y - r * 0.4, r * 0.22, { fill: "#6a6036", opacity: 0.6, blur: 1 })
      })
    })
  }
})
