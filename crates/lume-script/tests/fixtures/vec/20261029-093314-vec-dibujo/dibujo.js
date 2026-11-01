// dibujo.js: cada forma se dibuja encima de las anteriores.
// Lienzo de 720x1280: (0, 0) arriba a la izquierda, x hacia la derecha, y hacia abajo.
canvas({ width: 720, height: 1280, background: null }) // null = fondo transparente (objeto suelto)

const P = {
  ink: "#1e1c21",
  skin: "#e8b07e", skinSh: "#b9794f", skinLt: "#f6cf9c",
  hair: "#383a40", beard: "#a8a9a5", beardSh: "#86878a",
  jacket: "#7b7d5f", jacketSh: "#5c5e46", jacketLt: "#959877",
  lapel: "#514b5e", lapelSh: "#3d384a",
  shirt: "#c8cfd5", shirtSh: "#a3acb6",
  red: "#b5372b", glove: "#3b3f47", gloveLt: "#545964",
  lens: "#6fa9a2", gold: "#d8c14a", cigar: "#7a4a2e",
}
const L = (fill, w = 6) => ({ fill, stroke: P.ink, width: w, join: "round" })
const C = (pts) => pts.map(([x, y]) => [x, y, 0]) // polígono de esquinas afiladas, suave por blob
const ink = (pts, w = 4) => brush(pts, { width: w, color: P.ink, taper: [0.3, 0.4] })

group({ move: [0, 100] }, () => group({ scale: 0.8, around: [0, 0] }, () => {

  layer("chaqueta-derecha", () => {
    // panel oliva derecho, hueco oscuro y solapa violeta
    poly([[520, 560], [625, 700], [612, 850], [660, 1000], [715, 1125], [600, 1160], [560, 900]], L(P.jacket))
    poly([[545, 780], [592, 820], [662, 1020], [692, 1132], [612, 1158], [572, 900]], L(P.ink, 3))
    const lap = poly([[430, 470], [505, 462], [602, 488], [548, 570], [578, 690], [626, 703], [600, 800], [565, 845], [470, 700]], L(P.lapel))
    clip(lap, () => {
      poly([[548, 570], [578, 690], [626, 703], [600, 800], [565, 845], [530, 700]], { fill: P.lapelSh })
    })
    ink([[548, 572], [560, 640], [578, 690]], 3)
  })

  layer("cuello-camiseta", () => {
    const neck = poly([[335, 500], [505, 500], [512, 620], [462, 705], [348, 640]], L(P.skin))
    clip(neck, () => {
      poly([[335, 500], [505, 500], [500, 560], [420, 600], [340, 570]], { fill: P.skinSh })
      ink([[400, 630], [430, 645], [455, 640]], 3)
    })
    const shirt = poly([[348, 632], [410, 688], [462, 706], [520, 690], [562, 830], [602, 1000], [612, 1180], [472, 1180], [440, 1000], [392, 780]], L(P.shirt))
    clip(shirt, () => {
      poly([[520, 690], [562, 830], [602, 1000], [612, 1180], [560, 1180], [540, 900], [500, 760]], { fill: P.shirtSh })
      ink([[470, 880], [530, 870], [570, 880]], 2.5)
    })
  })

  layer("chaqueta-izquierda", () => {
    // cuerpo, cuello levantado, solapa, charretera roja y botón
    const body = poly([[160, 440], [330, 445], [340, 555], [350, 635], [395, 790], [440, 1000], [476, 1186], [330, 1186], [200, 1140], [150, 1082], [75, 1000], [48, 950], [90, 880], [72, 780], [75, 640], [160, 600], [195, 560]], L(P.jacket, 7))
    clip(body, () => {
      poly([[330, 445], [340, 555], [350, 635], [395, 790], [440, 1000], [476, 1186], [400, 1186], [370, 1000], [330, 800], [300, 700]], { fill: P.jacketSh })
      poly([[160, 440], [330, 445], [290, 480], [235, 530], [200, 562]], { fill: P.jacketSh })
      poly([[75, 640], [160, 600], [195, 560], [210, 620], [150, 700], [90, 760]], { fill: P.jacketLt, opacity: 0.8 })
      poly([[90, 880], [200, 840], [300, 870], [330, 960], [200, 940], [80, 960]], { fill: P.jacketSh, opacity: 0.7 })
    })
    // pliegues de la solapa y la tela
    ink([[200, 562], [250, 640], [300, 700], [330, 760]], 4)
    ink([[300, 700], [350, 640]], 3)
    ink([[240, 520], [290, 610]], 3)
    ink([[110, 760], [170, 800], [240, 790]], 2.5)
    ink([[150, 700], [190, 760]], 2.5)
    ink([[260, 820], [300, 880], [330, 960]], 3)
    ink([[330, 980], [360, 1060], [390, 1120]], 2.5)
    // charretera
    const ep = poly([[72, 642], [160, 614], [232, 640], [262, 700], [244, 738], [216, 664], [160, 652], [88, 664]], L(P.red, 5))
    clip(ep, () => poly([[216, 664], [262, 700], [244, 738], [232, 640]], { fill: shade(P.red, -0.35) }))
    // botón
    circle(340, 822, 28, L(P.gold, 5))
    circle(340, 822, 17, { fill: shade(P.gold, -0.3) })
    circle(333, 814, 8, { fill: shade(P.gold, 0.5) })
  })

  layer("brazo-izquierdo", () => {
    // manga con puño, antebrazo y guante
    const sl = poly([[48, 950], [250, 918], [288, 1062], [150, 1102], [75, 1010]], L(P.jacket, 7))
    clip(sl, () => poly([[180, 1000], [288, 1062], [150, 1102], [120, 1040]], { fill: P.jacketSh }))
    ink([[90, 980], [200, 965], [255, 1000]], 3)
    const arm = poly([[150, 1095], [280, 1060], [340, 1120], [330, 1192], [205, 1182]], L(P.skin, 6))
    clip(arm, () => poly([[150, 1095], [280, 1060], [300, 1090], [180, 1120]], { fill: P.skinSh }))
    const gl = poly([[298, 1132], [418, 1118], [500, 1188], [482, 1218], [328, 1205]], L(P.glove, 6))
    clip(gl, () => poly([[298, 1132], [418, 1118], [440, 1140], [320, 1160]], { fill: P.gloveLt }))
  })

  layer("brazo-derecho", () => {
    // antebrazo, guante sin dedos, dedos y cigarrillo
    const fa = poly([[640, 860], [712, 880], [692, 995], [662, 1005], [638, 950]], L(P.skin, 6))
    clip(fa, () => poly([[680, 860], [712, 880], [692, 995], [670, 990]], { fill: P.skinSh }))
    const finger = (f) => {
      const s = poly(f, L(P.skin, 5))
      clip(s, () => poly(shift(f, 8, 8), { fill: P.skinSh, opacity: 0.55 }))
    }
    finger([[680, 705], [684, 652], [714, 616], [738, 640], [722, 705]]) // pulgar
    finger([[762, 705], [764, 655], [760, 640], [812, 640], [844, 662], [846, 705]]) // anular
    finger([[772, 705], [774, 640], [744, 612], [754, 586], [804, 598], [834, 618], [836, 705]]) // medio
    // cigarrillo entre índice y medio
    poly([[812, 596], [866, 590], [868, 606], [814, 612]], L("#f2f0ea", 3))
    poly([[858, 591], [868, 590], [870, 606], [860, 607]], { fill: "#d2462e", glow: { blur: 6, color: "#ff6a3a" } })
    finger([[794, 705], [796, 604], [730, 546], [744, 514], [818, 562], [832, 600], [830, 705]]) // índice
    ink([[798, 572], [812, 584]], 2.5)
    const g = poly([[652, 700], [702, 652], [760, 640], [800, 652], [845, 682], [838, 722], [762, 850], [748, 885], [700, 884], [648, 822], [668, 760]], L(P.glove, 6))
    clip(g, () => {
      poly([[760, 640], [800, 652], [845, 682], [838, 722], [762, 850], [748, 885], [730, 860], [790, 720]], { fill: shade(P.glove, -0.3) })
      poly([[652, 700], [702, 652], [740, 650], [700, 720], [670, 780]], { fill: P.gloveLt })
    })
    ink([[662, 822], [748, 830]], 3)
    ink([[700, 720], [760, 700], [810, 700]], 2.5)
  })

  layer("cabeza", () => {
    // oreja, cara, sombras
    const ear = poly([[288, 318], [246, 312], [234, 352], [256, 384], [290, 388]], L(P.skin, 6))
    clip(ear, () => ink([[262, 330], [270, 355], [282, 365]], 3))
    const face = poly([[278, 300], [300, 238], [482, 238], [506, 300], [512, 420], [506, 530], [340, 532], [302, 425], [288, 410]], L(P.skin, 7))
    clip(face, () => {
      poly([[455, 330], [512, 330], [512, 430], [440, 400]], { fill: P.skinSh, opacity: 0.8 })
      poly([[278, 240], [510, 240], [510, 262], [278, 280]], { fill: P.skinSh, opacity: 0.6 })
      poly([[300, 350], [340, 345], [345, 400], [310, 410]], { fill: P.skinLt, opacity: 0.6 })
    })
    // nariz
    ink([[438, 330], [458, 368], [432, 378]], 4)
    ink([[318, 360], [330, 400]], 2.5)
  })

  layer("barba-bigote", () => {
    const beard = poly([[290, 380], [322, 405], [400, 400], [480, 418], [508, 420], [508, 530], [342, 534], [318, 450]], L(P.beard, 7))
    clip(beard, () => {
      poly([[440, 430], [508, 420], [508, 530], [460, 530]], { fill: P.beardSh })
      ink([[370, 500], [400, 492], [430, 500]], 2.5)
      ink([[330, 470], [345, 500]], 2)
    })
    ink([[395, 455], [430, 452], [455, 458]], 3.5) // boca
    poly([[318, 430], [355, 398], [420, 382], [470, 395], [502, 425], [478, 432], [440, 412], [400, 420], [356, 442]], L(P.hair, 4))
  })

  layer("pelo", () => {
    const h = poly([[218, 258], [276, 236], [268, 158], [310, 196], [345, 150], [370, 178], [430, 122], [420, 182], [562, 195], [495, 232], [548, 258], [480, 262], [452, 232], [402, 252], [350, 222], [312, 252], [300, 302], [276, 302]], L(P.hair, 6))
    clip(h, () => poly([[345, 150], [370, 178], [430, 122], [420, 182], [380, 200], [340, 190]], { fill: shade(P.hair, 0.25) }))
  })

  layer("gafas", () => {
    // cejas fruncidas asomando sobre las lentes
    brush([[322, 262], [370, 256], [412, 272]], { width: [4, 11, 6], color: P.hair, taper: [0.2, 0.3] })
    brush([[440, 276], [478, 264], [505, 270]], { width: [6, 10, 3], color: P.hair, taper: [0.2, 0.4] })
    line(286, 302, 330, 296, { stroke: P.ink, width: 10, cap: "round" })
    line(400, 300, 438, 302, { stroke: P.ink, width: 9, cap: "round" })
    for (const [cx, cy, rx, ry] of [[365, 306, 46, 40], [472, 312, 40, 37]]) {
      const lens = ellipse(cx, cy, rx, ry, { fill: P.lens, stroke: P.ink, width: 16 })
      clip(lens, () => {
        ellipse(cx + 8, cy + 12, rx, ry, { fill: shade(P.lens, -0.35), opacity: 0.7 })
        ellipse(cx - 8, cy - 8, rx * 0.5, ry * 0.4, { fill: shade(P.lens, 0.5), opacity: 0.8 })
      })
      // contorno superior grueso
      brush(arc(cx, cy, rx + 4, -70, 70, 8).map(([x, y]) => [x, y]), { width: 7, color: P.ink })
    }
  })

  layer("puro-humo", () => {
    poly([[505, 408], [590, 388], [600, 420], [512, 434]], L(P.cigar, 5))
    ink([[530, 410], [540, 425]], 2)
    ink([[560, 402], [566, 420]], 2)
    poly([[588, 390], [598, 388], [604, 418], [596, 420]], { fill: "#c8382a", glow: { blur: 8, color: "#ff6a3a" } })
    const smoke = (pts, w) => {
      brush(pts, { width: w.map((v) => v + 5), color: "#8d929a88", taper: [0.2, 0.5] })
      brush(pts, { width: w, color: "#eef0f2", taper: [0.2, 0.5] })
    }
    smoke([[610, 410], [650, 404], [705, 385], [690, 345], [745, 330], [725, 275], [765, 230], [785, 182]], [3, 12, 9, 5])
    smoke([[872, 594], [860, 560], [882, 520], [866, 480], [888, 450]], [2, 6, 3])
  })

  layer("textura", () => {
    // sin grano: rompería la transparencia del fondo
  })
}))
