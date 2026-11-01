// dibujo.js: pequeño caballero-insecto en una caverna azul
canvas({ width: 1024, height: 1024, background: linear(0, 0, 0, 1024, ["#1c3346", "#2c5068", "#14232f"]) })

const P = {
  deep: "#0f1c27",
  cave: "#22405a",
  fog: "#6f9bb3",
  glow: "#f3e6c4",
  mask: "#f4f1ea",
  maskSh: "#c9cbd6",
  ink: "#141821",
  cloak: "#3a4555",
  cloakDk: "#1f2631",
  cloakLt: "#6b7a8e",
  steel: "#8a93a3",
}
setLight(-1, -1)

layer("fondo", () => {
  // halo de luz detrás de la cabeza
  circle(512, 420, 460, { fill: radial(512, 420, 460, [[0, alpha(P.glow, 0.55)], [0.45, alpha(P.fog, 0.25)], [1, alpha(P.fog, 0)]]) })
  // columnas lejanas de roca
  seed(3)
  for (const x of [80, 190, 840, 950]) {
    const w = rand(60, 110)
    blob(roughen([[x - w / 2, -20], [x + w / 2, -20], [x + w * 0.4, 1040], [x - w * 0.45, 1040]], 10, { detail: 3 }), { fill: alpha(P.cave, 0.75), blur: 3 })
  }
  // estalactitas
  for (let i = 0; i < 14; i++) {
    let x = rand(0, 1024)
    if (x > 280 && x < 740) x = x < 512 ? x - 260 : x + 260
    const w = rand(25, 70), h = rand(80, 230)
    blob([[x - w, -10], [x + w, -10], [x + w * 0.2, h * 0.6, 0.5], [x, h, 0]], { fill: mix(P.deep, P.cave, rand(0.1, 0.5)) })
  }
  // niebla baja
  for (let i = 0; i < 4; i++) {
    ellipse(rand(100, 900), 900 + i * 20, rand(300, 500), 70, { fill: alpha(P.fog, 0.18), blur: 30 })
  }
  // motas de luz
  for (let i = 0; i < 40; i++) {
    const x = rand(0, 1024), y = rand(80, 900)
    circle(x, y, rand(1.5, 4), { fill: alpha(P.glow, rand(0.3, 0.8)), glow: { blur: 6, color: P.glow } })
  }
})

layer("suelo", () => {
  // repisa de roca
  const g = roughen([[-20, 975], [300, 965], [560, 972], [800, 962], [1044, 970], [1044, 1044], [-20, 1044]], 8, { detail: 3 })
  const ground = blob(g.map(p => [p[0], p[1]]), { fill: linear(0, 960, 0, 1024, [P.cave, P.deep]) })
  outline([ground], -4, alpha(P.fog, 0.6))
  ellipse(500, 985, 170, 14, { fill: alpha(P.deep, 0.7), blur: 6 })
})

layer("espada", () => {
  // hoja: de la empuñadura (detrás del hombro) hacia abajo a la derecha
  const blade = poly([[372, 655], [388, 640], [692, 922], [680, 928]], { fill: linear(372, 655, 692, 922, [P.steel, "#c5ccd8"]) })
  ink(blade, { width: 2.5, weight: 2, color: P.ink })
  // guarda y empuñadura
  const guard = poly([[355, 668], [395, 623], [404, 631], [364, 676]], { fill: "#5e6878" })
  ink(guard, { width: 2.5, color: P.ink })
  const hilt = poly([[322, 606], [338, 594], [384, 650], [368, 662]], { fill: "#58606e" })
  ink(hilt, { width: 2.5, color: P.ink })
  circle(326, 600, 9, { fill: "#6a7383", stroke: P.ink, width: 3 })
})

layer("capa", () => {
  // piernas finas
  for (const [x, d] of [[458, -6], [540, 6]]) {
    blob([[x - 20, 880], [x + 20, 880], [x + 16 + d * 0.3, 950], [x + 6 + d, 985, 0.4], [x - 8 + d, 986, 0.4], [x - 16, 940]], { fill: "#0b0e14" })
  }
  // silueta de la capa
  const pts = [[405, 632], [500, 644], [612, 632], [622, 700], [640, 790], [648, 880], [640, 930, 0], [612, 905], [600, 945, 0], [570, 915],
    [545, 935, 0], [520, 900], [495, 930, 0], [470, 905], [440, 942, 0], [418, 905], [392, 940, 0], [378, 900], [372, 820], [385, 720]]
  const cloak = blob(pts, { fill: P.cloak })
  clip(cloak, () => {
    freeform(blob(pts, { fill: P.cloak }), [[430, 700, P.cloakLt, 0.3], [520, 760, P.cloak, 0.4], [620, 850, P.cloakDk, 0.4], [400, 900, P.cloakDk, 0.2], [505, 900, P.deep, 0.2]])
    // dos solapas que se cruzan
    const flapR = blob([[560, 645], [610, 650], [628, 760], [600, 860], [540, 925, 0], [500, 900], [470, 820], [520, 720]], { fill: mix(P.cloak, P.cloakDk, 0.35) })
    cel(flapR, { depth: 0.3, color: P.cloakDk })
    ink(flapR, { width: 2.5, weight: 2, color: P.ink })
    const flapL = blob([[420, 645], [470, 650], [520, 740], [545, 840], [520, 935, 0], [470, 880], [420, 800], [395, 720]], { fill: mix(P.cloak, P.cloakLt, 0.3) })
    cel(flapL, { depth: 0.3, color: P.cloak })
    ink(flapL, { width: 2.5, weight: 2, color: P.ink })
    // pliegues de pincel
    seed(7)
    for (let i = 0; i < 16; i++) {
      const x0 = rand(395, 620), x1 = x0 + rand(-25, 25)
      brush([[x0 + rand(-10, 10), rand(650, 720)], [lerp(x0, x1, 0.5), rand(780, 820)], [x1, rand(890, 935)]], { width: rand(3, 7), color: alpha(i % 3 ? P.cloakDk : P.cloakLt, 0.75), taper: [0.4, 0.4] })
    }
    // sombra bajo la máscara
    ellipse(510, 640, 160, 40, { fill: alpha(P.deep, 0.7), blur: 14 })
  })
  ink(cloak, { width: 3, weight: 2.2, color: P.ink })
  // pliegues de tinta exteriores
  brush([[440, 700], [415, 800], [405, 930]], { width: [1, 4, 1], color: P.ink })
  brush([[600, 760], [625, 850], [622, 925]], { width: [1, 4, 1], color: P.ink })
})
layer("mascara", () => {
  const hornPts = [[305, 360], [252, 285], [236, 205], [262, 135], [318, 82], [372, 58], [412, 62, 0], [372, 80], [362, 90, 0], [386, 102, 0],
    [342, 135], [306, 195], [300, 252], [318, 292], [352, 318], [380, 360]]
  const maskPts = [[312, 322], [500, 306], [688, 322], [698, 410], [690, 520], [662, 598], [585, 632], [500, 640], [415, 632], [338, 600], [310, 520], [302, 410]]
  const shapes = []
  mirror(500, () => {
    shapes.push(blob(hornPts, { fill: P.mask }))
  })
  // cuernos: sombra interior y tinta antes de la máscara, que tapa sus bases
  for (const s of shapes) {
    cel(s, { depth: 0.2, color: P.maskSh })
    ink(s, { width: 4, weight: 2, color: P.ink })
  }
  const mask = blob(maskPts, { fill: P.mask })
  cel(mask, { depth: 0.18, color: P.maskSh })
  clip(mask, () => {
    // sombra que proyectan los cuernos/curva superior
    ellipse(500, 300, 230, 40, { fill: alpha(P.maskSh, 0.7), blur: 16 })
    // reflejo frío de la capa en la barbilla
    ellipse(510, 650, 200, 50, { fill: alpha(P.fog, 0.35), blur: 18 })
    // luz cálida
    ellipse(400, 400, 70, 90, { fill: alpha("#ffffff", 0.8), blur: 30 })
  })
  ink(mask, { width: 4, weight: 2, color: P.ink })
  // ojos
  for (const [cx, cy] of [[402, 522], [598, 520]]) {
    ellipse(cx, cy, 54, 58, { fill: P.ink })
    ellipse(cx - 18, cy - 24, 9, 7, { fill: alpha("#ffffff", 0.35), rotate: -30 })
  }
})
layer("retoques", () => {
  // contraluz frío en el borde derecho de la máscara y la capa
  brush([[690, 360], [694, 470], [672, 580]], { width: [1, 5, 1], color: alpha("#bfe3f2", 0.8) })
  brush([[632, 720], [644, 820], [642, 900]], { width: [1, 4, 1], color: alpha(P.fog, 0.8) })
  // viñeta y grano
  rect(0, 0, 1024, 1024, { fill: radial(512, 480, 720, [[0.55, alpha(P.deep, 0)], [1, alpha(P.deep, 0.7)]]) })
  grain({ opacity: 0.07 })
})
