// Zorro explorador al anochecer mirando una luciérnaga
canvas({ width: 1024, height: 1024, background: linear(0, 0, 0, 820, ["#3f4778", [0.55, "#8f74a6"], "#f2b98e"]) })

const P = {
  skyGlow: "#ffd9a8",
  farTree: "#7a77a8",
  midTree: "#565c88",
  nearTree: "#3e4a6e",
  ground: "#5e7d52",
  groundDark: "#3f5a45",
  rock: "#9a90a6",
  rockDark: "#5f5672",
  moss: "#86a84c",
  mossDark: "#4f7438",
  fox: "#e2793a",
  foxLight: "#f8ae62",
  foxDark: "#a9481f",
  foxInk: "#7a3016",
  cream: "#fcebd2",
  creamShade: "#e0c3b0",
  sock: "#5b3026",
  scarf: "#d93a3c",
  scarfDark: "#8f1e2c",
  pack: "#72904c",
  packDark: "#465f34",
  leather: "#a06a3a",
  eye: "#3a2018",
  fly: "#fff3a0",
}

const ink = (pts, w, color, taper = [0.3, 0.4]) => brush(pts, { width: w, color, taper })

layer("cielo", () => {
  ellipse(300, 640, 520, 260, { fill: alpha(P.skyGlow, 0.45), blur: 80 })
  seed(3)
  for (let i = 0; i < 40; i++) {
    const x = rand(20, 1004), y = rand(20, 360)
    circle(x, y, rand(1, 2.6), { fill: "#fff6dc", opacity: rand(0.4, 0.9) })
  }
  // luna creciente
  path("M 219 125.6 A 46 46 0 1 0 174.4 195.6 A 42 42 0 1 1 219 125.6 Z", { fill: "#fff1cf", glow: { blur: 18, color: "#ffe9b0" } })
  // nubes suaves
  blob([[600, 170], [660, 140], [740, 150], [800, 135], [870, 160], [850, 190], [640, 195]], { fill: "#b49ac0", opacity: 0.5 })
  blob([[60, 300], [140, 270], [230, 285], [300, 270], [340, 300], [220, 318], [90, 320]], { fill: "#c9a3b4", opacity: 0.45 })
})

layer("bosque lejano", () => {
  const pine = (x, base, h, w, color) => {
    const pts = [[x, base - h, 0]]
    const tiers = 4
    for (let i = 1; i <= tiers; i++) pts.push([x + w * (i / tiers) * 0.5 + 6, base - h + (h * i) / tiers, 0.4], [x + w * (i / tiers) * 0.28, base - h + (h * i) / tiers - 8, 0])
    const right = pts.slice(1)
    const left = right.map(([px, py, s]) => [2 * x - px, py, s]).reverse()
    blob([...pts, ...right.slice(0, -1).concat([[x + w * 0.5, base + 30, 0], [x - w * 0.5, base + 30, 0]]), ...left.slice(1)], { fill: color })
  }
  // colinas lejanas
  blob([[-40, 700], [120, 600], [300, 630], [480, 580], [700, 610], [880, 570], [1080, 620], [1080, 820], [-40, 820]], { fill: mix(P.farTree, "#f2b98e", 0.35) })
  seed(11)
  for (let x = -20; x < 1060; x += rand(55, 85)) pine(x, 690 + rand(-20, 20), rand(140, 220), rand(70, 100), P.farTree)
  blob([[-40, 760], [200, 700], [420, 730], [640, 700], [860, 720], [1080, 690], [1080, 860], [-40, 860]], { fill: P.midTree })
  for (let x = -10; x < 1060; x += rand(80, 130)) if (x < 160 || x > 820) pine(x, 760 + rand(-10, 15), rand(240, 330), rand(100, 140), P.nearTree)
  // bruma
  rect(0, 640, 1024, 200, { fill: linear(0, 640, 0, 840, [alpha("#e8b8a8", 0), alpha("#e8b8a8", 0.4)]) })
})

layer("suelo", () => {
  blob([[-40, 830], [200, 795], [500, 810], [800, 790], [1080, 815], [1080, 1080], [-40, 1080]], { fill: linear(0, 790, 0, 1024, ["#7d975c", P.ground, P.groundDark]) })
})

layer("roca", () => {
  const rockPts = [[200, 900], [215, 790], [270, 712], [380, 668], [520, 655], [650, 672], [740, 735], [790, 830], [800, 905], [500, 925]]
  // sombra proyectada
  ellipse(500, 905, 330, 34, { fill: "#2c3a33", opacity: 0.5, blur: 12 })
  const rock = blob(rockPts, { fill: P.rock })
  outline(rock, 3, P.rockDark)
  freeform(rock, [[330, 740, "#c7bccb", 0.5], [760, 860, P.rockDark, 0.4], [480, 910, "#776c8a", 0.5]])
  clip(rock, () => {
    // grietas y manchas
    ink([[600, 760], [630, 800], [620, 850]], 5, P.rockDark)
    ink([[300, 790], [330, 820]], 4, P.rockDark)
    circle(700, 830, 14, { fill: "#b8c27a", opacity: 0.5 })
    circle(330, 870, 10, { fill: "#b8c27a", opacity: 0.5 })
    ink([[740, 760], [780, 860], [770, 910]], 14, alpha(P.rockDark, 0.6), [0.4, 0.2])
  })
  ink([[210, 860], [216, 790], [268, 714]], [3, 7, 4], P.rockDark)
  ink([[742, 740], [790, 830], [798, 900]], [4, 10, 6], P.rockDark)
  // musgo
  seed(5)
  const mossPts = [[222, 790], [270, 710], [380, 664], [520, 650], [655, 668], [742, 738], [728, 760, 0], [700, 748], [672, 772, 0], [630, 742], [580, 770, 0], [530, 738], [470, 760, 0], [410, 735], [350, 772, 0], [300, 748], [255, 800, 0]]
  const moss = blob(mossPts, { fill: P.moss })
  freeform(moss, [[350, 680, "#b6cf68", 0.3], [520, 660, "#a6c45e", 0.3], [700, 740, P.mossDark, 0.2], [260, 780, "#6f9440", 0.2]])
  outline(moss, 2.5, P.mossDark)
  clip(moss, () => {
    for (let i = 0; i < 60; i++) {
      const x = rand(240, 730), y = rand(660, 760)
      circle(x, y, rand(3, 7), { fill: pick(["#c6dd78", "#6f9440", "#98b955"]), opacity: 0.6 })
    }
  })
})

layer("cola", () => {
  const tail = blob([[410, 600], [335, 582], [272, 545], [228, 490], [196, 425, 0], [178, 470], [182, 540], [222, 605], [290, 652], [370, 680], [420, 682]], { fill: P.fox })
  outline(tail, 3.5, P.foxInk)
  freeform(tail, [[300, 560, P.foxLight, 0.3], [230, 520, P.fox, 0.2], [300, 650, P.foxDark, 0.3]])
  clip(tail, () => {
    blob([[150, 400], [240, 430], [255, 480], [215, 520], [170, 530]], { fill: P.cream })
    ink([[228, 445], [242, 482], [214, 518]], 4, P.creamShade)
    ink([[230, 590], [280, 630], [340, 655]], 7, alpha(P.foxDark, 0.7))
    ink([[260, 560], [300, 585]], 4, alpha(P.foxInk, 0.5))
  })
  ink([[180, 480], [186, 545], [226, 607], [292, 654], [370, 682]], [3, 7, 9, 6], P.foxInk)
})

layer("zorro", () => {
  // patas traseras y cuerpo
  const body = blob([[455, 398], [545, 402], [598, 455], [612, 560], [614, 650], [590, 690], [440, 692], [355, 670], [335, 590], [362, 490], [408, 428]], { fill: P.fox })
  outline(body, 3.5, P.foxInk)
  freeform(body, [[560, 455, P.foxLight, 0.3], [430, 500, P.fox, 0.3], [380, 665, P.foxDark, 0.3], [590, 660, P.foxDark, 0.1]])
  clip(body, () => {
    // pecho crema
    blob([[495, 418], [582, 430], [616, 480], [614, 565], [588, 595], [560, 545], [528, 470]], { fill: P.cream })
    ink([[565, 470], [575, 500], [568, 525]], 3, P.creamShade)
    ink([[590, 520], [596, 550]], 3, P.creamShade)
  })
  // muslo
  const haunch = blob([[362, 560], [432, 538], [495, 578], [512, 645], [484, 690], [382, 690], [346, 640]], { fill: P.fox })
  freeform(haunch, [[440, 570, P.foxLight, 0.2], [400, 660, P.foxDark, 0.3], [490, 640, P.fox, 0.2]])
  ink([[372, 570], [432, 545], [490, 580], [508, 640], [488, 684]], [3, 6, 7, 4], P.foxInk)
  // pata trasera
  const hp = blob([[452, 674], [510, 670], [548, 682], [540, 698], [460, 700]], { fill: P.sock })
  ink([[455, 698], [540, 698], [548, 684]], [3, 5, 3], "#3a1c16")
  ink([[512, 690], [516, 698]], 2.5, "#3a1c16", [0, 0])
  ink([[530, 688], [532, 698]], 2.5, "#3a1c16", [0, 0])
  // patas delanteras
  const legSock = (pts) => {
    const leg = blob(pts, { fill: P.fox })
    clip(leg, () => {
      rect(500, 470, 160, 240, { fill: linear(0, 500, 0, 660, [mix(P.cream, P.foxLight, 0.5), [0.3, P.foxLight], [0.52, P.fox], [0.74, P.sock], "#3e2019"]) })
    })
    return leg
  }
  legSock([[540, 500], [576, 500], [582, 600], [580, 676], [592, 694], [544, 696], [546, 640]])
  legSock([[584, 490], [626, 495], [628, 600], [624, 676], [642, 694], [590, 697], [592, 620]])
  ink([[547, 560], [546, 640], [544, 696]], [1, 4, 4], P.foxInk, [0.4, 0])
  ink([[592, 560], [592, 620], [590, 697]], [1, 4, 4], P.foxInk, [0.4, 0])
  ink([[606, 494], [626, 505], [628, 600], [624, 676], [642, 694]], [2, 4, 5, 5, 4], P.foxInk, [0.15, 0])
  ink([[600, 697], [640, 695]], 4, "#3a1c16", [0.2, 0.2])
  ink([[614, 684], [616, 696]], 2.5, "#3a1c16", [0, 0])
  ink([[628, 684], [630, 696]], 2.5, "#3a1c16", [0, 0])
  ink([[566, 684], [568, 696]], 2.5, "#3a1c16", [0, 0])

  // orejas
  const earB = blob([[442, 290], [428, 172, 0], [505, 252]], { fill: P.foxDark })
  outline(earB, 3, P.foxInk)
  blob([[452, 275], [440, 200, 0], [488, 255]], { fill: "#5a2a1e" })
  const earF = blob([[502, 258], [552, 152, 0], [598, 276]], { fill: P.fox })
  outline(earF, 3.5, P.foxInk)
  clip(earF, () => rect(490, 140, 120, 140, { fill: linear(0, 150, 0, 270, ["#7a3418", [0.3, P.foxDark], P.fox]) }))
  blob([[520, 255], [552, 180, 0], [580, 262]], { fill: "#e9b7a4" })
  ink([[528, 254], [548, 205], [552, 182]], [2, 4, 1], "#c88878")
  // cabeza
  const head = blob([[425, 332], [440, 272], [500, 240], [572, 250], [618, 286], [655, 318], [686, 340, 0.5], [678, 360], [642, 380], [596, 402], [546, 428, 0], [522, 408], [482, 422, 0], [455, 396], [428, 372]], { fill: P.fox })
  outline(head, 3.5, P.foxInk)
  clip(head, () => {
    rect(400, 220, 300, 220, { fill: linear(660, 300, 430, 380, ["#ffb870", [0.5, P.fox], "#c45e2a"]) })
    ellipse(520, 270, 60, 30, { fill: P.foxLight, opacity: 0.5, blur: 18 })
    const muzzle = blob([[572, 338], [640, 326], [690, 344], [680, 364], [642, 382], [596, 404], [546, 432, 0], [528, 396], [548, 360]], { fill: P.cream })
    ink([[560, 400], [600, 395], [640, 380]], 5, alpha(P.creamShade, 0.8))
    // brillo cálido de la luciérnaga
    circle(720, 310, 120, { fill: radial(720, 310, 120, [alpha("#fff0a0", 0.55), alpha("#fff0a0", 0)]), blend: "screen" })
    // mejilla
    ellipse(568, 365, 18, 10, { fill: "#f2907a", opacity: 0.55 })
  })
  ink([[430, 365], [428, 330], [442, 274], [500, 242]], [3, 7, 5, 2], P.foxInk)
  ink([[546, 426], [598, 402], [644, 380], [678, 360]], [6, 5, 3], P.foxInk)
  // ojo mirando a la luciérnaga
  ellipse(603, 301, 13, 16, { fill: P.eye, rotate: -10 })
  circle(607, 294, 5, { fill: "#ffffff" })
  circle(598, 307, 2, { fill: "#ffffff", opacity: 0.8 })
  ink([[586, 280], [602, 274], [618, 278]], [2, 4, 2], P.foxInk)
  ink([[588, 285], [578, 280]], 2.5, P.eye)
  // nariz y boca
  ellipse(682, 343, 11, 9, { fill: "#3a2018", rotate: -15 })
  ellipse(684, 339, 4, 2.5, { fill: "#ffffff", opacity: 0.7 })
  ink([[674, 360], [658, 371], [640, 369]], [2, 3.5, 2], "#6a2f20")
  // bigotes
  ink([[650, 352], [700, 336]], 1.5, alpha("#6a2f20", 0.7))
  ink([[648, 358], [702, 356]], 1.5, alpha("#6a2f20", 0.7))
  // mechón
  ink([[488, 252], [470, 236], [478, 250]], 3, P.foxInk)
})

layer("mochila", () => {
  group({ rotate: -8, around: [355, 490] }, () => {
    // saco de dormir enrollado
    const roll = rect(300, 395, 110, 38, { fill: "#c9804a", r: 19 })
    outline(roll, 3, "#7a4220")
    ellipse(406, 414, 10, 19, { fill: "#e3a26a" })
    ink([[340, 395], [340, 433]], 4, "#7a4220", [0, 0])
    ink([[372, 395], [372, 433]], 4, "#7a4220", [0, 0])
    const pack = rect(298, 425, 118, 140, { fill: P.pack, r: 30 })
    outline(pack, 3.5, P.packDark)
    freeform(pack, [[390, 450, "#9cb46a", 0.3], [320, 550, P.packDark, 0.3], [360, 500, P.pack, 0.2]])
    const flap = blob([[300, 440], [356, 424], [414, 440], [410, 490], [358, 505], [304, 490]], { fill: "#839f58" })
    outline(flap, 2.5, P.packDark)
    ink([[310, 492], [358, 506], [406, 492]], [2, 4, 2], P.packDark)
    // hebilla
    rect(347, 492, 22, 26, { fill: P.leather, r: 4 })
    rect(353, 500, 10, 10, { fill: "#f1c46a", r: 2 })
    // bolsillo
    const pocket = rect(318, 520, 76, 36, { fill: "#6a8445", r: 10 })
    outline(pocket, 2, P.packDark)
    ink([[322, 528], [390, 528]], 2, alpha("#c8d89a", 0.6), [0.1, 0.1])
  })
  // correa
  ink([[402, 440], [470, 470], [546, 512]], [8, 9, 6], P.leather, [0, 0.2])
  ink([[404, 446], [470, 476], [544, 516]], 2, "#6e3f1e", [0, 0.3])
})

layer("bufanda", () => {
  const stripes = (shape) => clip(shape, () => {
    for (let x = 250; x < 640; x += 34) ink([[x, 380], [x + 30, 470]], 8, alpha("#f57a62", 0.7), [0, 0])
  })
  // extremo que ondea detrás
  const tailS = blob([[455, 404], [395, 392], [335, 368], [278, 372, 0], [302, 394], [272, 418, 0], [338, 422], [400, 434], [458, 436]], { fill: P.scarf })
  outline(tailS, 3, P.scarfDark)
  stripes(tailS)
  ink([[282, 374], [270, 368]], 3, P.scarfDark)
  ink([[300, 394], [285, 396]], 3, P.scarfDark)
  ink([[275, 416], [262, 420]], 3, P.scarfDark)
  // vuelta al cuello
  const wrap = blob([[436, 392], [500, 410], [570, 404], [610, 390], [620, 422], [584, 450], [510, 456], [444, 440], [426, 414]], { fill: P.scarf })
  outline(wrap, 3.5, P.scarfDark)
  freeform(wrap, [[590, 405, "#f46a5a", 0.3], [470, 430, "#b42c34", 0.3], [530, 420, P.scarf, 0.4]])
  stripes(wrap)
  ink([[440, 438], [510, 456], [584, 450]], [3, 7, 4], P.scarfDark)
  // extremo que cuelga delante
  const hang = blob([[566, 440], [604, 444], [618, 520, 0.5], [600, 536, 0], [590, 522, 0], [572, 534, 0], [560, 470]], { fill: P.scarf })
  outline(hang, 3, P.scarfDark)
  clip(hang, () => rect(540, 430, 100, 120, { fill: linear(610, 450, 570, 530, ["#f46a5a", P.scarf, "#b42c34"]) }))
  stripes(hang)
  // nudo
  const knot = ellipse(584, 440, 22, 18, { fill: "#e44a48" })
  outline(knot, 3, P.scarfDark)
  ellipse(590, 434, 8, 5, { fill: "#ff8a78", opacity: 0.8 })
})
layer("luciernaga", () => {
  const fx0 = 738, fy0 = 292
  // halo
  circle(fx0, fy0, 110, { fill: radial(fx0, fy0, 110, [alpha("#fff2a0", 0.5), [0.35, alpha("#ffd86a", 0.2)], alpha("#ffd86a", 0)]), blend: "screen" })
  circle(fx0 + 7, fy0 + 6, 34, { fill: "#ffe46a", opacity: 0.6, blur: 14 })
  circle(fx0 + 7, fy0 + 6, 18, { fill: "#fff7c0", blur: 3 })
  // alas
  ellipse(fx0 - 6, fy0 - 18, 14, 8, { fill: alpha("#ffffff", 0.55), rotate: -35 })
  ellipse(fx0 + 10, fy0 - 20, 13, 7, { fill: alpha("#ffffff", 0.45), rotate: 30 })
  // cuerpo
  const bug = ellipse(fx0 - 4, fy0 - 2, 11, 8, { fill: "#4a3a4e", rotate: 25 })
  outline(bug, 1.5, "#2c2234")
  circle(fx0 - 13, fy0 - 7, 5, { fill: "#4a3a4e" })
  ink([[fx0 - 15, fy0 - 11], [fx0 - 22, fy0 - 22], [fx0 - 18, fy0 - 28]], 1.6, "#2c2234", [0, 0.3])
  ink([[fx0 - 12, fy0 - 12], [fx0 - 14, fy0 - 25], [fx0 - 8, fy0 - 30]], 1.6, "#2c2234", [0, 0.3])
  ellipse(fx0 + 7, fy0 + 6, 9, 8, { fill: "#fffbe0" })
  // estela
  for (const [i, u] of [0, 1, 2, 3, 4].entries()) circle(fx0 + 30 + i * 18, fy0 + 25 + Math.sin(i) * 12, 2.5 - i * 0.35, { fill: "#fff2a0", opacity: 0.7 - i * 0.12, glow: { blur: 4, color: "#ffe46a" } })
  // otras luciérnagas lejanas
  seed(21)
  for (const [x, y] of [[860, 420], [120, 610], [930, 600], [300, 250], [800, 520], [90, 450]]) {
    circle(x, y, rand(2.5, 4), { fill: "#fff2a0", glow: { blur: 10, color: "#ffe46a" }, opacity: 0.9 })
  }
})

layer("primer plano", () => {
  seed(9)
  // hierba alrededor de la roca y del borde
  const tuft = (x, y, n, h, col) => {
    for (let i = 0; i < n; i++) {
      const a = rand(-0.5, 0.5), hh = h * rand(0.6, 1.1)
      ink([[x + i * 6 - n * 3, y], [x + i * 6 - n * 3 + a * hh * 0.5, y - hh * 0.6], [x + i * 6 - n * 3 + a * hh, y - hh]], rand(5, 8), col, [0, 0.8])
    }
  }
  for (const [x, y] of [[200, 910], [250, 925], [770, 915], [820, 900], [880, 930], [130, 900], [960, 900], [60, 960]]) tuft(x, y, 6, rand(40, 70), pick(["#4f7438", "#6a9246", "#3f5f34"]))
  // setas
  const shroom = (x, y, s) => {
    const stem = blob([[x - 7 * s, y], [x - 5 * s, y - 26 * s], [x + 5 * s, y - 26 * s], [x + 8 * s, y]], { fill: "#f5e6cc" })
    outline(stem, 2, "#a9876a")
    const cap = blob([[x - 26 * s, y - 22 * s, 0.5], [x - 14 * s, y - 42 * s], [x, y - 48 * s], [x + 14 * s, y - 42 * s], [x + 26 * s, y - 22 * s, 0.5]], { fill: "#d9534a" })
    outline(cap, 2.5, "#8f2a2c")
    clip(cap, () => {
      ellipse(x - 6 * s, y - 40 * s, 10 * s, 5 * s, { fill: "#ff8a74", opacity: 0.7 })
      circle(x - 12 * s, y - 32 * s, 4 * s, { fill: "#fff3e0" })
      circle(x + 8 * s, y - 38 * s, 3.5 * s, { fill: "#fff3e0" })
      circle(x + 15 * s, y - 27 * s, 3 * s, { fill: "#fff3e0" })
    })
  }
  shroom(770, 905, 1.1)
  shroom(805, 915, 0.75)
  shroom(255, 915, 0.8)
  // hojas grandes en las esquinas
  const leaf = (bx, by, tx, ty, col) => {
    const mx = (bx + tx) / 2, my = (by + ty) / 2, dx = ty - by, dy = bx - tx, k = 0.28
    const l = blob([[bx, by, 0], [mx + dx * k, my + dy * k], [tx, ty, 0], [mx - dx * k, my - dy * k]], { fill: col })
    outline(l, 2.5, shade(col, -0.45))
    ink([[bx, by], [mx + dx * 0.04, my + dy * 0.04], [tx, ty]], [3, 2, 0.5], shade(col, -0.35), [0, 0.5])
  }
  leaf(-10, 1040, 120, 880, "#3f6a3a")
  leaf(20, 1050, 190, 950, "#527f46")
  leaf(-20, 990, 60, 860, "#4a7640")
  leaf(1040, 1040, 900, 880, "#3f6a3a")
  leaf(1010, 1050, 840, 960, "#527f46")
  leaf(1040, 990, 960, 850, "#4a7640")
})

layer("acabado", () => {
  // viñeta suave
  rect(0, 0, 1024, 1024, { fill: radial(560, 460, 720, [alpha("#2a2448", 0), [0.7, alpha("#2a2448", 0)], alpha("#2a2448", 0.45)]), blend: "multiply" })
  grain({ opacity: 0.08, size: 1, blend: "overlay" })
})
