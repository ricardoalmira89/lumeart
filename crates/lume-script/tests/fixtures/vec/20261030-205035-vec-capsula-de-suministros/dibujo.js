// dibujo.js: cápsula de suministros de la flota humana, vista cenital, con esqueleto.
canvas({ width: 1024, height: 1024, background: null })
setLight(-1, -1)

const P = {
  casco: "#f4f7fb",     // blanco del casco
  cascoS: "#b4c2dc",    // sombra de un tono del casco
  tinta: "#1b2240",     // contorno oscuro
  teal: "#17a9a2",      // galones azul verdoso
  tealS: "#0e7479",
  cian: "#3fe6ff",      // cristal
  cianS: "#0a73a8",
  naranja: "#ff8a1f",   // acentos
  naranjaS: "#cf4c0c",
  carga: "#fff3a0",     // brillo de la carga
  metal: "#7d8cab",     // patas
  metalS: "#4f5b7c",
}
P.blanco = "#ffffff"
const BORDE = 30       // grosor del borde blanco de pegatina, fuera de la tinta
const INK = { width: 11, weight: 1.9, color: P.tinta }
const C = 512

// ---------- geometría compartida (pasada de borde blanco y pasada de color) ----------
const CUERPO = [347, 292, 330, 440, 160]           // x, y, w, h, r
const sx = (side, x) => side < 0 ? x : 2 * C - x
const finPts = (side) => [[372, 418], [270, 432], [176, 478], [118, 560, 0], [150, 612], [236, 604], [372, 600]]
  .map(([x, y, s]) => s === undefined ? [sx(side, x), y] : [sx(side, x), y, s])
const PATAS = [
  { n: "pata_sup_izq", j: [420, 392], f: [292, 262] },
  { n: "pata_sup_der", j: [604, 392], f: [732, 262] },
  { n: "pata_inf_izq", j: [420, 640], f: [292, 770] },
  { n: "pata_inf_der", j: [604, 640], f: [732, 770] },
]
const ANT = { j: [512, 318], top: [512, 168] }

// sombra de un solo tono: la pieza en sombra y encima la misma desplazada hacia la luz
function sombra(forma, dibujar, base, os, d = 34) {
  clip(forma, () => {
    rect(0, 0, 1024, 1024, { fill: os })
    group({ move: [-d, -d] }, () => dibujar(base))
  })
}

// rectángulo redondeado como trazado (para que ink siga el redondeo)
const rr = (x, y, w, h, r) => `M ${x + r} ${y} H ${x + w - r} A ${r} ${r} 0 0 1 ${x + w} ${y + r} V ${y + h - r} A ${r} ${r} 0 0 1 ${x + w - r} ${y + h} H ${x + r} A ${r} ${r} 0 0 1 ${x} ${y + h - r} V ${y + r} A ${r} ${r} 0 0 1 ${x + r} ${y} Z`
const cuerpoForma = (st) => path(rr(...CUERPO), st)
const finForma = (side, st) => blob(finPts(side), st)
const legPts = (p) => [p.j, [lerp(p.j[0], p.f[0], 0.5), lerp(p.j[1], p.f[1], 0.5)], p.f]

// ---------- esqueleto ----------
rig({ origin: [C, 512] })
bone("cuerpo", { at: [C, 512] })
bone("aleron_izq", { parent: "cuerpo", at: [380, 510] })
bone("aleron_der", { parent: "cuerpo", at: [644, 510] })
bone("antena", { parent: "cuerpo", at: ANT.j })
for (const p of PATAS) bone(p.n, { parent: "cuerpo", at: p.j })

// ---------- pasada 1: borde blanco de pegatina de todas las piezas ----------
layer("borde", () => {
  const B = { fill: P.blanco, stroke: P.blanco, width: BORDE * 2 + 12, join: "round", shadow: undefined }
  for (const p of PATAS) part(p.n, () => {
    curve(legPts(p), { stroke: P.blanco, width: 64 + BORDE * 2, cap: "round", shadow: B.shadow })
    circle(p.f[0], p.f[1], 40 + 6 + BORDE, { fill: P.blanco, shadow: B.shadow })
  })
  for (const s of [-1, 1]) part(s < 0 ? "aleron_izq" : "aleron_der", () => finForma(s, B))
  part("antena", () => {
    curve([ANT.j, ANT.top], { stroke: P.blanco, width: 26 + BORDE * 2, cap: "round", shadow: B.shadow })
    circle(ANT.top[0], ANT.top[1] - 6, 34 + 6 + BORDE, { fill: P.blanco, shadow: B.shadow })
  })
  part("cuerpo", () => cuerpoForma(B))
})

// ---------- pasada 2: color, sombra, tinta y detalle ----------
layer("patas", () => {
  for (const p of PATAS) part(p.n, () => {
    const pts = legPts(p)
    const [dx, dy] = [p.f[0] - p.j[0], p.f[1] - p.j[1]]
    const ang = Math.atan2(dy, dx) * 180 / Math.PI
    // brazo de la pata: cápsula con tinta, sombra de un tono abajo a la derecha
    curve(pts, { stroke: P.tinta, width: 64, cap: "round" })
    curve(pts, { stroke: P.metalS, width: 42, cap: "round" })
    curve(shift(pts, -4, -5), { stroke: P.metal, width: 30, cap: "round" })
    brush(shift(pts, -9, -9), { width: 7, color: "#c9d3ea", taper: [0.2, 0.3] })
    // pie de anclaje: garra redonda naranja con aro teal
    const pie = circle(p.f[0], p.f[1], 40, { fill: P.tealS })
    sombra(pie, (c) => circle(p.f[0], p.f[1], 40, { fill: c }), P.teal, P.tealS, 14)
    circle(p.f[0], p.f[1], 18, { fill: P.naranjaS })
    circle(p.f[0] - 3, p.f[1] - 3, 13, { fill: P.naranja })
    ink(pie, INK)
    circle(p.f[0] - 15, p.f[1] - 17, 6, { fill: "#ffffff" })
  })
})
layer("alerones", () => {
  for (const s of [-1, 1]) part(s < 0 ? "aleron_izq" : "aleron_der", () => {
    const X = (x) => sx(s, x)
    const f = finForma(s, { fill: P.casco })
    // el alerón derecho queda en el lado de sombra: algo más de sombra
    sombra(f, (c) => finForma(s, { fill: c }), P.casco, P.cascoS, s < 0 ? 22 : 34)
    clip(f, () => {
      // punta naranja
      blob([[X(200), 470], [X(110), 540], [X(130), 640], [X(205), 640], [X(185), 560]], { fill: P.naranjaS })
      blob([[X(196), 462], [X(104), 530], [X(118), 610], [X(186), 600], [X(176), 540]], { fill: P.naranja })
      // galón azul verdoso
      poly([[X(300), 430], [X(250), 430], [X(250), 610], [X(300), 610]], { fill: P.teal })
      poly([[X(300), 560], [X(250), 560], [X(250), 610], [X(300), 610]], { fill: P.tealS })
      // línea de panel
      brush([[X(372), 520], [X(300), 522]], { width: 5, color: alpha(P.tinta, 0.5), taper: [0, 0.3] })
    })
    ink(f, INK)
    // brillo marcado del canto superior
    brush([[X(352), 436], [X(290), 442], [X(222), 466]], { width: [3, 10, 3], color: "#ffffff" })
    brush([[X(178), 492], [X(150), 520]], { width: [3, 8, 2], color: "#ffffff" })
  })
})
layer("antena", () => {
  part("antena", () => {
    curve([ANT.j, ANT.top], { stroke: P.tinta, width: 26, cap: "round" })
    curve([ANT.j, ANT.top], { stroke: P.metal, width: 12, cap: "round" })
    line(507, 300, 507, 190, { stroke: "#d4dcf0", width: 4, cap: "round" })
    // anillo de la mitad del mástil
    ellipse(512, 236, 17, 7, { fill: P.teal, stroke: P.tinta, width: 5 })
    // baliza naranja con halo
    const [bx, by] = [ANT.top[0], ANT.top[1] - 6]
    circle(bx, by, 52, { fill: alpha(P.naranja, 0.35), blur: 14 })
    const b = circle(bx, by, 34, { fill: P.naranja })
    sombra(b, (c) => circle(bx, by, 34, { fill: c }), P.naranja, P.naranjaS, 12)
    circle(bx - 4, by - 4, 15, { fill: "#ffd27a" })
    ink(b, INK)
    circle(bx - 13, by - 14, 8, { fill: "#ffffff" })
  })
})
layer("cuerpo", () => {
  part("cuerpo", () => {
    const body = cuerpoForma({ fill: P.casco })
    sombra(body, (c) => cuerpoForma({ fill: c }), P.casco, P.cascoS, 40)
    clip(body, () => {
      // galones azul verdoso: chevrón arriba y banda abajo
      blob([[347, 330, 0], [512, 392, 0], [677, 330, 0], [677, 372, 0], [512, 434, 0], [347, 372, 0]], { fill: P.teal })
      blob([[600, 362, 0], [677, 334, 0], [677, 372, 0], [600, 400, 0]], { fill: P.tealS })
      rect(340, 652, 344, 34, { fill: P.teal })
      rect(586, 652, 100, 34, { fill: P.tealS })
      // costura central
      line(512, 292, 512, 400, { stroke: alpha(P.tinta, 0.45), width: 5 })
      line(512, 628, 512, 740, { stroke: alpha(P.tinta, 0.45), width: 5 })
    })
    cuerpoForma({ fill: "none", stroke: P.tinta, width: 13 })
    // contorno más grueso en el lado de sombra (abajo a la derecha)
    brush([[677, 420], [677, 572], [664, 633], [630, 685], [578, 719], [517, 732], [440, 732]], { width: [6, 14, 18, 14, 4], color: P.tinta, taper: [0.15, 0.4] })

    // visor de cristal cian con la carga brillando
    const vis = path(rr(432, 430, 160, 196, 74), { fill: P.cianS })
    clip(vis, () => {
      rect(0, 0, 1024, 1024, { fill: radial(500, 510, 120, [P.cian, "#22b6e8", P.cianS]) })
      // carga: núcleo de energía con estrella
      circle(512, 528, 58, { fill: alpha(P.carga, 0.55), blur: 16 })
      const star = repeat_radial(512, 528, 4, (i) => blob([[512, 470, 0], [526, 528], [512, 586, 0], [498, 528]], { fill: "#ffffff", rotate: i * 45 }))
      circle(512, 528, 22, { fill: P.carga })
      circle(512, 528, 11, { fill: "#ffffff" })
      // reflejos del cristal
      brush([[454, 560], [452, 500], [470, 456]], { width: [4, 14, 6], color: alpha("#ffffff", 0.9), taper: [0.2, 0.3] })
      circle(480, 452, 7, { fill: "#ffffff" })
      rect(432, 596, 160, 40, { fill: alpha(P.cianS, 0.55) })
    })
    path(rr(432, 430, 160, 196, 74), { fill: "none", stroke: P.tinta, width: 11 })
    // marco naranja del visor (tornillos)
    for (const [x, y] of [[452, 418], [572, 418], [452, 638], [572, 638]]) {
      circle(x, y, 9, { fill: P.naranja, stroke: P.tinta, width: 4 })
    }
    // luces naranjas de popa
    for (const x of [462, 562]) {
      rect(x - 18, 700, 36, 16, { r: 8, fill: P.naranja, stroke: P.tinta, width: 5 })
    }
    // brillos marcados del casco
    brush([[372, 560], [366, 470], [384, 380]], { width: [3, 13, 4], color: "#ffffff", taper: [0.25, 0.35] })
    brush([[404, 340], [440, 318], [478, 308]], { width: [3, 10, 3], color: "#ffffff" })
  })
})

// ---------- acciones ----------
// flotar: alerones que respiran, patas desfasadas, antena de lado a lado y un leve vaivén
const TAU = Math.PI * 2
action("flotar", { duration: 1.4, loop: true }, range(9).map((i) => {
  const t = i / 8, a = TAU * t
  const pata = (fase, lado) => lado * 6 * Math.sin(a + fase)
  return [t, {
    cuerpo: { rot: 0, x: 0, y: -6 * Math.sin(a) },
    aleron_izq: 8 * Math.sin(a),
    aleron_der: -8 * Math.sin(a),
    antena: 11 * Math.sin(a + Math.PI / 2),
    pata_sup_izq: pata(0.6, 1),
    pata_sup_der: pata(0.6 + Math.PI / 2, -1),
    pata_inf_izq: pata(0.6 + Math.PI, -1),
    pata_inf_der: pata(0.6 + 1.5 * Math.PI, 1),
  }]
}))

// golpe: todo se encoge hacia el centro, vibra y vuelve
const encoge = (k) => ({
  aleron_izq: { rot: -6 * k, x: 22 * k, y: 0 },
  aleron_der: { rot: 6 * k, x: -22 * k, y: 0 },
  antena: { rot: 0, x: 0, y: 22 * k },
  pata_sup_izq: { rot: 0, x: 16 * k, y: 16 * k },
  pata_sup_der: { rot: 0, x: -16 * k, y: 16 * k },
  pata_inf_izq: { rot: 0, x: 16 * k, y: -16 * k },
  pata_inf_der: { rot: 0, x: -16 * k, y: -16 * k },
})
action("golpe", { duration: 0.3, loop: false }, [
  [0, { cuerpo: { rot: 0, x: 0, y: 0 }, ...encoge(0) }],
  [0.15, { cuerpo: { rot: 4, x: 10, y: -4 }, ...encoge(1) }],
  [0.35, { cuerpo: { rot: -4, x: -9, y: 2 }, ...encoge(0.9) }],
  [0.55, { cuerpo: { rot: 3, x: 6, y: 0 }, ...encoge(0.5) }],
  [0.75, { cuerpo: { rot: -1.5, x: -3, y: 0 }, ...encoge(0.15) }],
  [1, { cuerpo: { rot: 0, x: 0, y: 0 }, ...encoge(0) }],
])
