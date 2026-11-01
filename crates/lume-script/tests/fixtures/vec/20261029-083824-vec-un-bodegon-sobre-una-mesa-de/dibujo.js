// dibujo.js: bodegón junto a una ventana con luz de tarde
canvas({ width: 1024, height: 768, background: linear(0, 0, 1024, 0, ["#4a362c", "#2a201e"]) })

const P = {
  wallDark: "#2a201e",   // pared en sombra
  wallLit: "#9a6440",    // pared tocada por la luz
  sun: "#ffd49a",        // luz cálida principal
  sky: "#f3c088",        // cielo de tarde
  cool: "#8fb2cf",       // contraluz fría
  wood: "#7a4528",       // madera media
  woodDark: "#3a2016",   // madera en sombra
  woodLit: "#c07a44",    // madera iluminada
  ceramic: "#c98e62",    // jarrón
  glass: "#3f8a4e",      // botella (verde vidrio)
  pear: "#c2b247",       // peras
  cup: "#e8ddcc",        // taza
  accent: "#ff9a4a",     // acento naranja
}
const LIGHT = { from: 165, height: 30 }

layer("pared", () => {
  rect(0, 0, 1024, 520, { fill: linear(0, 0, 1024, 300, ["#5a3f30", "#3a2a24", P.wallDark]) })
  // resplandor cálido alrededor de la ventana
  ellipse(260, 260, 380, 300, { fill: radial(260, 260, 380, [alpha(P.wallLit, 0.75), alpha(P.wallLit, 0)]), blur: 20 })
  // mancha de sol proyectada en la pared derecha
  poly([[620, 120], [860, 90], [880, 330], [640, 350]], { fill: alpha(P.sun, 0.18), blur: 26, blend: "screen" })
  // viñeta oscura en la esquina derecha
  rect(760, 0, 264, 520, { fill: linear(760, 0, 1024, 0, [alpha("#140e10", 0), alpha("#140e10", 0.6)]) })
  // textura de pared
  seed(3)
  for (let i = 0; i < 60; i++) {
    const x = rand(0, 1024), y = rand(0, 500)
    ellipse(x, y, rand(30, 80), rand(20, 50), { fill: alpha(pick(["#6a4a36", "#2a1c18"]), 0.08), blur: 18 })
  }
})

layer("cuchillos", () => {
  // barra magnética de nogal con juego de cuchillos colgados (hoja hacia abajo)
  const BAR = { x: 676, y: 146, w: 300, h: 20 }
  // sombra de la barra en la pared (luz desde arriba a la izquierda)
  rect(BAR.x + 8, BAR.y + 10, BAR.w, BAR.h, { fill: alpha("#100608", 0.6), blur: 6, blend: "multiply", r: 3 })
  function knife(x, w, len, hl, kind) {
    const y0 = BAR.y + 2          // arranque de la hoja (tras la virola)
    let pts
    if (kind === "cleaver") {
      pts = [[x, y0, 0], [x + w, y0, 0], [x + w, y0 + len, 0], [x + 2, y0 + len, 0]]
    } else if (kind === "santoku") {
      pts = [[x, y0, 0], [x + w, y0, 0], [x + w, y0 + len * 0.55], [x + w * 0.6, y0 + len * 0.92], [x, y0 + len, 0], [x, y0 + len * 0.5, 0.5]]
    } else {
      pts = [[x, y0, 0], [x + w, y0, 0], [x + w * 0.98, y0 + len * 0.6], [x + w * 0.55, y0 + len * 0.9], [x + 1, y0 + len, 0], [x, y0 + len * 0.5, 0.5]]
    }
    const hx = x + w / 2, hw = Math.min(w * 0.55, 16)
    // sombra de cuchillo y mango sobre la pared
    group({ move: [9, 12], opacity: 0.55, blur: 5 }, () => {
      blob(pts, { fill: "#100608" })
      rect(hx - hw / 2, y0 - hl - 10, hw, hl, { fill: "#100608", r: hw / 2 })
    })
    // hoja de acero: luz cálida a la izquierda, frío y oscuro a la derecha
    const blade = blob(pts, { fill: linear(x, 0, x + w, 0, ["#e8dccb", "#a9a7a8", "#5e6670"]) })
    clip(blade, () => {
      // bisel del filo y reflejo del cielo de la ventana
      rect(x + w * 0.72, y0, w * 0.4, len, { fill: alpha("#2a2a34", 0.45), blend: "multiply" })
      rect(x - 2, y0 + len * 0.15, w * 0.22, len * 0.6, { fill: alpha(P.sun, 0.55), blur: 4, blend: "screen" })
      rect(x, y0, w, len, { fill: linear(0, y0, 0, y0 + len, [alpha("#000000", 0), alpha("#1a0c14", 0.45)]), blend: "multiply" })
      if (kind === "santoku") for (let i = 0; i < 5; i++) ellipse(x + w * 0.62, y0 + 26 + i * 18, 3, 6, { fill: alpha("#3a3a44", 0.5) })
      if (kind === "bread") for (let y = y0 + 8; y < y0 + len * 0.85; y += 6) circle(x + w + 0.5, y, 2.2, { fill: "#4a4c56" })
      if (kind === "cleaver") circle(x + w * 0.74, y0 + 34, 5, { fill: "#2a1e1c" })
      // la pared se apaga hacia la derecha: el acero también
      rect(x, y0, w, len, { fill: alpha("#1a1018", clamp((x - 690) / 270, 0, 1) * 0.4), blend: "multiply" })
    })
    // filo brillante y contorno oscuro del lomo
    const edge = kind === "cleaver" ? [[x + w, y0 + 4], [x + w, y0 + len - 2]] : [[x + w, y0 + 4], [x + w * 0.98, y0 + len * 0.6], [x + w * 0.55, y0 + len * 0.9]]
    brush(edge, { width: 1.2, color: alpha("#f6efe2", 0.6), taper: [0, 0.4] })
    brush([[x, y0], [x, y0 + len * 0.5], [x + 1, y0 + len]], { width: [1.5, 1.2, 0.5], color: alpha("#2a2028", 0.7), taper: [0, 0.3] })
    // virola de acero
    rect(hx - hw / 2 - 2, y0 - 12, hw + 4, 13, { fill: linear(hx - hw, 0, hx + hw, 0, ["#efe4d0", "#9a9898", "#4e545e"]), r: 2 })
    // mango de madera oscura con remaches
    const handle = rect(hx - hw / 2, y0 - hl - 10, hw, hl, { fill: linear(hx - hw / 2, 0, hx + hw / 2, 0, ["#6a3a24", "#3a1e14", "#1e100c"]), r: hw / 2 })
    clip(handle, () => brush([[hx - hw * 0.25, y0 - hl - 4], [hx - hw * 0.28, y0 - 16]], { width: 2.5, color: alpha(P.sun, 0.45), taper: [0.3, 0.3] }))
    for (let i = 0; i < 3; i++) circle(hx, y0 - 22 - i * (hl - 30) / 2, 2.2, { fill: "#d8ccb8", stroke: "#3a2a24", width: 0.6 })
  }
  knife(690, 38, 196, 78, "chef")
  knife(742, 34, 150, 72, "santoku")
  knife(790, 22, 200, 76, "bread")
  knife(828, 22, 184, 74, "chef")
  knife(866, 16, 92, 58, "chef")
  knife(900, 58, 108, 76, "cleaver")
  // la barra va delante de las hojas: el imán las sujeta
  rect(BAR.x, BAR.y, BAR.w, BAR.h, { fill: linear(0, BAR.y, 0, BAR.y + BAR.h, ["#8a5434", "#4a2a1a", "#24140e"]), r: 3 })
  line(BAR.x + 3, BAR.y + 2, BAR.x + BAR.w - 3, BAR.y + 2, { stroke: alpha(P.sun, 0.45), width: 1.5 })
  rect(BAR.x, BAR.y, BAR.w, BAR.h, { fill: linear(BAR.x, 0, BAR.x + BAR.w, 0, [alpha("#100808", 0), alpha("#100808", 0.45)]), r: 3 })
  // tornillos de anclaje
  for (const sx of [BAR.x + 10, BAR.x + BAR.w - 10]) circle(sx, BAR.y + BAR.h / 2, 3, { fill: "#a89c90", stroke: "#2a1a14", width: 0.8 })
})

layer("ventana", () => {
  // hueco del muro (grosor)
  rect(56, 28, 360, 440, { fill: "#2c1e18", r: 4 })
  rect(66, 36, 342, 424, { fill: linear(0, 36, 0, 460, ["#7a5038", "#3a261c"]) })
  const glass = rect(90, 58, 300, 380, { fill: linear(0, 58, 0, 438, ["#9fb6c8", "#d9c3a6", P.sky, "#ffb070"]) })
  clip(glass, () => {
    // sol bajo y su halo
    circle(150, 330, 160, { fill: radial(150, 330, 160, [alpha("#fff2c8", 0.9), alpha(P.sky, 0)]) })
    circle(150, 330, 26, { fill: "#fff6dc", glow: { blur: 30, color: "#ffe0a0" } })
    // nubes alargadas
    seed(11)
    for (let i = 0; i < 6; i++) {
      const y = rand(90, 260), x = rand(60, 360)
      ellipse(x, y, rand(50, 110), rand(6, 12), { fill: alpha(pick(["#f7d6b0", "#c9b6b8"]), 0.6), blur: 4 })
    }
    // bandada de gaviotas lejanas: siluetas en "m" frías y suaves por la distancia
    seed(31)
    const gull = (x, y, s, flap) => {
      const d = s * flap
      brush([[x - s, y - d * 0.2], [x - s * 0.5, y - d], [x, y]], { width: [0.4, s * 0.22, s * 0.28], color: "#5e566a", taper: [0.6, 0] })
      brush([[x, y], [x + s * 0.5, y - d * 0.9], [x + s, y - d * 0.1]], { width: [s * 0.28, s * 0.22, 0.4], color: "#5e566a", taper: [0, 0.6] })
      circle(x, y + s * 0.05, s * 0.14, { fill: "#5e566a" })
    }
    const flock = [[208, 128, 7], [228, 140, 6], [252, 124, 6.5], [270, 150, 5.5], [296, 138, 5], [318, 162, 4.5], [336, 150, 4], [288, 172, 4.2], [352, 176, 3.5]]
    for (const [x, y, s] of flock) group({ opacity: lerp(0.9, 0.6, (x - 200) / 160) }, () => gull(x + rand(-3, 3), y + rand(-3, 3), s, rand(0.3, 0.7)))
    // colinas y árboles lejanos (fríos)
    blob([[80, 380], [160, 350], [260, 362], [400, 340], [400, 450], [80, 450]], { fill: alpha("#8a8aa0", 0.8), blur: 2 })
    // línea de árboles irregular
    const tl = [[80, 410]]
    for (let x = 80; x <= 400; x += 14) tl.push([x, 392 - rand(0, 40) - (x > 180 && x < 260 ? 25 : 0)])
    tl.push([400, 410])
    poly(roughen(tl, 5, { detail: 3 }), { fill: alpha("#6a6478", 0.9), blur: 2.5 })
    poly(roughen([[80, 410], [140, 384], [220, 392], [300, 378], [400, 390], [400, 412]], 4), { fill: "#544a5c", blur: 1.5 })
    rect(80, 400, 330, 60, { fill: "#4a4050" })
    // enredadera que cuelga del alero por fuera del cristal: contraluz del atardecer, algo velada por el vidrio
    group({ opacity: 0.9, blur: 0.4 }, () => {
      seed(140)
      const G = ["#2a2e22", "#33392a", "#3c4430", "#2e3426"]
      const leaf = (x, y, s, ang) => {
        const c = Math.cos(ang * Math.PI / 180), sn = Math.sin(ang * Math.PI / 180)
        const R = ([px, py, k]) => k === undefined ? [x + px * c - py * sn, y + px * sn + py * c] : [x + px * c - py * sn, y + px * sn + py * c, k]
        const lf = blob([[0, 0.1 * s, 0.4], [-0.45 * s, 0.05 * s], [-0.55 * s, 0.5 * s], [0, 1.2 * s, 0], [0.55 * s, 0.5 * s], [0.45 * s, 0.05 * s]].map(R), { fill: pick(G) })
        clip(lf, () => {
          // el sol bajo atraviesa la hoja
          const m = R([-0.15 * s, 0.45 * s])
          ellipse(m[0], m[1], s * 0.4, s * 0.3, { fill: alpha("#a89a4a", rand(0.2, 0.4)), blur: s * 0.3, blend: "screen" })
        })
        outline([lf], -0.8, alpha(P.sun, 0.22))
        brush([R([0, 0.1 * s]), R([0.02 * s, 0.6 * s]), R([0, 0.95 * s])], { width: [0.7, 0.4, 0.2], color: alpha("#14160e", 0.45), taper: [0, 0.5] })
      }
      // mata en el alero (asoma bajo el marco)
      for (let i = 0; i < 16; i++) leaf(rand(92, 228), rand(52, 66), rand(8, 13), rand(-70, 70))
      // tallos colgantes: [x, largo, vaivén]
      const vines = [[96, 170, 10], [112, 120, 12], [130, 196, 14], [150, 92, 9], [168, 182, 13], [188, 130, 10], [208, 70, 7], [222, 44, 5]]
      for (const [x0, len, sway] of vines) {
        const ph = rand(0, 6), fr = rand(2.5, 4.5), lean = rand(-6, 6), pts = []
        for (let i = 0; i <= 8; i++) {
          const t = i / 8
          pts.push([x0 + Math.sin(t * fr + ph) * sway * t + lean * t * t, 54 + t * len])
        }
        brush(pts, { width: [1.8, 1.2, 0.5], color: "#2e2a1e", taper: [0, 0.4] })
        const n = Math.max(2, Math.round(len / 22))
        for (let k = 1; k <= n; k++) {
          const t = clamp(k / (n + 0.3) + rand(-0.04, 0.04), 0.05, 0.99), i = Math.min(7, Math.floor(t * 8)), f = t * 8 - i
          const side = k % 2 ? 1 : -1
          leaf(lerp(pts[i][0], pts[i + 1][0], f), lerp(pts[i][1], pts[i + 1][1], f), lerp(12, 6.5, t) * rand(0.8, 1.15), side * rand(20, 65))
        }
      }
    })
  })
  // marco de la ventana
  const frame = { fill: "#4a3024" }
  rect(232, 58, 16, 380, frame)
  rect(90, 240, 300, 14, frame)
  rect(84, 52, 312, 392, { fill: "none", stroke: "#4a3024", width: 14 })
  // luz de borde cálida en el marco
  line(240, 60, 240, 436, { stroke: alpha(P.sun, 0.5), width: 2 })
  line(92, 247, 388, 247, { stroke: alpha(P.sun, 0.4), width: 2 })
  // reflejos en los cristales
  poly([[110, 70], [150, 70], [100, 160], [96, 140]], { fill: alpha("#ffffff", 0.12) })
  poly([[270, 270], [300, 270], [255, 340], [252, 320]], { fill: alpha("#ffffff", 0.1) })
  // alféizar
  rect(44, 456, 384, 22, { fill: linear(0, 456, 0, 478, ["#c48a5c", "#5a3826"]), r: 3 })
})

layer("haz de luz", () => {
  poly([[90, 60], [390, 60], [1024, 520], [1024, 760], [300, 760]], {
    fill: linear(200, 200, 800, 650, [alpha(P.sun, 0.22), alpha(P.sun, 0.05)]), blur: 30, blend: "screen",
  })
})

layer("mesa", () => {
  const top = poly([[0, 506], [1024, 496], [1024, 708], [0, 718]], { fill: linear(0, 500, 0, 718, [P.woodDark, P.wood, "#8a5030"]) })
  clip(top, () => {
    // vetas de la madera
    seed(5)
    for (let i = 0; i < 26; i++) {
      const y = 510 + i * 8 + rand(-3, 3)
      const k = (y - 500) / 210
      brush([[-20, y], [300, y + rand(-3, 3)], [700, y + rand(-4, 4)], [1044, y - 8]], {
        width: 1 + k * 3, color: alpha(pick(["#2a140c", "#a8643a"]), 0.25 + k * 0.15), taper: [0.1, 0.1],
      })
    }
    // zona iluminada por el sol
    ellipse(420, 600, 420, 90, { fill: alpha(P.woodLit, 0.55), blur: 40, blend: "screen" })
    ellipse(330, 590, 200, 50, { fill: alpha(P.sun, 0.3), blur: 30, blend: "screen" })
    // oscurecer la derecha
    rect(700, 490, 330, 230, { fill: linear(700, 0, 1024, 0, [alpha("#100808", 0), alpha("#100808", 0.5)]) })
  })
  // canto frontal
  poly([[0, 718], [1024, 708], [1024, 768], [0, 768]], { fill: linear(0, 710, 0, 768, ["#3a2014", "#1a0e0a"]) })
  line(0, 718, 1024, 708, { stroke: alpha(P.woodLit, 0.7), width: 2 })
  line(0, 506, 1024, 496, { stroke: alpha("#100806", 0.6), width: 3 })
})

const SH = { fill: alpha("#1a0c14", 0.8), blend: "multiply", blur: 10 }
layer("sombras proyectadas", () => {
  // sombras largas hacia la derecha (sol bajo desde la izquierda)
  // sombra larga: de la base del objeto hacia la derecha y algo hacia el fondo
  const castShadow = (x, y, w, len, th, fill) =>
    blob([[x - w, y], [x, y - th * 0.6], [x + len * 0.5, y - th * 0.8 - len * 0.02], [x + len, y - len * 0.04 - th * 0.4], [x + len * 1.02, y - len * 0.04 + th * 0.2], [x + len * 0.5, y + th * 0.35 - len * 0.02], [x, y + th * 0.25]], { fill, blur: 9 })
  castShadow(440, 597, 70, 360, 44, alpha("#1a0a10", 0.75)) // jarrón
  castShadow(600, 584, 48, 380, 32, alpha("#0a1a10", 0.7)) // botella
  ellipse(700, 576, 50, 7, { fill: alpha("#7ad08a", 0.12), blur: 12, blend: "screen" }) // luz verde a través del cristal
  castShadow(280, 662, 90, 240, 26, alpha("#1a0c14", 0.55)) // taza
  castShadow(726, 645, 56, 260, 34, alpha("#1a0c14", 0.6)) // pera delantera
  castShadow(814, 621, 46, 220, 28, alpha("#1a0c14", 0.6)) // pera trasera
  ellipse(722, 645, 48, 6, { fill: alpha("#100608", 0.7), blur: 4, blend: "multiply" })
  ellipse(812, 621, 40, 5, { fill: alpha("#100608", 0.7), blur: 4, blend: "multiply" })
  ellipse(275, 664, 100, 9, { fill: alpha("#100608", 0.55), blur: 6, blend: "multiply" })
  // oclusión de contacto
  ellipse(440, 598, 62, 8, { fill: alpha("#100608", 0.7), blur: 5, blend: "multiply" })
  ellipse(600, 586, 50, 7, { fill: alpha("#100608", 0.7), blur: 5, blend: "multiply" })
})

layer("ramas", () => {
  seed(21)
  const C = ["#3a2418", "#5a3a26", "#2a1a12"]
  function branch(x, y, ang, len, w, depth) {
    const pts = [[x, y]]
    let a = ang, cx = x, cy = y
    for (let i = 0; i < 4; i++) {
      a += rand(-12, 12)
      cx += Math.cos(a * Math.PI / 180) * len / 4
      cy += Math.sin(a * Math.PI / 180) * len / 4
      pts.push([cx, cy])
    }
    brush(pts, { width: [w, w * 0.7, w * 0.25], color: pick(C), taper: [0, 0.6] })
    // luz cálida en el lado izquierdo de la rama
    if (w > 2.5) brush(shift(pts, -w * 0.25, 0), { width: [w * 0.3, w * 0.2, 0], color: alpha("#e09a5a", 0.6), taper: [0, 0.6] })
    if (depth > 0) {
      const n = depth > 1 ? 3 : 2
      for (let k = 0; k < n; k++) {
        const t = rand(0.35, 0.95)
        const idx = Math.min(3, Math.floor(t * 4))
        const p = pts[idx + 1]
        branch(p[0], p[1], a + pick([-1, 1]) * rand(20, 45), len * rand(0.4, 0.6), w * 0.5, depth - 1)
      }
      // brotes secos en las puntas
      if (depth === 1) circle(cx, cy, rand(2, 3.5), { fill: "#6a4228" })
    }
  }
  branch(430, 400, -100, 300, 7, 2)
  branch(440, 400, -72, 260, 6, 2)
  branch(425, 405, -128, 240, 5, 2)
  branch(452, 405, -50, 220, 5, 2)
})

layer("jarron", () => {
  revolve(440, [[0, 384], [30, 384], [26, 392], [22, 410], [34, 440], [66, 478], [80, 515], [74, 555], [56, 588], [46, 598], [0, 598]], {
    color: P.ceramic, light: LIGHT,
  })
  // glaseado: banda decorativa y contraluz fría
  const body = ellipse(440, 516, 80, 84, { hidden: true })
  clip(body, () => {
    brush([[364, 500], [440, 508], [516, 500]], { width: 5, color: alpha("#7a4a30", 0.6), taper: [0, 0] })
    brush([[366, 512], [440, 520], [514, 512]], { width: 2, color: alpha("#7a4a30", 0.5), taper: [0, 0] })
    ellipse(520, 520, 22, 90, { fill: alpha(P.cool, 0.35), blur: 10, blend: "screen" })
  })
  // desgaste: silueta del búcaro a partir del perfil
  const prof = [[30, 386], [26, 392], [22, 410], [34, 440], [66, 478], [80, 515], [74, 555], [56, 588], [46, 597]]
  const sil = blob([...prof.map(([r, y]) => [440 + r, y]), ...prof.slice().reverse().map(([r, y]) => [440 - r, y])], { hidden: true })
  clip(sil, () => {
    seed(88)
    // manchas y moteado de la arcilla (pátina irregular)
    for (let i = 0; i < 26; i++) {
      const y = rand(400, 594), x = 440 + rand(-70, 70)
      ellipse(x, y, rand(4, 16), rand(3, 10), { fill: alpha(pick(["#5a3220", "#3a2014"]), rand(0.08, 0.18)), blur: rand(2, 5), blend: "multiply" })
    }
    for (let i = 0; i < 14; i++) {
      const y = rand(420, 590), x = 440 + rand(-60, 40)
      ellipse(x, y, rand(3, 9), rand(2, 6), { fill: alpha("#e8b88a", rand(0.12, 0.25)), blur: 2, blend: "screen" })
    }
    // la banda pintada, gastada a trozos: asoma la arcilla
    for (let i = 0; i < 6; i++) {
      const x = rand(372, 500), u = (x - 440) / 76, y = 500 + 8 * (1 - u * u) + rand(-1, 1)
      blob(jitter(ring(x, y, rand(2.5, 6), rand(1.5, 2.5), 7), 0.8), { fill: alpha(mix("#b07a54", "#5a3826", clamp(u, 0, 1)), 0.7), blur: 0.6 })
    }
    // marca de agua seca: anillo horizontal y suciedad en la base
    brush([[360, 472], [400, 476], [440, 479], [480, 477], [520, 473]], { width: [1, 3, 2, 3, 1], color: alpha("#3a2014", 0.16), blur: 2, taper: [0.3, 0.3] })
    rect(380, 572, 120, 30, { fill: linear(0, 572, 0, 600, [alpha("#2a1610", 0), alpha("#2a1610", 0.45)]), blend: "multiply" })
    // pie sin esmalte, rozado de tanto apoyarlo
    brush([[396, 592], [440, 595], [484, 592]], { width: 2, color: alpha("#e2b088", 0.45), taper: [0.3, 0.3] })
    // craquelado fino del esmalte
    const crack = (x, y, ang, len, depth) => {
      const pts = [[x, y]]
      let a = ang
      for (let k = 0; k < 4; k++) { a += rand(-35, 35); x += Math.cos(a * Math.PI / 180) * len / 4; y += Math.sin(a * Math.PI / 180) * len / 4; pts.push([x, y]) }
      brush(pts, { width: [0.9, 0.7, 0.3], color: alpha("#3a1e12", 0.45), taper: [0.1, 0.6] })
      if (depth > 0) crack(pts[2][0], pts[2][1], a + pick([-50, 50]), len * 0.55, depth - 1)
    }
    for (let i = 0; i < 6; i++) crack(rand(380, 500), rand(520, 570), rand(0, 360), rand(14, 26), 1)
    // grieta capilar que baja desde la boca
    const gx = [[453, 392], [455, 404], [452, 416], [456, 430], [455, 444]]
    brush(shift(gx, 0.8, 0.6), { width: [0.9, 0.7, 0.3], color: alpha("#f2c8a0", 0.3), taper: [0, 0.6] })
    brush(gx, { width: [1.2, 0.9, 0.3], color: alpha("#2a140c", 0.55), taper: [0, 0.6] })
    // arañazos claros
    for (let i = 0; i < 9; i++) {
      const x = rand(376, 500), y = rand(450, 580), l = rand(5, 14), a = rand(-30, 30) * Math.PI / 180
      line(x, y, x + Math.cos(a) * l, y + Math.sin(a) * l, { stroke: alpha("#f6d4b0", rand(0.2, 0.4)), width: 0.7 })
    }
  })
  // desconchón en el labio: arcilla cruda con su borde en sombra
  blob([[446, 389, 0], [452, 392], [460, 391, 0], [457, 395], [449, 395, 0]], { fill: "#d9a77a", stroke: alpha("#3a1e12", 0.7), width: 0.8 })
  blob([[423, 388, 0], [428, 390], [431, 389, 0], [428, 392, 0]], { fill: "#d9a77a", stroke: alpha("#3a1e12", 0.6), width: 0.6 })
  // brillo nítido
  ellipse(395, 495, 6, 18, { fill: alpha("#fff4e0", 0.8), blur: 2, rotate: 10 })
  // boca oscura
  ellipse(440, 386, 28, 6, { fill: "#2a1810" })
})
layer("botella", () => {
  group({ opacity: 0.92 }, () => {
    revolve(600, [[0, 236], [15, 236], [15, 250], [12, 256], [12, 320], [20, 352], [40, 388], [50, 420], [50, 568], [46, 584], [0, 584]], {
      color: P.glass, light: LIGHT,
    })
  })
  const bot = blob([[588, 236, 0], [612, 236, 0], [612, 320], [650, 400], [650, 570], [640, 584, 0], [560, 584, 0], [550, 570], [550, 400], [588, 320]], { hidden: true })
  clip(bot, () => {
    // interior translúcido: ventana vista a través del cristal
    rect(560, 430, 26, 130, { fill: alpha("#e8ffd8", 0.3), blur: 8, blend: "screen" })
    // líquido / fondo más denso
    rect(540, 440, 120, 150, { fill: linear(0, 440, 0, 590, [alpha("#143a1e", 0), alpha("#143a1e", 0.4)]), blend: "multiply" })
    // contraluz fría a la derecha
    brush([[640, 400], [646, 480], [644, 570]], { width: 6, color: alpha(P.cool, 0.6), taper: [0.2, 0.2] })
    // brillo de sol a la izquierda
    brush([[566, 408], [562, 480], [564, 556]], { width: 7, color: alpha("#fff0c0", 0.85), taper: [0.25, 0.3] })
    brush([[594, 262], [594, 318]], { width: 3, color: alpha("#fff0c0", 0.8) })
    // cáustica en la base
    ellipse(610, 576, 26, 6, { fill: alpha("#c8f8b0", 0.6), blur: 3, blend: "screen" })
  })
})

layer("peras", () => {
  function pear(cx, by, h, rot, seedN) {
    seed(seedN)
    group({ rotate: rot, around: [cx, by - h / 2] }, () => {
      const w = h * 0.42
      const pts = [
        [cx, by - h], [cx + w * 0.36, by - h * 0.9], [cx + w * 0.5, by - h * 0.62], [cx + w * 0.92, by - h * 0.32],
        [cx + w * 0.72, by - h * 0.03], [cx, by + 1], [cx - w * 0.72, by - h * 0.03], [cx - w * 0.92, by - h * 0.32],
        [cx - w * 0.5, by - h * 0.62], [cx - w * 0.36, by - h * 0.9],
      ]
      const shape = blob(pts, { fill: P.pear })
      clip(shape, () => {
        // sombra propia a la derecha y abajo
        ellipse(cx + w * 0.95, by - h * 0.3, w * 0.85, h * 0.7, { fill: alpha("#3a3a14", 0.9), blur: 20 })
        ellipse(cx + w * 0.7, by - h * 0.75, w * 0.4, h * 0.3, { fill: alpha("#5a5420", 0.7), blur: 12 })
        ellipse(cx, by + h * 0.05, w * 1.1, h * 0.14, { fill: alpha("#3a2a14", 0.6), blur: 10 })
        // luz cálida a la izquierda
        ellipse(cx - w * 0.4, by - h * 0.36, w * 0.5, h * 0.26, { fill: alpha("#ffe890", 0.8), blur: 14 })
        ellipse(cx - w * 0.15, by - h * 0.78, w * 0.25, h * 0.14, { fill: alpha("#f0dc70", 0.6), blur: 8 })
        // rubor cálido y moteado
        ellipse(cx - w * 0.3, by - h * 0.3, w * 0.5, h * 0.2, { fill: alpha(P.accent, 0.35), blur: 12, blend: "soft-light" })
        for (let i = 0; i < 30; i++) circle(cx + rand(-w, w), by - rand(0, h), rand(0.8, 1.6), { fill: alpha("#5a4a1a", 0.35) })
        // contraluz fría en el borde derecho
        brush([[cx + w * 0.4, by - h * 0.7], [cx + w * 0.95, by - h * 0.32], [cx + w * 0.7, by - h * 0.05]], { width: 5, color: alpha(P.cool, 0.45), blur: 2 })
      })
      // contorno con peso en el lado de la sombra
      brush([[cx + w * 0.3, by - h * 0.86], [cx + w * 0.5, by - h * 0.6], [cx + w * 0.9, by - h * 0.3], [cx + w * 0.6, by - h * 0.02], [cx - w * 0.2, by]], {
        width: [1, 3, 4, 3.5, 1], color: alpha("#3a2e10", 0.7),
      })
      // brillo
      ellipse(cx - w * 0.55, by - h * 0.38, 5, 10, { fill: alpha("#fffbe0", 0.85), blur: 2, rotate: 15 })
      // rabito
      brush([[cx, by - h + 4], [cx + 3, by - h - 14], [cx + 10, by - h - 26]], { width: [5, 3.5, 2], color: "#4a2e1a", taper: [0, 0.3] })
    })
  }
  pear(812, 622, 122, 10, 43)
  pear(722, 646, 150, -6, 41)
})

layer("taza", () => {
  // platillo
  ellipse(270, 656, 100, 20, { fill: "#6a5650" })
  ellipse(270, 652, 100, 19, { fill: linear(170, 0, 370, 0, ["#fff0d8", "#e0c8a8", "#8a7a80"]) })
  ellipse(270, 648, 76, 13, { fill: alpha("#7a6460", 0.4), blur: 3 })
  ellipse(310, 650, 50, 8, { fill: alpha("#3a2430", 0.4), blur: 4, blend: "multiply" })
  // asa (en sombra, a la derecha)
  curve([[318, 580], [352, 580], [356, 612], [318, 628]], { stroke: "#8a7c7c", width: 11, cap: "round" })
  curve([[320, 582], [348, 583], [351, 610], [322, 624]], { stroke: alpha(P.cool, 0.6), width: 3, cap: "round" })
  revolve(270, [[54, 560], [53, 580], [50, 610], [42, 638], [34, 648], [0, 648]], {
    color: P.cup, light: LIGHT,
  })
  // borde y café
  ellipse(270, 560, 54, 11, { fill: "#d8ccbc" })
  ellipse(270, 561, 48, 8.5, { fill: radial(260, 562, 50, ["#5a2e18", "#2a120a"]) })
  ellipse(256, 560, 16, 3, { fill: alpha(P.sun, 0.45), blur: 1.5 })
  // tinte cálido del sol en la porcelana y contraluz fría
  const cupBody = blob([[216, 562, 0], [324, 562, 0], [320, 610], [304, 646, 0.5], [236, 646, 0.5], [220, 610]], { hidden: true })
  clip(cupBody, () => {
    // marca impresa "Almirasofts": emblema y logotipo, tinta multiplicada sobre la porcelana
    const INK = { blend: "multiply", opacity: 0.95, move: [-5, 0] }
    group(INK, () => {
      circle(266, 591, 11, { fill: "none", stroke: "#8a3a22", width: 1.6 })
      poly([[266, 582], [274.5, 598], [270.5, 598], [266, 589], [261.5, 598], [257.5, 598]], { fill: "#8a3a22" })
      brush([[257, 596], [266, 593], [277, 588]], { width: 1.6, color: "#c8642c", taper: [0.2, 0.5] })
      text(266, 615, "ALMIRASOFTS", { size: 8.8, weight: 800, anchor: "middle", spacing: 0.3, fill: "#5a2a1c" })
      line(244, 620, 288, 620, { stroke: "#c8642c", width: 0.8 })
      text(266, 628, "COFFEE · CODE", { size: 5, anchor: "middle", spacing: 1.2, fill: "#7a4a36" })
    })
    ellipse(240, 600, 40, 50, { fill: alpha("#ffc070", 0.35), blur: 14, blend: "soft-light" })
    brush([[318, 568], [318, 610], [304, 642]], { width: 4, color: alpha(P.cool, 0.55), blur: 1.5 })
  })
  // brillo en la porcelana
  brush([[230, 572], [228, 600], [236, 628]], { width: 5, color: alpha("#fffaf0", 0.9) })
})

layer("manzana", () => {
  // C: [luz, medio, sombra, sombra baja, lateral, base]
  const RED = ["#f25a3e", "#c4262a", "#6a0e1c", "#7a1420", "#a81e26", "#b8222a"]
  function apple(ax, ab, s, C, seedN, rot = 0) {
  group({ scale: s, around: [ax, ab], rotate: rot }, () => {
  // sombra larga hacia la derecha y oclusión de contacto
  blob([[ax - 36, ab], [ax, ab - 14], [ax + 110, ab - 20], [ax + 200, ab - 16], [ax + 204, ab - 6], [ax + 110, ab + 4], [ax, ab + 8]], { fill: alpha("#1a0c14", 0.85), blur: 9, blend: "multiply" })
  ellipse(ax + 2, ab - 1, 38, 6, { fill: alpha("#100608", 0.9), blur: 4, blend: "multiply" })
  const apple = blob([
    [ax - 6, ab - 62, 0.5], [ax - 26, ab - 70], [ax - 42, ab - 52], [ax - 42, ab - 24], [ax - 26, ab - 2],
    [ax - 8, ab], [ax + 10, ab], [ax + 28, ab - 3], [ax + 42, ab - 24], [ax + 40, ab - 54], [ax + 20, ab - 70],
  ], { fill: C[5] })
  const body = freeform(apple, [
    [ax - 26, ab - 42, C[0], 0.1],
    [ax - 4, ab - 34, C[1], 0.1],
    [ax + 30, ab - 30, C[2], 0.1],
    [ax + 6, ab - 6, C[3], 0.05],
    [ax - 30, ab - 14, C[4], 0.05],
  ])
  clip(body, () => {
    // vetas amarillentas de la piel
    seed(seedN)
    for (let i = 0; i < 9; i++) {
      const x = ax + rand(-30, 20)
      brush([[x, ab - 60], [x + rand(-6, 6), ab - 34], [x + rand(-4, 8), ab - 6]], { width: rand(1, 2.2), color: alpha("#ffb060", 0.18), blur: 1 })
    }
    // luz cálida del sol a la izquierda
    ellipse(ax - 26, ab - 40, 14, 22, { fill: alpha(P.sun, 0.35), blur: 10, blend: "screen" })
    // contraluz fría en el borde derecho
    brush([[ax + 30, ab - 60], [ax + 42, ab - 34], [ax + 30, ab - 6]], { width: 4, color: alpha(P.cool, 0.4), blur: 2 })
    // rebote cálido de la mesa abajo
    ellipse(ax, ab + 2, 30, 7, { fill: alpha(P.woodLit, 0.35), blur: 4, blend: "screen" })
  })
  // contorno con peso en el lado de la sombra
  brush([[ax + 30, ab - 62], [ax + 39, ab - 46], [ax + 40, ab - 24], [ax + 27, ab - 4], [ax, ab - 1]], {
    width: [1, 3, 3.5, 3, 1], color: alpha("#3a0810", 0.7),
  })
  // hoyuelo del rabito y rabito
  ellipse(ax - 4, ab - 64, 9, 3.5, { fill: alpha("#4a0a14", 0.7), blur: 2 })
  brush([[ax - 4, ab - 64], [ax - 2, ab - 76], [ax + 4, ab - 86]], { width: [4, 3, 2], color: "#4a2e1a", taper: [0, 0.3] })
  // hojita
  blob([[ax + 2, ab - 78, 0], [ax + 14, ab - 90], [ax + 30, ab - 88, 0], [ax + 16, ab - 79]], { fill: linear(ax, 0, ax + 30, 0, ["#8a8a3a", "#3a3a18"]) })
  // brillo nítido
  ellipse(ax - 24, ab - 46, 4, 9, { fill: alpha("#fff4e0", 0.85), blur: 1.5, rotate: 20 })
  })
  }
  // grupo de manzanas detrás, entre el jarrón y la botella (más pequeñas por la distancia)
  apple(522, 604, 0.8, ["#f08a48", "#d0582c", "#7a2418", "#8a3020", "#c0502a", "#c85a2e"], 62, 6)   // anaranjada
  apple(566, 622, 0.88, ["#e2483a", "#a81c30", "#4e0a1c", "#5e1020", "#8e1828", "#9a1a2c"], 63, -4) // granate
  apple(468, 626, 0.9, ["#f6704a", "#d8382c", "#7e161c", "#8a2020", "#b82c28", "#cc3a2c"], 64, 3)   // roja más viva
  apple(500, 696, 1, RED, 61)
})

layer("vapor", () => {
  seed(9)
  for (let k = 0; k < 3; k++) {
    const x0 = 252 + k * 18
    const pts = []
    for (let i = 0; i <= 6; i++) pts.push([x0 + Math.sin(i * 1.1 + k * 2) * (8 + i * 4) + i * 3, 552 - i * 30])
    brush(pts, { width: (u) => 5 + 16 * Math.sin(u * Math.PI), color: alpha("#fff3dc", 0.22 - k * 0.04), blur: 5, taper: [0.2, 0.6] })
    brush(pts, { width: 2.5, color: alpha("#fffaf0", 0.35), blur: 1.5, taper: [0.2, 0.7] })
  }
})
layer("atmosfera", () => {
  // motas de polvo en el haz de luz
  seed(77)
  for (let i = 0; i < 50; i++) {
    const t = rand(), x = lerp(120, 700, t) + rand(-80, 80), y = lerp(120, 520, t) + rand(-80, 80)
    circle(x, y, rand(0.8, 2.2), { fill: alpha("#fff0c8", rand(0.3, 0.8)), glow: { blur: 3, color: "#ffd890" } })
  }
  // velo cálido general y viñeta
  rect(0, 0, 1024, 768, { fill: radial(300, 330, 700, [alpha("#ffb060", 0.1), alpha("#ffb060", 0)]), blend: "soft-light" })
  rect(0, 0, 1024, 768, { fill: radial(460, 420, 720, [alpha("#000000", 0), [0.7, alpha("#000000", 0.1)], alpha("#0a0408", 0.55)]), blend: "multiply" })
  grain({ opacity: 0.07 })
})
