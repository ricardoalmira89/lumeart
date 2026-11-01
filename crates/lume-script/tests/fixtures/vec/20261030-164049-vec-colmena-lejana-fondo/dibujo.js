canvas({ width: 1024, height: 1024, background: null });

const P = {
  deep: "#0c0e14",   // fondo de celdas vacías, sombra profunda
  dark: "#161a23",   // lado en sombra
  rock: "#212835",   // roca base
  lit: "#2b3342",    // roca iluminada
  wall: "#272e3c",   // borde de celdas
  gDark: "#2f4434",  // verde apagado
  gMid: "#44603f",   // verde de celda viva
  gLit: "#5b7a4e",   // núcleo más claro
};

seed(7);
const C = [512, 512], R = 446;
const ROCA = [[512, 70], [672, 96], [808, 190], [912, 330], [952, 500], [918, 672], [818, 820], [660, 930], [490, 958], [318, 900], [172, 790], [86, 620], [78, 440], [150, 262], [320, 112]];
const hueco = mix(P.deep, P.dark, 0.45);
// Zonas de roca maciza (manchas sin celdas)
const macizo = (x, y) => Math.sin(x * 0.011 + 1.3) + Math.cos(y * 0.014 - 0.6) + Math.sin((x + y) * 0.006) > 1.6;

// Proyecta un punto del panal plano sobre la esfera (se achata hacia el borde)
function proj(x, y) {
  const dx = x - C[0], dy = y - C[1];
  const d = Math.hypot(dx, dy) || 0.001;
  const t = Math.min(d / R, Math.PI / 2);
  const r = R * Math.sin(t);
  return [C[0] + dx / d * r, C[1] + dy / d * r];
}
function hexPts(cx, cy, s) {
  return range(6).map((i) => proj(cx + s * Math.cos((i * 60 + 30) * Math.PI / 180), cy + s * Math.sin((i * 60 + 30) * Math.PI / 180)));
}
// Luz: 1 en el lado iluminado (arriba-izquierda), 0 en la sombra
function luz(x, y) {
  const dx = (x - C[0]) / R, dy = (y - C[1]) / R;
  return clamp(0.5 - (dx + dy) * 0.55, 0, 1);
}

layer("colmena", () => {
  const roca = blob(ROCA, { fill: P.rock });

  clip(roca, () => {
    // Banda iluminada arriba a la izquierda
    const lit = blob(ROCA, { fill: P.lit });
    cut(lit, [circle(575, 575, 445, { hidden: true })]);

    // Panal de celdas
    const S = 40, W = S * Math.sqrt(3), H = S * 1.5;
    for (let row = -12; row <= 12; row++) {
      for (let col = -12; col <= 12; col++) {
        const fx = C[0] + col * W + (row & 1 ? W / 2 : 0);
        const fy = C[1] + row * H;
        const t = Math.hypot(fx - C[0], fy - C[1]) / R;
        if (t > 1.5) continue;
        if (macizo(fx, fy) || rand() < 0.04) continue;
        const [sx, sy] = proj(fx, fy);
        const L = luz(sx, sy);
        const borde = hexPts(fx, fy, S * 0.94);
        const celda = hexPts(fx, fy, S * 0.72);
        poly(borde, { fill: mix(P.dark, P.wall, L), join: "round" });
        const r = rand();
        if (r < 0.13) {
          // celda viva: brillo verde apagado
          const h = poly(celda, { fill: mix(P.gDark, P.gMid, 0.2 + L * 0.5) });
          clip(h, () => {
            poly(shift(celda, -6, -6), { fill: mix(P.deep, P.gDark, 0.6), opacity: 0.8 });
            poly(hexPts(fx + 4, fy + 4, S * 0.34), { fill: mix(P.gMid, P.gLit, L * 0.7), opacity: 0.6 });
          });
        } else {
          // celda vacía: hueco oscuro con la pared inferior derecha algo iluminada
          const h = poly(celda, { fill: mix(P.dark, P.wall, 0.2 + L * 0.6) });
          clip(h, () => {
            poly(shift(celda, 7, 7), { fill: hueco });
          });
        }
      }
    }

    // Luz suave arriba a la izquierda, en dos escalones planos
    const luz1 = blob(ROCA, { fill: P.lit, opacity: 0.18, blend: "screen" });
    cut(luz1, [circle(600, 600, 470, { hidden: true })]);
    const luz2 = blob(ROCA, { fill: P.lit, opacity: 0.18, blend: "screen" });
    cut(luz2, [circle(640, 640, 470, { hidden: true })]);

    // Sombra propia: lado inferior derecho casi negro
    const sombra = blob(ROCA, { fill: P.deep, opacity: 0.45 });
    cut(sombra, [circle(450, 450, 450, { hidden: true })]);
    const sombra2 = blob(ROCA, { fill: P.deep, opacity: 0.55 });
    cut(sombra2, [circle(400, 400, 470, { hidden: true })]);
    const sombra3 = blob(ROCA, { fill: P.deep, opacity: 0.6 });
    cut(sombra3, [circle(360, 360, 520, { hidden: true })]);
  });
});
