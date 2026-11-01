// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 1024x1024: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 1024, height: 1024, background: null }) // null = fondo transparente (objeto suelto)

// Energía: cristal dorado de cuatro puntas, luz arriba a la izquierda
const P = {
  ink: "#4a1e08",      // contorno oscuro (marrón cálido del propio dorado)
  white: "#ffffff",    // borde de pegatina y brillos
  f1: "#fffbe0",       // cara iluminada (casi blanca)
  f2: "#ffe866",       // luz
  f3: "#ffd21a",       // dorado medio
  f4: "#ffb400",       // dorado intenso
  f5: "#f08a00",       // sombra
  f6: "#c85a00",       // sombra profunda
  halo: "#ffcc22",
}
setLight(-1, -1)

const C = [512, 512]
const R = 410         // largo de las puntas
const W = 118         // cintura en diagonal
const T = { up: [512, 512 - R], right: [512 + R, 512], down: [512, 512 + R], left: [512 - R, 512] }
const Wd = { ul: [512 - W, 512 - W], ur: [512 + W, 512 - W], dr: [512 + W, 512 + W], dl: [512 - W, 512 + W] }
const star = [T.up, Wd.ur, T.right, Wd.dr, T.down, Wd.dl, T.left, Wd.ul]

layer("halo", () => {
  circle(512, 512, 330, { fill: P.halo, opacity: 0.45, blur: 70 })
  circle(512, 512, 200, { fill: "#fff0a0", opacity: 0.35, blur: 50 })
})

layer("pegatina", () => {
  // borde blanco grueso y contorno oscuro detrás de las facetas
  poly(star, { fill: P.white, stroke: P.white, width: 88, join: "round" })
  poly(star, { fill: P.ink, stroke: P.ink, width: 40, join: "round" })
})

layer("facetas", () => {
  const tri = (a, b, c, fill) => poly([a, b, c], { fill, stroke: fill, width: 1.5, join: "round" })
  // brazo de arriba
  tri(C, T.up, Wd.ul, P.f1)
  tri(C, T.up, Wd.ur, P.f2)
  // brazo izquierdo
  tri(C, T.left, Wd.ul, P.f2)
  tri(C, T.left, Wd.dl, P.f4)
  // brazo derecho
  tri(C, T.right, Wd.ur, P.f3)
  tri(C, T.right, Wd.dr, P.f5)
  // brazo de abajo
  tri(C, T.down, Wd.dl, P.f4)
  tri(C, T.down, Wd.dr, P.f6)
})

layer("aristas", () => {
  // aristas internas: finas en la luz, más marcadas en la sombra
  for (const [p, w] of [[T.up, 5], [T.left, 6], [T.right, 9], [T.down, 11]]) {
    brush([C, p], { width: [w * 1.4, w], taper: [0, 0.6], color: alpha(P.ink, 0.45) })
  }
  for (const [p, w] of [[Wd.ul, 4], [Wd.ur, 7], [Wd.dl, 7], [Wd.dr, 10]]) {
    brush([C, p], { width: [w, w * 0.8], taper: [0, 0.3], color: alpha(P.ink, 0.4) })
  }
  // contorno de tinta con peso hacia la sombra (abajo a la derecha)
  const s = poly(star, { fill: "none", hidden: true })
  ink(s, { width: 14, weight: 2.2, color: P.ink })
})

layer("brillos", () => {
  // franja de luz en la cara iluminada
  const face = poly([C, T.up, Wd.ul], { hidden: true })
  clip(face, () => {
    poly([[478, 170], [500, 150], [390, 420], [360, 430]], { fill: P.white, opacity: 0.9 })
  })
  const faceL = poly([C, T.left, Wd.ul], { hidden: true })
  clip(faceL, () => {
    poly([[170, 492], [150, 512], [420, 470], [430, 440]], { fill: P.white, opacity: 0.55 })
  })
  // destello de cuatro puntas en la esquina superior izquierda
  const sp = (x, y, r) => {
    poly([[x, y - r], [x + r * 0.2, y - r * 0.2], [x + r, y], [x + r * 0.2, y + r * 0.2], [x, y + r], [x - r * 0.2, y + r * 0.2], [x - r, y], [x - r * 0.2, y - r * 0.2]], { fill: P.white, stroke: P.f4, width: 8, behind: true, join: "round", glow: { blur: 12, color: P.halo } })
  }
  sp(300, 290, 70)
  circle(395, 395, 14, { fill: P.white })
  // reflejo de borde cálido en la sombra
  const faceDR = poly([C, T.down, Wd.dr], { hidden: true })
  clip(faceDR, () => {
    brush([[575, 760], [520, 880]], { width: 12, taper: [0.3, 0.3], color: alpha(P.f3, 0.8) })
  })
})
