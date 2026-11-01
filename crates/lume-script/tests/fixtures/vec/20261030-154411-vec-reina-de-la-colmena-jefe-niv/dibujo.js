// Reina de la Colmena: jefe del nivel 1. Vista cenital, mira hacia abajo.
canvas({ width: 1024, height: 768, background: null })
setLight(-1, -1)

const P = {
  lima: "#a6e22e",     // placas iluminadas
  verde: "#5fae24",    // tono medio
  oscuro: "#2f6b1f",   // placas en sombra
  tinta: "#14310f",    // contorno
  ambar: "#ffb21f",    // detalles
  ambarO: "#c46a0a",
  rojo: "#ff2b3d",     // visor y núcleo
  rojoO: "#8c0c1f",
  ala: "#bfe6ff",      // membrana translúcida
  metal: "#3a4740",
  blanco: "#ffffff",
}
const CX = 512

// ---------- geometría (lado izquierdo; se refleja con mirror) ----------
const G = {
  alaSup: [[455, 300], [330, 165, 0.5], [160, 62, 0], [58, 88, 0.5], [66, 170], [190, 320], [350, 418], [455, 410]],
  alaSupNervio: [[455, 318], [330, 190], [175, 96], [118, 112], [210, 230], [360, 360], [455, 372]],
  alaInf: [[445, 425], [300, 440], [150, 478, 0], [118, 528, 0.5], [210, 548], [350, 528], [445, 500]],
  alaInfNervio: [[445, 440], [310, 455], [190, 490], [230, 515], [350, 505], [445, 485]],
  mandibula: [[440, 556], [352, 538], [276, 584], [250, 652], [262, 712], [330, 744, 0], [336, 696], [352, 648], [384, 624], [430, 618]],
}
const abdomen = ring(CX, 190, 118, 152, 12)
const torax = [[CX, 272], [610, 282], [688, 330], [700, 410], [660, 492], [580, 528], [CX, 532], [444, 528], [364, 492], [324, 410], [336, 330], [414, 282]]
const cabeza = [[CX, 500], [600, 506], [668, 540], [690, 600], [652, 662], [580, 694], [CX, 702], [444, 694], [372, 662], [334, 600], [356, 540], [424, 506]]

// Piezas de la silueta completa (para el borde blanco de pegatina)
function silueta(st) {
  mirror(CX, () => {
    blob(G.alaSup, st); blob(G.alaInf, st); blob(G.mandibula, st)
  })
  blob(abdomen, st); blob(torax, st); blob(cabeza, st)
}

layer("borde-pegatina", () => {
  silueta({ fill: P.blanco, stroke: P.blanco, width: 34, join: "round" })
})

layer("alas", () => {
  mirror(CX, () => {
    for (const [forma, nervio] of [[G.alaInf, G.alaInfNervio], [G.alaSup, G.alaSupNervio]]) {
      // membrana: base blanca + azul translúcido
      blob(forma, { fill: "#eef8ff", stroke: P.tinta, width: 10, join: "round" })
      const m = blob(forma, { fill: linear(80, 100, 450, 400, [P.ala, "#8fc8f0"]), opacity: 0.85 })
      clip(m, () => {
        // brillo en el borde de ataque
        brush(shift(forma.slice(1, 4).map(p => [p[0], p[1]]), 6, 14), { width: 14, color: alpha(P.blanco, 0.8), taper: [0.3, 0.5] })
      })
      // armazón rígido verde
      const n = blob(nervio, { fill: P.verde, stroke: P.tinta, width: 8, join: "round" })
      clip(n, () => {
        blob(shift(nervio, 10, 16), { fill: P.oscuro })
        blob(shift(grow(nervio, 0.75), -8, -10), { fill: P.lima })
      })
    }
    // luces ámbar en las puntas
    circle(150, 112, 13, { fill: P.ambar, stroke: P.tinta, width: 6 })
    circle(146, 108, 4, { fill: P.blanco })
  })
})

layer("abdomen", () => {
  const a = blob(abdomen, { fill: P.oscuro, stroke: P.tinta, width: 12, join: "round" })
  clip(a, () => {
    // placas superpuestas, de la punta hacia el tórax
    const ys = [72, 132, 194, 258]
    ys.forEach((y, i) => {
      const w = 70 + i * 16
      const placa = blob([[CX, y - 34], [CX + w, y - 18], [CX + w + 20, y + 30], [CX, y + 46], [CX - w - 20, y + 30], [CX - w, y - 18]],
        { fill: P.verde, stroke: P.tinta, width: 8, join: "round" })
      clip(placa, () => {
        ellipse(CX - w * 0.35, y - 14, w * 0.75, 26, { fill: P.lima })
        ellipse(CX + w * 0.2, y + 52, w * 1.3, 30, { fill: P.oscuro })
      })
      circle(CX, y + 14, 9, { fill: P.ambar, stroke: P.tinta, width: 5 })
    })
  })
})

layer("torax", () => {
  blob(torax, { fill: "#1d3a14", stroke: P.tinta, width: 12, join: "round" })
  // núcleo rojo (punto débil)
  circle(CX, 400, 74, { fill: P.rojoO, glow: { blur: 30, color: alpha(P.rojo, 0.9) } })
  const nucleo = circle(CX, 400, 60, { fill: radial(CX - 18, 380, 70, ["#ffd0c8", P.rojo, P.rojoO]), stroke: P.tinta, width: 8 })
  clip(nucleo, () => {
    circle(CX + 22, 422, 46, { fill: P.rojoO, opacity: 0.45, blend: "multiply" })
  })
  ellipse(CX - 22, 376, 18, 12, { fill: P.blanco, rotate: -30 })
  circle(CX + 18, 368, 5, { fill: P.blanco })
  // placas entreabiertas a cada lado del núcleo
  mirror(CX, () => {
    const pl = [[498, 284], [414, 284], [338, 330], [326, 412], [366, 492], [450, 524], [496, 520], [470, 470], [448, 400], [462, 334]]
    const p = blob(pl, { fill: P.verde, stroke: P.tinta, width: 10, join: "round" })
    clip(p, () => {
      blob([[330, 270], [480, 270], [430, 330], [380, 420], [320, 430]], { fill: P.lima })
      blob([[360, 470], [440, 430], [520, 470], [520, 540], [340, 540]], { fill: P.oscuro })
      brush([[350, 340], [400, 300], [460, 292]], { width: 8, color: alpha(P.blanco, 0.85) })
    })
    // junta entre placas
    brush([[350, 420], [400, 432], [446, 440]], { width: 7, color: P.tinta, taper: [0.2, 0.4] })
    circle(372, 380, 10, { fill: P.ambar, stroke: P.tinta, width: 5 })
  })
})

layer("mandibulas", () => {
  mirror(CX, () => {
    // cañón
    // mandíbula
    const m = blob(G.mandibula, { fill: P.verde, stroke: P.tinta, width: 12, join: "round" })
    clip(m, () => {
      blob([[310, 660], [360, 640], [440, 600], [440, 760], [300, 760]], { fill: P.oscuro })
      blob([[240, 560], [360, 530], [330, 570], [280, 620], [240, 660]], { fill: P.lima })
    })
    // cañón apoyado sobre la mandíbula, apuntando abajo
    const c = rect(276, 612, 40, 118, { fill: P.metal, stroke: P.tinta, width: 9, r: 12, shadow: { x: 6, y: 6, blur: 0, color: alpha(P.tinta, 0.5) } })
    clip(c, () => {
      rect(276, 612, 13, 118, { fill: "#7d9084" })
      rect(270, 650, 52, 12, { fill: P.ambar })
      rect(270, 690, 52, 12, { fill: P.ambar })
      rect(270, 658, 52, 4, { fill: P.ambarO })
      rect(270, 698, 52, 4, { fill: P.ambarO })
    })
    ellipse(296, 726, 12, 5, { fill: "#0b1408" })
    circle(296, 630, 8, { fill: P.rojo, stroke: P.tinta, width: 4 })
  })
})

layer("cabeza", () => {
  const h = blob(cabeza, { fill: P.verde, stroke: P.tinta, width: 12, join: "round" })
  clip(h, () => {
    blob([[330, 490], [700, 490], [680, 560], [512, 545], [340, 570]], { fill: P.lima })
    blob([[330, 650], [512, 640], [700, 640], [700, 720], [330, 720]], { fill: P.oscuro })
    brush([[400, 530], [470, 512], [540, 512]], { width: 9, color: alpha(P.blanco, 0.85) })
  })
  // visor rojo largo
  rect(392, 576, 240, 52, { fill: P.tinta, r: 26 })
  const v = rect(400, 583, 224, 38, { fill: linear(0, 583, 0, 621, ["#ff8a8a", P.rojo, P.rojoO]), r: 19, glow: { blur: 14, color: alpha(P.rojo, 0.7) } })
  clip(v, () => {
    rect(410, 588, 150, 9, { fill: alpha(P.blanco, 0.85), r: 4 })
  })
  circle(612, 594, 4, { fill: P.blanco })
  // antenas cortas ámbar
  mirror(CX, () => {
    circle(452, 666, 9, { fill: P.ambar, stroke: P.tinta, width: 5 })
  })
})
