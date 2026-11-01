// dibujo.js: tortuga ninja original sentada en un muro bajo el cielo azul.
canvas({ width: 720, height: 1280, background: linear(0, 0, 0, 1100, ["#11679a", "#1c86bd", "#2a93c6"]) })

const P = {
  ink: "#1f2a1a",
  skin: "#6d9a43", skinD: "#46692f", skinL: "#97bd5c",
  plast: "#cfae6a", plastD: "#a17f47",
  pad: "#7a5236", padD: "#4c3222", padL: "#9c7350",
  red: "#c9302c", redD: "#86201f",
  brick: "#c9714d", brickD: "#9a4f36",
  stone: "#e6d6b0", stoneD: "#b8a682",
  plaster: "#f1e2b8", plasterD: "#cdb98a",
  steel: "#c9ced2",
}
const S = (extra = {}) => ({ stroke: P.ink, width: 2.6, join: "round", ...extra })
seed(7)

layer("muro", () => {
  // yeso
  const wall = rect(0, 1085, 720, 200, { fill: P.plaster })
  clip(wall, () => {
    rect(0, 1085, 720, 30, { fill: P.plasterD, opacity: 0.5 })
    // grietas
    const cracks = [
      [[20, 1120], [60, 1150], [55, 1190], [100, 1230], [130, 1280]],
      [[150, 1110], [175, 1160], [160, 1200], [200, 1250]],
      [[260, 1140], [300, 1175], [350, 1170], [380, 1210]],
      [[600, 1120], [640, 1150], [630, 1200], [690, 1230]],
      [[0, 1210], [40, 1220], [70, 1270]],
      [[560, 1240], [600, 1255], [650, 1250], [700, 1275]],
      [[680, 1120], [700, 1160], [720, 1170]],
    ]
    // desconchones
    for (const [cx, cy, r] of [[60, 1160, 14], [175, 1180, 10], [320, 1172, 12], [640, 1180, 15], [690, 1240, 11], [40, 1230, 9], [250, 1250, 8]]) {
      blob(roughen(ring(cx, cy, r * 1.6, r, 9, rand(-30, 30)), 4), { fill: P.plasterD, stroke: alpha(P.ink, 0.6), width: 1.2 })
    }
    for (const c of cracks) brush(jitter(subdivide(c, 25), 4), { width: [0.6, 2, 0.8], color: alpha(P.ink, 0.75) })
    for (let i = 0; i < 40; i++) {
      const x = rand(0, 720), y = rand(1120, 1280)
      brush([[x, y], [x + rand(-8, 8), y + rand(3, 9)]], { width: 1.2, color: alpha(P.ink, 0.5) })
    }
    // sombra bajo la hilada
    rect(0, 1085, 720, 14, { fill: "#6b5a3a", opacity: 0.35, blend: "multiply", blur: 4 })
  })
  // hilada de ladrillos claros
  let x = -20
  const rows = [[1043, 22], [1064, 22]]
  for (const [y, h] of rows) {
    x = -rand(0, 40)
    while (x < 720) {
      const w = rand(70, 115)
      rect(x + 2, y, w - 4, h, S({ fill: mix(P.stone, P.stoneD, rand(0, 0.5)), r: 2, width: 2 }))
      rect(x + 4, y + h - 6, w - 8, 5, { fill: P.stoneD, opacity: 0.6 })
      x += w
    }
  }
  // remates rotos bajo la hilada
  for (let i = 0; i < 10; i++) {
    const cx = rand(0, 720)
    brush(jitter([[cx, 1088], [cx + 12, 1098], [cx + 6, 1112], [cx + 18, 1124]], 3), { width: [0.8, 2, 0.6], color: alpha(P.ink, 0.7) })
  }
  // albardilla de ladrillo rojo
  x = -rand(0, 30)
  while (x < 720) {
    const w = rand(65, 110)
    const light = rand() < 0.18
    const col = light ? P.stone : mix(P.brick, P.brickD, rand(0, 0.35))
    poly([[x + 2, 1018 + rand(-2, 2)], [x + w - 2, 1018 + rand(-2, 2)], [x + w - 1, 1044], [x + 1, 1044]], S({ fill: col, width: 2.2 }))
    rect(x + 4, 1018, w - 8, 5, { fill: shade(col, 0.35), opacity: 0.7 })
    rect(x + 4, 1036, w - 8, 7, { fill: shade(col, -0.35), opacity: 0.6 })
    x += w
  }
  // piedrecitas sobre el muro
  blob(roughen(ring(232, 1005, 20, 14, 9), 3), S({ fill: "#b49a6c" }))
  blob([[220, 1000], [235, 994], [244, 1006], [228, 1012]], { fill: shade("#b49a6c", 0.4) })
  blob(roughen(ring(664, 1005, 22, 15, 9), 3), S({ fill: "#a88862" }))
  circle(658, 1000, 3, { fill: P.red, opacity: 0.6 })
})

layer("sombras de los pies", () => {
  blob([[408, 1150], [438, 1160], [446, 1215], [436, 1262], [410, 1262], [402, 1205]], { fill: "#7f93a3", opacity: 0.85, blur: 1 })
  blob([[504, 1165], [536, 1172], [546, 1225], [536, 1272], [508, 1270], [500, 1215]], { fill: "#7f93a3", opacity: 0.85, blur: 1 })
})

layer("tortuga", () => {
  const skinShade = (shape, pts) => clip(shape, () => {
    blob(pts, { fill: P.skinD, opacity: 0.85 })
  })
  // caparazón detrás de los hombros
  const shell = blob([[395, 830], [420, 790], [475, 775], [530, 790], [560, 830], [545, 860], [405, 860]], S({ fill: P.pad }))
  clip(shell, () => {
    brush([[405, 815], [440, 795], [475, 790], [512, 795], [548, 815]], { width: 3, color: P.padD })
    rect(380, 770, 200, 22, { fill: P.padL, opacity: 0.5 })
  })

  // cuello
  blob([[445, 785], [505, 785], [515, 835], [435, 835]], S({ fill: P.skin }))
  // cabeza
  const head = blob([[473, 700], [505, 708], [520, 740], [515, 778], [497, 800], [473, 805], [449, 800], [431, 778], [427, 740], [441, 708]], S({ fill: P.skin, width: 3 }))
  clip(head, () => {
    blob([[500, 704], [530, 740], [522, 790], [480, 812], [496, 790], [506, 750]], { fill: P.skinD, opacity: 0.8 })
    blob([[430, 785], [473, 798], [515, 785], [510, 815], [430, 815]], { fill: P.skinD, opacity: 0.5 })
    ellipse(455, 715, 18, 9, { fill: P.skinL, opacity: 0.8, rotate: -20 })
    // cinta
    poly([[420, 727], [528, 727], [528, 752], [420, 752]], { fill: P.red })
    rect(420, 727, 108, 5, { fill: shade(P.red, 0.3) })
    rect(420, 747, 108, 5, { fill: P.redD })
  })
  // ojos con ceño
  mirror(473, () => {
    blob([[452, 737, 0], [465, 742], [466, 748], [455, 748], [448, 742]], { fill: "#fbf3df", stroke: P.ink, width: 1.5 })
    brush([[444, 732], [458, 737], [469, 742]], { width: 3, color: P.redD, taper: [0.1, 0.5] })
  })
  brush([[455, 784], [470, 781], [488, 782], [494, 779]], { width: [1.5, 3, 1.5], color: P.ink })
  brush([[488, 788], [492, 785]], { width: 2, color: P.ink })
  brush([[466, 770], [470, 772]], { width: 2, color: P.ink })
  brush([[478, 770], [482, 772]], { width: 2, color: P.ink })
  // colas de la cinta al viento
  blob([[518, 735], [560, 728], [600, 730, 0.5], [622, 720, 0], [610, 735], [572, 742], [525, 750]], S({ fill: P.red, width: 2 }))
  blob([[520, 745], [555, 752], [590, 770, 0.5], [606, 790, 0], [580, 776], [548, 765], [520, 758]], S({ fill: P.red, width: 2 }))
  brush([[540, 740], [580, 737], [608, 728]], { width: 2, color: P.redD })
  brush([[540, 758], [575, 768], [598, 782]], { width: 2, color: P.redD })

  // torso
  const torso = blob([[365, 875], [378, 835], [420, 815], [475, 828], [530, 815], [572, 835], [585, 875], [572, 925], [540, 960], [475, 968], [410, 960], [378, 925]], S({ fill: P.skin, width: 3 }))
  clip(torso, () => {
    blob([[530, 820], [580, 845], [585, 920], [545, 960], [555, 890]], { fill: P.skinD, opacity: 0.85 })
    blob([[380, 840], [415, 825], [410, 860], [385, 880]], { fill: P.skinL, opacity: 0.8 })
    // trapecios y pectorales asomando junto al plastrón
    brush([[430, 822], [452, 832], [462, 845]], { width: [1, 2.4, 1], color: P.ink })
    brush([[520, 822], [498, 832], [488, 845]], { width: [1, 2.4, 1], color: P.ink })
    ellipse(440, 828, 14, 5, { fill: P.skinL, opacity: 0.7, rotate: 15 })
  })
  // plastrón
  const plast = blob([[420, 838, 0.5], [475, 848], [530, 838, 0.5], [535, 895], [522, 950], [475, 962], [428, 950], [415, 895]], S({ fill: P.plast, width: 2.6 }))
  clip(plast, () => {
    blob([[505, 845], [540, 850], [535, 960], [495, 965], [512, 900]], { fill: P.plastD, opacity: 0.8 })
    rect(410, 838, 130, 14, { fill: shade(P.plast, 0.35), opacity: 0.7 })
    for (const y of [878, 914]) brush([[418, y], [450, y + 4], [475, y], [500, y + 4], [532, y]], { width: [1.5, 2.6, 1.5], color: P.ink })
    brush([[475, 850], [475, 962]], { width: [1.5, 2.6, 1.5], color: P.ink })
  })

  // cinturón
  blob([[378, 925], [475, 945], [572, 925], [578, 948], [475, 970], [372, 948]], S({ fill: P.pad }))
  // dagas envueltas en el cinto
  for (const [hx, dir] of [[440, -1], [508, 1]]) {
    line(hx, 880, hx - dir * 4, 950, { stroke: P.ink, width: 6, cap: "round" })
    line(hx, 880, hx - dir * 4, 945, { stroke: P.red, width: 4, cap: "round" })
    for (let k = 0; k < 6; k++) line(hx - 3 - dir * k * 0.4, 890 + k * 9, hx + 3 - dir * k * 0.4, 894 + k * 9, { stroke: P.redD, width: 1.5 })
    circle(hx, 878, 4, S({ fill: P.steel, width: 1.5 }))
    poly([[hx - dir * 4 - 14, 948], [hx - dir * 4 + 14, 948], [hx - dir * 4 + 10, 956], [hx - dir * 4 - 10, 956]], S({ fill: P.steel, width: 1.8 }))
  }

  // brazos: hombro → codo a los lados
  mirror(475, () => {
    const arm = blob([[372, 845], [395, 835], [410, 880], [400, 925], [375, 940], [355, 915], [358, 875]], S({ fill: P.skin, width: 3 }))
    clip(arm, () => {
      blob([[395, 880], [412, 870], [405, 935], [380, 945]], { fill: P.skinD, opacity: 0.7 })
      ellipse(372, 858, 12, 7, { fill: P.skinL, opacity: 0.8, rotate: -40 })
    })
    // deltoides y bíceps
    brush([[362, 880], [380, 868], [400, 872]], { width: [1, 2.4, 1], color: P.ink })
    brush([[372, 900], [385, 912]], { width: [1, 2, 1], color: P.skinD })
  })
  // codera
  mirror(475, () => {
    const ep = blob([[350, 905], [380, 895], [400, 910], [395, 940], [365, 948], [345, 930]], S({ fill: P.pad }))
    clip(ep, () => rect(340, 930, 70, 30, { fill: P.padD, opacity: 0.7 }))
  })

  // muslos (escorzo, hacia el espectador)
  const thigh = blob([[385, 955], [475, 965], [565, 955], [585, 1000], [560, 1035], [475, 1035], [390, 1035], [365, 1000]], S({ fill: P.skin, width: 3 }))
  clip(thigh, () => rect(360, 1010, 240, 40, { fill: P.skinD, opacity: 0.8 }))
  // taparrabos
  blob([[455, 950], [495, 950], [500, 985], [485, 1020], [465, 1020], [450, 985]], S({ fill: P.pad }))
  circle(475, 958, 10, S({ fill: P.padL }))
  brush([[470, 970], [466, 1012]], { width: 2, color: P.padD })
  brush([[482, 970], [486, 1012]], { width: 2, color: P.padD })

  // rodilleras
  for (const [kx, s] of [[392, -1], [558, 1]]) {
    const kp = ellipse(kx, 982, 50, 38, S({ fill: P.pad, width: 3 }))
    clip(kp, () => {
      ellipse(kx + s * 8, 1000, 45, 22, { fill: P.padD, opacity: 0.8 })
      ellipse(kx - 12, 965, 20, 9, { fill: P.padL, opacity: 0.8 })
    })
    brush([[kx - 40, 990], [kx, 1000], [kx + 40, 990]], { width: [1, 2.4, 1], color: P.ink })
  }

  // antebrazos hacia las rodillas, con vendas
  mirror(475, () => {
    const fa = blob([[372, 925], [400, 920], [440, 950], [452, 975], [428, 985], [385, 955]], S({ fill: P.skin, width: 3 }))
    clip(fa, () => blob([[395, 950], [440, 965], [430, 990], [380, 960]], { fill: P.skinD, opacity: 0.7 }))
    const wrap = blob([[420, 945], [445, 950], [455, 975], [430, 982], [415, 965]], S({ fill: P.pad, width: 2.2 }))
    clip(wrap, () => { for (let k = 0; k < 3; k++) brush([[418 + k * 6, 960 - k * 4], [440 + k * 5, 978 - k * 6]], { width: 1.5, color: P.padD }) })
  })
  // manos colgando sobre las rodillas (tres dedos)
  mirror(475, () => {
    // palma
    const hand = blob([[428, 972], [458, 966], [468, 990], [460, 1002], [430, 1002], [424, 988]], S({ fill: P.skin, width: 2.6 }))
    clip(hand, () => blob([[452, 970], [472, 985], [462, 1005], [448, 1003]], { fill: P.skinD, opacity: 0.8 }))
    // tres dedos gruesos colgando
    for (const [dx, len] of [[-12, 30], [0, 36], [12, 32]]) {
      const cx = 446 + dx
      const f = blob([[cx - 7, 996], [cx + 7, 996], [cx + 7, 996 + len - 5], [cx, 996 + len + 2], [cx - 7, 996 + len - 5]], S({ fill: P.skin, width: 2.2 }))
      clip(f, () => rect(cx + 2, 990, 10, 50, { fill: P.skinD, opacity: 0.8 }))
      brush([[cx - 4, 996 + len * 0.55], [cx + 4, 996 + len * 0.55]], { width: 1.2, color: P.skinD })
    }
    // pulgar
    blob([[424, 980], [436, 984], [436, 1010], [426, 1014], [420, 998]], S({ fill: P.skinL, width: 2 }))
  })

  // espinillas colgando delante del muro
  const leg = (top, ank, foot) => {
    const L = (t, dx) => [lerp(top[0], ank[0], t) + dx * lerp(1, 0.72, t), lerp(top[1], ank[1], t)]
    // espinilla con pantorrilla
    const sh = blob([L(0, -28), L(0, 26), L(0.35, 27), L(1, 20), L(1, -20), L(0.4, -25)], S({ fill: P.skin, width: 3 }))
    clip(sh, () => blob([L(0, 8), L(0, 40), L(1, 30), L(1, 4), L(0.5, 10)], { fill: P.skinD, opacity: 0.85 }))
    // vendas en tiras
    for (let k = 0; k < 6; k++) {
      const t = 0.38 + k * 0.11
      const [cx, cy] = L(t, 0)
      const w = 25 * lerp(1, 0.75, t)
      const b = blob([[cx - w - 1, cy - 5 + (k % 2) * 3], [cx + w + 1, cy - 8 + (k % 2) * 4], [cx + w + 1, cy + 6], [cx - w - 1, cy + 9]], S({ fill: k % 2 ? P.pad : P.padL, width: 1.8 }))
      clip(b, () => rect(cx + w * 0.2, cy - 12, w, 24, { fill: P.padD, opacity: 0.6 }))
    }
    // pie visto desde arriba, colgando
    const [fx, fy] = foot
    const ft = blob([[fx - 18, fy - 34], [fx + 18, fy - 34], [fx + 26, fy - 6], [fx + 22, fy + 16], [fx, fy + 22], [fx - 22, fy + 16], [fx - 26, fy - 6]], S({ fill: P.skin, width: 3 }))
    clip(ft, () => blob([[fx + 4, fy - 34], [fx + 30, fy - 10], [fx + 25, fy + 22], [fx + 2, fy + 22]], { fill: P.skinD, opacity: 0.85 }))
    brush([[fx - 6, fy + 2], [fx - 7, fy + 21]], { width: [1, 2.4, 1], color: P.ink })
    brush([[fx + 9, fy + 2], [fx + 10, fy + 19]], { width: [1, 2.2, 1], color: P.ink })
    ellipse(fx - 12, fy - 14, 7, 4, { fill: P.skinL, opacity: 0.8 })
    // tira de la sandalia
    blob([[fx - 24, fy - 28], [fx + 23, fy - 30], [fx + 25, fy - 18], [fx - 25, fy - 15]], S({ fill: P.pad, width: 2 }))
  }
  leg([395, 1015], [412, 1118], [416, 1150])
  leg([552, 1015], [516, 1125], [514, 1160])
})

layer("textura", () => {
  grain({ opacity: 0.07 })
})
