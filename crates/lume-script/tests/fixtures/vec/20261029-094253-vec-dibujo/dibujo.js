// dibujo.js: farol de queroseno antiguo encendido
canvas({ width: 720, height: 1280, background: null })

const P = {
  dark: "#2a1a10",
  base: "#6e4c2e",
  mid: "#8f6842",
  light: "#c99a60",
  rust: "#7a3a1c",
  amber: "#e8963e",
  glow: "#ffd27a",
  core: "#fff6dc",
}
const CX = 360

// Asa de alambre (detrás de todo)
layer("asa", () => {
  mirror(CX, () => {
    curve([[168, 445], [174, 330], [184, 190], [205, 132], [270, 106], [338, 94], [360, 84]],
      { stroke: P.dark, width: 9, cap: "round" })
    curve([[166, 440], [172, 330], [182, 190], [203, 134], [268, 107], [336, 95], [358, 86]],
      { stroke: P.mid, width: 3, cap: "round", opacity: 0.8 })
    // bisagra
    circle(166, 448, 14, { fill: "none", stroke: P.dark, width: 7 })
  })
})

// Resplandor suave alrededor del globo
layer("halo", () => {
})

// Marco: tubos laterales en U
layer("marco", () => {
  mirror(CX, () => {
    const tube = [[268, 436], [210, 430], [165, 446], [142, 520], [126, 700], [114, 875], [128, 948], [180, 986]]
    brush(tube, { width: 40, taper: [0, 0], color: P.dark })
    brush(tube, { width: 32, taper: [0, 0], color: P.base })
    // sombra exterior
    brush(shift(tube, -8, 0), { width: 10, taper: [0.1, 0.1], color: shade(P.base, -0.45) })
    // luz del farol en el borde interior
    brush(shift(tube.slice(2, 7), 10, 0), { width: 7, taper: [0.3, 0.3], color: P.glow, opacity: 0.75 })
    brush(shift(tube.slice(1, 4), -2, -4), { width: 5, taper: [0.3, 0.3], color: P.light, opacity: 0.8 })
  })
})

// Globo de vidrio encendido
layer("globo", () => {
  const gpts = [[230, 625], [360, 615], [490, 625], [510, 700], [508, 800], [480, 890], [360, 900], [240, 890], [212, 800], [210, 700]]
  const g = blob(gpts, { fill: P.amber })
  const gf = freeform(g, [
    [360, 745, P.core, 0.3],
    [225, 760, "#d9812f", 0.1],
    [495, 760, "#d47a2c", 0.1],
    [360, 885, "#e89a40", 0.1],
  ])
  clip(gf, () => {
    // reflejos verticales del vidrio
    brush([[255, 650], [242, 760], [258, 870]], { width: 14, color: "#fff1cf", opacity: 0.35 })
    brush([[470, 660], [482, 760], [468, 860]], { width: 8, color: "#fff1cf", opacity: 0.3 })
    // vetas/luz refractada
    curve([[230, 700], [300, 760], [360, 740], [430, 770], [500, 720]], { stroke: alpha(P.core, 0.25), width: 3 })
    curve([[230, 830], [300, 800], [380, 820], [490, 790]], { stroke: alpha(P.core, 0.2), width: 3 })
    // portamechas
    rect(338, 790, 44, 60, { fill: linear(338, 0, 382, 0, ["#7a4a24", "#e0a560", "#7a4a24"]), r: 4, opacity: 0.85 })
    ellipse(CX, 790, 22, 6, { fill: "#d89a50" })
    ellipse(CX, 850, 70, 14, { fill: "#9a5a2a", opacity: 0.6 })
    // llama
    ellipse(CX, 740, 90, 110, { fill: radial(CX, 745, 100, [alpha("#ffffff", 0.7), alpha(P.core, 0)]) })
    blob([[CX, 650, 0], [380, 712], [386, 760], [CX, 792], [334, 760], [340, 712]],
      { fill: "#ffc860", glow: { blur: 34, color: "#ffb04a" } })
    blob([[CX, 678, 0], [373, 728], [375, 764], [CX, 786], [345, 764], [347, 728]], { fill: P.core, blur: 3 })
    blob([[CX, 708, 0], [366, 745], [CX, 778], [354, 745]], { fill: "#ffffff" })
    ellipse(CX, 783, 6, 4, { fill: "#e07a30", opacity: 0.7, blur: 1.5 })
    // borde oscuro del vidrio
  })
  outline(gf, -6, "#8a4318", { opacity: 0.5, blur: 4 })
})

// Cuerpo superior: tapa, chimenea, collar
layer("superior", () => {
  const M = { color: "#7a5634", light: { from: 125, height: 40 } }
  revolve(CX, [[112, 582], [114, 628]], M) // collar
  revolve(CX, [[76, 504], [76, 584]], M) // tubo de ventilación
  revolve(CX, [[94, 352], [94, 498]], M) // chimenea
  revolve(CX, [[124, 492], [120, 506]], M) // ala
  revolve(CX, [[100, 422], [100, 444]], M) // aro
  revolve(CX, [[0, 316], [40, 318], [108, 350], [100, 358]], M) // tapa
})

// Cuerpo inferior: quemador y depósito
layer("deposito", () => {
  const M = { color: "#7a5634", light: { from: 125, height: 40 } }
  revolve(CX, [[214, 1100], [226, 1108], [226, 1170], [214, 1184]], M) // base
  revolve(CX, [[76, 962], [150, 970], [200, 990], [214, 1012], [214, 1104]], M) // depósito
  revolve(CX, [[76, 912], [76, 966]], M) // cuello
  revolve(CX, [[102, 888], [102, 914]], M) // anillo bajo el globo
})

// Rejilla de alambre sobre el globo
layer("rejilla", () => {
  const W = { stroke: P.dark, width: 5, cap: "round" }
  const H = { stroke: alpha(P.glow, 0.6), width: 1.6, cap: "round" }
  // aros horizontales siguiendo el globo
  const top = [[212, 690], [290, 712], [360, 716], [430, 712], [508, 690]]
  const bot = [[226, 868], [290, 884], [360, 888], [430, 884], [494, 868]]
  curve(top, W); curve(shift(top, 0, -1.5), H)
  curve(bot, W); curve(shift(bot, 0, -1.5), H)
  // cruz en X
  const a = [[212, 692], [300, 760], [360, 800], [420, 840], [494, 870]]
  const b = [[508, 692], [420, 760], [360, 800], [300, 840], [226, 870]]
  curve(a, W); curve(shift(a, 0, -1.5), H)
  curve(b, W); curve(shift(b, 0, -1.5), H)
  // varillas que sujetan la rejilla al marco
  mirror(CX, () => {
    curve([[130, 860], [175, 870], [226, 868]], { stroke: P.dark, width: 5 })
  })
})

// Detalles: anilla, tapón, palanca, óxido y grano
layer("detalles", () => {
  // anilla superior
  circle(CX, 272, 38, { fill: "none", stroke: P.dark, width: 11 })
  circle(CX, 272, 38, { fill: "none", stroke: P.mid, width: 4, dash: [70, 400], opacity: 0.9 })
  rect(350, 304, 20, 16, { fill: P.base, r: 3 })
  // agujeros de ventilación en la chimenea y el tubo
  for (const x of [-60, -20, 20, 60]) {
    ellipse(CX + x * 1.1, 380, 9, 10, { fill: P.dark })
  }
  for (const x of [-50, -18, 18, 50]) {
    ellipse(CX + x, 545, 8, 9, { fill: P.dark })
    ellipse(CX + x + 1, 547, 4, 4, { fill: alpha(P.glow, 0.6), blur: 1.5 })
  }
  // luz cálida del globo sobre el metal cercano
  clip(rect(244, 600, 232, 30, { hidden: true }), () => {
    ellipse(CX, 632, 120, 20, { fill: alpha("#ff9a30", 0.6), blend: "color-dodge", blur: 8 })
  })
  clip(rect(256, 884, 208, 32, { hidden: true }), () => {
    ellipse(CX, 886, 110, 22, { fill: alpha("#ff9a30", 0.6), blend: "color-dodge", blur: 8 })
  })
  clip(blob([[160, 990], [360, 960], [560, 990], [575, 1030], [360, 1040], [145, 1030]], { hidden: true }), () => {
    ellipse(CX, 975, 170, 40, { fill: alpha(P.amber, 0.45), blend: "screen", blur: 16 })
  })
  // tapón del depósito
  revolve(352, [[0, 958], [36, 960], [38, 980], [22, 988], [0, 988]], { color: "#7a5634", light: { from: 125, height: 40 } })
  // palanca de mecha
  curve([[176, 850], [158, 852], [150, 880]], { stroke: P.dark, width: 6, cap: "round" })
  rect(146, 815, 12, 50, { fill: P.base, stroke: P.dark, width: 3, r: 4 })
  // manchas de óxido y desgaste en el depósito y la base
  seed(7)
  const tank = blob([[150, 1000], [360, 960], [570, 1000], [586, 1180], [360, 1196], [134, 1180]], { hidden: true })
  clip(tank, () => {
    for (let i = 0; i < 26; i++) {
      const x = rand(175, 545), y = rand(1015, 1170)
      blob(jitter(ring(x, y, rand(6, 26), rand(4, 14), 7), 3), { fill: pick([P.rust, P.dark, "#5a3a20"]), opacity: rand(0.15, 0.4), blend: "multiply", blur: 2 })
    }
    for (let i = 0; i < 14; i++) {
      const x = rand(160, 400), y = rand(1010, 1170)
      brush([[x, y], [x + rand(10, 40), y + rand(-3, 3)]], { width: rand(1.5, 3), color: P.light, opacity: 0.35 })
    }
  })
  const upper = blob([[260, 330], [460, 330], [490, 500], [480, 630], [240, 630], [230, 500]], { hidden: true })
  clip(upper, () => {
    for (let i = 0; i < 16; i++) {
      const x = rand(290, 430), y = rand(360, 620)
      blob(jitter(ring(x, y, rand(5, 16), rand(3, 9), 7), 2), { fill: pick([P.rust, P.dark]), opacity: rand(0.15, 0.35), blend: "multiply", blur: 1.5 })
    }
  })
})
