// dibujo.js: hoja de proyectiles (4x3 celdas de 256x256)
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

const P = {
  ink: "#1c0f2e",     // contorno oscuro
  white: "#ffffff",   // borde de pegatina y núcleos
  magenta: "#ff2fb4",
  lime: "#9ef01a",
  red: "#ff2a3d",
  yellow: "#ffd51f",
  violet: "#9b4dff",
  ivory: "#f4ead0",
  acid: "#4be03a",
  amber: "#ff9a1f",
  lilac: "#c79bff",
  purple: "#8a2be2",
}
const W = 26   // grosor del borde blanco
const K = 9    // grosor del contorno oscuro
const C = [[128, 128], [384, 128], [640, 128], [896, 128],
           [128, 384], [384, 384], [640, 384], [896, 384],
           [128, 640], [384, 640], [640, 640]]

// pegatina: copia blanca gruesa detrás + figura con contorno oscuro
function sticker(make, fill, extra = {}) {
  make({ fill: P.white, stroke: P.white, width: W + K, join: "round" })
  return make({ fill, stroke: P.ink, width: K, join: "round", behind: true, ...extra })
}
function shine(x, y, rx, ry, rot = -35) {
  ellipse(x, y, rx, ry, { fill: P.white, rotate: rot })
}

layer("fila 1", () => {
  // (1) PLASMA
  element("plasma", { pivot: C[0] }, () => {
    const [x, y] = C[0]
    circle(x, y, 84, { stroke: P.white, width: 40 })
    circle(x, y, 84, { stroke: P.ink, width: 22 })
    circle(x, y, 84, { stroke: "#ff8ee0", width: 9 })
    const orb = sticker(s => circle(x, y + 4, 58, s), radial(x - 14, y - 10, 70, ["#ff7fd6", P.magenta, "#b0127a"]))
    clip(orb, () => {
      circle(x + 20, y + 26, 52, { fill: "#a0106e", opacity: 0.55, blend: "multiply" })
      circle(x, y + 6, 32, { fill: "#ffc4ef", glow: { blur: 14, color: "#ffd0f4" } })
      circle(x, y + 6, 21, { fill: P.white })
    })
    shine(x - 26, y - 22, 10, 6)
    for (const a of [30, 150, 260]) {
      const [px, py] = polar(x, y, 84, a)
      circle(px, py, 6, { fill: P.white })
    }
  })

  // (2) ESPORA
  element("espora", { pivot: C[1] }, () => {
    const [x, y] = C[1]
    seed(4)
    const pts = range(14).map(i => {
      const a = i / 14 * Math.PI * 2
      const r = i % 2 ? 66 : 76
      return [x + Math.cos(a) * r, y + Math.sin(a) * r, i % 2 ? 1 : 0.5]
    })
    const sp = sticker(s => blob(pts, s), P.lime)
    clip(sp, () => {
      circle(x + 26, y + 30, 70, { fill: "#3f9a12", opacity: 0.6, blend: "multiply" })
      circle(x - 4, y - 2, 38, { fill: "#e9ffb0", glow: { blur: 14, color: "#f4ffd0" } })
      circle(x - 4, y - 2, 20, { fill: P.white })
      for (let i = 0; i < 16; i++) {
        const a = rand(0, 360), r = rand(42, 64)
        const [px, py] = polar(x, y, r, a)
        circle(px, py, rand(3.5, 6.5), { fill: pick(["#2f7d0c", "#4ea316", "#d8ff8a"]) })
      }
    })
    shine(x - 34, y - 34, 11, 6)
  })

  // (3) DARDO
  element("dardo", { pivot: C[2] }, () => {
    const [x, y] = C[2]
    const pts = [[x, y + 106, 0], [x + 28, y + 22], [x + 30, y - 40], [x + 60, y - 98, 0], [x + 16, y - 74],
                 [x, y - 86, 0], [x - 16, y - 74], [x - 60, y - 98, 0], [x - 30, y - 40], [x - 28, y + 22]]
    const d = sticker(s => blob(pts, s), P.red)
    clip(d, () => {
      rect(x, y - 120, 80, 240, { fill: "#a3001e", opacity: 0.55, blend: "multiply" })
      blob([[x, y + 80, 0], [x + 8, y + 10], [x + 7, y - 50], [x, y - 66, 0], [x - 7, y - 50], [x - 8, y + 10]],
        { fill: "#ffd0d4", glow: { blur: 10, color: "#ff9aa6" } })
      blob([[x, y + 56, 0], [x + 4, y], [x, y - 44, 0], [x - 4, y]], { fill: P.white })
    })
    shine(x - 14, y - 30, 4, 12, 0)
  })

  // (4) ORBE
  element("orbe", { pivot: C[3] }, () => {
    const [x, y] = C[3]
    const o = sticker(s => circle(x, y, 78, s), radial(x - 18, y - 18, 90, ["#fff27a", P.yellow, "#f08a00"]))
    clip(o, () => {
      circle(x + 22, y + 26, 66, { fill: "#d06000", opacity: 0.45, blend: "multiply" })
      circle(x, y + 4, 40, { fill: "#ff9a00", stroke: "#b85400", width: 4 })
      circle(x, y + 4, 28, { fill: "#fff6c8", glow: { blur: 14, color: "#ffffff" } })
      circle(x, y + 4, 18, { fill: P.white })
    })
    shine(x - 32, y - 32, 13, 7)
  })
})

layer("fila 2", () => {
  // (5) ESQUIRLA: cristal de cuatro puntas, la larga abajo
  element("esquirla", { pivot: C[4] }, () => {
    const [x, y] = C[4]
    const T = [x, y - 84], R = [x + 74, y - 6], B = [x, y + 104], L = [x - 74, y - 6], M = [x, y - 2]
    const a = [x + 26, y - 30], b = [x + 26, y + 22], c = [x - 26, y + 22], d = [x - 26, y - 30]
    const pts = [T, a, R, b, B, c, L, d].map(p => [p[0], p[1], 0])
    const s = sticker(st => blob(pts, st), P.violet)
    clip(s, () => {
      // facetas: claras arriba-izquierda, oscuras abajo-derecha
      poly([T, a, M], { fill: "#d2b0ff" }); poly([T, d, M], { fill: "#ead9ff" })
      poly([L, d, M], { fill: "#c39bff" }); poly([L, c, M], { fill: "#8c3cf0" })
      poly([R, a, M], { fill: "#7a2fe0" }); poly([R, b, M], { fill: "#5a1bb0" })
      poly([B, c, M], { fill: "#9b4dff" }); poly([B, b, M], { fill: "#6420c4" })
      blob([[x, y - 40, 0], [x + 14, y, 0], [x, y + 50, 0], [x - 14, y, 0]], { fill: P.white, glow: { blur: 12, color: "#e6d0ff" } })
    })
    blob(pts, { fill: "none", stroke: P.ink, width: K, join: "round" })
  })

  // (6) TAJO: media luna de hueso con el filo abajo
  element("tajo", { pivot: C[5] }, () => {
    const [x, y] = C[5]
    const n = 8, half = 92, top = y - 62
    const arcPts = (depth, u0 = 0, u1 = 1, m = n) => range(m + 1).map(i => {
      const u = u0 + (u1 - u0) * i / m
      return [x - half + 2 * half * u, top + depth * Math.pow(Math.sin(Math.PI * u), 0.8)]
    })
    const outer = arcPts(140).map((p, i) => [p[0], p[1], i === 0 || i === n ? 0 : 1])
    const inner = arcPts(62).reverse().slice(1, -1)
    const pts = [...outer, ...inner]
    const t = sticker(s => blob(pts, s), P.ivory)
    clip(t, () => {
      // sombra de un tono junto al filo
      blob([...arcPts(146), ...arcPts(112).reverse()], { fill: "#d2bd8e" })
      // filo cortante brillante
      curve(arcPts(126, 0.14, 0.86), { stroke: "#fffdf4", width: 8, cap: "round", glow: { blur: 10, color: "#ffffff" } })
      curve(arcPts(80, 0.22, 0.6, 5), { stroke: P.white, width: 6, cap: "round" })
    })
  })

  // (7) ÁCIDO: gota con la punta arriba y una burbuja
  element("acido", { pivot: C[6] }, () => {
    const [x, y] = C[6]
    const pts = [[x, y - 102, 0], [x + 30, y - 46], [x + 66, y + 10], [x + 62, y + 66], [x, y + 98],
                 [x - 62, y + 66], [x - 66, y + 10], [x - 30, y - 46]]
    const g = sticker(s => blob(pts, s), P.acid)
    clip(g, () => {
      circle(x + 40, y + 50, 80, { fill: "#1d8a2a", opacity: 0.55, blend: "multiply" })
      ellipse(x - 4, y + 30, 36, 42, { fill: "#d9ffa0", glow: { blur: 14, color: "#efffc8" } })
      ellipse(x - 6, y + 32, 20, 24, { fill: P.white })
    })
    circle(x + 32, y - 4, 13, { fill: alpha("#d9ffa0", 0.6), stroke: P.white, width: 4 })
    circle(x + 28, y - 8, 4, { fill: P.white })
    shine(x - 30, y - 12, 7, 15, 20)
  })

  // (8) ÁMBAR: gota de resina translúcida
  element("ambar", { pivot: C[7] }, () => {
    const [x, y] = C[7]
    const pts = [[x, y - 92, 0], [x + 24, y - 50], [x + 62, y - 4], [x + 70, y + 46], [x + 40, y + 88], [x, y + 98],
                 [x - 40, y + 88], [x - 70, y + 46], [x - 62, y - 4], [x - 24, y - 50]]
    const g = sticker(s => blob(pts, s), radial(x, y + 30, 90, ["#ffd36a", P.amber, "#d9580a"]))
    clip(g, () => {
      blob(shift(grow(pts, 0.9), 14, 12), { fill: "#b8400a", opacity: 0.4, blend: "multiply" })
      blob(shift(grow(pts, 0.8), -6, -2), { fill: "#ffb84a", opacity: 0.55 })
      ellipse(x, y + 36, 30, 34, { fill: "#fff0b8", glow: { blur: 16, color: "#ffe9a0" } })
      ellipse(x, y + 38, 16, 18, { fill: P.white })
      circle(x + 30, y - 6, 5, { fill: "#8a3a06", opacity: 0.7 })
      circle(x - 22, y + 66, 3.5, { fill: "#8a3a06", opacity: 0.6 })
    })
    shine(x - 34, y + 4, 8, 20, 18)
    circle(x - 18, y - 36, 5, { fill: P.white })
  })
})

layer("fila 3", () => {
  // (9) SEDA: bolita lila envuelta en hebras
  element("seda", { pivot: C[8] }, () => {
    const [x, y] = C[8]
    seed(9)
    // hebras sueltas detrás (estela hacia arriba)
    for (const dx of [-30, 4, 34]) {
      const pts = [[x + dx * 0.5, y - 40], [x + dx, y - 80], [x + dx * 0.6 + 10, y - 104]]
      curve(pts, { stroke: P.white, width: 20, cap: "round" })
      curve(pts, { stroke: P.ink, width: 12, cap: "round" })
      curve(pts, { stroke: "#e2ccff", width: 4, cap: "round" })
    }
    const b = sticker(s => circle(x, y + 10, 72, s), P.lilac)
    clip(b, () => {
      circle(x + 26, y + 36, 74, { fill: "#7a4fc4", opacity: 0.5, blend: "multiply" })
      // hebras que envuelven la bola (bandas elípticas)
      for (const [rot, ry, col] of [[-25, 26, "#8e64d8"], [35, 34, "#8e64d8"], [80, 20, "#efe4ff"], [-60, 40, "#efe4ff"], [10, 50, "#a37ff0"]]) {
        ellipse(x, y + 10, 76, ry, { fill: "none", stroke: P.ink, width: 7, rotate: rot, opacity: 0.35 })
        ellipse(x, y + 10, 76, ry, { fill: "none", stroke: col, width: 4, rotate: rot })
      }
      circle(x, y + 12, 30, { fill: "#f3eaff", glow: { blur: 14, color: P.white } })
      circle(x, y + 12, 17, { fill: P.white })
    })
    shine(x - 36, y - 26, 11, 6)
  })

  // (10) FUEGO DEL OJO: llamarada en lágrima, punta arriba
  element("fuego", { pivot: C[9] }, () => {
    const [x, y] = C[9]
    const flame = (s, sx, sy) => [[x, y - 108 * sy, 0], [x + 18 * sx, y - 54 * sy], [x + 40 * sx, y - 74 * sy, 0], [x + 50 * sx, y - 20 * sy],
      [x + 64 * sx, y + 30 * sy], [x + 40 * sx, y + 84 * sy], [x, y + 98 * sy], [x - 40 * sx, y + 84 * sy], [x - 64 * sx, y + 30 * sy],
      [x - 52 * sx, y - 26 * sy], [x - 42 * sx, y - 64 * sy, 0], [x - 20 * sx, y - 40 * sy]]
    const f = sticker(s => blob(flame(0, 1, 1), s), "#e8202a")
    clip(f, () => {
      blob(shift(flame(0, 0.74, 0.74), 0, 22), { fill: "#ff7a1a" })
      blob(shift(flame(0, 0.48, 0.5), 0, 44), { fill: "#ffd23a" })
      ellipse(x, y + 52, 20, 26, { fill: P.white, glow: { blur: 12, color: "#fff2a0" } })
    })
  })

  // (11) COHETE del jugador, mirando hacia arriba
  element("cohete", { pivot: C[10] }, () => {
    const x = C[10][0], y = C[10][1] - 8
    const nose = s => blob([[x, y - 104, 0], [x + 18, y - 76], [x + 24, y - 44], [x - 24, y - 44], [x - 18, y - 76]], s)
    const body = s => rect(x - 24, y - 52, 48, 120, { r: 10, ...s })
    const finR = s => blob([[x + 20, y + 4], [x + 54, y + 42], [x + 60, y + 84, 0], [x + 20, y + 62]], s)
    const finL = s => blob([[x - 20, y + 4], [x - 54, y + 42], [x - 60, y + 84, 0], [x - 20, y + 62]], s)
    // aletas traseras: asoman por debajo de las laterales
    const finC = s => { blob([[x - 16, y + 26], [x - 36, y + 66], [x - 34, y + 102, 0], [x - 14, y + 84]], s)
                        return blob([[x + 16, y + 26], [x + 36, y + 66], [x + 34, y + 102, 0], [x + 14, y + 84]], s) }
    const nozzle = s => blob([[x - 18, y + 64, 0], [x + 18, y + 64, 0], [x + 22, y + 88, 0], [x - 22, y + 88, 0]], s)
    const wh = { fill: P.white, stroke: P.white, width: W + K, join: "round" }
    // llamita
    const fl = [[x - 13, y + 84], [x + 13, y + 84], [x, y + 116, 0]]
    for (const part of [finC, finL, finR, nozzle, body, nose]) part(wh)
    blob(fl, { ...wh, width: 18 })
    blob(fl, { fill: "#ffb020", stroke: P.ink, width: 6, behind: true })
    ellipse(x, y + 97, 5, 8, { fill: P.white })
    const dk = { stroke: P.ink, width: K, join: "round", behind: true }
    finC({ ...dk, fill: "#5a1aa0" })
    finL({ ...dk, fill: P.purple }); finR({ ...dk, fill: P.purple })
    nozzle({ ...dk, fill: "#3a2a4f" })
    const bd = body({ ...dk, fill: "#f6f3ff" })
    clip(bd, () => {
      rect(x + 6, y - 60, 30, 140, { fill: "#c8bde6" })
      rect(x - 16, y - 46, 6, 100, { fill: P.white, r: 3 })
      rect(x - 30, y + 30, 60, 10, { fill: P.purple })
    })
    const ns = nose({ ...dk, fill: P.purple })
    clip(ns, () => {
      rect(x + 4, y - 110, 30, 70, { fill: "#5a1a9e" })
      ellipse(x - 8, y - 70, 4, 12, { fill: "#d9b8ff" })
    })
    circle(x, y - 18, 10, { fill: "#7ad8ff", stroke: P.ink, width: 5 })
    circle(x - 3, y - 21, 3, { fill: P.white })
  })
})
