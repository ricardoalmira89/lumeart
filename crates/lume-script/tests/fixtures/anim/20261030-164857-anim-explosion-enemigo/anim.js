// Animación "explosion_enemigo" · celdas de 80x80 · la base empieza en (8, 8) dentro de cada celda.
config({ name: "explosion_enemigo", base: "base.png", cell: [80, 80], origin: [8, 8], colors: [] });

// Receta del dibujo base: los mismos comandos de pxcli que lo dibujaron, en orden.
// Repártela en piezas con nombre para poder moverlas. El orden de dibujo hace de capas:
// lo que se dibuja después queda encima.
part("humo", () => {
  fill_rect(47, 28, 9, 3, "#26142f");
  fill_rect(48, 27, 7, 1, "#26142f");
  fill_rect(48, 31, 7, 1, "#26142f");
  fill_rect(49, 26, 5, 1, "#26142f");
  fill_rect(49, 32, 5, 1, "#26142f");
  fill_rect(51, 25, 7, 3, "#26142f");
  fill_rect(52, 24, 5, 1, "#26142f");
  fill_rect(52, 28, 5, 1, "#26142f");
  fill_rect(17, 49, 9, 3, "#26142f");
  fill_rect(18, 48, 7, 1, "#26142f");
  fill_rect(18, 52, 7, 1, "#26142f");
  fill_rect(19, 47, 5, 1, "#26142f");
  fill_rect(19, 53, 5, 1, "#26142f");
  fill_rect(14, 51, 7, 3, "#26142f");
  fill_rect(15, 50, 5, 1, "#26142f");
  fill_rect(15, 54, 5, 1, "#26142f");
  fill_rect(8, 30, 9, 3, "#26142f");
  fill_rect(9, 29, 7, 1, "#26142f");
  fill_rect(9, 33, 7, 1, "#26142f");
  fill_rect(10, 28, 5, 1, "#26142f");
  fill_rect(10, 34, 5, 1, "#26142f");
  fill_rect(7, 26, 7, 3, "#26142f");
  fill_rect(8, 25, 5, 1, "#26142f");
  fill_rect(8, 29, 5, 1, "#26142f");
  fill_rect(41, 49, 9, 3, "#26142f");
  fill_rect(42, 48, 7, 1, "#26142f");
  fill_rect(42, 52, 7, 1, "#26142f");
  fill_rect(43, 47, 5, 1, "#26142f");
  fill_rect(43, 53, 5, 1, "#26142f");
  fill_rect(46, 51, 7, 3, "#26142f");
  fill_rect(47, 50, 5, 1, "#26142f");
  fill_rect(47, 54, 5, 1, "#26142f");
  fill_rect(48, 28, 7, 3, "#4b2a68");
  fill_rect(49, 27, 5, 1, "#4b2a68");
  fill_rect(49, 31, 5, 1, "#4b2a68");
  fill_rect(52, 26, 5, 1, "#4b2a68");
  fill_rect(53, 25, 3, 1, "#4b2a68");
  fill_rect(53, 27, 3, 1, "#4b2a68");
  fill_rect(18, 49, 7, 3, "#4b2a68");
  fill_rect(19, 48, 5, 1, "#4b2a68");
  fill_rect(19, 52, 5, 1, "#4b2a68");
  fill_rect(15, 52, 5, 1, "#4b2a68");
  fill_rect(16, 51, 3, 1, "#4b2a68");
  fill_rect(16, 53, 3, 1, "#4b2a68");
  fill_rect(9, 30, 7, 3, "#4b2a68");
  fill_rect(10, 29, 5, 1, "#4b2a68");
  fill_rect(10, 33, 5, 1, "#4b2a68");
  fill_rect(8, 27, 5, 1, "#4b2a68");
  fill_rect(9, 26, 3, 1, "#4b2a68");
  fill_rect(9, 28, 3, 1, "#4b2a68");
  fill_rect(42, 49, 7, 3, "#4b2a68");
  fill_rect(43, 48, 5, 1, "#4b2a68");
  fill_rect(43, 52, 5, 1, "#4b2a68");
  fill_rect(47, 52, 5, 1, "#4b2a68");
  fill_rect(48, 51, 3, 1, "#4b2a68");
  fill_rect(48, 53, 3, 1, "#4b2a68");
  fill_rect(49, 28, 2, 1, "#6f4c90");
  set_pixel(50, 27, "#6f4c90");
  set_pixel(53, 25, "#6f4c90");
  fill_rect(19, 49, 2, 1, "#6f4c90");
  set_pixel(19, 48, "#6f4c90");
  set_pixel(16, 51, "#6f4c90");
  fill_rect(10, 30, 2, 1, "#6f4c90");
  set_pixel(10, 29, "#6f4c90");
  set_pixel(9, 26, "#6f4c90");
  fill_rect(43, 49, 2, 1, "#6f4c90");
  set_pixel(44, 48, "#6f4c90");
  set_pixel(48, 51, "#6f4c90");
});

part("fuego", () => {
  fill_rect(17, 29, 31, 7, "#4a0a14");
  fill_rect(18, 26, 29, 3, "#4a0a14");
  fill_rect(23, 44, 19, 1, "#4a0a14");
  fill_rect(25, 45, 15, 1, "#4a0a14");
  fill_rect(13, 20, 17, 5, "#4a0a14");
  fill_rect(14, 17, 15, 2, "#4a0a14");
  fill_rect(14, 25, 15, 2, "#4a0a14");
  fill_rect(15, 16, 13, 1, "#4a0a14");
  fill_rect(15, 27, 13, 1, "#4a0a14");
  fill_rect(16, 15, 11, 1, "#4a0a14");
  fill_rect(16, 28, 11, 1, "#4a0a14");
  fill_rect(17, 14, 9, 1, "#4a0a14");
  fill_rect(17, 29, 9, 1, "#4a0a14");
  fill_rect(36, 19, 15, 5, "#4a0a14");
  fill_rect(37, 16, 13, 2, "#4a0a14");
  fill_rect(37, 24, 13, 2, "#4a0a14");
  fill_rect(38, 15, 11, 1, "#4a0a14");
  fill_rect(38, 26, 11, 1, "#4a0a14");
  fill_rect(39, 14, 9, 1, "#4a0a14");
  fill_rect(39, 27, 9, 1, "#4a0a14");
  fill_rect(38, 36, 17, 5, "#4a0a14");
  fill_rect(39, 33, 15, 2, "#4a0a14");
  fill_rect(39, 41, 15, 2, "#4a0a14");
  fill_rect(40, 32, 13, 1, "#4a0a14");
  fill_rect(40, 43, 13, 1, "#4a0a14");
  fill_rect(41, 31, 11, 1, "#4a0a14");
  fill_rect(41, 44, 11, 1, "#4a0a14");
  fill_rect(42, 30, 9, 1, "#4a0a14");
  fill_rect(42, 45, 9, 1, "#4a0a14");
  fill_rect(26, 45, 15, 5, "#4a0a14");
  fill_rect(27, 50, 13, 2, "#4a0a14");
  fill_rect(28, 52, 11, 1, "#4a0a14");
  fill_rect(29, 53, 9, 1, "#4a0a14");
  fill_rect(12, 38, 15, 5, "#4a0a14");
  fill_rect(13, 35, 13, 2, "#4a0a14");
  fill_rect(13, 43, 13, 2, "#4a0a14");
  fill_rect(14, 34, 11, 1, "#4a0a14");
  fill_rect(14, 45, 11, 1, "#4a0a14");
  fill_rect(15, 33, 9, 1, "#4a0a14");
  fill_rect(15, 46, 9, 1, "#4a0a14");
  fill_rect(26, 14, 13, 5, "#4a0a14");
  fill_rect(27, 13, 11, 1, "#4a0a14");
  fill_rect(28, 12, 9, 1, "#4a0a14");
  fill_rect(29, 11, 7, 1, "#4a0a14");
  fill_rect(18, 29, 29, 7, "#e8401c");
  fill_rect(19, 26, 27, 3, "#e8401c");
  fill_rect(21, 40, 23, 2, "#e8401c");
  fill_rect(22, 42, 21, 1, "#e8401c");
  fill_rect(23, 43, 19, 1, "#e8401c");
  fill_rect(25, 20, 15, 1, "#e8401c");
  fill_rect(25, 44, 15, 1, "#e8401c");
  fill_rect(27, 19, 11, 1, "#e8401c");
  fill_rect(14, 20, 15, 5, "#e8401c");
  fill_rect(15, 17, 13, 2, "#e8401c");
  fill_rect(15, 25, 13, 2, "#e8401c");
  fill_rect(16, 16, 11, 1, "#e8401c");
  fill_rect(16, 27, 11, 1, "#e8401c");
  fill_rect(17, 15, 9, 1, "#e8401c");
  fill_rect(17, 28, 9, 1, "#e8401c");
  fill_rect(37, 19, 13, 5, "#e8401c");
  fill_rect(38, 18, 11, 1, "#e8401c");
  fill_rect(38, 24, 11, 1, "#e8401c");
  fill_rect(39, 17, 9, 1, "#e8401c");
  fill_rect(39, 25, 9, 1, "#e8401c");
  fill_rect(40, 16, 7, 1, "#e8401c");
  fill_rect(40, 26, 7, 1, "#e8401c");
  fill_rect(39, 36, 15, 5, "#e8401c");
  fill_rect(40, 33, 13, 2, "#e8401c");
  fill_rect(40, 41, 13, 2, "#e8401c");
  fill_rect(41, 32, 11, 1, "#e8401c");
  fill_rect(41, 43, 11, 1, "#e8401c");
  fill_rect(42, 31, 9, 1, "#e8401c");
  fill_rect(42, 44, 9, 1, "#e8401c");
  fill_rect(27, 45, 13, 5, "#e8401c");
  fill_rect(28, 44, 11, 1, "#e8401c");
  fill_rect(28, 50, 11, 1, "#e8401c");
  fill_rect(29, 43, 9, 1, "#e8401c");
  fill_rect(29, 51, 9, 1, "#e8401c");
  fill_rect(30, 42, 7, 1, "#e8401c");
  fill_rect(30, 52, 7, 1, "#e8401c");
  fill_rect(13, 38, 13, 5, "#e8401c");
  fill_rect(14, 37, 11, 1, "#e8401c");
  fill_rect(14, 43, 11, 1, "#e8401c");
  fill_rect(15, 36, 9, 1, "#e8401c");
  fill_rect(15, 44, 9, 1, "#e8401c");
  fill_rect(16, 35, 7, 1, "#e8401c");
  fill_rect(16, 45, 7, 1, "#e8401c");
  fill_rect(27, 14, 11, 5, "#e8401c");
  fill_rect(28, 13, 9, 1, "#e8401c");
  fill_rect(28, 19, 9, 1, "#e8401c");
  fill_rect(29, 12, 7, 1, "#e8401c");
  fill_rect(29, 20, 7, 1, "#e8401c");
  fill_rect(20, 28, 23, 7, "#ff8c1a");
  fill_rect(21, 26, 21, 2, "#ff8c1a");
  fill_rect(21, 35, 21, 2, "#ff8c1a");
  fill_rect(22, 25, 19, 1, "#ff8c1a");
  fill_rect(22, 37, 19, 1, "#ff8c1a");
  fill_rect(23, 23, 17, 2, "#ff8c1a");
  fill_rect(23, 38, 17, 2, "#ff8c1a");
  fill_rect(25, 40, 13, 1, "#ff8c1a");
  fill_rect(26, 21, 11, 1, "#ff8c1a");
  fill_rect(26, 41, 11, 1, "#ff8c1a");
  fill_rect(17, 21, 11, 5, "#ff8c1a");
  fill_rect(18, 20, 9, 1, "#ff8c1a");
  fill_rect(18, 26, 9, 1, "#ff8c1a");
  fill_rect(19, 19, 7, 1, "#ff8c1a");
  fill_rect(19, 27, 7, 1, "#ff8c1a");
  fill_rect(38, 21, 9, 3, "#ff8c1a");
  fill_rect(39, 20, 7, 1, "#ff8c1a");
  fill_rect(39, 24, 7, 1, "#ff8c1a");
  fill_rect(40, 19, 5, 1, "#ff8c1a");
  fill_rect(40, 25, 5, 1, "#ff8c1a");
  fill_rect(39, 35, 11, 5, "#ff8c1a");
  fill_rect(40, 34, 9, 1, "#ff8c1a");
  fill_rect(40, 40, 9, 1, "#ff8c1a");
  fill_rect(41, 33, 7, 1, "#ff8c1a");
  fill_rect(41, 41, 7, 1, "#ff8c1a");
  fill_rect(29, 44, 9, 3, "#ff8c1a");
  fill_rect(30, 43, 7, 1, "#ff8c1a");
  fill_rect(30, 47, 7, 1, "#ff8c1a");
  fill_rect(31, 42, 5, 1, "#ff8c1a");
  fill_rect(31, 48, 5, 1, "#ff8c1a");
  fill_rect(16, 38, 9, 3, "#ff8c1a");
  fill_rect(17, 37, 7, 1, "#ff8c1a");
  fill_rect(17, 41, 7, 1, "#ff8c1a");
  fill_rect(18, 36, 5, 1, "#ff8c1a");
  fill_rect(18, 42, 5, 1, "#ff8c1a");
  fill_rect(29, 16, 7, 3, "#ff8c1a");
  fill_rect(30, 15, 5, 1, "#ff8c1a");
  fill_rect(30, 19, 5, 1, "#ff8c1a");
  fill_rect(22, 28, 17, 5, "#ffd23a");
  fill_rect(23, 25, 15, 2, "#ffd23a");
  fill_rect(23, 33, 15, 2, "#ffd23a");
  fill_rect(24, 24, 13, 1, "#ffd23a");
  fill_rect(24, 35, 13, 1, "#ffd23a");
  fill_rect(25, 23, 11, 1, "#ffd23a");
  fill_rect(25, 36, 11, 1, "#ffd23a");
  fill_rect(26, 22, 9, 1, "#ffd23a");
  fill_rect(26, 37, 9, 1, "#ffd23a");
  fill_rect(20, 24, 7, 3, "#ffd23a");
  fill_rect(21, 23, 5, 1, "#ffd23a");
  fill_rect(21, 27, 5, 1, "#ffd23a");
  fill_rect(35, 34, 7, 3, "#ffd23a");
  fill_rect(36, 33, 5, 1, "#ffd23a");
  fill_rect(36, 37, 5, 1, "#ffd23a");
  fill_rect(34, 23, 5, 1, "#ffd23a");
  fill_rect(35, 22, 3, 1, "#ffd23a");
  fill_rect(35, 24, 3, 1, "#ffd23a");
  fill_rect(23, 37, 5, 1, "#ffd23a");
  fill_rect(24, 36, 3, 1, "#ffd23a");
  fill_rect(24, 38, 3, 1, "#ffd23a");
  fill_rect(24, 27, 11, 5, "#fff6a8");
  fill_rect(25, 26, 9, 1, "#fff6a8");
  fill_rect(25, 32, 9, 1, "#fff6a8");
  fill_rect(26, 25, 7, 1, "#fff6a8");
  fill_rect(26, 33, 7, 1, "#fff6a8");
  fill_rect(25, 27, 7, 3, "#ffffff");
  fill_rect(26, 26, 5, 1, "#ffffff");
  fill_rect(26, 30, 5, 1, "#ffffff");
  set_pixel(14, 19, "#4a0a14");
  fill_rect(15, 19, 4, 1, "#e8401c");
  set_pixel(53, 35, "#4a0a14");
  fill_rect(50, 35, 3, 1, "#e8401c");
  set_pixel(26, 19, "#e8401c");
});

// Chispas, una pieza por chispa para poder alejarlas del centro por separado.
part("chispa_no", () => {
  line(11, 12, 13, 14, "#ffe14a");
  set_pixel(11, 12, "#ffffff");
  set_pixel(14, 15, "#ff8c1a");
  set_pixel(12, 12, "#ffe14a");
  set_pixel(13, 13, "#ffe14a");
});
part("chispa_ne", () => {
  line(52, 11, 54, 9, "#ffe14a");
  set_pixel(54, 9, "#ffffff");
  set_pixel(51, 12, "#ff8c1a");
  set_pixel(53, 11, "#ffe14a");
  set_pixel(54, 10, "#ffe14a");
});
part("chispa_se", () => {
  line(55, 44, 57, 46, "#ffe14a");
  set_pixel(57, 46, "#ffffff");
  set_pixel(54, 43, "#ff8c1a");
  set_pixel(56, 44, "#ffe14a");
  set_pixel(57, 45, "#ffe14a");
});
part("chispa_s", () => {
  set_pixel(29, 55, "#ffe14a");
  set_pixel(28, 56, "#ffe14a");
  set_pixel(28, 57, "#ffffff");
  set_pixel(29, 54, "#ff8c1a");
  set_pixel(29, 56, "#ffe14a");
});
part("chispa_so", () => {
  line(7, 44, 9, 42, "#ffe14a");
  set_pixel(7, 44, "#ffffff");
  set_pixel(10, 41, "#ff8c1a");
  set_pixel(8, 44, "#ffe14a");
  set_pixel(9, 43, "#ffe14a");
});
part("chispa_n", () => {
  set_pixel(37, 10, "#ff8c1a");
  set_pixel(37, 9, "#ffe14a");
  set_pixel(38, 8, "#ffe14a");
  set_pixel(38, 7, "#ffffff");
  set_pixel(37, 8, "#ffe14a");
});

// Dirección de salida de cada chispa (hacia fuera desde el centro).
const CHISPAS = [
  ["chispa_no", -1, -1],
  ["chispa_ne", 1, -1],
  ["chispa_se", 1, 1],
  ["chispa_s", 0, 1],
  ["chispa_so", -1, 0.5],
  ["chispa_n", 0.3, -1],
];

// d > 0 aleja las chispas, d < 0 las acerca al centro.
part("chispas", ({ d = 0 } = {}) => {
  for (const [nombre, dx, dy] of CHISPAS) {
    move(Math.round(dx * d), Math.round(dy * d), () => draw(nombre));
  }
});

// Lóbulos de la bola de fuego: centro y dirección hacia fuera.
const LOBULOS = [
  [21, 22, -1, -1],
  [43, 21, 1, -1],
  [46, 38, 1, 0.4],
  [33, 47, 0, 1],
  [19, 40, -1, 0.5],
  [32, 16, 0, -1],
];

// Bocanada redonda por capas: cada color es un círculo un poco más pequeño encima del anterior.
part("bocanada", ({ x, y, r, capas }) => {
  capas.forEach((c, i) => {
    // El primer color es un contorno de 1 px; el resto se reparte hacia el centro.
    const paso = Math.max(1, Math.round(r / capas.length));
    const ri = i === 0 ? r : r - 1 - (i - 1) * paso;
    if (ri >= 1) ellipse(x, y, ri, ri, c);
    else if (ri === 0) set_pixel(x, y, c);
  });
});

const FUEGO = ["#4a0a14", "#e8401c", "#ff8c1a", "#ffd23a", "#fff6a8", "#ffffff"];
const HUMO = ["#26142f", "#4b2a68"];

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("humo");
  draw("fuego");
  draw("chispas");
});

// Una función por acción: dibuja el frame f desde cero. t va de 0 a 1.
action("estallar", { frames: 8, fps: 15, loop: false }, (f, { t }) => {
  const P = (x, y, dx, dy, k) => [Math.round(x + dx * k), Math.round(y + dy * k)];
  if (f === 0) {
    // Destello pequeño
    ellipse(30, 30, 2, 2, "#ffd23a");
    set_pixel(30, 27, "#ffd23a");
    set_pixel(30, 33, "#ffd23a");
    set_pixel(27, 30, "#ffd23a");
    set_pixel(33, 30, "#ffd23a");
    fill_rect(29, 30, 3, 1, "#ffffff");
    fill_rect(30, 29, 1, 3, "#ffffff");
  } else if (f === 1) {
    draw("bocanada", { x: 30, y: 30, r: 9, capas: FUEGO.slice(1) });
  } else if (f === 2) {
    for (const [x, y, dx, dy] of LOBULOS) {
      const [px, py] = P(30, 30, x - 30, y - 30, 0.85);
      draw("bocanada", { x: px, y: py, r: 7, capas: FUEGO.slice(0, 3) });
    }
    draw("bocanada", { x: 30, y: 30, r: 15, capas: FUEGO });
    draw("chispas", { d: -6 });
  } else if (f === 3) {
    draw("humo");
    draw("fuego");
    draw("chispas");
  } else if (f === 4 || f === 5) {
    const k = f === 4 ? 2 : 5;
    const r = f === 4 ? 8 : 6;
    for (const [x, y, dx, dy] of LOBULOS) {
      const [px, py] = P(x, y, dx, dy, k + 6);
      draw("bocanada", { x: px, y: py, r: r - 3, capas: HUMO });
    }
    for (const [x, y, dx, dy] of LOBULOS) {
      const [px, py] = P(x, y, dx, dy, k);
      draw("bocanada", { x: px, y: py, r, capas: f === 4 ? FUEGO.slice(0, 3) : FUEGO.slice(0, 2) });
    }
    draw("bocanada", { x: 31, y: 31, r: f === 4 ? 9 : 5, capas: f === 4 ? FUEGO.slice(0, 3) : FUEGO.slice(0, 2) });
    draw("chispas", { d: f === 4 ? 3 : 7 });
  } else if (f === 6) {
    LOBULOS.forEach(([x, y, dx, dy], i) => {
      const [px, py] = P(x, y, dx, dy, 8);
      draw("bocanada", { x: px, y: py, r: 6, capas: HUMO });
      set_pixel(px - 2, py - 2, "#6f4c90");
      if (i % 2 === 0) draw("bocanada", { x: px + 1, y: py + 1, r: 2, capas: ["#4a0a14", "#e8401c"] });
    });
  } else {
    LOBULOS.forEach(([x, y, dx, dy], i) => {
      if (i === 2) return;
      const [px, py] = P(x, y, dx, dy, 12);
      draw("bocanada", { x: px, y: py, r: i % 2 ? 2 : 1, capas: HUMO });
    });
  }
});

