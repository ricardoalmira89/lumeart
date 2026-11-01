// Animación "explosion_jugador" · celdas de 80x80 · la base empieza en (8, 8) dentro de cada celda.
config({ name: "explosion_jugador", base: "base.png", cell: [80, 80], origin: [8, 8], colors: [] });

// Capas de la onda, de fuera hacia dentro. Cada una acepta { c } para cambiar su color.
part("contorno", ({ c = "#0a1238" } = {}) => {
  fill_rect(13, 28, 39, 9, c);
  fill_rect(14, 25, 37, 3, c);
  fill_rect(14, 37, 37, 3, c);
  fill_rect(15, 23, 35, 2, c);
  fill_rect(15, 40, 35, 2, c);
  fill_rect(16, 22, 33, 1, c);
  fill_rect(16, 42, 33, 1, c);
  fill_rect(17, 20, 31, 2, c);
  fill_rect(17, 43, 31, 2, c);
  fill_rect(18, 19, 29, 1, c);
  fill_rect(18, 45, 29, 1, c);
  fill_rect(19, 18, 27, 1, c);
  fill_rect(19, 46, 27, 1, c);
  fill_rect(20, 17, 25, 1, c);
  fill_rect(20, 47, 25, 1, c);
  fill_rect(24, 15, 17, 1, c);
  fill_rect(24, 49, 17, 1, c);
  fill_rect(26, 14, 13, 1, c);
  fill_rect(26, 50, 13, 1, c);
  fill_rect(29, 13, 7, 1, c);
  fill_rect(29, 51, 7, 1, c);
  fill_rect(39, 18, 11, 5, c);
  fill_rect(40, 17, 9, 1, c);
  fill_rect(41, 16, 7, 1, c);
  fill_rect(43, 15, 3, 1, c);
  fill_rect(10, 26, 11, 5, c);
  fill_rect(11, 25, 9, 1, c);
  fill_rect(11, 31, 9, 1, c);
  fill_rect(12, 24, 7, 1, c);
  fill_rect(12, 32, 7, 1, c);
  fill_rect(14, 23, 3, 1, c);
  fill_rect(33, 46, 11, 5, c);
  fill_rect(34, 51, 9, 1, c);
  fill_rect(35, 52, 7, 1, c);
  fill_rect(37, 53, 3, 1, c);
  fill_rect(15, 42, 11, 5, c);
  fill_rect(16, 47, 9, 1, c);
  fill_rect(17, 48, 7, 1, c);
  fill_rect(19, 49, 3, 1, c);
  fill_rect(44, 33, 11, 5, c);
  fill_rect(45, 32, 9, 1, c);
  fill_rect(45, 38, 9, 1, c);
  fill_rect(46, 31, 7, 1, c);
  fill_rect(46, 39, 7, 1, c);
  fill_rect(48, 40, 3, 1, c);
  fill_rect(22, 14, 11, 5, c);
  fill_rect(23, 13, 9, 1, c);
  fill_rect(24, 12, 7, 1, c);
  fill_rect(26, 11, 3, 1, c);
});

part("capa_oscura", ({ c = "#1631a8" } = {}) => {
  fill_rect(14, 28, 37, 9, c);
  fill_rect(15, 25, 35, 3, c);
  fill_rect(15, 37, 35, 3, c);
  fill_rect(16, 23, 33, 2, c);
  fill_rect(16, 40, 33, 2, c);
  fill_rect(17, 22, 31, 1, c);
  fill_rect(17, 42, 31, 1, c);
  fill_rect(18, 20, 29, 2, c);
  fill_rect(18, 43, 29, 2, c);
  fill_rect(19, 19, 27, 1, c);
  fill_rect(19, 45, 27, 1, c);
  fill_rect(20, 18, 25, 1, c);
  fill_rect(20, 46, 25, 1, c);
  fill_rect(22, 17, 21, 1, c);
  fill_rect(24, 16, 17, 1, c);
  fill_rect(24, 48, 17, 1, c);
  fill_rect(26, 15, 13, 1, c);
  fill_rect(26, 49, 13, 1, c);
  fill_rect(29, 14, 7, 1, c);
  fill_rect(29, 50, 7, 1, c);
  fill_rect(40, 18, 9, 5, c);
  fill_rect(41, 17, 7, 1, c);
  fill_rect(43, 16, 3, 1, c);
  fill_rect(11, 26, 9, 5, c);
  fill_rect(12, 25, 7, 1, c);
  fill_rect(12, 31, 7, 1, c);
  fill_rect(14, 24, 3, 1, c);
  fill_rect(14, 32, 3, 1, c);
  fill_rect(34, 46, 9, 5, c);
  fill_rect(35, 51, 7, 1, c);
  fill_rect(37, 52, 3, 1, c);
  fill_rect(16, 42, 9, 5, c);
  fill_rect(17, 47, 7, 1, c);
  fill_rect(19, 48, 3, 1, c);
  fill_rect(45, 33, 9, 5, c);
  fill_rect(46, 32, 7, 1, c);
  fill_rect(46, 38, 7, 1, c);
  fill_rect(48, 31, 3, 1, c);
  fill_rect(48, 39, 3, 1, c);
  fill_rect(23, 14, 9, 5, c);
  fill_rect(24, 13, 7, 1, c);
  fill_rect(26, 12, 3, 1, c);
});

part("capa_azul", ({ c = "#1d4be8" } = {}) => {
  fill_rect(15, 28, 35, 9, c);
  fill_rect(16, 25, 33, 3, c);
  fill_rect(16, 37, 33, 3, c);
  fill_rect(17, 23, 31, 2, c);
  fill_rect(17, 40, 31, 2, c);
  fill_rect(18, 22, 29, 1, c);
  fill_rect(18, 42, 29, 1, c);
  fill_rect(19, 20, 27, 2, c);
  fill_rect(19, 43, 27, 2, c);
  fill_rect(20, 19, 25, 1, c);
  fill_rect(20, 45, 25, 1, c);
  fill_rect(22, 18, 21, 1, c);
  fill_rect(22, 46, 21, 1, c);
  fill_rect(24, 17, 17, 1, c);
  fill_rect(24, 47, 17, 1, c);
  fill_rect(26, 16, 13, 1, c);
  fill_rect(26, 48, 13, 1, c);
  fill_rect(29, 15, 7, 1, c);
  fill_rect(29, 49, 7, 1, c);
  fill_rect(41, 18, 7, 5, c);
  fill_rect(43, 17, 3, 1, c);
  fill_rect(43, 23, 3, 1, c);
  fill_rect(12, 26, 7, 5, c);
  fill_rect(14, 25, 3, 1, c);
  fill_rect(14, 31, 3, 1, c);
  fill_rect(35, 46, 7, 5, c);
  fill_rect(37, 51, 3, 1, c);
  fill_rect(17, 42, 7, 5, c);
  fill_rect(19, 41, 3, 1, c);
  fill_rect(19, 47, 3, 1, c);
  fill_rect(46, 33, 7, 5, c);
  fill_rect(48, 32, 3, 1, c);
  fill_rect(48, 38, 3, 1, c);
  fill_rect(24, 14, 7, 5, c);
  fill_rect(26, 13, 3, 1, c);
});

part("capa_verde", ({ c = "#18c0b0" } = {}) => {
  fill_rect(18, 29, 29, 7, c);
  fill_rect(19, 26, 27, 3, c);
  fill_rect(19, 36, 27, 3, c);
  fill_rect(20, 25, 25, 1, c);
  fill_rect(20, 39, 25, 1, c);
  fill_rect(21, 23, 23, 2, c);
  fill_rect(21, 40, 23, 2, c);
  fill_rect(22, 22, 21, 1, c);
  fill_rect(22, 42, 21, 1, c);
  fill_rect(23, 21, 19, 1, c);
  fill_rect(23, 43, 19, 1, c);
  fill_rect(25, 20, 15, 1, c);
  fill_rect(25, 44, 15, 1, c);
  fill_rect(27, 19, 11, 1, c);
  fill_rect(27, 45, 11, 1, c);
  fill_rect(30, 18, 5, 1, c);
  fill_rect(30, 46, 5, 1, c);
  fill_rect(38, 22, 7, 3, c);
  fill_rect(39, 21, 5, 1, c);
  fill_rect(39, 25, 5, 1, c);
  fill_rect(40, 20, 3, 1, c);
  fill_rect(40, 26, 3, 1, c);
  fill_rect(16, 28, 7, 3, c);
  fill_rect(17, 27, 5, 1, c);
  fill_rect(17, 31, 5, 1, c);
  fill_rect(18, 26, 3, 1, c);
  fill_rect(18, 32, 3, 1, c);
  fill_rect(33, 43, 7, 3, c);
  fill_rect(34, 42, 5, 1, c);
  fill_rect(34, 46, 5, 1, c);
  fill_rect(35, 47, 3, 1, c);
  fill_rect(20, 40, 7, 3, c);
  fill_rect(21, 39, 5, 1, c);
  fill_rect(21, 43, 5, 1, c);
  fill_rect(22, 38, 3, 1, c);
  fill_rect(22, 44, 3, 1, c);
  fill_rect(42, 33, 7, 3, c);
  fill_rect(43, 32, 5, 1, c);
  fill_rect(43, 36, 5, 1, c);
  fill_rect(44, 31, 3, 1, c);
  fill_rect(44, 37, 3, 1, c);
  fill_rect(25, 18, 7, 3, c);
  fill_rect(26, 17, 5, 1, c);
  fill_rect(26, 21, 5, 1, c);
  fill_rect(27, 16, 3, 1, c);
});

part("capa_cian", ({ c = "#9af8ff" } = {}) => {
  fill_rect(21, 28, 21, 7, c);
  fill_rect(22, 26, 19, 2, c);
  fill_rect(22, 35, 19, 2, c);
  fill_rect(23, 25, 17, 1, c);
  fill_rect(23, 37, 17, 1, c);
  fill_rect(24, 24, 15, 1, c);
  fill_rect(24, 38, 15, 1, c);
  fill_rect(25, 23, 13, 1, c);
  fill_rect(25, 39, 13, 1, c);
  fill_rect(27, 22, 9, 1, c);
  fill_rect(27, 40, 9, 1, c);
  fill_rect(29, 21, 5, 1, c);
  fill_rect(29, 41, 5, 1, c);
  fill_rect(37, 23, 5, 3, c);
  fill_rect(38, 22, 3, 1, c);
  fill_rect(38, 26, 3, 1, c);
  fill_rect(19, 35, 5, 3, c);
  fill_rect(20, 34, 3, 1, c);
  fill_rect(20, 38, 3, 1, c);
  fill_rect(33, 40, 5, 3, c);
  fill_rect(34, 39, 3, 1, c);
  fill_rect(34, 43, 3, 1, c);
  fill_rect(40, 32, 5, 3, c);
  fill_rect(41, 31, 3, 1, c);
  fill_rect(41, 35, 3, 1, c);
});

part("nucleo", ({ c = "#ffffff" } = {}) => {
  fill_rect(25, 28, 13, 5, c);
  fill_rect(26, 27, 11, 1, c);
  fill_rect(26, 33, 11, 1, c);
  fill_rect(27, 26, 9, 1, c);
  fill_rect(27, 34, 9, 1, c);
  fill_rect(28, 25, 7, 1, c);
  fill_rect(28, 35, 7, 1, c);
  fill_rect(30, 24, 3, 1, c);
  fill_rect(30, 36, 3, 1, c);
  fill_rect(36, 25, 2, 2, c);
  fill_rect(24, 33, 2, 2, c);
});

part("anillo", () => {
  line(59, 32, 59, 37, "#9af8ff");
  line(59, 37, 57, 43, "#9af8ff");
  line(57, 43, 54, 48, "#9af8ff");
  line(54, 48, 51, 51, "#9af8ff");
  line(51, 51, 48, 54, "#9af8ff");
  line(48, 54, 43, 57, "#9af8ff");
  line(43, 57, 37, 59, "#9af8ff");
  line(37, 59, 32, 59, "#9af8ff");
  line(5, 32, 5, 37, "#9af8ff");
  line(5, 37, 7, 43, "#9af8ff");
  line(7, 43, 10, 48, "#9af8ff");
  line(10, 48, 13, 51, "#9af8ff");
  line(13, 51, 16, 54, "#9af8ff");
  line(16, 54, 21, 57, "#9af8ff");
  line(21, 57, 27, 59, "#9af8ff");
  line(27, 59, 32, 59, "#9af8ff");
  line(5, 32, 5, 27, "#9af8ff");
  line(5, 27, 7, 21, "#9af8ff");
  line(7, 21, 10, 16, "#9af8ff");
  line(10, 16, 13, 13, "#9af8ff");
  line(13, 13, 16, 10, "#9af8ff");
  line(16, 10, 21, 7, "#9af8ff");
  line(21, 7, 27, 5, "#9af8ff");
  line(27, 5, 32, 5, "#9af8ff");
  line(59, 32, 59, 27, "#9af8ff");
  line(59, 27, 57, 21, "#9af8ff");
  line(57, 21, 54, 16, "#9af8ff");
  line(54, 16, 51, 13, "#9af8ff");
  line(51, 13, 48, 10, "#9af8ff");
  line(48, 10, 43, 7, "#9af8ff");
  line(43, 7, 37, 5, "#9af8ff");
  line(37, 5, 32, 5, "#9af8ff");
  fill_rect(50, 12, 3, 3, "transparent");
  fill_rect(12, 50, 3, 3, "transparent");
  fill_rect(39, 57, 3, 3, "transparent");
  fill_rect(5, 24, 2, 3, "transparent");
});

// Fragmentos del casco con su estela.
part("frag_ne", () => {
  fill_rect(53, 5, 6, 4, "#0a1238");
  fill_rect(54, 6, 4, 2, "#ffffff");
  set_pixel(57, 7, "#a9bfd9");
  set_pixel(56, 7, "#18c0b0");
  set_pixel(53, 5, "transparent");
  set_pixel(58, 8, "transparent");
  line(50, 10, 52, 8, "#18c0b0");
});

part("frag_so", () => {
  fill_rect(6, 53, 6, 5, "#0a1238");
  fill_rect(7, 54, 4, 3, "#18c0b0");
  set_pixel(7, 54, "#ffffff");
  set_pixel(8, 54, "#ffffff");
  set_pixel(10, 56, "#0e7f86");
  set_pixel(11, 57, "transparent");
  line(12, 52, 10, 54, "#9af8ff");
});

part("frag_no", () => {
  fill_rect(6, 6, 4, 4, "#0a1238");
  fill_rect(7, 7, 2, 2, "#ffffff");
  set_pixel(8, 8, "#a9bfd9");
  set_pixel(6, 9, "transparent");
  line(11, 11, 13, 13, "#9af8ff");
  set_pixel(10, 6, "#0a1238");
  set_pixel(10, 7, "#0a1238");
  set_pixel(9, 7, "#ffffff");
  set_pixel(9, 8, "#18c0b0");
});

part("frag_se", () => {
  fill_rect(53, 53, 5, 4, "#0a1238");
  fill_rect(54, 54, 3, 2, "#ffffff");
  set_pixel(56, 55, "#18c0b0");
  set_pixel(55, 55, "#a9bfd9");
  set_pixel(57, 53, "transparent");
  set_pixel(53, 56, "transparent");
  line(50, 50, 52, 52, "#18c0b0");
});

// Chispas en cruz.
part("chispa_e", () => {
  fill_rect(53, 24, 3, 1, "#ffffff");
  fill_rect(54, 23, 1, 3, "#ffffff");
  set_pixel(53, 23, "#9af8ff");
  set_pixel(55, 25, "#9af8ff");
  set_pixel(55, 23, "#1d4be8");
  set_pixel(53, 25, "#1d4be8");
});

part("chispa_o", () => {
  fill_rect(9, 38, 3, 1, "#ffffff");
  fill_rect(10, 37, 1, 3, "#ffffff");
  set_pixel(9, 37, "#9af8ff");
  set_pixel(11, 39, "#9af8ff");
  set_pixel(11, 37, "#1d4be8");
  set_pixel(9, 39, "#1d4be8");
});

part("chispa_s", () => {
  fill_rect(27, 55, 3, 1, "#ffffff");
  fill_rect(28, 54, 1, 3, "#ffffff");
  set_pixel(27, 54, "#9af8ff");
  set_pixel(29, 56, "#9af8ff");
  set_pixel(29, 54, "#1d4be8");
  set_pixel(27, 56, "#1d4be8");
});

part("chispa_n1", () => {
  set_pixel(21, 11, "#ffffff");
  set_pixel(20, 11, "#9af8ff");
  set_pixel(22, 11, "#9af8ff");
  set_pixel(21, 10, "#9af8ff");
  set_pixel(21, 12, "#9af8ff");
});

part("chispa_n2", () => {
  set_pixel(38, 9, "#ffffff");
  set_pixel(37, 9, "#9af8ff");
  set_pixel(39, 9, "#9af8ff");
  set_pixel(38, 8, "#9af8ff");
  set_pixel(38, 10, "#9af8ff");
});

part("chispa_n3", () => {
  set_pixel(15, 18, "#ffffff");
  set_pixel(14, 18, "#9af8ff");
  set_pixel(16, 18, "#9af8ff");
  set_pixel(15, 17, "#9af8ff");
  set_pixel(15, 19, "#9af8ff");
});

// Onda completa; cada capa puede cambiar de color o desaparecer (null).
const ONDA = ["contorno", "capa_oscura", "capa_azul", "capa_verde", "capa_cian", "nucleo"];
function onda(colores = {}) {
  for (const capa of ONDA) {
    const c = colores[capa];
    if (c === null) continue;
    draw(capa, c ? { c } : {});
  }
}

// Fragmentos y chispas, cada uno con su dirección de huida desde el centro (32, 32).
const FRAGMENTOS = [["frag_ne", 1, -1], ["frag_so", -1, 1], ["frag_no", -1, -1], ["frag_se", 1, 1]];
const CHISPAS = [
  ["chispa_e", 22, -8], ["chispa_o", -22, 6], ["chispa_s", -4, 23],
  ["chispa_n1", -11, -21], ["chispa_n2", 6, -23], ["chispa_n3", -17, -14],
];
function fragmentos(d) {
  for (const [p, dx, dy] of FRAGMENTOS) move(Math.round(dx * d), Math.round(dy * d), () => draw(p));
}
function chispas(d) {
  for (const [p, x, y] of CHISPAS) {
    const l = Math.hypot(x, y);
    move(Math.round((x / l) * d), Math.round((y / l) * d), () => draw(p));
  }
}

// Anillo de choque de radio r; `quedan` (0-10) controla cuántos tramos sobreviven.
function anillo(r, c, quedan = 10, tramos = 24) {
  const pasos = Math.ceil(2 * Math.PI * r * 2);
  for (let i = 0; i < pasos; i++) {
    const a = (i / pasos) * 2 * Math.PI;
    const tramo = Math.floor((i / pasos) * tramos);
    if ((tramo * 7 + 3) % 10 >= quedan) continue;
    set_pixel(Math.round(32 + Math.cos(a) * r), Math.round(32 + Math.sin(a) * r), c);
  }
}

// Jirones: tajos radiales transparentes que deshilachan la onda.
function jirones(n, rmin, rmax) {
  for (let i = 0; i < n; i++) {
    const a = rand() * 2 * Math.PI;
    const r = lerp(rmin, rmax, rand());
    const largo = 3 + Math.floor(rand() * 5);
    const x = 32 + Math.cos(a) * r, y = 32 + Math.sin(a) * r;
    for (const [ox, oy] of [[0, 0], [1, 0], [0, 1]]) {
      line(Math.round(x) + ox, Math.round(y) + oy, Math.round(x + Math.cos(a) * largo) + ox, Math.round(y + Math.sin(a) * largo) + oy, "transparent");
    }
  }
}

// Restos sueltos repartidos en una corona.
function restos(n, rmin, rmax, colores) {
  for (let i = 0; i < n; i++) {
    const a = rand() * 2 * Math.PI;
    const r = lerp(rmin, rmax, rand());
    const x = Math.round(32 + Math.cos(a) * r), y = Math.round(32 + Math.sin(a) * r);
    const c = colores[Math.floor(rand() * colores.length)];
    set_pixel(x, y, c);
    if (rand() < 0.4) set_pixel(x + 1, y, c);
  }
}

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  onda();
  draw("anillo");
  for (const [p] of FRAGMENTOS) draw(p);
  for (const [p] of CHISPAS) draw(p);
});

// Una función por acción: dibuja el frame f desde cero. t va de 0 a 1.
action("estallar", { frames: 8, fps: 15, loop: false }, (f, { t }) => {
  if (f === 0) {
    // Destello pequeño con rayos en cruz.
    line(32, 27, 32, 37, "#9af8ff");
    line(27, 32, 37, 32, "#9af8ff");
    fill_rect(31, 31, 3, 3, "#ffffff");
    line(32, 29, 32, 35, "#ffffff");
    line(29, 32, 35, 32, "#ffffff");
  } else if (f === 1) {
    // Esfera pequeña blanca y cian.
    ellipse(32, 32, 7, 7, "#0a1238");
    ellipse(32, 32, 6, 6, "#18c0b0");
    ellipse(32, 32, 5, 5, "#9af8ff");
    ellipse(32, 32, 3, 3, "#ffffff");
    line(32, 22, 32, 24, "#9af8ff");
    line(32, 40, 32, 42, "#9af8ff");
    line(22, 32, 24, 32, "#9af8ff");
    line(40, 32, 42, 32, "#9af8ff");
  } else if (f === 2) {
    // La onda casi al tamaño de la base, fragmentos cerca del centro.
    ellipse(32, 32, 17, 16, "#0a1238");
    ellipse(32, 32, 16, 15, "#1631a8");
    ellipse(32, 32, 15, 14, "#1d4be8");
    ellipse(32, 32, 12, 11, "#18c0b0");
    ellipse(32, 32, 9, 8, "#9af8ff");
    ellipse(32, 32, 5, 4, "#ffffff");
    fragmentos(-12);
    chispas(-12);
  } else if (f === 3) {
    // Máxima intensidad: la base.
    base();
  } else if (f === 4) {
    // El núcleo se apaga, la onda se enfría y empieza a deshilacharse.
    onda({ nucleo: "#9af8ff", capa_cian: "#18c0b0" });
    jirones(14, 17, 24);
    anillo(30, "#9af8ff", 8);
    fragmentos(3);
    chispas(3);
  } else if (f === 5) {
    // Onda azul verdosa y azul, hueca y en jirones; anillo más grande y fino.
    onda({ nucleo: null, capa_cian: "#0e7f86", capa_verde: "#1d4be8", capa_azul: "#1631a8" });
    ellipse(32, 32, 7, 6, "transparent");
    jirones(30, 5, 26);
    anillo(33, "#18c0b0", 6);
    fragmentos(6);
    chispas(6);
  } else if (f === 6) {
    // Solo restos azules y un anillo roto muy fino.
    restos(70, 12, 30, ["#1d4be8", "#1631a8", "#0e7f86", "#1d4be8"]);
    anillo(35, "#1d4be8", 4, 30);
    restos(6, 30, 38, ["#9af8ff", "#ffffff"]);
  } else {
    // Apenas unas chispas y píxeles azules a punto de desaparecer.
    restos(14, 18, 38, ["#1631a8", "#1d4be8"]);
    restos(4, 30, 39, ["#9af8ff", "#ffffff"]);
  }
});
