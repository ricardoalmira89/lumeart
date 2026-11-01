// dibujo.js: personaje con esqueleto (mira a la derecha).
// Lienzo de 720x1280: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 720, height: 1280, background: null })
setLight(-1, -1)

const P = {
  skin: "#f4b08a", skinD: "#d9876a", nose: "#f08868",
  hair: "#1d1b24",
  shirt: "#f2f1eb", shirtD: "#b9bfc4",
  tie: "#16161c",
  pants: "#d4ab74", pantsD: "#a98250",
  shoe: "#8a5236", shoeL: "#c99d8e",
  sock: "#e9ecee", sockS: "#7d8790",
  line: "#5a4a48",
}

// zonas de recorte (invisibles) para partir piezas en las articulaciones
const C = (x, y, r) => circle(x, y, r, { fill: "#000", hidden: true })
const R = (x, y, w, h) => rect(x, y, w, h, { fill: "#000", hidden: true })

// eje de la lanza en reposo: u a lo largo (+ hacia la punta) y v de través, desde el agarre.
// Se lleva casi horizontal, con la punta 10° arriba.
const LG = [392, 796], LA = -10 * Math.PI / 180
const LS = (u, v = 0, s) => {
  const p = [LG[0] + u * Math.cos(LA) - v * Math.sin(LA), LG[1] + u * Math.sin(LA) + v * Math.cos(LA)]
  return s === undefined ? p : [...p, s]
}
const L_TIP = 272, L_COLA = -318, L_OJAL = L_COLA + 14

// ---------- esqueleto ----------
rig({ origin: [425, 1144] })
bone("cadera", { at: [414, 590] })
bone("torso", { parent: "cadera", at: [395, 556] })
bone("cabeza", { parent: "torso", at: [432, 326] })
bone("pelo", { parent: "cabeza", at: [430, 250] })
bone("brazo_izq", { parent: "torso", at: [378, 398] })
bone("antebrazo_izq", { parent: "brazo_izq", at: [389, 565] })
bone("mano_izq", { parent: "antebrazo_izq", at: [404, 730] })
bone("brazo_der", { parent: "torso", at: [362, 398] })
bone("antebrazo_der", { parent: "brazo_der", at: [373, 565] })
bone("mano_der", { parent: "antebrazo_der", at: [388, 730] })
bone("lanza", { parent: "mano_der", at: LG })
bone("lanza_punta", { parent: "lanza", at: LS(L_TIP) })
bone("lanza_cola", { parent: "lanza", at: LS(L_OJAL) })
bone("muslo_izq", { parent: "cadera", at: [424, 600] })
bone("pierna_izq", { parent: "muslo_izq", at: [427, 810] })
bone("pie_izq", { parent: "pierna_izq", at: [425, 1090] })
bone("muslo_der", { parent: "cadera", at: [414, 600] })
bone("pierna_der", { parent: "muslo_der", at: [418, 820] })
bone("pie_der", { parent: "pierna_der", at: [388, 1092] })

// ---------- piezas reutilizables ----------
// brazo (manga entera + puño); c = colores, se recorta por partes
// agujero desgarrado con la piel asomando (puntos fijos: sin rand, para que coincida entre piezas)
function tornHole(pts, skin, skinD, rim) {
  const h = blob(pts, { fill: mix(skin, skinD, 0.3) })
  clip(h, () => {
    const b = bbox(h)
    rect(b.x - 2, b.y - 2, b.w + 4, b.h * 0.45, { fill: alpha(skinD, 0.7), blur: 2 })
  })
  // borde deshilachado: más marcado abajo, casi nada arriba
  brush([...pts.slice(3), pts[0]].map(p => [p[0], p[1]]), { width: [0.6, 1.4, 1], color: alpha(rim, 0.75), taper: [0.3, 0.3] })
  // hilos que cruzan el agujero
  const a = pts[1], b2 = pts[pts.length - 3], c2 = pts[2], d = pts[pts.length - 2]
  curve([[a[0], a[1]], [(a[0] + b2[0]) / 2 + 1, (a[1] + b2[1]) / 2 + 2], [b2[0], b2[1]]], { stroke: alpha(shade(rim, 0.5), 0.9), width: 0.8 })
  curve([[c2[0], c2[1]], [(c2[0] + d[0]) / 2, (c2[1] + d[1]) / 2 + 3], [d[0], d[1]]], { stroke: alpha(shade(rim, 0.5), 0.8), width: 0.7 })
}
const DIRT = "#b39a6a"
function armShape(c) {
  const sl = shade(c.shirtD, -0.25)
  // antebrazo al aire bajo la manga rota
  const fore = blob([[373, 686], [401, 684], [404, 736], [377, 738]], { fill: c.skin })
  clip(fore, () => {
    brush([[370, 712], [388, 720], [406, 712]], { width: 10, color: alpha(c.skinD, 0.6), blur: 3 })
    brush([[398, 690], [401, 736]], { width: 5, color: alpha(c.skinD, 0.5), blur: 2 })
  })
  const arm = blob([[356, 370], [380, 380], [392, 450], [394, 520], [398, 600], [402, 728], [374, 732], [366, 640], [350, 570], [338, 500], [336, 420]], { fill: c.shirt })
  const armF = freeform(arm, [[352, 400, c.hi, 0.5], [345, 500, c.shirt, 0.4], [386, 500, mix(c.shirt, c.shirtD, 0.4), 0.2], [362, 620, c.shirt, 0.4], [378, 700, c.shirt, 0.3], [396, 690, mix(c.shirt, c.shirtD, 0.4), 0.2]])
  // bajo de la manga arrancado en jirones
  cut(armF, [poly([[350, 780], [350, 704], [372, 699], [377, 714], [383, 702], [389, 722], [394, 706], [399, 716], [404, 697], [420, 700], [420, 780]], { fill: "#000", hidden: true })])
  clip(armF, () => {
    brush([[352, 560], [362, 580], [372, 590]], { width: 3, color: alpha(c.shirtD, 0.9) })
    brush([[362, 600], [372, 610]], { width: 2, color: alpha(c.shirtD, 0.9) })
    // manchas de sal y suciedad
    blob([[340, 430], [356, 420], [366, 436], [352, 452], [338, 446]], { fill: alpha(DIRT, 0.22), blur: 4 })
    blob([[366, 620], [386, 612], [396, 634], [380, 650], [368, 640]], { fill: alpha(DIRT, 0.25), blur: 5 })
    brush([[370, 690], [386, 696], [402, 690]], { width: 12, color: alpha(DIRT, 0.35), blur: 4 })
    // borde deshilachado
    brush([[372, 699], [377, 714], [383, 702], [389, 722], [394, 706], [399, 716], [404, 697]], { width: 2, color: alpha(c.shirtD, 0.9) })
  })
  // hilos sueltos colgando del jirón
  brush([[378, 712], [377, 721]], { width: 0.9, color: c.shirtD, taper: [0, 0.8] })
  brush([[389, 720], [391, 731]], { width: 0.9, color: c.shirtD, taper: [0, 0.8] })
  brush([[399, 715], [397, 724]], { width: 0.8, color: c.shirtD, taper: [0, 0.8] })
  // agujeros en la manga
  tornHole([[346, 462, 0], [352, 457], [358, 463, 0], [361, 470], [357, 479, 0], [350, 481], [345, 474, 0], [343, 468]], c.skin, c.skinD, sl)
  tornHole([[382, 640, 0], [388, 637], [391, 645, 0], [387, 652, 0], [381, 648]], c.skin, c.skinD, sl)
  brush([[336, 430], [340, 500], [352, 572], [366, 640], [372, 700]], { width: [1.5, 2.5, 2.5, 1.5], color: alpha(sl, 0.7) })
  brush([[382, 384], [392, 450], [396, 560], [403, 697]], { width: [1, 2, 2.5, 2], color: alpha(sl, 0.6) })
}
// mano de caricatura colgando relajada: dorso grande, tres dedos gruesos algo curvados,
// pulgar delante y uñas sucias. Sin rand: debe salir igual siempre.
function handShape(c) {
  const ink = mix(c.skinD, "#4a3028", 0.5)
  const nailC = "#d8c48a", nailD = "#8a7434"
  // uña sucia en la punta de un dedo
  const nail = (x, y, r) => {
    blob([[x - 2.5, y - 4], [x + 2, y - 4], [x + 3 + r, y + 2], [x + r * 0.8, y + 5, 0], [x - 2, y + 1]], { fill: nailC, stroke: nailD, width: 0.7 })
    brush([[x - 1.5, y + 1], [x + 1.5 + r, y + 2.5]], { width: 1, color: "#4a3a22", taper: [0.2, 0.2] })
  }
  // dedo (cápsula curvada) con contorno detrás para separarlo de la pierna
  const finger = (pts, col) => {
    brush(pts, { width: [14.5, 13.5, 12], color: ink, taper: [0, 0.25] })
    return brush(pts, { width: [12, 11, 9.5], color: col, taper: [0, 0.25] })
  }
  // pulgar: va por el lado de la palma (hacia el cuerpo), así que queda detrás
  // del dorso y solo asoma por delante, apuntando abajo y adelante
  const thumb = blob([[396, 740], [404, 743], [412, 758], [415, 774], [412, 784, 0.5], [406, 783], [401, 768], [396, 756]], { fill: mix(c.skin, c.skinD, 0.4), stroke: ink, width: 1.4 })
  clip(thumb, () => brush([[410, 752], [414, 770], [412, 782]], { width: 5, color: alpha(c.skinD, 0.7), blur: 2 }))
  nail(411, 780, 1)
  // dedos del fondo (meñique y anular), más oscuros
  const back = mix(c.skin, c.skinD, 0.45)
  finger([[379, 768], [377, 790], [380, 806]], back)
  nail(381, 805, 0)
  // dedo corazón
  finger([[387, 770], [386, 796], [391, 815]], mix(c.skin, c.skinD, 0.15))
  nail(392, 814, 1)
  // índice, delante
  finger([[396, 770], [397, 794], [403, 810]], c.skin)
  nail(404, 809, 2)
  // separación y pliegues de los dedos
  brush([[384, 780], [383, 798], [386, 808]], { width: 1.2, color: alpha(ink, 0.7), taper: [0.3, 0.3] })
  brush([[392, 782], [392, 798], [397, 808]], { width: 1.2, color: alpha(ink, 0.7), taper: [0.3, 0.3] })
  for (const [x, y] of [[379, 792], [387, 797], [397, 795]]) brush([[x - 4, y], [x, y + 1.5], [x + 4, y]], { width: 0.9, color: alpha(ink, 0.6) })
  // dorso de la mano
  const hand = blob([[377, 724], [400, 724], [406, 748], [404, 774], [390, 781], [374, 776], [372, 750]], { fill: c.skin, stroke: ink, width: 1.6, behind: true })
  clip(hand, () => {
    brush([[404, 732], [406, 754], [402, 776]], { width: 8, color: alpha(c.skinD, 0.6), blur: 3 })
    ellipse(380, 748, 5, 14, { fill: alpha(shade(c.skin, 0.35), 0.6), blur: 3 })
    blob([[376, 760], [396, 758], [400, 774], [380, 778]], { fill: alpha(DIRT, 0.25), blur: 3 })
  })
  // nudillos
  for (const [x, y] of [[379, 771], [388, 774], [398, 772]]) brush([[x - 4, y + 1], [x, y - 1.5], [x + 4, y + 1]], { width: 1.1, color: alpha(c.skinD, 0.9), taper: [0.3, 0.3] })
  // tendones y pelillos del dorso
  brush([[386, 740], [388, 766]], { width: 1, color: alpha(c.skinD, 0.5), taper: [0.4, 0.4] })
  brush([[394, 742], [396, 766]], { width: 1, color: alpha(c.skinD, 0.5), taper: [0.4, 0.4] })
  for (const [x, y] of [[382, 750], [390, 754], [384, 760], [395, 750]]) brush([[x, y], [x + 1.5, y + 3], [x, y + 5]], { width: 0.7, color: alpha(HAIR, 0.8), taper: [0.05, 0.8] })
  // peso de línea en el lado de la sombra
  brush([[406, 748], [404, 774]], { width: [1, 2, 1.2], color: alpha(ink, 0.8), taper: [0.3, 0.3] })
}
const armNear = { shirt: P.shirt, shirtD: P.shirtD, hi: "#ffffff", skin: P.skin, skinD: P.skinD }
const armFar = { shirt: shade(P.shirt, -0.22), shirtD: shade(P.shirtD, -0.25), hi: shade(P.shirt, -0.1), skin: mix(P.skin, P.skinD, 0.5), skinD: shade(P.skinD, -0.2) }

// pantalón corto roto (se recorta por cadera y muslo). Sin rand: se dibuja dos veces y debe coincidir.
function pantNear() {
  const pl = shade(P.pantsD, -0.2)
  const pant = blob([[380, 556], [444, 556, 0.5], [449, 640], [453, 712], [447, 724, 0], [439, 712], [431, 732, 0], [422, 715], [412, 729, 0], [404, 713], [395, 726, 0], [387, 708], [384, 660]], { fill: P.pants })
  const pantS = freeform(pant, [[395, 600, shade(P.pants, 0.12), 0.4], [442, 640, mix(P.pants, P.pantsD, 0.5), 0.2], [405, 690, P.pants, 0.5], [440, 715, mix(P.pants, P.pantsD, 0.6), 0.2]])
  clip(pantS, () => {
    brush([[440, 580], [445, 650], [446, 712]], { width: 8, color: alpha(P.pantsD, 0.6), blur: 2 })
    // manchas de agua salada
    blob([[392, 620], [414, 612], [420, 634], [400, 644]], { fill: alpha(DIRT, 0.35), blur: 4 })
    brush([[388, 704], [420, 712], [450, 706]], { width: 12, color: alpha(shade(P.pantsD, -0.2), 0.45), blur: 4 })
    // remiendo cosido a mano
    rect(400, 662, 18, 16, { fill: mix(P.pantsD, "#7a8a8e", 0.4), rotate: -8 })
    for (let i = 0; i < 4; i++) line(399 + i * 6, 660, 401 + i * 6, 666, { stroke: "#3a3028", width: 0.8 })
  })
  tornHole([[428, 630, 0], [434, 626], [440, 632, 0], [440, 640], [436, 647, 0], [429, 646], [426, 640, 0], [425, 635]], P.skin, P.skinD, pl)
  // hilos colgando del bajo
  for (const [x, y, l] of [[400, 718, 9], [416, 724, 12], [428, 726, 8], [443, 718, 10]]) brush([[x, y], [x + 1, y + l * 0.6], [x - 1, y + l]], { width: 0.9, color: shade(P.pantsD, -0.1), taper: [0, 0.8] })
  brush([[446, 560], [449, 640], [452, 710]], { width: [2, 3, 2.5], color: alpha(pl, 0.8) })
  brush([[453, 712], [447, 724], [439, 712], [431, 732], [422, 715], [412, 729], [404, 713], [395, 726], [387, 708]], { width: [1.5, 2, 1.5], color: alpha(pl, 0.8), taper: [0.1, 0.1] })
}
const fp = mix(P.pants, P.pantsD, 0.35), fpD = mix(P.pantsD, "#6e5a48", 0.3)
function pantFar() {
  const pantF = blob([[396, 566], [446, 566], [452, 690], [447, 706, 0], [440, 694], [432, 715, 0], [424, 699], [414, 711, 0], [405, 696], [399, 680]], { fill: fp })
  clip(pantF, () => {
    brush([[444, 600], [448, 660], [450, 700]], { width: 7, color: alpha(fpD, 0.7), blur: 2 })
    brush([[400, 692], [450, 696]], { width: 10, color: alpha(fpD, 0.5), blur: 4 })
  })
  brush([[448, 640], [452, 690]], { width: [2, 3, 2], color: alpha(shade(fpD, -0.2), 0.8) })
  brush([[452, 690], [447, 706], [440, 694], [432, 715], [424, 699], [414, 711], [405, 696], [399, 680]], { width: 1.5, color: alpha(shade(fpD, -0.2), 0.8), taper: [0.1, 0.1] })
}

// piernas desnudas y peludas
const HAIR = "#2a2228"
// pelos rizados a lo largo de un tramo de A a B (w = ancho a cada lado)
function legHair(ax, ay, bx, by, wa, wb, n, col) {
  for (let i = 0; i < n; i++) {
    const t = rand(), x0 = lerp(ax, bx, t), y0 = lerp(ay, by, t), w = lerp(wa, wb, t)
    const x = x0 + rand(-1, 1.15) * w, y = y0
    const l = rand(4, 13), s = pick([-1, 1]), dx = rand(-5, 3)
    const kind = rand()
    const pts = kind < 0.6
      ? [[x, y], [x + dx * 0.5 + s * 1.8, y + l * 0.5], [x + dx, y + l]] // pelo suelto algo curvado
      : [[x, y], [x + s * l * 0.3, y + l * 0.4], [x + dx * 0.3, y + l * 0.75], [x + s * l * 0.35, y + l * 0.9]] // rizo
    brush(pts, { width: rand(0.6, 1.3), color: alpha(col, rand(0.55, 0.95)), taper: [0.05, 0.8] })
  }
}
function skinLeg(pts, c, shadowLine) {
  const leg = blob(pts, { fill: c.skin })
  clip(leg, () => {
    brush(shadowLine, { width: 9, color: alpha(c.skinD, 0.65), blur: 3 })
  })
  return leg
}

// ---------- brazo lejano (detrás de todo, más oscuro) ----------
const FAR = 16
part("brazo_izq", () => {
  clip([R(300, 300, 160, 265), C(373 + FAR, 565, 26)], () => group({ move: [FAR, 0] }, () => armShape(armFar)))
})
part("mano_izq", () => group({ move: [FAR, 0] }, () => handShape(armFar)))
part("antebrazo_izq", () => {
  clip([C(373 + FAR, 565, 26), R(300, 565, 180, 200)], () => group({ move: [FAR, 0] }, () => armShape(armFar)))
})

// ---------- pierna lejana ----------
const legFar = { skin: mix(P.skin, P.skinD, 0.45), skinD: shade(P.skinD, -0.2) }
part("muslo_izq", () => {
  seed(31)
  skinLeg([[405, 670], [446, 670], [444, 760], [441, 800], [440, 815], [432, 828], [420, 828], [413, 815], [411, 780], [407, 730]], legFar, [[446, 690], [442, 780], [436, 826]])
  legHair(426, 700, 427, 815, 16, 11, 40, shade(HAIR, -0.1))
  clip([C(424, 600, 34), R(380, 600, 90, 210), C(427, 810, 26)], pantFar)
})
part("pierna_izq", () => {
  seed(32)
  skinLeg([[414, 800], [434, 797], [443, 812], [441, 850], [435, 910], [430, 990], [434, 1040], [434, 1094], [417, 1094], [416, 1040], [410, 990], [410, 930], [413, 870], [411, 830]], legFar, [[442, 820], [436, 910], [432, 1000], [434, 1090]])
  brush([[434, 806], [442, 818], [440, 834]], { width: 1.5, color: legFar.skinD, taper: [0.3, 0.3] })
  legHair(426, 840, 424, 1050, 13, 8, 70, shade(HAIR, -0.1))
})
// zapato gastado con la silueta entera: agujero en la puntera por el que sale el dedo gordo
// con su uña larga, y la suela abierta por delante. Coordenadas del pie cercano.
function brokenShoe(c) {
  const ln = c.sole
  const shoe = blob([[364, 1112, 0.5], [394, 1104], [425, 1116], [470, 1118], [489, 1128], [484, 1143, 0.5], [362, 1144, 0.5], [361, 1128]], { fill: c.shoe })
  clip(shoe, () => {
    ellipse(462, 1128, 30, 14, { fill: c.shoeL })
    ellipse(430, 1121, 14, 4, { fill: "#ffffff30", blur: 2 })
    rect(355, 1138, 140, 8, { fill: ln })
    // suela abierta por la punta: una boca oscura entre cuero y suela
    blob([[452, 1138], [472, 1135], [486, 1133], [490, 1137], [478, 1140], [458, 1140]], { fill: "#140a08" })
    brush([[456, 1140], [474, 1141], [488, 1138]], { width: 1.2, color: shade(c.shoe, 0.25), taper: [0.3, 0.3] })
    // rozaduras, grietas y mugre
    brush([[380, 1130], [392, 1126], [400, 1132]], { width: 1, color: alpha(ln, 0.7) })
    brush([[436, 1118], [440, 1126], [446, 1128]], { width: 1, color: alpha(ln, 0.7) })
    brush([[472, 1126], [478, 1131], [476, 1136]], { width: 1, color: alpha(ln, 0.7) })
    blob([[364, 1118], [384, 1114], [388, 1130], [368, 1134]], { fill: alpha(DIRT, 0.3), blur: 3 })
  })
  brush([[489, 1130], [484, 1143], [364, 1144]], { width: 2.5, color: ln, taper: [0.2, 0.1] })
  // agujero en la puntera
  blob([[450, 1122, 0], [457, 1118], [463, 1120, 0], [470, 1118], [477, 1123, 0], [470, 1128], [458, 1128]], { fill: "#1c100c" })
  // dedo gordo que asoma hacia arriba y adelante
  const toe = blob([[455, 1125], [455, 1114], [461, 1106], [470, 1104], [477, 1109], [476, 1119], [471, 1126]], { fill: c.skin })
  clip(toe, () => {
    brush([[472, 1106], [477, 1114], [474, 1124]], { width: 5, color: alpha(c.skinD, 0.7), blur: 2 })
    blob([[456, 1116], [466, 1112], [470, 1122], [458, 1124]], { fill: alpha(DIRT, 0.45), blur: 2 })
  })
  brush([[459, 1113], [465, 1114]], { width: 1, color: c.skinD })
  brush([[458, 1117], [464, 1119]], { width: 1, color: c.skinD })
  brush([[462, 1108], [459, 1101], [462, 1097]], { width: 0.8, color: HAIR, taper: [0.05, 0.8] })
  brush([[458, 1112], [453, 1106], [455, 1101]], { width: 0.8, color: HAIR, taper: [0.05, 0.8] })
  // uña larga, amarilla y curvada que sale de la punta del dedo
  const nail = blob([[468, 1104], [476, 1102], [486, 1104], [494, 1110], [497, 1119, 0], [489, 1112], [480, 1110], [474, 1110]], { fill: c.nail, stroke: c.nailD, width: 1 })
  clip(nail, () => {
    brush([[471, 1104], [482, 1105], [491, 1110]], { width: 1.8, color: "#ffffff70", blur: 0.5 })
    blob([[476, 1108], [486, 1108], [494, 1115], [484, 1112]], { fill: alpha(c.nailD, 0.6), blur: 1 })
    line(481, 1104, 482, 1109, { stroke: alpha(c.nailD, 0.8), width: 0.7 })
  })
  brush([[470, 1110], [480, 1111]], { width: 1.3, color: c.grime, taper: [0.2, 0.2] })
  // bordes del cuero rasgado, por encima de la base del dedo
  brush([[449, 1122], [453, 1126], [458, 1124], [462, 1129], [468, 1126], [472, 1130], [478, 1124]], { width: [1.5, 2.2, 1.5], color: ln, taper: [0.1, 0.1] })
  blob([[451, 1123, 0], [457, 1127], [460, 1123, 0], [458, 1130], [452, 1129]], { fill: c.shoe })
  blob([[466, 1126, 0], [473, 1129], [478, 1124, 0], [476, 1131], [468, 1131]], { fill: shade(c.shoeL, -0.1) })
  // agujero en el empeine con el calcetín asomando
  blob([[401, 1121, 0], [408, 1117], [414, 1120], [412, 1127, 0], [404, 1128]], { fill: mix(c.sockS, "#2a2228", 0.35), stroke: ln, width: 1.2 })
  brush([[404, 1121], [410, 1125]], { width: 1.2, color: alpha(c.sock, 0.6) })
  brush([[456, 1121], [449, 1126], [458, 1130], [452, 1137]], { width: [1.5, 2, 1.5], color: ln, taper: [0.1, 0.1] })
  brush([[364, 1112], [362, 1130], [364, 1144]], { width: 2.5, color: ln, taper: [0.1, 0.2] })
  // cordones
  curve([[400, 1110], [412, 1096], [432, 1088], [430, 1102], [412, 1110]], { stroke: P.line, width: 1.5 })
  curve([[402, 1110], [396, 1094], [404, 1088], [408, 1104]], { stroke: P.line, width: 1.5 })
}
// calcetín con el borde deshilachado y agujeros (coordenadas del pie cercano)
function brokenSock(c) {
  const sock = blob([[377, 1070, 0], [382, 1077], [387, 1069, 0], [392, 1076], [399, 1070, 0], [404, 1114], [366, 1116], [369, 1094]], { fill: c.sock })
  clip(sock, () => {
    for (let i = 0; i < 4; i++) rect(360, 1081 + i * 9, 50, 4, { fill: c.sockS, rotate: -6 })
    blob([[366, 1096], [384, 1092], [400, 1108], [372, 1114]], { fill: alpha(DIRT, 0.35), blur: 3 })
  })
  tornHole([[383, 1084, 0], [389, 1081], [395, 1086, 0], [394, 1093], [388, 1096, 0], [382, 1091]], c.skin, c.skinD, c.sockS)
  tornHole([[368, 1100, 0], [374, 1098], [377, 1104, 0], [372, 1110], [367, 1107]], c.skin, c.skinD, c.sockS)
  brush([[386, 1072], [385, 1079]], { width: 0.8, color: c.sockS, taper: [0, 0.8] })
}
const footFar = { shoe: shade(P.shoe, -0.15), shoeL: shade(P.shoeL, -0.15), sole: "#2e1912", skin: legFar.skin, skinD: legFar.skinD, nail: "#bfae6e", nailD: "#6f5e2c", grime: "#3a2e1c", sock: shade(P.sock, -0.12), sockS: shade(P.sockS, -0.1) }
const footNear = { shoe: P.shoe, shoeL: P.shoeL, sole: "#3a2018", skin: P.skin, skinD: P.skinD, nail: "#dcc67e", nailD: "#8a7434", grime: "#4a3a22", sock: P.sock, sockS: P.sockS }
part("pie_izq", () => {
  group({ move: [37, -3] }, () => { brokenSock(footFar); brokenShoe(footFar) })
})

// ---------- cadera: parte alta del pantalón ----------
part("cadera", () => {
  clip(R(360, 545, 100, 70), pantNear)
})

// ---------- pierna cercana ----------
const legNear = { skin: P.skin, skinD: P.skinD }
part("muslo_der", () => {
  seed(21)
  const th = skinLeg([[393, 676], [442, 676], [440, 740], [434, 790], [435, 820], [427, 836], [412, 838], [402, 826], [400, 790], [395, 740]], legNear, [[442, 690], [438, 760], [432, 830]])
  clip(th, () => {
    brush([[396, 700], [399, 770]], { width: 6, color: alpha(shade(P.skin, 0.3), 0.6), blur: 3 })
    // sombra del bajo del pantalón sobre el muslo
    brush([[390, 722], [420, 734], [446, 726]], { width: 14, color: alpha(shade(P.skinD, -0.15), 0.6), blur: 4 })
  })
  brush([[436, 730], [434, 790], [435, 820]], { width: [1, 2, 1.5], color: alpha(shade(P.skinD, -0.2), 0.7) })
  legHair(418, 718, 418, 825, 18, 13, 60, HAIR)
  clip([C(414, 600, 34), R(340, 600, 130, 220), C(418, 820, 26)], pantNear)
})
part("pierna_der", () => {
  seed(22)
  const sh = skinLeg([[406, 808], [424, 803], [436, 816], [433, 842], [427, 870], [419, 925], [404, 990], [398, 1040], [397, 1096], [380, 1096], [381, 1040], [378, 990], [381, 930], [392, 872], [401, 834]], legNear, [[434, 826], [424, 900], [404, 990], [397, 1090]])
  clip(sh, () => {
    // luz en el gemelo y en la rodilla
    brush([[390, 880], [384, 940], [382, 1000]], { width: 6, color: alpha(shade(P.skin, 0.3), 0.6), blur: 3 })
    circle(424, 820, 7, { fill: alpha(shade(P.skin, 0.25), 0.6), blur: 2 })
  })
  // rodilla huesuda, tibia y tobillo
  brush([[414, 808], [430, 812], [434, 828], [428, 840]], { width: [1, 1.8, 1.2], color: P.skinD, taper: [0.3, 0.3] })
  brush([[427, 870], [419, 925], [404, 990], [398, 1040]], { width: [1.5, 2.2, 1.5], color: alpha(shade(P.skinD, -0.2), 0.7) })
  brush([[392, 872], [381, 930], [378, 990]], { width: [1, 1.6, 1], color: alpha(P.skinD, 0.6) })
  brush([[394, 1045], [395, 1080]], { width: 3, color: P.skinD, taper: [0.2, 0.2] })
  legHair(412, 845, 392, 1050, 15, 9, 100, HAIR)
})
part("pie_der", () => {
  brokenSock(footNear)
  brokenShoe(footNear)
})

// ---------- torso: camisa, cinturón, corbata ----------
part("torso", () => {
  const shirt = blob([[432, 322], [404, 332], [372, 348], [350, 372], [338, 420], [334, 500], [338, 560], [380, 566], [430, 552], [444, 530], [438, 504], [426, 488], [425, 400], [430, 336]], { fill: P.shirt })
  freeform(shirt, [[365, 380, "#ffffff", 0.5], [410, 345, P.shirt, 0.4], [350, 500, P.shirt, 0.4], [400, 470, P.shirt, 0.4], [425, 530, mix(P.shirt, P.shirtD, 0.5), 0.2], [380, 555, mix(P.shirt, P.shirtD, 0.3), 0.3], [415, 420, shade(P.shirt, -0.04), 0.3]])
  clip(shirt, () => {
    brush([[396, 340], [412, 360], [416, 400]], { width: 3, color: alpha(P.shirtD, 0.8) })
    brush([[380, 470], [400, 500], [410, 530]], { width: 4, color: alpha(P.shirtD, 0.5) })
    // manchas de sal y suciedad
    blob([[398, 380], [420, 372], [428, 396], [410, 410], [396, 400]], { fill: alpha(DIRT, 0.22), blur: 5 })
    blob([[400, 520], [430, 512], [444, 540], [414, 556], [398, 544]], { fill: alpha(DIRT, 0.3), blur: 5 })
    blob([[360, 350], [384, 342], [390, 360], [368, 370]], { fill: alpha(DIRT, 0.2), blur: 4 })
  })
  const sl = shade(P.shirtD, -0.25)
  // desgarrones en el pecho
  tornHole([[404, 440, 0], [411, 436], [418, 443, 0], [416, 452], [419, 461, 0], [410, 463], [403, 456, 0], [405, 449]], P.skin, P.skinD, sl)
  // rasgón en diagonal con la tela levantada
  // abertura oscura del rasgón
  blob([[398, 494, 0], [409, 486], [420, 480, 0], [414, 490], [403, 498]], { fill: mix(P.skin, P.skinD, 0.7) })
  // solapa de tela que cuelga, con su sombra
  blob([[403, 498, 0], [414, 490], [420, 480, 0], [419, 492], [412, 503, 0], [408, 499], [404, 504, 0]], { fill: shade(P.shirt, -0.08) })
  brush([[420, 480], [419, 492], [412, 503], [408, 499], [404, 504]], { width: [0.8, 1.4, 1], color: alpha(sl, 0.8), taper: [0.2, 0.2] })
  brush([[398, 494], [409, 486], [420, 480]], { width: [1, 1.5, 0.6], color: alpha(shade(P.skinD, -0.2), 0.8), taper: [0.2, 0.3] })
  brush([[404, 332], [372, 346], [348, 372], [337, 420], [334, 500], [338, 562]], { width: [1, 2, 2.5, 2.5], color: alpha(sl, 0.7) })
  // faldón roto que se sale del cinturón
  const tail = blob([[398, 552], [440, 545], [447, 560], [440, 574, 0], [432, 564], [424, 582, 0], [415, 566], [406, 577, 0], [397, 562]], { fill: shade(P.shirt, -0.05) })
  clip(tail, () => {
    rect(390, 540, 60, 12, { fill: alpha(P.shirtD, 0.6), blur: 3 })
    blob([[404, 560], [430, 556], [436, 572], [410, 578]], { fill: alpha(DIRT, 0.3), blur: 3 })
  })
  brush([[447, 560], [440, 574], [432, 564], [424, 582], [415, 566], [406, 577], [397, 562]], { width: [1, 1.6, 1.2], color: alpha(sl, 0.8), taper: [0.1, 0.1] })
  brush([[424, 582], [425, 590]], { width: 0.8, color: P.shirtD, taper: [0, 0.8] })
  // cinturón por encima del faldón
  brush([[382, 560], [412, 556], [442, 547]], { width: 6, color: shade(P.pantsD, -0.1), taper: [0.1, 0.3] })
  brush([[428, 330], [426, 400], [424, 470], [426, 500], [440, 520], [444, 545], [445, 586]], { width: [5, 4, 4, 5, 6, 6], color: P.tie, taper: [0.05, 0.02] })
})

// ---------- cabeza y cuello largo ----------
part("cabeza", () => {
  const kl = shade(P.skinD, -0.2)
  const head = blob([[440, 196], [470, 170], [500, 174], [506, 200], [507, 224], [510, 246], [514, 251, 0.5], [506, 255], [509, 262, 0.5], [498, 271], [474, 292], [446, 318], [432, 340, 0.5], [424, 334], [428, 300], [438, 272], [426, 236]], { fill: P.skin })
  freeform(head, [[496, 215, shade(P.skin, 0.18), 0.4], [470, 180, P.skin, 0.3], [480, 270, P.skin, 0.3], [460, 290, P.skin, 0.3], [436, 310, mix(P.skin, P.skinD, 0.6), 0.3], [450, 262, mix(P.skin, P.skinD, 0.5), 0.2]])
  clip(head, () => {
    brush([[500, 266], [476, 282], [458, 300]], { width: 7, color: alpha(P.skinD, 0.55), blur: 2 })
  })
  brush([[509, 258], [509, 263], [504, 268], [498, 271]], { width: 2, color: P.skinD, taper: [0.2, 0.3] })

  // boca: dientes de conejo torcidos que no le caben
  const T = { t: "#f1e6c2", d: "#c8b078", st: "#a88d52", ln: mix(P.skinD, "#4a3028", 0.45) }
  // labio inferior algo caído y su sombra
  brush([[497, 258], [504, 260], [509, 261]], { width: 3, color: alpha(mix(P.skinD, P.nose, 0.4), 0.8), taper: [0.4, 0.2] })
  // paleta trasera, más amarilla y torcida hacia atrás
  const t2 = blob([[502, 248], [508, 248], [509, 257], [507, 262, 0.5], [502, 261, 0.5], [501, 254]], { fill: mix(T.t, T.d, 0.45), stroke: T.ln, width: 1 })
  clip(t2, () => rect(498, 256, 14, 8, { fill: alpha(T.st, 0.6), blur: 1 }))
  // paleta grande, salida hacia delante y abajo, montada sobre el labio
  const t1 = blob([[507, 247], [514, 247], [520, 258], [520, 266, 0.5], [514, 267, 0.5], [510, 257]], { fill: T.t, stroke: T.ln, width: 1.2 })
  clip(t1, () => {
    rect(500, 261, 26, 8, { fill: alpha(T.d, 0.7), blur: 1 })
    brush([[511, 250], [513, 259]], { width: 2, color: "#ffffffb0", blur: 0.5 })
    line(515, 253, 517, 260, { stroke: alpha(T.st, 0.7), width: 0.8 })
  })
  // labio superior que tapa la raíz de los dientes (la nariz lo cubre por arriba)
  blob([[504, 245], [512, 243], [516, 249, 0.5], [512, 252], [502, 251], [499, 248]], { fill: linear(0, 245, 0, 252, [P.skin, mix(P.skin, P.skinD, 0.2)]) })
  brush([[516, 249], [510, 252], [502, 251], [494, 253]], { width: [1.6, 1.8, 1.2], color: P.skinD, taper: [0.2, 0.4] })
  // colmillo inferior torcido que asoma en la comisura
  const fang = blob([[496, 257], [501, 257], [500, 251, 0.5], [499, 247, 0], [496, 251]], { fill: mix(T.t, T.d, 0.25), stroke: T.ln, width: 1 })
  clip(fang, () => rect(494, 254, 10, 4, { fill: alpha(T.d, 0.7) }))
  // comisura caída y pliegue de la mejilla
  brush([[494, 252], [490, 256], [489, 259]], { width: [1.5, 1], color: P.skinD, taper: [0.1, 0.6] })
  brush([[488, 244], [486, 252]], { width: 1.2, color: alpha(P.skinD, 0.6), taper: [0.4, 0.4] })
  brush([[440, 270], [430, 300], [428, 326]], { width: [1, 2], color: alpha(kl, 0.7) })
  brush([[498, 268], [474, 292], [446, 318]], { width: [2, 1.5], color: alpha(kl, 0.7) })

  // ceja y ojo cerrado
  brush([[493, 188], [500, 185], [508, 187]], { width: 3, color: P.hair, taper: [0.2, 0.4] })
  brush([[496, 199], [502, 202], [508, 200]], { width: 2, color: P.hair, taper: [0.3, 0.3] })
  brush([[496, 195], [506, 195]], { width: 1.5, color: P.hair, taper: [0.3, 0.3] })
  // nariz triangular: cuña ancha que nace de la cara con su mismo tono
  const faceLit = shade(P.skin, 0.16)
  const nl = mix(P.skinD, "#7a3a30", 0.25)
  const nose = blob([[506, 203], [530, 211], [553, 222], [573, 233], [583, 240], [577, 246], [560, 246], [540, 245], [522, 244], [508, 241], [500, 226], [500, 210]], { fill: linear(500, 0, 586, 0, [faceLit, [0.55, P.skin], mix(P.skin, P.nose, 0.35)]) })
  clip(nose, () => {
    // luz suave a lo largo del caballete
    brush([[514, 208], [542, 219], [570, 233]], { width: 8, color: alpha(shade(P.skin, 0.3), 0.6), blur: 3 })
    // plano inferior en sombra, que se funde antes de llegar a la cara
    brush([[528, 246], [552, 244], [576, 244]], { width: 10, color: alpha(P.skinD, 0.5), blur: 4, taper: [0.6, 0.2] })
    // punta apenas sonrosada
    circle(576, 238, 9, { fill: alpha(P.nose, 0.45), blur: 4 })
  })
  // aleta pequeña y fosa, cerca de la base
  brush([[540, 245], [535, 236], [526, 235], [520, 241]], { width: [1, 1.8, 1.2], color: alpha(nl, 0.85), taper: [0.3, 0.4] })
  ellipse(547, 242.5, 4, 1.5, { fill: alpha(shade(nl, -0.3), 0.8), rotate: 4 })
  // contorno con peso solo en la punta y la parte de abajo (no en la base)
  brush([[582, 240], [577, 245], [562, 245.5], [546, 244.5]], { width: [1.5, 2.2, 1], color: nl, taper: [0.3, 0.6] })
  brush([[530, 212], [556, 224], [578, 237]], { width: [0.6, 1.2, 1.6], color: alpha(P.skinD, 0.6), taper: [0.6, 0.2] })
  // brillo pequeño en la punta
  ellipse(574, 233, 3.5, 1.8, { fill: "#ffffff70", blur: 1, rotate: 30 })
})

// el pelo (y la oreja, que queda por encima), en su propio hueso: el juego lo mueve con un muelle
// (bajo el agua flota y se queda atrás; fuera, pegado). Gira alrededor de la oreja.
part("pelo", () => {
  // pelo rizado
  seed(7)
  const hairBase = blob(roughen([[420, 236], [422, 196], [444, 166], [472, 150], [494, 144], [504, 156], [498, 176], [490, 196], [474, 214], [468, 240], [462, 260], [452, 258], [448, 236], [432, 238]], 5, { detail: 3 }), { fill: P.hair })
  const curls = [[492, 152, 12], [476, 158, 14], [458, 170, 15], [440, 188, 15], [430, 212, 13], [470, 190, 16], [454, 210, 15], [462, 236, 10], [484, 176, 11], [440, 226, 11]]
  const hair = curls.map(([x, y, r]) => circle(x + rand(-1, 1), y + rand(-1, 1), r, { fill: P.hair }))
  clip([hairBase, ...hair], () => {
    for (const [x, y, r] of curls) brush(arc(x - 2, y - 1, r * 0.7, 220, 330, 6), { width: 2.5, color: "#3c3a4c" })
    brush([[452, 164], [474, 152], [492, 148]], { width: 3, color: "#55526a", blur: 1 })
  })
  // oreja
  const ear = blob([[428, 238], [438, 240], [440, 256], [432, 268], [420, 264], [417, 248]], { fill: P.skin })
  clip(ear, () => ellipse(436, 262, 8, 10, { fill: alpha(P.skinD, 0.6), blur: 3 }))
  brush([[431, 246], [425, 254], [431, 262]], { width: 3, color: P.skinD })
})

// cuello de la camisa (va con el torso, encima del cuello)
part("torso", () => {
  blob([[422, 318], [446, 320], [450, 334], [428, 342], [410, 336]], { fill: P.shirt })
  brush([[426, 320], [436, 332], [430, 342]], { width: 2, color: P.shirtD })
})

// ---------- lanza de pesca: rama recta, punta de chapa oxidada y cola con ojal ----------
const LZ = { wood: "#94704a", woodL: "#c8a173", woodD: "#5c4029", ink: "#3e2a1c", cord: "#cfb47c", cordD: "#7a6034",
  rust: "#9c4f2a", rustD: "#552a18", rustL: "#d08a50", steel: "#aaa69c", cloth: "#c9c3b4" }
// vueltas de cordel apretadas entre u0 y u1 (r = medio ancho)
function lash(u0, u1, r, col, colD) {
  blob([LS(u0, -r, 0), LS(u1, -r, 0), LS(u1, r, 0), LS(u0, r, 0)], { fill: colD })
  for (let u = u0 + 1.2; u < u1 - 0.5; u += 2.6) {
    brush([LS(u - 0.6, -r), LS(u, 0), LS(u + 0.6, r)], { width: 2.1, color: col, taper: [0.15, 0.15] })
    brush([LS(u - 0.9, -r * 0.6), LS(u - 0.6, -r * 0.1)], { width: 0.7, color: "#fff4d070", taper: [0.2, 0.2] })
  }
}
function spear() {
  seed(51)
  // astil: rama recta con un leve engrosamiento en cada nudo
  const shaft = blob([LS(L_COLA, -4.2, 0.5), LS(-212, -5.2), LS(-198, -6.6), LS(-184, -5.2), LS(40, -5.2), LS(52, -6.4), LS(64, -5),
    LS(200, -4.4), LS(204, 0, 0.5), LS(200, 4.4), LS(-184, 5.4), LS(-198, 6.2), LS(-212, 5.4), LS(L_COLA, 4.4, 0.5), LS(L_COLA - 3, 0, 0.5)],
    { fill: LZ.wood, stroke: LZ.ink, width: 1.8, behind: true })
  clip(shaft, () => {
    brush([LS(L_COLA, 3.6), LS(204, 3.4)], { width: 4, color: alpha(LZ.woodD, 0.75), blur: 1, taper: [0, 0] })
    brush([LS(L_COLA + 4, -2.8), LS(200, -2.6)], { width: 2, color: alpha(LZ.woodL, 0.85), taper: [0.1, 0.1] })
    // vetas
    for (let i = 0; i < 16; i++) {
      const u = rand(L_COLA + 4, 180), l = rand(30, 90), v = rand(-3, 3)
      brush([LS(u, v), LS(u + l / 2, v + rand(-0.7, 0.7)), LS(u + l, v)], { width: rand(0.5, 0.9), color: alpha(LZ.woodD, rand(0.4, 0.7)), taper: [0.3, 0.3] })
    }
    // zona del agarre, oscurecida por el uso
    brush([LS(-30, 0), LS(30, 0)], { width: 12, color: alpha("#3a2618", 0.35), blur: 3, taper: [0.3, 0.3] })
    // grieta reparada
    brush([LS(-138, -1.5), LS(-112, 0.5), LS(-90, -0.5)], { width: 0.9, color: "#2a1c12", taper: [0.2, 0.3] })
  })
  // nudos de la rama
  for (const [u, v, s] of [[-198, 0.8, 1], [52, -0.8, 0.8]]) {
    ellipse(...LS(u, v), 4.4 * s, 2.8 * s, { fill: LZ.woodD, rotate: -10 })
    ellipse(...LS(u, v), 2.2 * s, 1.3 * s, { fill: "#2e1e14", rotate: -10 })
    brush([LS(u - 9, v - 2.6), LS(u, v - 3.6), LS(u + 9, v - 2.6)], { width: 0.7, color: alpha(LZ.woodD, 0.8) })
  }
  // tablilla atada sobre la grieta y tira de tela de camisa enrollada
  const slat = blob([LS(-152, -8.4, 0.5), LS(-74, -8, 0.5), LS(-72, -4.4, 0.5), LS(-154, -4.6, 0.5)], { fill: "#b29770", stroke: LZ.ink, width: 1.2, behind: true })
  clip(slat, () => brush([LS(-152, -5.4), LS(-72, -5.2)], { width: 1.4, color: alpha("#6e5a3c", 0.7) }))
  lash(-146, -134, 8.6, LZ.cord, LZ.cordD)
  const cloth = blob([LS(-104, -9.4, 0), LS(-88, -9.4, 0), LS(-84, 6.4, 0), LS(-100, 6.4, 0)], { fill: LZ.cloth, stroke: "#7d7a70", width: 1, behind: true })
  clip(cloth, () => {
    brush([LS(-100, -5), LS(-93, 4)], { width: 1, color: "#8f8a7c" })
    brush([LS(-94, -9), LS(-88, 2)], { width: 1, color: "#8f8a7c" })
    blob([LS(-104, 0), LS(-84, 0), LS(-84, 7), LS(-104, 7)], { fill: alpha(DIRT, 0.5), blur: 2 })
  })
  brush([LS(-86, 6.4), LS(-82, 12), LS(-86, 17)], { width: 1.4, color: LZ.cloth, taper: [0, 0.7] })

  // punta: chapa oxidada recortada en arpón, con dos lengüetas hacia atrás
  const tip = blob([LS(184, -4.8, 0), LS(206, -7, 0), LS(194, -14.5, 0), LS(226, -7.4, 0.5), LS(252, -4), LS(L_TIP, 0, 0),
    LS(252, 4), LS(226, 7.4, 0.5), LS(194, 14.5, 0), LS(206, 7, 0), LS(184, 4.8, 0)], { fill: LZ.rust, stroke: LZ.rustD, width: 1.4, behind: true })
  clip(tip, () => {
    brush([LS(196, 3.5), LS(272, 1)], { width: 6, color: alpha(LZ.rustD, 0.7), blur: 1.5, taper: [0, 0.2] })
    // filo gastado donde asoma el metal
    brush([LS(222, -6.2), LS(250, -3.4), LS(270, -0.4)], { width: 1.8, color: LZ.steel, taper: [0.3, 0.1] })
    brush([LS(214, -0.8), LS(262, -0.4)], { width: 1.2, color: alpha(LZ.rustL, 0.8), taper: [0.3, 0.3] })
    for (let i = 0; i < 9; i++) circle(...LS(rand(200, 258), rand(-5, 5)), rand(0.8, 2.2), { fill: alpha(pick([LZ.rustD, LZ.rustL, "#6a3a1e"]), 0.75) })
  })
  // atadura de la punta
  lash(182, 208, 6.6, LZ.cord, LZ.cordD)
  brush([LS(206, 6.4), LS(211, 13), LS(207, 21)], { width: 1.3, color: LZ.cord, taper: [0, 0.6] })
  brush([LS(204, 6.6), LS(203, 11)], { width: 1.1, color: LZ.cordD, taper: [0, 0.6] })

  // cola: ojal taladrado, vueltas de cordel para que no se raje y muesca en el extremo
  lash(L_OJAL + 9, L_OJAL + 20, 5.6, LZ.cord, LZ.cordD)
  ellipse(...LS(L_OJAL, 0), 3.4, 2.6, { fill: "#20140c", stroke: LZ.woodD, width: 0.8, rotate: -10 })
  brush([LS(L_OJAL - 2.5, 2.2), LS(L_OJAL + 2.5, 2.2)], { width: 0.8, color: alpha(LZ.woodL, 0.8) })
  poly([LS(L_COLA - 3, -2.4), LS(L_COLA + 6, -0.3), LS(L_COLA + 6, 0.3), LS(L_COLA - 3, 2.4)], { fill: "#2a1c12" })
}

// ---------- brazo cercano ----------
part("brazo_der", () => {
  clip([R(300, 300, 160, 265), C(373, 565, 26)], () => armShape(armNear))
})
// la lanza va entre el brazo y la mano: los dedos la tapan
part("lanza", spear)
part("mano_der", () => handShape(armNear))
part("antebrazo_der", () => {
  clip([C(373, 565, 26), R(300, 565, 160, 200)], () => armShape(armNear))
})

// ---------- acciones ----------
// giro total de la cadena del brazo cercano hasta la mano
const rotOf = (p, k) => (typeof p[k] === "object" ? p[k].rot : p[k]) || 0
const armSum = p => ["cadera", "torso", "brazo_der", "antebrazo_der", "mano_der"].reduce((s, k) => s + rotOf(p, k), 0)
// lanza en la mano: mantiene el ángulo "target" en pantalla y acompaña una fracción del brazo
const carry = (p, target, follow = 0.3) => ({ ...p, lanza: target - (1 - follow) * armSum(p) })
// lanza apuntando "up" grados sobre la horizontal (en reposo va 10° arriba)
const aimAt = (p, up) => ({ ...p, lanza: (10 - up) - armSum(p) })
// reposo: respiración adormilada, la cabeza cabecea un poco
const I0 = { cadera: { rot: 0, y: 0 }, torso: 0, cabeza: 0, brazo_der: 0, brazo_izq: 0, antebrazo_der: 0, antebrazo_izq: 0, mano_der: 0 }
action("reposo", { duration: 3, loop: true }, [
  [0, I0],
  [0.45, { cadera: { rot: 0, y: 1.5 }, torso: 1.5, cabeza: 3.5, brazo_der: -1.5, brazo_izq: -1, antebrazo_der: -2, antebrazo_izq: -2, mano_der: 2 }],
  [1, I0],
])

// andar: zancada larga y floja de chico larguirucho; rodillas que rebotan,
// cabeza que se adelanta en cada paso y brazos colgando que llegan tarde
const W0 = { cadera: { rot: 0, y: 18 }, torso: 5, cabeza: { rot: 4, x: 3 },
  muslo_der: -20, pierna_der: 4, pie_der: 8, muslo_izq: 16, pierna_izq: 10, pie_izq: -12,
  brazo_der: 16, antebrazo_der: -8, mano_der: 4, brazo_izq: -14, antebrazo_izq: -18, mano_izq: -2 }
const W1 = { cadera: { rot: 0, y: 22 }, torso: 6, cabeza: { rot: 7, x: 5 },
  muslo_der: -26, pierna_der: 26, pie_der: 0, muslo_izq: 8, pierna_izq: 40, pie_izq: -12,
  brazo_der: 13, antebrazo_der: -6, mano_der: 10, brazo_izq: -11, antebrazo_izq: -20, mano_izq: -8 }
const W2 = { cadera: { rot: 0, y: 1 }, torso: 4, cabeza: { rot: 3, x: 0 },
  muslo_der: 0, pierna_der: 2, pie_der: -2, muslo_izq: -14, pierna_izq: 48, pie_izq: -6,
  brazo_der: 1, antebrazo_der: -6, mano_der: 8, brazo_izq: -1, antebrazo_izq: -10, mano_izq: 4 }
const W3 = { cadera: { rot: 0, y: -4 }, torso: 4, cabeza: { rot: 1, x: -2 },
  muslo_der: 10, pierna_der: 0, pie_der: 2, muslo_izq: -24, pierna_izq: 20, pie_izq: -10,
  brazo_der: -10, antebrazo_der: -12, mano_der: 0, brazo_izq: 10, antebrazo_izq: -4, mano_izq: 10 }
action("andar", { duration: 1.2, loop: true, move: 547 }, [
  [0, W0], [0.125, W1], [0.25, W2], [0.375, W3],
  [0.5, swapSides(W0)], [0.625, swapSides(W1)], [0.75, swapSides(W2)], [0.875, swapSides(W3)],
  [1, W0],
].map(([t, p]) => [t, carry(p, 4, 0.35)]))

// correr: desgarbado, inclinado hacia delante, cuello estirado y codos a 90°
const R0 = { cadera: { rot: 0, y: 20 }, torso: 14, cabeza: { rot: 8, x: 4 },
  muslo_der: -24, pierna_der: 10, pie_der: 4, muslo_izq: 30, pierna_izq: 70, pie_izq: -10,
  brazo_der: 40, antebrazo_der: -95, mano_der: 10, brazo_izq: -40, antebrazo_izq: -80, mano_izq: -5 }
const R1 = { cadera: { rot: 0, y: 30 }, torso: 15, cabeza: { rot: 11, x: 6 },
  muslo_der: -8, pierna_der: 30, pie_der: -22, muslo_izq: -10, pierna_izq: 95, pie_izq: -15,
  brazo_der: 20, antebrazo_der: -90, mano_der: 14, brazo_izq: -20, antebrazo_izq: -85, mano_izq: 0 }
const R2 = { cadera: { rot: 0, y: 10 }, torso: 14, cabeza: { rot: 8, x: 3 },
  muslo_der: 24, pierna_der: 6, pie_der: 22, muslo_izq: -45, pierna_izq: 70, pie_izq: -5,
  brazo_der: -10, antebrazo_der: -90, mano_der: 4, brazo_izq: 15, antebrazo_izq: -95, mano_izq: 10 }
const R3 = { cadera: { rot: 0, y: -12 }, torso: 13, cabeza: { rot: 6, x: 0 },
  muslo_der: 35, pierna_der: 70, pie_der: -10, muslo_izq: -38, pierna_izq: 30, pie_izq: 8,
  brazo_der: -30, antebrazo_der: -90, mano_der: 0, brazo_izq: 30, antebrazo_izq: -95, mano_izq: 12 }
action("correr", { duration: 0.64, loop: true, move: 2360 }, [
  [0, R0], [0.125, R1], [0.25, R2], [0.375, R3],
  [0.5, swapSides(R0)], [0.625, swapSides(R1)], [0.75, swapSides(R2)], [0.875, swapSides(R3)],
  [1, R0],
].map(([t, p]) => [t, carry(p, 8, 0.2)]))

// apuntar: lanza echada atrás por encima de la cabeza, peso en la pierna de atrás (cercana)
// y el brazo lejano adelantado. En el bucle solo respira y la punta tiembla.
const AIM = { cadera: { rot: 0, y: 50 }, torso: -8, cabeza: { rot: 6, x: 0 },
  muslo_izq: -28, pierna_izq: 4, pie_izq: 24, muslo_der: 18, pierna_der: 8, pie_der: -26,
  brazo_der: 198, antebrazo_der: -70, mano_der: 30,
  brazo_izq: -80, antebrazo_izq: -15, mano_izq: -5 }
// ---------- pies clavados: IK de dos huesos para que el tobillo no se mueva ----------
const dg = a => a * Math.PI / 180, gd = a => a * 180 / Math.PI
const vAdd = (a, b) => [a[0] + b[0], a[1] + b[1]], vSub = (a, b) => [a[0] - b[0], a[1] - b[1]]
const vRot = ([x, y], a) => [x * Math.cos(dg(a)) - y * Math.sin(dg(a)), x * Math.sin(dg(a)) + y * Math.cos(dg(a))]
const vAng = (p, q) => gd(Math.atan2(q[1] - p[1], q[0] - p[0]))
const vLen = v => Math.hypot(v[0], v[1])
const HIP0 = [414, 590]
const LEG = { der: { h: [414, 600], k: [418, 820], a: [388, 1092] }, izq: { h: [424, 600], k: [427, 810], a: [425, 1090] } }
const hipOf = (p, s) => {
  const c = p.cadera || {}, r = c.rot || 0
  return vAdd([HIP0[0] + (c.x || 0), HIP0[1] + (c.y || 0)], vRot(vSub(LEG[s].h, HIP0), r))
}
// tobillo en el lienzo para una pose
function ankleOf(p, s) {
  const L = LEG[s], r = (p.cadera || {}).rot || 0, m = rotOf(p, "muslo_" + s), q = rotOf(p, "pierna_" + s)
  const knee = vAdd(hipOf(p, s), vRot(vSub(L.k, L.h), r + m))
  return vAdd(knee, vRot(vSub(L.a, L.k), r + m + q))
}
// coloca la pierna "s" para que el tobillo caiga en A; pie = giro extra del pie (0 = plano)
function plant(p, s, A, pie = 0) {
  const L = LEG[s], r = (p.cadera || {}).rot || 0, H = hipOf(p, s)
  const a = vLen(vSub(L.k, L.h)), b = vLen(vSub(L.a, L.k))
  const D = vSub(A, H), d = clamp(vLen(D), Math.abs(a - b) + 0.5, a + b - 0.5)
  const sg = Math.sign(vAng(L.h, L.k) - vAng(L.h, L.a)) || 1
  const thW = gd(Math.atan2(D[1], D[0])) + sg * gd(Math.acos((a * a + d * d - b * b) / (2 * a * d)))
  const knee = vAdd(H, [a * Math.cos(dg(thW)), a * Math.sin(dg(thW))])
  const m = thW - vAng(L.h, L.k) - r
  const q = vAng(knee, A) - vAng(L.k, L.a) - (thW - vAng(L.h, L.k))
  return { ...p, ["muslo_" + s]: m, ["pierna_" + s]: q, ["pie_" + s]: -(r + m + q) + pie }
}
const stand = (p, feet) => plant(plant(p, "der", feet.der), "izq", feet.izq)

// equilibrio: llega al borde, el pie de delante se le va al vacío, se inclina, agita los brazos
// en círculos hacia atrás (el lejano más amplio), echa la cadera atrás y se recupera.
const FEET_R = { der: LEG.der.a, izq: LEG.izq.a }

// ---------- apuntar, alzar, lanzar y sostener, con los pies clavados (IK) ----------
// La guardia de apuntar se coge dando un paso con el pie de delante (izquierdo); el de atrás se queda
// donde estaba, bajo la cadera, cargando el peso: así ningún pie patina y el cuerpo no se desplaza.
const FEET_A = { der: FEET_R.der, izq: [ankleOf(AIM, "izq")[0], FEET_R.izq[1]] }
const enGuardia = p => stand(p, FEET_A)
// apuntar: lanza echada atrás por encima de la cabeza y el brazo lejano adelantado. En el bucle solo
// respira y la punta tiembla.
const AIM0 = aimAt(enGuardia(AIM), 15)
const breathe = (k, dl) => {
  const p = aimAt(enGuardia({ ...AIM, cadera: { rot: 0, y: 50 + k * 2 }, torso: -8 - k * 1.5, cabeza: { rot: 6 + k, x: 0 }, brazo_izq: -80 + k * 2 }), 15)
  return { ...p, lanza: p.lanza + dl }
}
action("apuntar", { duration: 1.2, loop: true }, [
  [0, AIM0], [0.15, breathe(0.3, 1.2)], [0.3, breathe(0.6, -0.8)], [0.5, breathe(1, 1)],
  [0.65, breathe(0.7, -1.3)], [0.82, breathe(0.35, 0.9)], [1, AIM0],
])
// alzar: de estar de pie a la guardia de apuntar, sin molinete: primero dobla el codo y sube la lanza
// al hombro (la punta sigue mirando adelante) mientras el pie de delante se levanta, y luego echa el
// brazo atrás y apoya el pie
const PASO = [(FEET_R.izq[0] + FEET_A.izq[0]) / 2, FEET_R.izq[1] - 45]
action("alzar", { duration: 0.25, loop: false }, [
  [0, aimAt(stand({ cadera: { x: 0, y: 0, rot: 0 }, torso: 0, cabeza: { rot: 0, x: 0 },
    brazo_der: 0, antebrazo_der: 0, mano_der: 0, brazo_izq: 0, antebrazo_izq: 0, mano_izq: 0 }, FEET_R), 10)],
  [0.5, aimAt(plant(plant({ cadera: { rot: 0, x: 0, y: 28 }, torso: -4, cabeza: { rot: 4, x: 0 },
    brazo_der: 40, antebrazo_der: -125, mano_der: 18, brazo_izq: -45, antebrazo_izq: -20, mano_izq: 0 },
    "der", FEET_R.der), "izq", PASO, 20), 18)],
  [1, AIM0],
])
// lanzar: anticipación, impulso de cadera, torso y brazo; suelta en t = 0,4 a 20° sobre la
// horizontal. El pie de delante frena (se queda clavado) y el de atrás pivota sobre la punta y avanza
// un poco en la continuación, con la cadera.
const PUNTA = { der: [480, 1144], izq: [517, 1141] }          // de puntillas: el apoyo bajo los dedos
const TALON = { der: vSub(LEG.der.a, PUNTA.der), izq: vSub(LEG.izq.a, PUNTA.izq) }
const lz = (c, to, ca, bd, ad, md, bi, ai, mi, up, atras) => {
  const p = { cadera: { rot: c[2], x: c[0], y: c[1] }, torso: to, cabeza: { rot: ca[0], x: ca[1] },
    brazo_der: bd, antebrazo_der: ad, mano_der: md, brazo_izq: bi, antebrazo_izq: ai, mano_izq: mi }
  // el pie de atrás pivota sobre la punta: el tobillo sube lo que haga falta para que los dedos sigan
  // en la cubierta (no se hunden en ella)
  const der = atras ? plant(p, "der", vAdd(vAdd(PUNTA.der, [atras[0], 0]), vRot(TALON.der, atras[2])), atras[2]) : plant(p, "der", FEET_A.der)
  return aimAt(plant(der, "izq", FEET_A.izq), up)
}
const CLAVES_LANZAR = [
  [0.2, [0, 54, -4], -10, [7, 0], 206, -80, 36, -70, -20, -5, 12, null],
  [0.4, [0, 48, 8], 6, [2, 2], 206, -6, 10, -20, -35, 0, 20, null],
  [0.65, [10, 48, 10], 14, [4, 3], 270, -14, 4, 10, -40, 5, 0, [24, -18, 25]],
  [1, [22, 50, 10], 18, [8, 3], 288, -22, 8, 20, -45, 8, -20, [50, -28, 40]],
]
action("lanzar", { duration: 0.5, loop: false }, [[0, AIM0], ...CLAVES_LANZAR.map(([t, ...a]) => [t, lz(...a)])])

// sostener: tras lanzar se queda con las piernas abiertas donde las dejó (el pie de atrás ya apoyado),
// sujetando la soga a la altura de la cadera; respira despacio y pasa el peso de un pie a otro.
const FEET_L = { der: vAdd(FEET_A.der, [50, 0]), izq: FEET_A.izq }
const CENTRO_L = (FEET_L.der[0] + FEET_L.izq[0]) / 2 - 406
const hold = (k, w) => aimAt(stand({
  cadera: { rot: 5 + w * 1.5, x: CENTRO_L + w * 7, y: 40 + k * 2.5 - Math.abs(w) * 1.5 }, torso: 3 - k * 1.5, cabeza: { rot: 6 + k * 2.5, x: 2 },
  brazo_der: 345 + k * 1.5, antebrazo_der: -64 - k, mano_der: -12,
  brazo_izq: -4 + k * 2, antebrazo_izq: -38 - k * 2, mano_izq: -8,
}, FEET_L), -20)
action("sostener", { duration: 2.4, loop: true }, [
  [0, hold(0, 0)], [0.25, hold(0.5, 1)], [0.5, hold(1, 0)], [0.75, hold(0.5, -1)], [1, hold(0, 0)],
])

// recoger: tira de la soga mano sobre mano hacia el pecho, echado atrás y con los pies donde quedaron
// al lanzar (sin moverlos)
const PULL = { cadera: { rot: 0, y: 28 }, torso: -10, cabeza: { rot: 10, x: 2 } }
const tirando = p => stand({ ...p, cadera: { ...p.cadera, x: CENTRO_L } }, FEET_L)
const reachD = { brazo_der: -62, antebrazo_der: -32, mano_der: -20 }, pullD = { brazo_der: 40, antebrazo_der: -140, mano_der: -25 }
const reachI = { brazo_izq: -62, antebrazo_izq: -32, mano_izq: -20 }, pullI = { brazo_izq: 40, antebrazo_izq: -140, mano_izq: -25 }
const REC0 = tirando({ ...PULL, ...reachD, ...pullI })
const REC1 = tirando({ ...PULL, cadera: { rot: 0, y: 30 }, torso: -12, ...pullD, ...reachI })
action("recoger", { duration: 0.8, loop: true }, [
  [0, REC0],
  [0.25, tirando({ ...PULL, cadera: { rot: 0, y: 29 }, torso: -11, brazo_der: -10, antebrazo_der: -90, mano_der: -22, brazo_izq: 10, antebrazo_izq: -80, mano_izq: -22 })],
  [0.5, REC1],
  [0.75, tirando({ ...PULL, cadera: { rot: 0, y: 29 }, torso: -11, brazo_izq: -10, antebrazo_izq: -90, mano_izq: -22, brazo_der: 10, antebrazo_der: -80, mano_der: -22 })],
  [1, REC0],
])
// volver: con la lanza ya en la mano, de la guardia a estar de pie: el pie de delante vuelve atrás con
// un paso (el de atrás, que quedó algo adelantado, se arrastra un poco)
const DE_PIE = aimAt(stand({ cadera: { x: 0, y: 0, rot: 0 }, torso: 0, cabeza: { rot: 0, x: 0 },
  brazo_der: 0, antebrazo_der: 0, mano_der: 0, brazo_izq: 0, antebrazo_izq: 0, mano_izq: 0 }, FEET_R), 10)
const PASO_ATRAS = [(FEET_L.izq[0] + FEET_R.izq[0]) / 2, FEET_R.izq[1] - 40]
action("volver", { duration: 0.25, loop: false }, [
  [0, aimAt(REC0, 10)],
  [0.5, aimAt(plant(plant({ cadera: { rot: 0, x: CENTRO_L / 2, y: 14 }, torso: -4, cabeza: { rot: 4, x: 1 },
    brazo_der: 4, antebrazo_der: -20, mano_der: 4, brazo_izq: -10, antebrazo_izq: -30, mano_izq: -6 },
    "der", vAdd(FEET_R.der, [(FEET_L.der[0] - FEET_R.der[0]) / 2, 0])), "izq", PASO_ATRAS, 18), 10)],
  [1, DE_PIE],
])
const EQ = [
  // t, cadera [x, y, rot], torso, cabeza [rot, x], brazo_izq, antebrazo_izq, mano_izq, brazo_der, antebrazo_der, mano_der, lanza (grados arriba), pie izq [dx, dy, giro]
  [0, [0, 2, 0], 4, [4, 2], 0, 0, 0, 0, 0, 0, 10, [0, 0, 0]],
  [0.12, [14, 8, 3], 14, [10, 8], -110, -30, -10, -60, -30, -5, 40, [10, -18, 16]],
  [0.26, [-6, 16, 4], 20, [12, 10], -200, -10, 20, -95, -40, 10, 62, [22, -40, 28]],
  [0.4, [-28, 18, 3], 18, [6, 6], -130, -100, -20, -60, -60, -10, 45, [16, -30, 18]],
  [0.54, [-30, 12, 1], 10, [-2, 2], -60, -20, 10, -25, -20, 5, 20, [6, -10, 6]],
  [0.68, [-16, 8, 0], 10, [-8, -2], -160, -40, -15, -75, -40, -10, 50, [0, 0, 0]],
  [0.84, [-4, 3, 0], 4, [2, 0], -40, -60, 5, -25, -40, 0, 20, [0, 0, 0]],
  [1, [0, 0, 0], 0, [0, 0], 0, -2, 0, 0, -2, 0, 10, [0, 0, 0]],
]
action("equilibrio", { duration: 0.5, loop: false }, EQ.map(([t, c, to, ca, bi, ai, mi, bd, ad, md, up, f]) => {
  const p = { cadera: { x: c[0], y: c[1], rot: c[2] }, torso: to, cabeza: { rot: ca[0], x: ca[1] },
    brazo_izq: bi, antebrazo_izq: ai, mano_izq: mi, brazo_der: bd, antebrazo_der: ad, mano_der: md }
  const q = plant(plant(p, "der", FEET_R.der), "izq", vAdd(FEET_R.izq, [f[0], f[1]]), f[2])
  return [t, aimAt(q, up)]
}))

// golpe: estocada a dos manos. Recoge la lanza a la cadera (la mano lejana guía el asta por delante),
// y clava con un paso del pie de delante (~0,3 m) que mete el peso del cuerpo: la lanza avanza a lo
// largo de su eje (mismo ángulo de la anticipación a la estocada), no barre. Luego vuelve con el pie.
// pie: el pie de delante (izquierdo) [adelanto, altura, giro]; el de atrás no se mueve.
const golpe = (c, to, ca, bd, ad, md, bi, ai, mi, up, pie = [0, 0, 0]) => aimAt(plant(plant({
  cadera: { x: c[0], y: c[1], rot: c[2] }, torso: to, cabeza: { rot: ca[0], x: ca[1] },
  brazo_der: bd, antebrazo_der: ad, mano_der: md, brazo_izq: bi, antebrazo_izq: ai, mano_izq: mi,
}, "der", FEET_R.der), "izq", vAdd(FEET_R.izq, [pie[0], pie[1]]), pie[2]), up)
action("golpe", { duration: 0.45, loop: false }, [
  [0, golpe([0, 0, 0], 0, [0, 0], 0, 0, 0, 0, 0, 0, 10)],
  [0.22, golpe([-10, 14, -2], -6, [2, -2], 36, -86, 6, -46, -44, 0, -4, [16, -18, 10])],
  [0.36, golpe([14, 22, 2], 6, [4, 1], -26, -44, 5, -58, -26, 0, -4, [95, -26, 8])],
  [0.48, golpe([40, 28, 5], 14, [6, 4], -82, -6, 4, -74, -8, 0, -4, [170, 0, 0])],
  [0.64, golpe([36, 25, 4], 11, [5, 3], -74, -12, 4, -68, -12, 0, -4, [170, 0, 0])],
  [0.84, golpe([14, 12, 1], 4, [2, 1], -20, -34, 2, -24, -30, 0, 4, [80, -22, 8])],
  [1, golpe([0, 0, 0], 0, [0, 0], 0, 0, 0, 0, 0, 0, 10)],
])
// golpe_abajo: desde el borde, hacia el agua. Se agacha doblando cadera y rodillas, con las dos manos
// en el asta, y clava casi en vertical (unos 78° abajo): así la punta entra en el agua junto a la balsa
// La muñeca se dobla (mano_der) para que el asta no gire más de ~100° respecto a la mano: sin molinete.
// desliza: la lanza corre por las manos hacia delante (la empuja la de atrás) para llegar más hondo
const abajo = (c, to, ca, bd, ad, md, bi, ai, mi, up, desliza = 0) => {
  const p = aimAt(stand({
    cadera: { x: c[0], y: c[1], rot: c[2] }, torso: to, cabeza: { rot: ca[0], x: ca[1] },
    brazo_der: bd, antebrazo_der: ad, mano_der: md, brazo_izq: bi, antebrazo_izq: ai, mano_izq: mi,
  }, FEET_R), up)
  return { ...p, lanza: { rot: p.lanza, x: desliza * Math.cos(dg(p.lanza - 10)), y: desliza * Math.sin(dg(p.lanza - 10)) } }
}
action("golpe_abajo", { duration: 0.7, loop: false }, [
  [0, golpe([0, 0, 0], 0, [0, 0], 0, 0, 0, 0, 0, 0, 10)],
  [0.22, abajo([6, 90, 6], 26, [-10, 0], -62, -52, 40, -50, -48, 0, -55, 0)],
  [0.48, abajo([34, 250, 14], 48, [-24, 0], -86, -4, 30, -76, -10, 0, -82, 400)],
  [0.66, abajo([32, 240, 13], 46, [-22, 0], -84, -8, 30, -74, -12, 0, -80, 370)],
  [1, golpe([0, 0, 0], 0, [0, 0], 0, 0, 0, 0, 0, 0, 10)],
])

// ---------- agua ----------
// Las acciones de nadar van en las coordenadas del cuerpo, de pie: el juego gira el cuerpo entero
// hacia donde nada (tumbado boca abajo, con la cabeza por delante). "Delante" del cuerpo es el
// vientre: al nadar queda hacia el fondo, y la espalda hacia la superficie.
// nadar, nadar_rapido y flotar_agua duran lo mismo (1 s): el juego las mezcla por velocidad y lleva
// el ritmo de la patada.
const sn = a => Math.sin(dg(a))
// patada de crol: sale de la cadera. Sube con la pierna casi recta; al empezar a bajar, el muslo
// tira y la espinilla se queda atrás (la rodilla se dobla) y luego se estira de golpe (latigazo). El
// pie va en punta y llega tarde, empujado por el agua.
// (medido frente a nadadores: el pie recorre 0,30–0,40 m de pico a pico y la rodilla se dobla 45–78°,
// cuando la cadera ya empieza a bajar: el talón apenas pasa la línea del cuerpo)
const kick = (ph, amp, knee) => ({
  muslo: -12 + amp * sn(ph),
  pierna: knee / 2 + 2 + (knee / 2) * sn(ph - 110),
  pie: 82 + 6 * sn(ph - 90),   // en punta: el pie sigue a la pierna
})
function swimPose(ph, k) {
  // k: 0 nadar, 1 nadar rápido (el esfuerzo lo pone sobre todo el ritmo, que lleva el juego, y el
  // brazo libre, que rema; la patada solo algo más amplia)
  const amp = 10 + 2 * k, knee = 44 + 8 * k
  const d = kick(ph, amp, knee), i = kick(ph + 180, amp, knee)
  // el cuerpo ondula un poco con cada patada (dos por ciclo); la cabeza se queda quieta, mirando
  // hacia delante y algo abajo, hacia donde nada
  const und = sn(2 * ph + 40)
  const cadRot = 1.5 * und, torso = -1.2 * und
  const p = {
    cadera: { rot: cadRot, x: 5 * und, y: 0 }, torso, cabeza: { rot: -50 - torso - cadRot, x: 0 },
    muslo_der: d.muslo, pierna_der: d.pierna, pie_der: d.pie,
    muslo_izq: i.muslo, pierna_izq: i.pierna, pie_izq: i.pie,
    // la lanza, por delante: nadando normal, con el codo doblado unos 75° (lista para clavar: la
    // estocada estira el brazo y la punta llega a lo que tiene a 1–1,5 m); rápido, estirado, en flecha. El otro brazo, pegado al cuerpo, rema un poco
    brazo_der: -150 - 26 * k + 2 * sn(2 * ph), antebrazo_der: -75 + 70 * k, mano_der: 10 - 6 * k,
    brazo_izq: 6 - 30 * k + (6 + 22 * k) * sn(ph + 30), antebrazo_izq: -18 - 12 * k + (10 + 6 * k) * sn(ph - 30), mano_izq: 10,
  }
  return aimAt(p, 88)
}
const cycle = (f, n = 12) => Array.from({ length: n + 1 }, (_, j) => [j / n, f(360 * j / n)])
action("nadar", { duration: 1, loop: true }, cycle(ph => swimPose(ph, 0)))
action("nadar_rapido", { duration: 1, loop: true }, cycle(ph => swimPose(ph, 1)))

// flotar quieto bajo el agua: casi de pie, pedaleo lento con las piernas dobladas, la mano libre
// remando a los lados y la lanza a punto, por delante a la altura del pecho
function treadPose(ph, surf) {
  const legL = ph + 180
  const p = {
    cadera: { rot: 0, x: 0, y: 6 * sn(2 * ph) }, torso: surf ? 2 : 6, cabeza: { rot: surf ? -12 : -4 + 2 * sn(ph), x: 0 },
    muslo_der: -38 + 12 * sn(ph), pierna_der: 72 + 18 * sn(ph - 90), pie_der: 25 + 12 * sn(ph - 150),
    muslo_izq: -38 + 12 * sn(legL), pierna_izq: 72 + 18 * sn(legL - 90), pie_izq: 25 + 12 * sn(legL - 150),
    brazo_der: -38 + 4 * sn(ph), antebrazo_der: -68 + 5 * sn(ph + 60), mano_der: 6,
    brazo_izq: -24 + (surf ? 18 : 12) * sn(ph + 90), antebrazo_izq: -48 + (surf ? 20 : 14) * sn(ph), mano_izq: 0,
  }
  return aimAt(p, 6)
}
action("flotar_agua", { duration: 1, loop: true }, cycle(ph => treadPose(ph, false)))
// en la superficie: igual, con la cabeza fuera, recta, y remando con más fuerza
action("superficie", { duration: 1.6, loop: true }, cycle(ph => treadPose(ph, true)))

// clavado (desde parado), medido en vídeo: un salto de cabeza desde el borde de una piscina, de
// perfil, a 100 fps. No salta hacia arriba: sube los brazos, se deja caer hacia delante girando
// sobre las puntas de los pies (la cadera se dobla hasta ~135° y las rodillas un poco) y, cuando el
// tronco llega casi a la horizontal, empuja con las piernas. Esta acción es la parte en cubierta
// (los dedos no se mueven). En el aire va clavado_aire, en las coordenadas del cuerpo (de pie): el
// juego gira el cuerpo entero alrededor del centro de masas a velocidad constante y lleva esa
// animación al ritmo del vuelo (0 al despegar, 1 al entrar en el agua).
// tronco girado T° hacia delante (desde de pie), cadera a "cad"° (180 = recta), rodillas a "rod"° y
// de puntillas "fi"°; en "o" el resto (torso, cabeza, brazos). La cadera va donde las piernas, con
// esos ángulos, dejan los dedos en su sitio.
function sobrePuntas(T, cad, rod, fi, o) {
  const torso = o.torso ?? 8, r = T - torso, m = torso - (180 - cad), q = 180 - rod, L = LEG.der
  const A = vAdd(PUNTA.der, vRot(TALON.der, fi))
  const H = vSub(vSub(A, vRot(vSub(L.a, L.k), r + m + q)), vRot(vSub(L.k, L.h), r + m))
  const c = vSub(vSub(H, HIP0), vRot(vSub(L.h, HIP0), r))
  const p = { ...o, torso, cadera: { x: c[0], y: c[1], rot: r }, muslo_der: m, pierna_der: q }
  return plant(plant(p, "der", A, fi), "izq", vAdd(PUNTA.izq, vRot(TALON.izq, fi)), fi)
}
// la lanza a lo largo del brazo cercano (d grados más arriba)
const conBrazo = (p, d = 0) => aimAt(p, -(88 + armSum(p)) + d)
const brazos = (b, a, cab, torso) => ({ torso, cabeza: { rot: cab, x: 0 },
  brazo_der: b, antebrazo_der: a, mano_der: 2, brazo_izq: b - 3, antebrazo_izq: a, mano_izq: 0 })
// [t, giro del tronco, cadera, rodillas, puntillas, brazo, antebrazo, cabeza, torso]
// empieza a caer mientras sube los brazos (sin quedarse de pie con ellos arriba); la caída se
// acelera (como un péndulo invertido: 120, 180 y 215°/s) y sigue a ~215°/s hasta despegar, lo que
// girará en el aire: el giro no frena al dejar la cubierta. Dura 0,55 s (0,69 en el vídeo). La
// cabeza va entre los brazos: el flaco la lleva adelantada, así que se echa atrás (como al nadar)
const CAIDA = [
  [0.15, 3, 178, 176, 6, -100, -12, 0, 4],
  [0.35, 14, 171, 171, 14, -164, -6, -14, 6],
  [0.55, 34, 157, 162, 22, -172, -4, -26, 8],
  [0.78, 62, 142, 150, 30, -158, -4, -34, 8],
  // carga (rodillas dobladas) y empuje: las piernas se estiran y sube a las puntas al despegar
  [0.88, 75, 138, 148, 34, -150, -4, -36, 8],
  [1, 92, 145, 174, 60, -140, -4, -38, 8],
]
const SALIDA = conBrazo(sobrePuntas(92, 145, 174, 60, brazos(-140, -4, -38, 8)))
// tras la clave del despegue va otra que sigue al mismo ritmo TRAS segundos más. No se ve (el juego
// pasa antes a la acción del aire), pero con la interpolación cúbica, sin ella, el movimiento
// frenaría al llegar al despegue. El juego despega TRAS segundos antes del final.
const TRAS = 0.05
function hastaDespegar(keys, dur) {
  const [ta, a] = keys[keys.length - 2], [tb, b] = keys[keys.length - 1]
  const total = dur + TRAS
  return { total, keys: [...keys.map(([t, p]) => [t * dur / total, p]), [1, mezclar(a, b, 1 + TRAS / ((tb - ta) * dur))]] }
}
const caida = hastaDespegar([
  [0, aimAt(stand({ cadera: { x: 0, y: 0, rot: 0 }, torso: 0, cabeza: { rot: 0, x: 0 },
    brazo_der: 0, antebrazo_der: 0, mano_der: 0, brazo_izq: 0, antebrazo_izq: 0, mano_izq: 0 }, FEET_R), 10)],
  ...CAIDA.map(([t, T, cad, rod, fi, b, a, cab, to]) => [t, conBrazo(sobrePuntas(T, cad, rod, fi, brazos(b, a, cab, to)))]),
], 0.55)
action("clavado", { duration: caida.total, loop: false }, caida.keys)
// en el aire, de pie (el juego pone el giro): la cadera se abre de 136° a 166°, las rodillas se
// estiran, los pies en punta y los brazos se alinean con el tronco para entrar por el mismo agujero
const deFrente = p => ({ ...p, cadera: { x: 0, y: 0, rot: 0 } })   // la misma forma, sin el giro
const enAire = (torso, cad, rod, pie, b, cab) => conBrazo({ ...brazos(b, -3, cab, torso), cadera: { x: 0, y: 0, rot: 0 },
  muslo_der: torso - (180 - cad), pierna_der: 180 - rod, pie_der: pie,
  muslo_izq: torso - (180 - cad) - 2, pierna_izq: 180 - rod, pie_izq: pie })
action("clavado_aire", { duration: 1, loop: false }, [
  [0, deFrente(SALIDA)],
  [0.35, enAire(8, 145, 176, 58, -158, -39)],
  [0.7, enAire(6, 155, 178, 62, -168, -40)],
  [1, enAire(4, 166, 178, 64, -175, -40)],
])

// clavado en carrera: llega corriendo y bate en el borde sin pararse. El pie de batida se queda
// clavado mientras el cuerpo pasa por encima (resbala hacia atrás a la velocidad de correr, como en
// correr), se agacha un poco, sube la otra rodilla y lleva los brazos de atrás hacia delante y
// arriba. Deja la cubierta a los 0,16 s (lo que dura un apoyo corriendo); el vuelo va en su acción
// _aire, como en clavado. Hay una por pie: el juego elige la del pie que iba a pisar.
const V_CORRER = 2360
// mezcla dos poses (también los giros y desplazamientos anidados, como cadera: { x, y, rot })
function mezclar(a, b, f) {
  const out = {}
  for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) {
    const x = a[k] ?? b[k], y = b[k] ?? a[k]
    out[k] = typeof x === "object" ? mezclar(x, y, f) : x + (y - x) * f
  }
  return out
}
function batida(s) {
  const o = s === "der" ? "izq" : "der"
  const contacto = s === "der" ? R0 : swapSides(R0)
  const A0 = ankleOf(contacto, s)
  const pisa = t => [A0[0] - V_CORRER * t, A0[1]]
  const cuerpo = (c, to, ca, b) => ({ cadera: { x: c[0], y: c[1], rot: c[2] }, torso: to, cabeza: { rot: ca, x: 0 },
    brazo_der: b[0], antebrazo_der: b[1], mano_der: b[2], brazo_izq: b[3], antebrazo_izq: b[4], mano_izq: b[5] })
  const otra = (m, q, f) => ({ ["muslo_" + o]: m, ["pierna_" + o]: q, ["pie_" + o]: f })
  const toes = pisa(0.16)
  // pisa como en correr
  const k0 = carry(contacto, 8, 0.2)
  // carga: la cadera baja, el cuerpo se adelanta y los dos brazos se van atrás
  const k1 = aimAt(plant({ ...cuerpo([0, 44, 3], 22, 4, [52, -38, 0, 58, -32, 0]), ...otra(-22, 105, -20) }, s, pisa(0.08)), -12)
  // batida: la pierna de atrás estirada y de puntillas, la otra rodilla arriba, brazos arriba. Muy
  // echado hacia delante (en el vídeo de referencia, el tronco sale a ~25° de la horizontal; aquí a
  // ~45°): así casi no sube y el giro del vuelo ya viene empezado.
  const k2 = aimAt(plant({ ...cuerpo([34, 44, 36], 26, -26, [-118, -26, 5, -128, -20, 0]), ...otra(-96, 92, -10) }, s, [toes[0], toes[1] - 50], 38), 45)
  // entre medias, el pie de batida vuelve a clavarse donde toca (si no, resbala entre fotogramas)
  const entre = (a, b, apoyo, pie) => plant(mezclar(a, b, 0.5), s, apoyo, pie)
  // en cubierta: deja la cubierta al final (0,16 s, lo que dura un apoyo corriendo)
  const cubierta = [
    [0, k0],
    [1 / 4, entre(k0, k1, pisa(0.04), 0)],
    [1 / 2, k1],
    [3 / 4, entre(k1, k2, [pisa(0.12)[0], pisa(0.12)[1] - 12], 12)],
    [1, k2],
  ]
  // en el aire, de pie (el juego pone el giro): junta las piernas, endereza el tronco y estira los
  // brazos por encima de la cabeza; al final dobla un poco la cadera, como en el vídeo
  const aire = (to, cab, b, ms, qs, ps, mo, qo, po) => conBrazo({ ...cuerpo([0, 0, 0], to, cab, b),
    ["muslo_" + s]: ms, ["pierna_" + s]: qs, ["pie_" + s]: ps, ...otra(mo, qo, po) })
  const vuelo = [
    [0, deFrente(k2)],
    [0.3, aire(16, -30, [-160, -8, 3, -165, -8, 0], 12, 14, 50, 4, 30, 40)],
    [0.65, aire(8, -42, [-172, -4, 2, -175, -4, 0], 6, 10, 60, 4, 12, 58)],
    [1, aire(4, -40, [-175, -3, 2, -177, -3, 0], -10, 10, 64, -12, 10, 64)],
  ]
  return { cubierta, vuelo }
}
for (const [nombre, s] of [["clavado_carrera", "der"], ["clavado_carrera_izq", "izq"]]) {
  const b = batida(s), c = hastaDespegar(b.cubierta, 0.16)
  action(nombre, { duration: c.total, loop: false }, c.keys)
  action(nombre + "_aire", { duration: 1, loop: false }, b.vuelo)
}

// ---------- manos apoyadas: IK de dos huesos para el brazo ----------
const ARM = { der: { s: [362, 398], e: [373, 565], h: [388, 730] }, izq: { s: [378, 398], e: [389, 565], h: [404, 730] } }
const TORSO0 = [395, 556]
const wrap = a => ((a + 180) % 360 + 360) % 360 - 180
function shoulderOf(p, s) {
  const c = p.cadera || {}, r = c.rot || 0
  const hip = [HIP0[0] + (c.x || 0), HIP0[1] + (c.y || 0)]
  const torso = vAdd(hip, vRot(vSub(TORSO0, HIP0), r))
  return vAdd(torso, vRot(vSub(ARM[s].s, TORSO0), r + rotOf(p, "torso")))
}
// pone la mano "s" en H con el codo doblado hacia delante (antebrazo < 0); giro = hacia dónde
// señalan los dedos en pantalla (0 = hacia delante, 90 = hacia abajo)
function reach(p, s, H, giro = 0) {
  const A = ARM[s], S = shoulderOf(p, s), base = ((p.cadera || {}).rot || 0) + rotOf(p, "torso")
  const a = vLen(vSub(A.e, A.s)), b = vLen(vSub(A.h, A.e))
  const D = vSub(H, S), d = clamp(vLen(D), Math.abs(a - b) + 0.5, a + b - 0.5)
  let best = null
  for (const sg of [1, -1]) {
    const thW = gd(Math.atan2(D[1], D[0])) + sg * gd(Math.acos((a * a + d * d - b * b) / (2 * a * d)))
    const E = vAdd(S, [a * Math.cos(dg(thW)), a * Math.sin(dg(thW))])
    const brazo = wrap(thW - vAng(A.s, A.e) - base)
    const ante = wrap(vAng(E, H) - vAng(A.e, A.h) - base - brazo)
    if (!best || (ante <= 0 && best.ante > 0)) best = { brazo, ante }
  }
  const total = base + best.brazo + best.ante
  return { ...p, ["brazo_" + s]: best.brazo, ["antebrazo_" + s]: best.ante, ["mano_" + s]: wrap(giro - 90 - total) }
}

// subir a la balsa: desde el agua, agarrado al borde con las dos manos (la lanza sobre la cubierta),
// tira hasta sacar el pecho, empuja con los brazos estirados, sube una rodilla, apoya el pie y se
// levanta. El origen es donde acaba de pie, mirando hacia dentro; el borde le queda 162 detrás y
// el agua, 183 por debajo de la cubierta (como en el juego).
const DECK = LEG.der.a[1] + 4, EDGE = 425 - 162
const HAND_D = [EDGE + 70, DECK], HAND_I = [EDGE + 120, DECK]
const hands = p => reach(reach(p, "der", HAND_D, 0), "izq", HAND_I, 0)
// Como de verdad (2–5 s): cuelga con el agua por los hombros, se hunde un poco con los brazos
// estirados (preparación), da una patada fuerte y tira, echa el vientre sobre la cubierta, sube una
// rodilla y se levanta. La patada es en 0,3 (el juego salpica ahí, en los pies).
action("subir", { duration: 1.9, loop: false }, [
  [0, aimAt(hands({ cadera: { x: -262, y: 990, rot: 0 }, torso: 8, cabeza: { rot: -26, x: 0 },
    muslo_der: -10, pierna_der: 30, pie_der: 50, muslo_izq: 10, pierna_izq: 26, pie_izq: 55 }), 2)],
  [0.16, aimAt(hands({ cadera: { x: -268, y: 1075, rot: 0 }, torso: 4, cabeza: { rot: -30, x: 0 },
    muslo_der: -48, pierna_der: 96, pie_der: 40, muslo_izq: -40, pierna_izq: 88, pie_izq: 45 }), 2)],
  [0.3, aimAt(hands({ cadera: { x: -240, y: 760, rot: 0 }, torso: 18, cabeza: { rot: -12, x: 0 },
    muslo_der: 14, pierna_der: 4, pie_der: 62, muslo_izq: 8, pierna_izq: 6, pie_izq: 60 }), 1)],
  [0.42, aimAt(hands({ cadera: { x: -200, y: 470, rot: 0 }, torso: 62, cabeza: { rot: -18, x: 0 },
    muslo_der: 20, pierna_der: 10, pie_der: 55, muslo_izq: 12, pierna_izq: 14, pie_izq: 55 }), 0)],
  [0.56, aimAt(hands({ cadera: { x: -165, y: 280, rot: 0 }, torso: 48, cabeza: { rot: -4, x: 0 },
    muslo_der: -95, pierna_der: 125, pie_der: 30, muslo_izq: 12, pierna_izq: 22, pie_izq: 50 }), 0)],
  // de pie a medias: el pie cercano ya delante y el lejano apoyado atrás, junto al borde
  [0.75, aimAt(plant(plant({ cadera: { x: -70, y: 170, rot: 0 }, torso: 30, cabeza: { rot: 4, x: 0 },
    brazo_der: -30, antebrazo_der: -60, mano_der: 0, brazo_izq: -20, antebrazo_izq: -50, mano_izq: 0 },
    "der", vAdd(LEG.der.a, [20, 0])), "izq", [EDGE + 50, LEG.izq.a[1]], 20), 8)],
  // el pie lejano da el paso, levantado, hasta su sitio
  [0.88, aimAt(plant(plant({ cadera: { x: -24, y: 60, rot: 0 }, torso: 12, cabeza: { rot: 2, x: 0 },
    brazo_der: -12, antebrazo_der: -30, mano_der: 0, brazo_izq: -8, antebrazo_izq: -25, mano_izq: 0 },
    "der", vAdd(LEG.der.a, [8, 0])), "izq", vAdd(LEG.izq.a, [-60, -70]), 10), 9)],
  [1, aimAt(stand({ cadera: { x: 0, y: 0, rot: 0 }, torso: 0, cabeza: { rot: 0, x: 0 },
    brazo_der: 0, antebrazo_der: 0, mano_der: 0, brazo_izq: 0, antebrazo_izq: 0, mano_izq: 0 }, FEET_R), 10)],
])

// despertar: tras el desmayo, en cuclillas en la cubierta con las manos apoyadas; tose dos veces (el
// tronco se encoge y la cabeza cae) y se levanta. Los pies no se mueven en ningún momento (IK) y las
// manos, mientras apoyan, tampoco.
const SUELO = LEG.der.a[1]
const cuclillas = (cy, rot, torso, cab, mano) => {
  const p = stand({ cadera: { x: -40, y: cy, rot }, torso, cabeza: { rot: cab, x: 0 } }, FEET_R)
  return aimAt(reach(reach(p, "der", [mano, SUELO], 0), "izq", [mano + 30, SUELO], 0), 0)
}
const levantando = (cy, rot, torso, cab, b) => aimAt(stand({ cadera: { x: -20, y: cy, rot }, torso, cabeza: { rot: cab, x: 0 },
  brazo_der: b, antebrazo_der: -30, mano_der: 0, brazo_izq: b - 10, antebrazo_izq: -25, mano_izq: 0 }, FEET_R), 6)
action("despertar", { duration: 1.6, loop: false }, [
  [0, cuclillas(330, 40, 30, 40, 560)],
  [0.12, cuclillas(345, 34, 44, 52, 560)],
  [0.24, cuclillas(330, 42, 28, 34, 560)],
  [0.36, cuclillas(347, 34, 46, 54, 560)],
  [0.5, cuclillas(325, 42, 24, 16, 560)],
  [0.64, levantando(250, 30, 18, 8, -50)],
  [0.82, levantando(110, 12, 8, 4, -24)],
  [1, stand({ cadera: { x: 0, y: 0, rot: 0 }, torso: 0, cabeza: { rot: 0, x: 0 },
    brazo_der: 0, antebrazo_der: 0, mano_der: 0, brazo_izq: 0, antebrazo_izq: 0, mano_izq: 0, lanza: 0 }, FEET_R)],
])

// pinchar bajo el agua: recoge el brazo de la lanza hasta la cabeza (sin dejar de apuntar), da una
// patada fuerte y clava con el brazo estirado del todo; luego vuelve a la pose de nadar. El juego gira
// el cuerpo hacia la mira, así que la lanza va siempre a lo largo del cuerpo.
const SW0 = swimPose(0, 0)
const stab = (o) => aimAt({ ...SW0, ...o, cadera: { rot: 0, x: 0, y: 0, ...(o.cadera || {}) }, cabeza: { rot: -50, x: 0, ...(o.cabeza || {}) } }, 89)
// el brazo libre: en la anticipación se adelanta (por debajo de la lanza) y en la estocada barre hacia
// atrás pegado al cuerpo, como una brazada, en vez de colgar hacia el fondo
// Las claves se interpolan cada 0,05 (en línea recta, con la lanza recolocada en cada una): entre
// claves lejanas, Godot curva cada hueso por su lado y el asta daba bandazos de hasta 60°.
const aplana = o => Object.fromEntries(Object.entries(o).flatMap(([k, v]) => typeof v === "object" ? Object.entries(v).map(([kk, vv]) => [k + "." + kk, vv]) : [[k, v]]))
const desaplana = r => { const o = {}; for (const [k, v] of Object.entries(r)) { if (k.includes(".")) { const [a, b] = k.split("."); (o[a] = o[a] || {})[b] = v } else o[k] = v } return o }
function densas(claves, paso, pose) {
  const base = aplana({ ...SW0, lanza: undefined, cabeza: { rot: -50, x: 0 } })
  delete base.lanza
  const llenas = claves.map(([t, o]) => [t, { ...base, ...aplana(o) }])
  const out = []
  for (let t = 0; t <= 1 + 1e-9; t += paso) {
    const j = Math.min(llenas.length - 2, llenas.findIndex(([tb]) => tb >= t - 1e-9) - 1 < 0 ? 0 : llenas.findIndex(([tb]) => tb >= t - 1e-9) - 1)
    const [ta, a] = llenas[j], [tb, b] = llenas[j + 1]
    const f = Math.max(0, Math.min(1, (t - ta) / (tb - ta)))
    const r = {}
    for (const k of Object.keys(a)) r[k] = a[k] + ((b[k] ?? a[k]) - a[k]) * f
    out.push([Math.round(t * 1000) / 1000, pose(desaplana(r))])
  }
  return out
}
action("golpe_agua", { duration: 0.4, loop: false }, densas([
  [0, {}],
  [0.3, { torso: 4, cadera: { x: -6, y: 10 }, brazo_der: -140, antebrazo_der: -112, mano_der: 12,
    brazo_izq: -160, antebrazo_izq: -8, muslo_der: 16, pierna_der: 34, pie_der: 50, muslo_izq: -8, pierna_izq: 10, pie_izq: 62 }],
  [0.55, { torso: -4, cadera: { x: 8, y: -40 }, brazo_der: -179, antebrazo_der: 0, mano_der: 0,
    brazo_izq: 14, antebrazo_izq: -10, muslo_der: -16, pierna_der: 4, pie_der: 70, muslo_izq: 12, pierna_izq: 18, pie_izq: 55 }],
  [0.72, { torso: -3, cadera: { x: 6, y: -30 }, brazo_der: -178, antebrazo_der: -2, mano_der: 0,
    brazo_izq: 10, antebrazo_izq: -12, muslo_der: -10, pierna_der: 8, pie_der: 66, muslo_izq: 8, pierna_izq: 16, pie_izq: 58 }],
  [1, {}],
], 0.05, stab))
