// dibujo.js: hoja 2x2 de módulos de morro (vista cenital, apuntan hacia arriba)
canvas({ width: 1024, height: 1024, background: null })

const P = {
  tinta: "#1b1d33",     // contorno oscuro
  blanco: "#ffffff",    // borde de pegatina y brillos
  metal: "#eef1f8",     // metal blanco
  metalSh: "#a4afca",   // sombra del metal
  garra: "#3d4362",     // abrazadera
  garraSh: "#242840",
  garraHi: "#7480aa",
  cian: "#c6ecff", cianSh: "#4f9fe0",
  verde: "#62f26e", verdeSh: "#17a043",
  violeta: "#b05cff", violetaSh: "#5a1fb5",
  oro: "#ffcf2e", oroSh: "#d4801a",
  ambar: "#ff9a2e", ambarSh: "#c4471a",
}
const S = 0.8 // escala de cada módulo dentro de su cuadrante

// barra recta de (x1,y1) a (x2,y2) con grosor w
function bar(x1, y1, x2, y2, w) {
  const dx = x2 - x1, dy = y2 - y1, l = Math.hypot(dx, dy)
  const nx = -dy / l * w / 2, ny = dx / l * w / 2
  return [[x1 + nx, y1 + ny], [x2 + nx, y2 + ny], [x2 - nx, y2 - ny], [x1 - nx, y1 - ny]]
}
const orb = (x, y, r) => ring(x, y, r, r, 14)

// piezas comunes
const BODY = [[-72, -30, 0.5], [-42, -54, 0.5], [42, -54, 0.5], [72, -30, 0.5], [76, 46, 0.5], [52, 70, 0.5], [-52, 70, 0.5], [-76, 46, 0.5]]
const CLAMP = [[-88, 46], [88, 46], [88, 136], [74, 152], [46, 152], [46, 104], [-46, 104], [-46, 152], [-74, 152], [-88, 136]]

// dibuja un módulo: parts = [{ f: blob|poly, p, base, sh, d }], extras(tp) = detalles
function modulo(nombre, cx, cy, off, parts, extras) {
  const tp = (x, y) => [cx + x * S, cy + (y + off) * S]
  const tps = (pts) => pts.map(([x, y, k]) => k === undefined ? tp(x, y) : [...tp(x, y), k])
  element(nombre, { pivot: tp(0, 152) }, () => {
    const all = [{ f: poly, p: CLAMP, base: P.garra, sh: P.garraSh, d: 10 }, ...parts]
    // borde blanco de pegatina y contorno oscuro
    for (const q of all) q.f(tps(q.p), { fill: P.blanco, stroke: P.blanco, width: 50 * S, join: "round" })
    for (const q of all) q.f(tps(q.p), { fill: P.tinta, stroke: P.tinta, width: 20 * S, join: "round" })
    // rellenos con sombra de un tono (abajo a la derecha)
    for (const q of all) {
      const pts = tps(q.p)
      const s = q.f(pts, { fill: q.sh, stroke: P.tinta, width: 5 * S, join: "round" })
      const d = (q.d ?? 14) * S
      clip(s, () => q.f(shift(pts, -d, -d), { fill: q.base }))
    }
    extras(tp, tps)
  })
}

// detalles comunes: tornillos de la abrazadera y brillo del cuerpo
function comunes(tp, tps) {
  brush(tps([[-76, 86], [-76, 136]]), { width: 7 * S, color: P.garraHi, taper: [0.2, 0.4] })
  for (const x of [-62, 62]) {
    const [bx, by] = tp(x, 88)
    circle(bx, by, 9 * S, { fill: P.garraHi, stroke: P.tinta, width: 4 * S })
  }
  brush(tps([[-60, 30], [-62, -20], [-36, -42]]), { width: [6, 13, 6].map(v => v * S), color: P.blanco })
}

layer("modulos", () => {
  // ── LANZA (arriba izquierda) ──
  modulo("lanza", 256, 256, 64, [
    { f: poly, p: [[-17, -50], [17, -50], [17, -210], [-17, -210]], base: P.metal, sh: P.metalSh, d: 7 },
    { f: poly, p: [[-27, -222], [27, -222], [27, -198], [-27, -198]], base: P.garra, sh: P.garraSh, d: 6 },
    { f: blob, p: BODY, base: P.metal, sh: P.metalSh },
    { f: poly, p: [[0, -290], [22, -248], [15, -222], [-15, -222], [-22, -248]], base: P.cian, sh: P.cianSh, d: 9 },
  ], (tp, tps) => {
    comunes(tp, tps)
    brush(tps([[-9, -60], [-9, -195]]), { width: 6 * S, color: P.blanco, taper: [0.2, 0.3] })
    brush(tps([[-6, -275], [-12, -248], [-6, -230]]), { width: [4, 9, 4].map(v => v * S), color: P.blanco })
  })

  // ── HORQUILLA (arriba derecha) ──
  modulo("horquilla", 768, 256, 26, [
    { f: poly, p: bar(-30, -30, -112, -172, 38), base: P.metal, sh: P.metalSh, d: 8 },
    { f: poly, p: bar(30, -30, 112, -172, 38), base: P.metal, sh: P.metalSh, d: 8 },
    { f: blob, p: BODY, base: P.metal, sh: P.metalSh },
    { f: blob, p: orb(-115, -178, 27), base: P.verde, sh: P.verdeSh, d: 9 },
    { f: blob, p: orb(115, -178, 27), base: P.verde, sh: P.verdeSh, d: 9 },
  ], (tp, tps) => {
    comunes(tp, tps)
    brush(tps([[-36, -50], [-96, -152]]), { width: 6 * S, color: P.blanco, taper: [0.2, 0.3] })
    brush(tps([[26, -58], [88, -162]]), { width: 5 * S, color: P.blanco, taper: [0.2, 0.3], opacity: 0.8 })
    for (const x of [-115, 115]) {
      const [ox, oy] = tp(x - 9, -188)
      ellipse(ox, oy, 9 * S, 6 * S, { fill: P.blanco, rotate: -35 })
    }
  })

  // ── RASTREADOR (abajo izquierda) ──
  modulo("rastreador", 256, 768, 52, [
    { f: poly, p: bar(-46, -165, -82, -232, 11), base: P.metal, sh: P.metalSh, d: 3 },
    { f: poly, p: bar(46, -165, 82, -232, 11), base: P.metal, sh: P.metalSh, d: 3 },
    { f: blob, p: orb(-84, -238, 15), base: P.violeta, sh: P.violetaSh, d: 6 },
    { f: blob, p: orb(84, -238, 15), base: P.violeta, sh: P.violetaSh, d: 6 },
    { f: blob, p: BODY, base: P.metal, sh: P.metalSh },
    { f: blob, p: orb(0, -100, 102), base: P.metal, sh: P.metalSh, d: 16 },
  ], (tp, tps) => {
    comunes(tp, tps)
    // cuenco de la antena (cóncavo: sombra arriba a la izquierda)
    const [dx, dy] = tp(0, -100)
    const cuenco = circle(dx, dy, 72 * S, { fill: P.metalSh, stroke: P.tinta, width: 6 * S })
    clip(cuenco, () => circle(dx + 12 * S, dy + 12 * S, 70 * S, { fill: P.metal }))
    // ojo
    const ojo = circle(dx, dy, 34 * S, { fill: P.violetaSh, stroke: P.tinta, width: 8 * S })
    clip(ojo, () => circle(dx - 6 * S, dy - 6 * S, 30 * S, { fill: P.violeta }))
    circle(dx - 11 * S, dy - 12 * S, 10 * S, { fill: P.blanco })
    brush(tps([[-88, -70], [-80, -150], [-30, -190]]), { width: [5, 12, 5].map(v => v * S), color: P.blanco })
    for (const x of [-84, 84]) {
      const [ox, oy] = tp(x - 5, -243)
      circle(ox, oy, 5 * S, { fill: P.blanco })
    }
  })

  // ── REBOTE (abajo derecha) ──
  modulo("rebote", 768, 768, 30, [
    { f: poly, p: bar(-50, -10, -128, -112, 34), base: P.metal, sh: P.metalSh, d: 8 },
    { f: poly, p: bar(50, -10, 128, -112, 34), base: P.metal, sh: P.metalSh, d: 8 },
    { f: blob, p: orb(-130, -116, 22), base: P.ambar, sh: P.ambarSh, d: 8 },
    { f: blob, p: orb(130, -116, 22), base: P.ambar, sh: P.ambarSh, d: 8 },
    { f: blob, p: BODY, base: P.metal, sh: P.metalSh },
    { f: poly, p: [[0, -215], [-92, -40], [92, -40]], base: P.oro, sh: P.oro, d: 0 },
  ], (tp, tps) => {
    comunes(tp, tps)
    // cara derecha del prisma en sombra, arista central
    poly(tps([[0, -215], [92, -40], [0, -40]]), { fill: P.oroSh, stroke: P.tinta, width: 5 * S, join: "round" })
    brush(tps([[0, -205], [0, -45]]), { width: [3, 6, 4].map(v => v * S), color: P.tinta, taper: [0.1, 0.1] })
    brush(tps([[-12, -180], [-66, -60]]), { width: [4, 10, 4].map(v => v * S), color: P.blanco })
    brush(tps([[-82, -54], [-114, -98]]), { width: 6 * S, color: P.blanco, taper: [0.2, 0.3] })
    for (const x of [-130, 130]) {
      const [ox, oy] = tp(x - 7, -124)
      ellipse(ox, oy, 8 * S, 5 * S, { fill: P.blanco, rotate: -35 })
    }
  })
})
