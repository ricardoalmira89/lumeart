// dibujo.js: Núcleo de La Muralla (jefe de la Colmena), vista cenital, luz arriba-izquierda.
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

const P = {
  ink: "#0c1a10",       // contorno oscuro (verde casi negro)
  chitin: "#1f4a2a",    // quitina verde oscuro (base)
  chitinD: "#12301b",   // sombra de quitina
  chitinL: "#3f8a4a",   // luz de quitina
  resin: "#2c5e33",     // nudo de resina
  plate: "#6a2f8f",     // placas moradas
  plateD: "#3e1a5e",
  plateL: "#a565cf",
  amber: "#f2a52a",     // colmillos y detalles
  amberD: "#b0600f",
  red: "#ff2a2a",       // ojo-núcleo
  redD: "#8e0b1c",
  white: "#ffffff",
}
const CX = 512, CY = 345
const pt = (r, a, cx = CX, cy = CY, sy = 1) => [cx + r * Math.cos(a * Math.PI / 180), cy + r * Math.sin(a * Math.PI / 180) * sy]

// ---- siluetas ----
// banda de pared que sale por ambos lados del lienzo (sin contorno visible en los bordes)
const bandPts = [
  [-120, 262, 0], [60, 270], [180, 252], [260, 240],
  [764, 240], [844, 252], [964, 270], [1144, 262, 0],
  [1144, 470, 0], [964, 462], [850, 478], [760, 500],
  [264, 500], [174, 478], [60, 462], [-120, 470, 0],
]
const bodyPts = [
  [512, 140], [640, 152], [748, 200], [800, 280], [806, 380], [760, 470],
  [650, 532], [512, 548], [374, 532], [264, 470], [218, 380], [224, 280], [276, 200], [384, 152],
]
const fangX = [400, 438, 475, 512, 549, 586, 624]
const fangPts = fangX.map((x, i) => {
  const d = (x - 512) / 112
  const len = 80 - Math.abs(d) * 26
  const w = 17
  const tipx = x - d * 8
  return [[x - w, 520], [x + w, 520], [x + w * 0.5, 520 + len * 0.6], [tipx, 528 + len, 0], [x - w * 0.6, 520 + len * 0.55]]
})

layer("contorno-pegatina", () => {
  // borde blanco grueso de pegatina detrás de todo
  blob(bandPts, { fill: P.white, stroke: P.white, width: 44, join: "round" })
  blob(bodyPts, { fill: P.white, stroke: P.white, width: 44, join: "round" })
  fangPts.forEach(f => blob(f, { fill: P.white, stroke: P.white, width: 40, join: "round" }))
  // contorno oscuro
  blob(bandPts, { fill: P.ink, stroke: P.ink, width: 16, join: "round" })
  blob(bodyPts, { fill: P.ink, stroke: P.ink, width: 18, join: "round" })
  fangPts.forEach(f => blob(f, { fill: P.ink, stroke: P.ink, width: 14, join: "round" }))
})

layer("colmillos", () => {
  fangPts.forEach((f, i) => {
    const s = blob(f, { fill: P.amber })
    const b = bbox(s)
    clip(s, () => {
      rect(b.cx + 2, b.y, b.w, b.h, { fill: P.amberD, opacity: 0.85 })
      brush([[b.cx - 8, b.y + 8], [b.cx - 6, b.y + b.h * 0.6]], { width: 5, color: "#fff2c0", taper: [0.1, 0.8] })
    })
  })
})

layer("pared-lateral", () => {
  const band = blob(bandPts, { fill: P.chitin })
  freeform(band, [
    [40, 285, P.resin, 0.2], [984, 285, P.resin, 0.2], [40, 455, P.chitinD, 0.2],
    [984, 455, P.chitinD, 0.2], [250, 360, P.chitin, 0.2], [780, 360, P.chitin, 0.2],
  ])
})

// venas que salen del nudo hacia los dos bordes (simétricas); se dibujan sobre el cuerpo
const drawVeins = () => {
  mirror(512, () => {
    const veins = [
      { pts: [[160, 300], [120, 276], [70, 270]], w: 10, t: [0.5, 0] },
      { pts: [[150, 418], [110, 446], [50, 450]], w: 9, t: [0.5, 0] },
      { pts: [[284, 280], [200, 296], [110, 312], [-40, 300]], w: 26, t: [0.25, 0] },
      { pts: [[288, 442], [190, 420], [100, 404], [-40, 420]], w: 22, t: [0.25, 0] },
      { pts: [[280, 352], [170, 352], [80, 362], [-40, 356]], w: 14, t: [0.25, 0] },
    ]
    veins.forEach(v => {
      brush(shift(v.pts, 2, 6), { width: v.w + 6, color: P.ink, taper: v.t, opacity: 0.6 })
      brush(v.pts, { width: v.w, color: P.plate, taper: v.t })
      brush(shift(v.pts, -1, -v.w * 0.22), { width: v.w * 0.35, color: P.plateL, taper: [v.t[0] + 0.1, 0.1], opacity: 0.9 })
    })
    // nódulos ámbar a lo largo de las venas
    ;[[200, 297, 9], [118, 310, 7], [190, 420, 8], [96, 405, 6]].forEach(([x, y, r]) => {
      circle(x, y, r + 4, { fill: P.ink })
      circle(x, y, r, { fill: P.amber })
      circle(x - r * 0.3, y - r * 0.35, r * 0.35, { fill: "#fff3c8" })
    })
  })
}

layer("cuerpo", () => {
  const body = blob(bodyPts, { fill: P.resin })
  const bf = freeform(body, [
    [330, 190, P.chitinL, 0.2], [700, 210, P.resin, 0.2], [260, 400, P.resin, 0.1],
    [720, 470, P.chitinD, 0.3], [512, 530, P.chitinD, 0.2], [512, 345, P.chitin, 0.3],
  ])
  clip(bf, () => {
    // sombra de un tono en el lado opuesto a la luz
    cel(body, { depth: 0.22, base: P.resin, color: P.chitinD })
    // anillo de placas de quitina segmentadas alrededor del ojo
    for (let i = 0; i < 12; i++) {
      const a0 = i * 30 + 4, a1 = a0 + 24
      const outer = range(5).map(k => pt(225, lerp(a0, a1, k / 4), CX, CY, 0.78))
      const inner = range(5).map(k => pt(178, lerp(a1, a0, k / 4), CX, CY, 0.8))
      const seg = blob([...outer.map(p => [...p, 0.5]), ...inner.map(p => [...p, 0.5])], { fill: P.chitin, stroke: P.ink, width: 6 })
      const mid = (a0 + a1) / 2
      const lit = Math.cos((mid + 135) * Math.PI / 180)
      clip(seg, () => {
        brush(range(5).map(k => pt(218, lerp(a0 + 2, a1 - 2, k / 4), CX, CY, 0.78)), { width: 7, color: lit > 0 ? P.chitinL : P.resin, taper: [0.3, 0.3] })
      })
    }
    // brillo de borde arriba-izquierda
    brush([[262, 300], [300, 215], [400, 165], [500, 152]], { width: 12, color: "#8fd17a", opacity: 0.8, taper: [0.4, 0.5] })
  })
})

layer("ojo", () => {
  // cuenca
  circle(CX, CY, 136, { fill: P.ink })
  circle(CX, CY, 128, { fill: "#2a0610" })
  // resplandor
  circle(CX, CY, 114, { fill: P.red, glow: { blur: 30, color: "#ff3a2a" } })
  const eye = circle(CX, CY, 114, { fill: P.red })
  freeform(eye, [
    [480, 310, "#ffb04a", 0.25], [512, 345, "#ff4a2a", 0.2], [585, 410, P.redD, 0.3],
    [440, 410, "#c4142a", 0.1], [600, 290, "#e21f2a", 0.1],
  ])
  clip(eye, () => {
    // venitas del iris
    repeat_radial(CX, CY, 14, i => {
      brush([[CX, CY - 40], [CX + 4, CY - 70], [CX - 2, CY - 100]], { width: 4, color: "#ffd27a", opacity: 0.45 })
    })
    // pupila rasgada vertical
    blob([[CX, CY - 78, 0], [CX + 17, CY - 20], [CX + 19, CY + 20], [CX, CY + 78, 0], [CX - 19, CY + 20], [CX - 17, CY - 20]], { fill: "#1a0008" })
    blob([[CX, CY - 60, 0], [CX + 6, CY], [CX, CY + 60, 0], [CX - 6, CY]], { fill: "#ff7a3a", opacity: 0.55 })
    // sombra inferior derecha
    circle(CX + 34, CY + 34, 110, { fill: "none", stroke: P.redD, width: 36, opacity: 0.55 })
  })
  // brillos blancos marcados
  ellipse(CX - 34, CY - 36, 20, 12, { fill: P.white, rotate: -40 })
  circle(CX - 8, CY - 58, 6, { fill: P.white })
})

layer("parpados", () => {
  // cuatro párpados de placas moradas medio abiertos, en diagonal
  ;[225, 315, 45, 135].forEach(th => {
    const outer = range(6).map(k => [...pt(172, lerp(th - 34, th + 34, k / 5)), 0.6])
    const inner = range(5).map(k => [...pt(102, lerp(th + 22, th - 22, k / 4)), 0.8])
    outer[0][2] = 0; outer[5][2] = 0
    const pts = [...outer, ...inner]
    blob(shift(pts, 6, 9), { fill: "#000000", opacity: 0.45 })
    blob(pts, { fill: P.ink, stroke: P.ink, width: 16, join: "round" })
    const lid = blob(pts, { fill: P.plate })
    const lit = Math.cos((th - 225) * Math.PI / 180) // 1 = cara a la luz
    clip(lid, () => {
      // sombra de un tono: mitad alejada de la luz
      const sh = pt(60, th + 45)
      circle(sh[0] + 40, sh[1] + 40, 150, { fill: P.plateD, opacity: lit > 0.5 ? 0.35 : 0.85 })
      // segmentos de la placa
      ;[125, 150].forEach(r => {
        curve(range(7).map(k => pt(r, lerp(th - 38, th + 38, k / 6))), { stroke: P.plateD, width: 6 })
      })
      // labio interior claro (borde que abraza el ojo)
      brush(range(5).map(k => pt(110, lerp(th - 20, th + 20, k / 4))), { width: 10, color: P.plateL, taper: [0.3, 0.3] })
      // brillo blanco en el lado de la luz
      brush(range(4).map(k => pt(165, lerp(th - 30, th - 5, k / 3))), { width: 8, color: P.white, opacity: 0.4 + 0.5 * Math.max(0, lit), taper: [0.4, 0.4] })
    })
    // punta ámbar en el centro exterior de cada párpado
    const [tx, ty] = pt(140, th)
    circle(tx, ty, 13, { fill: P.ink })
    circle(tx, ty, 8, { fill: P.amber })
    circle(tx - 3, ty - 3, 3, { fill: "#fff3c8" })
  })
})
layer("venas", () => drawVeins())

layer("retoques", () => {
  const sil = [blob(bandPts, { hidden: true }), blob(bodyPts, { hidden: true })]
  clip(sil, () => grain({ opacity: 0.06, size: 1, blend: "overlay" }))
})
