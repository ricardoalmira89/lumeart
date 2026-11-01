canvas({ width: 1024, height: 768, background: null });
setLight(-1, -1);

const P = {
  ink: "#0f2a14",      // contorno oscuro verde
  dark: "#24642c",     // quitina oscura
  base: "#47b03a",     // quitina verde
  light: "#8fe05a",    // luz de la quitina
  purple: "#7b3fb0",   // placas
  purpleD: "#46206e",  // sombra de placas
  purpleL: "#b07ae0",  // luz de placas
  amber: "#f4a52c",    // detalles ámbar
  amberD: "#b0600f",
  lime: "#d4ff4f",     // brillo de incubación
  white: "#ffffff",
};
const CX = 512;

// ---------- Geometría ----------
const wingL = [[340, 205], [250, 150], [120, 112, 0], [78, 168], [118, 238, 0], [92, 318, 0], [190, 362], [330, 420]];
const wingR = wingL.map(([x, y, s]) => s === undefined ? [2 * CX - x, y] : [2 * CX - x, y, s]);
const abdPts = ring(CX, 330, 262, 278, 14);
const tailPts = [[CX, 22, 0], [CX + 46, 80], [CX - 46, 80]];
const headPts = [[CX - 170, 610], [CX - 120, 560], [CX, 545], [CX + 120, 560], [CX + 170, 610], [CX + 120, 690], [CX, 712], [CX - 120, 690]];
const mandL = [[CX - 138, 650], [CX - 134, 700], [CX - 96, 734], [CX - 28, 742, 0], [CX - 74, 714], [CX - 90, 680]];
const mandR = mandL.map(([x, y, s]) => s === undefined ? [2 * CX - x, y] : [2 * CX - x, y, s]);
// espolones laterales
const spurs = [];
for (const [y, len] of [[150, 40], [300, 34], [470, 40]]) {
  const t = Math.asin((y - 330) / 278);
  const x = 262 * Math.cos(t);
  spurs.push([[CX - x + 14, y - 26], [CX - x - len, y + 6, 0], [CX - x + 14, y + 22]]);
  spurs.push([[CX + x - 14, y - 26], [CX + x + len, y + 6, 0], [CX + x - 14, y + 22]]);
}

function silhouette(st) {
  blob(wingL, st); blob(wingR, st);
  for (const s of spurs) blob(s, st);
  blob(tailPts, st);
  blob(abdPts, st);
  blob(mandL, st); blob(mandR, st);
  blob(headPts, st);
}

layer("borde", () => {
  // borde blanco de pegatina y contorno oscuro
  silhouette({ fill: P.white, stroke: P.white, width: 44, join: "round" });
  silhouette({ fill: P.ink, stroke: P.ink, width: 16, join: "round" });
});

layer("alas", () => {
  // alas cortas y rígidas
  for (const [pts, side] of [[wingL, -1], [wingR, 1]]) {
    const w = blob(pts, { fill: P.dark });
    clip(w, () => {
      // placa clara superior
      blob(shift(grow(pts, 0.86), side * 8, -14), { fill: P.base });
      // nervaduras moradas
      const root = [CX + side * 250, 300];
      for (const tip of [[CX + side * 400, 125], [CX + side * 440, 205], [CX + side * 420, 310], [CX + side * 330, 380]]) {
        brush([root, [lerp(root[0], tip[0], 0.5), lerp(root[1], tip[1], 0.5) + 6], tip], { width: [10, 7, 3], taper: [0, 0.6], color: P.purpleD });
      }
    });
    // borde de ataque ámbar
    const a = pts[1], b = pts[2];
    brush([[a[0], a[1] + 6], [lerp(a[0], b[0], 0.5), lerp(a[1], b[1], 0.5) + 6], [b[0] + side * -6, b[1] + 10]], { width: [6, 9, 4], taper: [0.2, 0.4], color: P.amber });
  }
});

const abd = blob(abdPts, { fill: P.dark, hidden: true });
layer("abdomen", () => {
  // espolones y cola
  for (const s of spurs) {
    const sp = blob(s, { fill: P.amber });
    clip(sp, () => blob(shift(s, 0, 12), { fill: P.amberD }));
  }
  const tl = blob(tailPts, { fill: P.dark });
  // cuerpo: sombra de un tono, base desplazada hacia la luz
  blob(abdPts, { fill: P.dark });
  clip(abd, () => {
    blob(shift(grow(abdPts, 0.97), -22, -26), { fill: P.base });
    // segmentos transversales
    for (let i = 0; i < 8; i++) {
      const y = 95 + i * 66;
      brush([[CX - 280, y - 22], [CX - 140, y + 4], [CX, y + 12], [CX + 140, y + 4], [CX + 280, y - 22]], { width: [4, 11, 13, 11, 4], taper: [0.05, 0.05], color: P.ink });
      // luz bajo cada junta (el segmento sobresale)
      brush([[CX - 230, y - 6], [CX - 120, y + 17], [CX - 20, y + 24]], { width: 7, taper: [0.4, 0.5], color: alpha(P.light, 0.85) });
    }
    // placa dorsal central morada
    const spine = blob([[CX, 60], [CX + 44, 140], [CX + 52, 330], [CX + 40, 520], [CX, 590], [CX - 40, 520], [CX - 52, 330], [CX - 44, 140]], { fill: P.purple, stroke: P.ink, width: 8 });
    clip(spine, () => {
      rect(CX, 40, 80, 600, { fill: P.purpleD });
      for (let i = 0; i < 8; i++) brush([[CX - 60, 95 + i * 66], [CX, 107 + i * 66], [CX + 60, 95 + i * 66]], { width: 7, taper: [0, 0], color: P.ink });
      brush([[CX - 26, 120], [CX - 34, 330], [CX - 24, 520]], { width: 8, taper: [0.3, 0.3], color: P.purpleL });
    });
    // nódulos ámbar a lo largo de la espina
    for (const y of [190, 330, 470]) {
      circle(CX, y, 17, { fill: P.amber, stroke: P.ink, width: 6 });
      circle(CX - 5, y - 5, 5, { fill: P.white });
    }
  });
});

layer("compuertas", () => {
  // cuatro compuertas de incubación
  function hatch_(cx, cy, rx, ry, rot) {
    const pts = turn(ring(cx, cy, rx, ry, 12), rot);
    blob(pts, { fill: P.ink, stroke: P.ink, width: 16 });
    // halo verde tenue alrededor
    const h = blob(pts, { fill: P.purple });
    clip(h, () => {
      blob(shift(pts, 12, 14), { fill: P.purpleD });
      blob(shift(grow(pts, 0.55), -16, -22), { fill: P.purpleL, opacity: 0.7 });
      // brillo de incubación por las juntas (cuatro placas que se juntan)
      const c = [cx, cy];
      for (const a of [rot, rot + 90]) {
        const p1 = polar(cx, cy, ry * 1.2, a - 90), p2 = polar(cx, cy, ry * 1.2, a + 90);
        line(p1[0], p1[1], p2[0], p2[1], { stroke: P.lime, width: 12, glow: { blur: 8, color: alpha(P.lime, 0.7) }, opacity: 0.85 });
        line(p1[0], p1[1], p2[0], p2[1], { stroke: P.ink, width: 4 });
      }
      circle(cx, cy, 13, { fill: P.lime, glow: { blur: 10, color: P.lime } });
      circle(cx, cy, 6, { fill: P.ink });
      // remaches ámbar en cada placa
      for (const a of [45, 135, 225, 315]) {
        const ang = (a + rot - 90) * Math.PI / 180;
        const px = cx + Math.cos(ang) * rx * 0.62, py = cy + Math.sin(ang) * ry * 0.62;
        circle(px, py, 6, { fill: P.amber, stroke: P.ink, width: 3 });
      }
    });
    // brillo blanco marcado
    brush(turn([[cx - rx * 0.62, cy - ry * 0.1], [cx - rx * 0.5, cy - ry * 0.55], [cx - rx * 0.15, cy - ry * 0.78]], rot), { width: [3, 8, 3], color: P.white });
  }
  mirror(CX, () => {
    hatch_(CX - 160, 210, 62, 80, -16);
    hatch_(CX - 168, 425, 64, 82, 14);
  });
});

layer("cabeza", () => {
  // mandíbulas ámbar
  for (const m of [mandL, mandR]) {
    const md = blob(m, { fill: P.amber });
    clip(md, () => blob(shift(m, 8, 10), { fill: P.amberD }));
  }
  brush([[CX - 124, 700], [CX - 100, 724], [CX - 60, 734]], { width: [2, 6, 2], color: alpha(P.white, 0.9) });
  // placa frontal
  const hd = blob(headPts, { fill: P.dark });
  clip(hd, () => {
    blob(shift(grow(headPts, 0.95), -14, -14), { fill: P.base });
    // caparazón morado frontal
    const cap = blob([[CX - 150, 600], [CX - 100, 568], [CX, 558], [CX + 100, 568], [CX + 150, 600], [CX + 70, 612], [CX, 640], [CX - 70, 612]], { fill: P.purple, stroke: P.ink, width: 7 });
    clip(cap, () => {
      rect(CX - 160, 590, 320, 60, { fill: P.purpleD });
      brush([[CX - 120, 586], [CX - 40, 570], [CX + 30, 570]], { width: [3, 7, 3], color: P.purpleL });
    });
    // ojos ámbar
    mirror(CX, () => {
      ellipse(CX - 118, 630, 18, 12, { fill: P.amber, stroke: P.ink, width: 5, rotate: 25 });
      circle(CX - 122, 627, 4, { fill: P.white });
    });
  });
  // junta entre abdomen y cabeza
  brush([[CX - 150, 572], [CX, 548], [CX + 150, 572]], { width: [6, 12, 6], taper: [0.1, 0.1], color: P.ink });
  // boca-cañón
  circle(CX, 676, 44, { fill: P.ink });
  circle(CX, 676, 34, { fill: "#1d4a1c", stroke: P.dark, width: 4 });
  circle(CX, 680, 20, { fill: P.lime, glow: { blur: 16, color: alpha(P.lime, 0.8) } });
  circle(CX, 682, 9, { fill: "#f6ffd0" });
  // dientes del cañón
  repeat_radial(CX, 676, 8, (i) => blob([[CX - 6, 676 - 42], [CX + 6, 676 - 42], [CX, 676 - 30, 0]], { fill: P.amber, stroke: P.ink, width: 2 }));
});

layer("acabado", () => {
  // brillos y grano
  brush([[330, 128], [380, 96], [440, 76]], { width: [3, 11, 3], color: alpha(P.white, 0.9) });
  brush([[268, 400], [262, 340], [268, 310]], { width: [2, 7, 2], color: alpha(P.white, 0.8) });
  brush([[120, 150], [200, 140], [260, 158]], { width: [2, 6, 2], color: alpha(P.white, 0.8) });
  brush([[CX - 14, 34], [CX - 26, 66]], { width: [2, 5], taper: [0.3, 0.3], color: alpha(P.white, 0.8) });
  clip(abd, () => grain({ opacity: 0.06, blend: "overlay" }));
});
