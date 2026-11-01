// Tejedor de la Colmena: jefe araña, vista cenital, cabeza hacia abajo.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  ink: "#1a0f2e",       // contorno oscuro
  green: "#3fbf3a",     // quitina verde
  greenD: "#1f6b2c",    // verde oscuro / sombra
  greenL: "#a6f05a",    // luz verde
  purple: "#7a2fb0",    // placas moradas
  purpleD: "#3e1466",   // sombra morada
  purpleL: "#b46ae6",   // luz morada
  lime: "#c8ff4a",      // panal tenue
  lilac: "#e8c8ff",     // hileras / seda
  amber: "#ffb020",     // detalles ámbar
  amberD: "#b0560e",
  red: "#ff2a2a",       // ojos
  white: "#ffffff",
}

const CX = 512
// Patas del lado derecho: base, rodilla, tobillo, punta (se reflejan)
const LEGS = [
  [[575, 662], [722, 708], [805, 822], [838, 968]],
  [[612, 622], [792, 604], [905, 700], [992, 818]],
  [[618, 578], [792, 452], [922, 402], [1006, 438]],
  [[592, 540], [770, 330], [862, 196], [930, 52]],
]
const SEG_W = [40, 32, 22]

const ABD = { cx: CX, cy: 330, rx: 200, ry: 215 }
const abdPts = ring(ABD.cx, ABD.cy, ABD.rx, ABD.ry, 12)
const cephPts = [[512, 478], [590, 500], [636, 560], [628, 628], [580, 680], [512, 692], [444, 680], [396, 628], [388, 560], [434, 500]]
const headPts = [[512, 668], [580, 688], [606, 740], [580, 792], [512, 808], [444, 792], [418, 740], [444, 688]]
const fangR = [[540, 790], [578, 808], [576, 860], [548, 902, 0], [546, 852], [528, 820]]

function legPass(extra, color, segColors) {
  mirror(CX, () => {
    for (const L of LEGS) {
      for (let s = 0; s < 3; s++) {
        curve([L[s], L[s + 1]], { stroke: segColors ? segColors[s] : color, width: SEG_W[s] + extra, cap: "round" })
      }
    }
  })
}
function bodyPass(extra, color) {
  const st = { fill: color, stroke: color, width: extra, join: "round" }
  blob(abdPts, st)
  blob(cephPts, st)
  blob(headPts, st)
  mirror(CX, () => blob(fangR, st))
}

group({ scale: 0.9, around: [512, 512] }, () => {
layer("hilo", () => {
  // hilo de seda del que cuelga
  line(CX, -80, CX, 130, { stroke: P.white, width: 40 })
  line(CX, -80, CX, 130, { stroke: P.ink, width: 20 })
  rect(CX - 5, -80, 10, 210, { fill: P.lilac, glow: { blur: 12, color: P.lilac } })
  rect(CX - 2, -80, 4, 210, { fill: P.white })
})

layer("borde-pegatina", () => {
  // silueta blanca gruesa
  legPass(56, P.white)
  bodyPass(56, P.white)
})

layer("patas", () => {
  // contorno oscuro
  legPass(18, P.ink)
  // color base: verde, verde, punta morada
  legPass(0, null, [P.green, P.green, P.purple])
  // sombra de un tono en el lado inferior derecho de cada segmento
  mirror(CX, () => {
    for (const L of LEGS) {
      for (let s = 0; s < 3; s++) {
        const c = s < 2 ? P.greenD : P.purpleD
        curve(shift([L[s], L[s + 1]], 0, SEG_W[s] * 0.22), { stroke: c, width: SEG_W[s] * 0.45, cap: "round" })
      }
    }
  })
  // brillos blancos en el lado superior
  mirror(CX, () => {
    for (const L of LEGS) {
      for (let s = 0; s < 3; s++) {
        const a = L[s], b = L[s + 1]
        const p0 = [lerp(a[0], b[0], 0.2), lerp(a[1], b[1], 0.2) - SEG_W[s] * 0.2]
        const p1 = [lerp(a[0], b[0], 0.6), lerp(a[1], b[1], 0.6) - SEG_W[s] * 0.2]
        curve([p0, p1], { stroke: s < 2 ? P.greenL : P.purpleL, width: SEG_W[s] * 0.22, cap: "round" })
      }
      // articulaciones ámbar
      for (let s = 1; s < 3; s++) {
        circle(L[s][0], L[s][1], SEG_W[s] * 0.62, { fill: P.amber, stroke: P.ink, width: 7 })
        circle(L[s][0] - 4, L[s][1] - 5, SEG_W[s] * 0.2, { fill: P.white })
      }
      // espina en la punta
      const t = L[3], k = L[2]
      const dx = t[0] - k[0], dy = t[1] - k[1], d = Math.hypot(dx, dy)
      blob([[t[0] - dy / d * 11, t[1] + dx / d * 11], [t[0] + dx / d * 34, t[1] + dy / d * 34, 0], [t[0] + dy / d * 11, t[1] - dx / d * 11]], { fill: P.purpleD, stroke: P.ink, width: 6 })
    }
  })
})

layer("abdomen", () => {
  // quitina morada, panal lima, hileras lilas
  const abd = blob(abdPts, { fill: P.purple, stroke: P.ink, width: 22, behind: true })
  clip(abd, () => {
    // panal tenue
    const hr = 34
    for (let row = -7; row <= 7; row++) {
      for (let col = -5; col <= 5; col++) {
        const x = CX + col * hr * 1.73 + (row % 2 ? hr * 0.865 : 0)
        const y = ABD.cy + row * hr * 1.5
        poly(ring(x, y, hr, hr, 6), { fill: "none", stroke: P.lime, width: 5, opacity: 0.22 })
      }
    }
    // sombra de un tono (abajo-derecha)
    const sh = ellipse(ABD.cx, ABD.cy, ABD.rx + 10, ABD.ry + 10, { fill: P.purpleD })
    cut(sh, [ellipse(ABD.cx - 38, ABD.cy - 44, ABD.rx, ABD.ry, { hidden: true })])
    // placa dorsal central y bandas
    blob([[512, 150], [560, 230], [548, 400], [512, 520], [476, 400], [464, 230]], { fill: P.purpleD, opacity: 0.55 })
    // chevrones apuntando a la cabeza
    for (const [y, w] of [[250, 70], [330, 95], [410, 80]]) {
      mirror(CX, () => brush([[CX - w, y - 34], [CX - w * 0.45, y - 6], [CX - 4, y + 18]], { width: 13, color: P.purpleD, taper: [0.8, 0] }))
    }
    // manchas ámbar
    mirror(CX, () => {
      for (const [x, y, r] of [[450, 230, 16], [420, 330, 13], [440, 430, 11]]) {
        circle(x, y, r, { fill: P.amber, stroke: P.amberD, width: 4 })
      }
    })
    // brillo blanco arriba-izquierda
    blob([[350, 230], [400, 165], [470, 140], [440, 185], [385, 240], [355, 280]], { fill: P.purpleL })
    blob([[380, 200], [420, 170], [445, 160], [405, 200]], { fill: P.white, opacity: 0.95 })
    circle(352, 300, 9, { fill: P.white })
  })
  // hileras (glándulas de seda) brillando arriba
  circle(CX, 140, 60, { fill: radial(CX, 140, 60, [alpha(P.lilac, 0.75), alpha(P.lilac, 0)]) })
  mirror(CX, () => {
    ellipse(CX - 24, 140, 17, 26, { fill: P.lilac, stroke: P.ink, width: 7, rotate: 20, glow: { blur: 14, color: P.lilac } })
    circle(CX - 27, 133, 5, { fill: P.white })
  })
  ellipse(CX, 126, 16, 24, { fill: P.lilac, stroke: P.ink, width: 7, glow: { blur: 18, color: P.lilac } })
  ellipse(CX - 3, 120, 6, 9, { fill: P.white })
})

layer("cefalotorax", () => {
  // placa verde con surco
  const ceph = blob(cephPts, { fill: P.green, stroke: P.ink, width: 22, behind: true })
  clip(ceph, () => {
    blob(shift(cephPts, 26, 30), { fill: "none", stroke: P.greenD, width: 50 })
    // surco central y estrías radiales
    brush([[512, 500], [512, 560], [512, 600]], { width: 16, color: P.greenD, taper: [0.3, 0.3] })
    circle(512, 600, 12, { fill: P.greenD })
    mirror(CX, () => {
      for (const a of [200, 225, 250, 160]) {
        const p = polar(512, 600, 40, a + 90), q = polar(512, 600, 105, a + 90)
        brush([p, q], { width: 10, color: P.greenD, taper: [0.2, 0.6] })
      }
    })
    // placas moradas laterales
    mirror(CX, () => blob([[420, 530], [452, 512], [462, 560], [430, 600], [404, 590]], { fill: P.purple, stroke: P.ink, width: 6 }))
    // brillo
    blob([[430, 520], [470, 498], [500, 500], [460, 530], [430, 560]], { fill: P.greenL })
    blob([[448, 516], [478, 504], [462, 520]], { fill: P.white })
  })
})

layer("cabeza", () => {
  // quelíceros ámbar (bajo la cabeza)
  mirror(CX, () => {
    const f = blob(fangR, { fill: P.amber, stroke: P.ink, width: 16, behind: true })
    clip(f, () => {
      blob(shift(fangR, 10, 4), { fill: "none", stroke: P.amberD, width: 16 })
      brush([[548, 805], [560, 830], [556, 858]], { width: 7, color: P.white, taper: [0.2, 0.6] })
    })
  })
  const head = blob(headPts, { fill: P.greenD, stroke: P.ink, width: 20, behind: true })
  clip(head, () => {
    blob(shift(headPts, 14, 18), { fill: "none", stroke: shade(P.greenD, -0.4), width: 30 })
    blob([[450, 700], [490, 680], [520, 680], [470, 705]], { fill: P.green })
    circle(512, 742, 90, { fill: radial(512, 742, 90, [alpha(P.red, 0.55), alpha(P.red, 0)]) })
  })
  // ojos: punto débil
  const eye = (x, y, r) => {
    circle(x, y, r + 5, { fill: P.ink })
    circle(x, y, r, { fill: radial(x - r * 0.3, y - r * 0.3, r * 1.2, ["#ff8a6a", P.red, "#9a0010"]), glow: { blur: r * 0.9, color: alpha(P.red, 0.9) } })
    circle(x - r * 0.35, y - r * 0.38, r * 0.3, { fill: P.white })
  }
  mirror(CX, () => {
    eye(486, 742, 25)
    eye(452, 706, 13)
    eye(440, 752, 11)
    eye(470, 784, 9)
  })
  eye(512, 700, 10)
})
})
