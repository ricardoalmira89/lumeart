// dibujo.js: tortuga ninja sentada en un muro bajo un cielo azul (calco de referencia-1.png).
// Lienzo de 720x1280: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 720, height: 1280, background: linear(0, 0, 0, 1040, ["#0679a6", [0.55, "#0d82ae"], "#2391bb"]) })

const P = {
  cieloA: "#0679a6", cieloB: "#2391bb",
  tinta: "#1b140d",
  verdeO: "#3f5a2c", verde: "#5f8a3a", verdeC: "#8db357",
  ocre: "#c9a86a", ocreO: "#9b7b49",
  cuero: "#6b4a34", cueroO: "#4a3122",
  rojo: "#c8373a", rojoO: "#8f2427",
  ladrillo: "#d9805c", crema: "#f6e2b4", cremaO: "#d9bf8c",
  sombra: "#5f7380",
}

layer("cielo", () => {
  // degradado del fondo + una veladura suave cerca del horizonte
  rect(0, 700, 720, 340, { fill: linear(0, 700, 0, 1040, [alpha("#3aa2c8", 0), alpha("#3aa2c8", 0.35)]) })
})

layer("nubes", () => {
  // cúmulos de cómic: base crema, sombra azulada abajo a la derecha, luz arriba a la izquierda y tinta
  setLight(-1, -1)
  seed(5)
  // cada nube: base plana redondeada + bultos redondos encima (de mayor a menor hacia los lados)
  const nube = (cx, yb, w, h, n) => {
    const piezas = [{ x: cx - w / 2, y: yb - h * 0.32, w, h: h * 0.32, r: h * 0.16 }]
    const bultos = []
    for (let k = 0; k < n; k++) {
      const t = (k + 0.5) / n
      const r = h * (0.32 + 0.38 * Math.sin(Math.PI * t)) * rand(0.85, 1.1)
      bultos.push({ x: cx - w / 2 + w * t + rand(-4, 4), y: yb - h * 0.3 - r * 0.55, r })
    }
    const formas = () => [
      ...piezas.map((p) => rect(p.x, p.y, p.w, p.h, { r: p.r, hidden: true })),
      ...bultos.map((b) => circle(b.x, b.y, b.r, { hidden: true })),
    ]
    outline(formas(), 2.2, "#1f4258")
    const sil = formas()
    clip(sil, () => {
      rect(cx - w / 2 - 50, yb - h * 2, w + 100, h * 2.2, { fill: "#9fc0d4" })                  // sombra
      for (const b of bultos) circle(b.x - b.r * 0.2, b.y - b.r * 0.2, b.r * 0.9, { fill: "#f1ece0" })  // luz
      for (const b of bultos) circle(b.x - b.r * 0.38, b.y - b.r * 0.42, b.r * 0.4, { fill: "#fffbef", opacity: 0.55 }) // brillo
      rect(cx - w / 2 - 10, yb - h * 0.12, w + 20, h * 0.2, { fill: "#8fb3c9" })                // panza
    })
    // rizos de tinta donde un bulto se monta sobre otro
    for (let k = 1; k < n; k++) {
      const b = bultos[k], a = bultos[k - 1]
      const lado = b.r < a.r ? b : a
      const s = lado === b ? -1 : 1
      brush(arc(lado.x, lado.y, lado.r * 0.95, 180 + s * 30, 180 + s * 75, 5),
        { width: 1.7, color: alpha("#1f4258", 0.75), taper: [0.4, 0.5] })
    }
  }
  nube(165, 300, 280, 95, 5)
  nube(555, 420, 300, 105, 5)
  nube(235, 412, 130, 40, 4)
})

layer("ciudad", () => {
  // skyline en tres niveles de profundidad, visto desde la azotea (horizonte oculto tras el muro)
  const cam = camera({ eye: [0, 20, 0], look: [0, 20, 100], fov: 50, center: [360, 990], light: [-0.6, 0.8, -0.5] })
  seed(11)
  const velo = (op) => rect(0, 420, 720, 630, { fill: linear(0, 300, 0, 1040, [alpha(P.cieloA, op * 0.7), alpha(P.cieloB, op)]) })

  const fila = (z, ancho, alto, colores, edge, detalle) => {
    const edificios = []
    let x = -z * 0.56
    while (x < z * 0.56) {
      const w = rand(ancho[0], ancho[1])
      edificios.push({ x, w, h: rand(alto[0], alto[1]), d: rand(15, 30), z: z + rand(-12, 12), c: pick(colores) })
      x += w + rand(-2, 4)
    }
    // de fuera hacia el centro, para que los laterales se tapen bien
    edificios.sort((a, b) => Math.abs(b.x + b.w / 2) - Math.abs(a.x + a.w / 2))
    for (const e of edificios) {
      const b = box3d(cam, { at: [e.x, 0, e.z], size: [e.w, e.h, e.d], color: e.c, edge })
      if (detalle) detalle(b, e)
    }
  }

  // 1 · al fondo: torres altas y pálidas, casi fundidas con el cielo
  fila(520, [22, 46], [150, 390], ["#8cc0d8", "#97c6dc", "#80b6d0"], null, (b, e) => {
    if (rand() < 0.35) cam.line([e.x + e.w / 2, e.h, e.z], [e.x + e.w / 2, e.h + rand(20, 45), e.z], { stroke: "#86bad3", width: 1.2 }) // antenas
  })
  velo(0.32)

  // 2 · plano medio: bloques azulados con bandas de ventanas
  fila(300, [18, 36], [70, 170], ["#5f93b2", "#6b9cba", "#5689a8"], { color: "#2f5c78", width: 0.7 }, (b, e) => {
    if (!b.front.visible) return
    for (let v = 6; v < e.h - 4; v += 5) b.front.quad(1.5, v, e.w - 3, 2, { fill: alpha("#244d68", 0.55) })
  })
  velo(0.16)

  // 3 · primer plano: edificios bajos y cálidos de piedra y ladrillo, con ventanas, cornisas y depósitos de agua
  fila(160, [14, 26], [26, 58], ["#d2bb94", "#c98f6e", "#b98a70", "#a8a493"], { color: "#3a3a3a", width: 1.1 }, (b, e) => {
    const caras = [b.front, b.left, b.right].filter((f) => f.visible)
    for (const f of caras) {
      const ancho = f === b.front ? e.w : e.d
      const cols = Math.max(2, Math.floor(ancho / 3.4))
      const paso = ancho / cols
      for (let v = 8; v < e.h - 5; v += 4.2) {
        for (let i = 0; i < cols; i++) {
          f.quad(i * paso + paso * 0.28, v, paso * 0.44, 2.2, { fill: rand() < 0.15 ? "#9cc6d6" : "#3e5865", opacity: 0.85 })
        }
      }
      f.quad(0, e.h - 2.2, ancho, 1.2, { fill: alpha("#2a2a2a", 0.35) })   // cornisa
    }
    if (rand() < 0.4) {
      const tx = e.x + e.w * rand(0.3, 0.7), tz = e.z + e.d * 0.5
      for (const dx of [-1.6, 1.6]) cam.line([tx + dx, e.h, tz - 1], [tx + dx, e.h + 3, tz - 1], { stroke: "#3a2a1d", width: 0.9 })
      cylinder3d(cam, { at: [tx, e.h + 3, tz], r: 2.2, h: 4, rTop: 2.3, color: "#8a6a4c", edge: { color: "#3a2a1d", width: 0.8 } })
      cylinder3d(cam, { at: [tx, e.h + 7, tz], r: 2.4, h: 1.8, rTop: 0, color: "#6b4a34", edge: { color: "#3a2a1d", width: 0.8 } })
    }
  })
})

layer("muro", () => {
  // base crema del enlucido para tapar el calco
  rect(0, 1080, 720, 200, { fill: P.crema })
})

let silueta
const AZULES = ["#077cab", "#1c8ab4", "#1685b2", "#0f81ae", "#0779a5", "#228eb8", "#4e8ca7"]
layer("calco", () => {
  // regiones de color de la referencia, sin los azules del cielo (muro y escena)
  trace({
    colors: 28, detail: 0.9, drop: AZULES,
    palette: [P.tinta, "#2e261c", "#4a4236", "#6b4a34", "#8a6a4c", P.verdeO, P.verde, P.verdeC,
      P.ocre, P.ocreO, "#e3c48a", P.rojo, P.rojoO, P.ladrillo, P.crema, P.cremaO, "#b9a582", P.sombra, "#8b9592"],
  })
  // la tortuga, recalcada con una paleta sin azules para que la piel salga verde
  silueta = blob([[465, 702], [505, 718], [516, 768], [560, 798], [586, 858], [594, 908], [612, 950], [610, 995],
    [592, 1012], [560, 1032], [549, 1095], [550, 1174], [492, 1176], [507, 1116], [497, 1076], [466, 1034], [434, 1110], [456, 1172],
    [400, 1172], [398, 1150], [404, 1100], [364, 1032], [333, 1000], [336, 958], [358, 928], [364, 878], [380, 818], [418, 790], [424, 740]],
    { hidden: true })
  clip(silueta, () => {
    trace({
      colors: 28, detail: 0.9, drop: AZULES,
      palette: [P.tinta, "#2a2016", "#3b2a1d", P.cueroO, P.cuero, "#2c3f1f", P.verdeO, "#4d7230", P.verde, "#7a9e48", P.verdeC, "#8fae5a",
        P.ocre, "#8a6a4c", "#e0c58e", P.rojo, P.rojoO, P.ladrillo, P.crema],
    })
  })
})

layer("tinta", () => {
  trace({ mode: "lines", threshold: 60, ink: P.tinta })
})

layer("retoques", () => {
  // piel de cabeza y hombros recalcada sin ocres, para que las luces salgan verdes
  const piel = [
    ellipse(466, 752, 47, 53, { hidden: true }),
    blob([[338, 880], [366, 822], [400, 806], [428, 812], [428, 880], [400, 896], [360, 900]], { hidden: true }),
    blob([[518, 812], [540, 806], [570, 822], [592, 880], [578, 900], [540, 896], [516, 880]], { hidden: true }),
    poly([[424, 798], [508, 798], [502, 816], [430, 816]], { hidden: true }),                       // cuello
    blob([[370, 897], [425, 882], [430, 928], [418, 942], [384, 934]], { hidden: true }),           // antebrazo izq.
    blob([[522, 882], [566, 897], [552, 934], [524, 942], [520, 928]], { hidden: true }),           // antebrazo der.
    blob([[428, 952], [458, 945], [470, 995], [444, 1008], [424, 985]], { hidden: true }),          // mano izq.
    blob([[478, 945], [510, 952], [516, 985], [496, 1008], [474, 995]], { hidden: true }),          // mano der.
  ]
  clip(silueta, () => clip(piel, () => {
    trace({
      colors: 28, detail: 0.9, drop: AZULES,
      palette: [P.tinta, "#2a2016", "#3b2a1d", P.cueroO, "#2c3f1f", P.verdeO, "#4d7230", P.verde, "#7a9e48",
        P.verdeC, "#a9c873", P.rojo, P.rojoO, "#f4ecd2"],
    })
  }))
})

layer("saludo", () => {
  // el antebrazo izquierdo (del espectador) ya no descansa en el muslo: se tapa lo que había y se levanta saludando
  const linea = (pts, w = 2.2) => brush(pts, { width: w, color: P.tinta, taper: [0.25, 0.3] })

  // 1 · lo que quedaba detrás del brazo: costado, cinturón, taparrabos y muslo
  const hueco = blob([[372, 893, 0], [412, 903], [415, 933, 0], [452, 933], [471, 944], [487, 996], [483, 1019, 0], [425, 1019, 0],
    [404, 990], [400, 955], [380, 948], [370, 925]], { fill: P.verde })
  clip(hueco, () => {
    rect(360, 885, 70, 50, { fill: "#7a9e48" })                                                         // costado
    const muslo = poly([[395, 948], [480, 948], [492, 1022], [395, 1022]], { fill: P.verde })
    blob([[448, 975], [472, 960], [492, 1022], [462, 1022]], { fill: "#4d7230" })                     // sombra hacia la entrepierna
    hatch(muslo, { angle: -35, gap: 5, width: 0.8, color: alpha(P.tinta, 0.3), wobble: 0.6 })
    poly([[395, 946], [472, 946], [470, 960], [398, 958]], { fill: "#2c3f1f", opacity: 0.7 })          // sombra del cinturón
    poly([[365, 922], [470, 926], [472, 948], [368, 946]], { fill: P.cuero })                          // cinturón
    poly([[365, 922], [470, 926], [470, 931], [366, 928]], { fill: "#8a6a4c" })
    blob([[458, 940], [482, 940], [488, 1000], [470, 1012], [460, 990]], { fill: P.cuero })            // taparrabos
    brush([[466, 948], [470, 975], [472, 1002]], { width: 1.4, color: P.cueroO })
    brush([[476, 946], [481, 972], [482, 996]], { width: 1.2, color: P.cueroO })
  })
  linea([[380, 922], [420, 924], [470, 926]], 2)
  linea([[384, 946], [420, 947], [468, 948]], 2.4)
  linea([[458, 942], [462, 980], [470, 1010]], 1.8)
  linea([[412, 968], [438, 982], [458, 1000]], 1.4)                                                  // pliegue del muslo

  // 2 · antebrazo levantado desde el codo (todo el brazo se desplaza para nacer del bíceps)
  group({ move: [28, 0] }, () => {
  const ante = blob([[326, 918], [374, 910], [366, 878], [350, 846], [340, 834], [304, 838], [306, 862], [318, 890]], { fill: P.verde })
  cel(ante, { color: P.verdeO, highlight: true })
  ink(ante, { width: 1.8, weight: 2.2, color: P.tinta })
  linea([[338, 900], [330, 872], [324, 850]], 1.3)                                                   // músculo

  // vendas de la muñeca
  const venda = blob([[300, 846], [344, 840, 0], [342, 820, 0], [300, 826, 0]], { fill: "#8a6a4c" })
  cel(venda, { color: P.cuero })
  ink(venda, { width: 1.5, weight: 2, color: P.tinta })
  for (const y of [828, 835, 842]) brush([[302, y + 1], [322, y - 1], [342, y - 4]], { width: 1.1, color: P.cueroO })

  // 3 · mano abierta: palma a la cámara, tres dedos y el pulgar hacia la cabeza
  const dedo = (bx, by, tx, ty, w) => {
    const dx = tx - bx, dy = ty - by, l = Math.hypot(dx, dy), nx = -dy / l * w / 2, ny = dx / l * w / 2
    return blob([[bx + nx, by + ny], [tx + nx * 0.9, ty + ny * 0.9], [tx + dx / l * w * 0.45, ty + dy / l * w * 0.45],
      [tx - nx * 0.9, ty - ny * 0.9], [bx - nx, by - ny]], { fill: P.verde })
  }
  const dedos = [
    dedo(302, 796, 292, 762, 17),
    dedo(319, 792, 318, 756, 18),
    dedo(336, 796, 345, 763, 17),
    dedo(340, 812, 362, 795, 15),
  ]
  const palma = blob([[298, 826], [342, 822], [350, 802], [338, 786], [302, 788], [290, 806]], { fill: P.verde })
  for (const f of [...dedos, palma]) cel(f, { color: P.verdeO, highlight: true })
  clip(palma, () => ellipse(316, 806, 15, 12, { fill: "#8fae5a" }))                                 // hueco de la palma, más claro
  for (const f of [...dedos, palma]) ink(f, { width: 1.6, weight: 2.2, color: P.tinta })
  // pliegues de la palma y de los nudillos
  linea([[300, 800], [316, 806], [334, 801]], 1.2)
  linea([[340, 812], [328, 818], [312, 820]], 1.1)
  for (const [x, y] of [[296, 776], [318, 772], [341, 777]]) linea([[x - 4, y], [x + 4, y - 1]], 1)

  // codera encima de la articulación para que el giro quede limpio
  const codera = blob([[322, 900], [352, 892], [376, 902], [372, 930], [346, 938], [324, 926]], { fill: P.cuero })
  cel(codera, { color: P.cueroO, highlight: true })
  ink(codera, { width: 1.8, weight: 2.2, color: P.tinta })
  linea([[330, 914], [350, 910], [370, 914]], 1.2)
  // rayado de sombra en el lado derecho, como el trazo del calco
  for (const f of [ante, palma]) clip(f, () => hatch(f, { angle: 60, gap: 4.5, width: 0.7, color: alpha(P.tinta, 0.18) }))

  // líneas de movimiento del saludo
  for (const [r, op] of [[54, 0.7], [66, 0.45]]) brush(arc(318, 800, r, -75, -40, 6), { width: 2, color: alpha("#f6e2b4", op), taper: [0.4, 0.4] })
  })
})

layer("acabado", () => {
  grain({ opacity: 0.06 })
})
