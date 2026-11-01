// Animación "rotura_roca" · celdas de 72x72 · la base empieza en (4, 4) dentro de cada celda.
config({ name: "rotura_roca", base: "base.png", cell: [72, 72], origin: [4, 4], colors: [] });

// Nube de polvo de la base (incluye los recortes y bultos de su borde).
part("polvo", () => {
  fill_rect(26, 18, 12, 28, "#8a7e72");
  fill_rect(22, 20, 20, 24, "#8a7e72");
  fill_rect(20, 22, 24, 20, "#8a7e72");
  fill_rect(18, 25, 28, 14, "#8a7e72");
  fill_rect(16, 28, 3, 7, "#8a7e72");
  fill_rect(45, 27, 3, 8, "#8a7e72");
  fill_rect(29, 16, 6, 2, "#8a7e72");
  fill_rect(28, 46, 8, 2, "#8a7e72");
  fill_rect(39, 17, 5, 4, "#8a7e72");
  fill_rect(19, 41, 5, 4, "#8a7e72");
  fill_rect(18, 19, 5, 4, "#8a7e72");
  fill_rect(41, 41, 5, 4, "#8a7e72");
  fill_rect(27, 22, 10, 20, "#a39686");
  fill_rect(24, 24, 16, 16, "#a39686");
  fill_rect(22, 27, 20, 10, "#a39686");
  fill_rect(21, 22, 3, 3, "#a39686");
  fill_rect(40, 39, 3, 3, "#a39686");
  set_pixel(39, 17, "transparent");
  set_pixel(43, 17, "transparent");
  set_pixel(18, 19, "transparent");
  set_pixel(19, 44, "transparent");
  set_pixel(23, 44, "transparent");
  set_pixel(16, 28, "transparent");
  set_pixel(16, 34, "transparent");
  set_pixel(47, 27, "transparent");
  set_pixel(47, 34, "transparent");
  set_pixel(26, 18, "transparent");
  set_pixel(37, 18, "transparent");
  set_pixel(29, 16, "transparent");
  set_pixel(34, 16, "transparent");
  fill_rect(19, 17, 3, 2, "#8a7e72");
  fill_rect(42, 15, 2, 2, "#8a7e72");
  fill_rect(48, 29, 2, 4, "#8a7e72");
  fill_rect(13, 29, 3, 4, "#8a7e72");
  fill_rect(30, 48, 4, 1, "#8a7e72");
  fill_rect(21, 45, 2, 2, "#8a7e72");
  fill_rect(46, 41, 2, 2, "#8a7e72");
  fill_rect(19, 20, 3, 2, "#a39686");
  fill_rect(40, 18, 2, 2, "#a39686");
  fill_rect(17, 29, 2, 2, "#a39686");
  fill_rect(20, 42, 2, 2, "#a39686");
  fill_rect(30, 17, 3, 1, "#a39686");
});

part("destello", () => {
  fill_rect(28, 27, 8, 10, "#bdb1a2");
  fill_rect(27, 28, 10, 8, "#bdb1a2");
  fill_rect(26, 30, 12, 4, "#bdb1a2");
  fill_rect(31, 25, 2, 14, "#f5e9a8");
  fill_rect(25, 31, 14, 2, "#f5e9a8");
  fill_rect(29, 29, 6, 6, "#f5e9a8");
  fill_rect(30, 30, 4, 4, "#ffffff");
  fill_rect(31, 28, 2, 8, "#ffffff");
  fill_rect(28, 31, 8, 2, "#ffffff");
  set_pixel(28, 28, "#f5e9a8");
  set_pixel(35, 28, "#f5e9a8");
  set_pixel(28, 35, "#f5e9a8");
  set_pixel(35, 35, "#f5e9a8");
});

part("roca_no", () => {
  line(11, 5, 12, 5, "#2a1c14");
  line(10, 6, 15, 6, "#2a1c14");
  line(10, 7, 19, 7, "#2a1c14");
  line(9, 8, 19, 8, "#2a1c14");
  line(9, 9, 20, 9, "#2a1c14");
  line(8, 10, 20, 10, "#2a1c14");
  line(8, 11, 20, 11, "#2a1c14");
  line(7, 12, 21, 12, "#2a1c14");
  line(8, 13, 21, 13, "#2a1c14");
  line(10, 14, 21, 14, "#2a1c14");
  line(11, 15, 19, 15, "#2a1c14");
  line(13, 16, 18, 16, "#2a1c14");
  line(14, 17, 16, 17, "#2a1c14");
  line(10, 8, 18, 8, "#7f6c5c");
  line(10, 9, 19, 9, "#7f6c5c");
  line(9, 10, 19, 10, "#7f6c5c");
  line(9, 11, 19, 11, "#7f6c5c");
  line(8, 12, 20, 12, "#7f6c5c");
  line(10, 13, 20, 13, "#7f6c5c");
  line(11, 14, 19, 14, "#7f6c5c");
  line(11, 6, 12, 6, "#a8927c");
  line(11, 7, 15, 7, "#a8927c");
  line(10, 8, 16, 8, "#a8927c");
  line(10, 9, 14, 9, "#a8927c");
  line(9, 10, 12, 10, "#a8927c");
  line(9, 11, 10, 11, "#a8927c");
  line(11, 7, 13, 7, "#c9b49c");
  line(11, 8, 12, 8, "#c9b49c");
  line(17, 13, 20, 13, "#5a4a3e");
  line(14, 14, 19, 14, "#5a4a3e");
  line(13, 15, 18, 15, "#5a4a3e");
  line(14, 16, 16, 16, "#5a4a3e");
  line(19, 12, 20, 12, "#5a4a3e");
  set_pixel(19, 11, "#5a4a3e");
  set_pixel(19, 10, "#5a4a3e");
  set_pixel(15, 11, "#4a3b31");
  set_pixel(16, 11, "#4a3b31");
  set_pixel(15, 12, "#4a3b31");
  set_pixel(16, 12, "#a8927c");
});

part("roca_ne", () => {
  line(49, 4, 51, 4, "#2a1c14");
  line(46, 5, 52, 5, "#2a1c14");
  line(42, 6, 53, 6, "#2a1c14");
  line(42, 7, 54, 7, "#2a1c14");
  line(43, 8, 55, 8, "#2a1c14");
  line(43, 9, 55, 9, "#2a1c14");
  line(43, 10, 55, 10, "#2a1c14");
  line(43, 11, 54, 11, "#2a1c14");
  line(44, 12, 52, 12, "#2a1c14");
  line(46, 13, 51, 13, "#2a1c14");
  line(48, 14, 50, 14, "#2a1c14");
  line(46, 6, 52, 6, "#7f6c5c");
  line(43, 7, 53, 7, "#7f6c5c");
  line(44, 8, 54, 8, "#7f6c5c");
  line(44, 9, 54, 9, "#7f6c5c");
  line(44, 10, 54, 10, "#7f6c5c");
  line(44, 11, 52, 11, "#7f6c5c");
  line(46, 12, 51, 12, "#7f6c5c");
  line(49, 5, 51, 5, "#a8927c");
  line(46, 6, 51, 6, "#a8927c");
  line(43, 7, 49, 7, "#a8927c");
  line(44, 8, 47, 8, "#a8927c");
  line(44, 9, 45, 9, "#a8927c");
  line(47, 6, 49, 6, "#c9b49c");
  line(44, 7, 46, 7, "#c9b49c");
  line(50, 11, 52, 11, "#5a4a3e");
  line(47, 12, 51, 12, "#5a4a3e");
  line(48, 13, 50, 13, "#5a4a3e");
  line(53, 10, 54, 10, "#5a4a3e");
  set_pixel(54, 9, "#5a4a3e");
  set_pixel(50, 8, "#4a3b31");
  set_pixel(51, 8, "#4a3b31");
  set_pixel(51, 9, "#a8927c");
});

part("roca_n", () => {
  line(33, 5, 34, 5, "#2a1c14");
  line(29, 6, 35, 6, "#2a1c14");
  line(28, 7, 35, 7, "#2a1c14");
  line(28, 8, 36, 8, "#2a1c14");
  line(27, 9, 36, 9, "#2a1c14");
  line(27, 10, 36, 10, "#2a1c14");
  line(29, 11, 34, 11, "#2a1c14");
  line(31, 12, 32, 12, "#2a1c14");
  line(29, 7, 34, 7, "#7f6c5c");
  line(29, 8, 35, 8, "#7f6c5c");
  line(28, 9, 35, 9, "#7f6c5c");
  line(29, 10, 34, 10, "#7f6c5c");
  line(33, 6, 34, 6, "#a8927c");
  line(29, 7, 32, 7, "#a8927c");
  line(29, 8, 31, 8, "#a8927c");
  line(28, 9, 29, 9, "#a8927c");
  set_pixel(30, 7, "#c9b49c");
  line(32, 10, 34, 10, "#5a4a3e");
  line(31, 11, 32, 11, "#5a4a3e");
  set_pixel(35, 9, "#5a4a3e");
  set_pixel(35, 8, "#5a4a3e");
});

part("roca_o", () => {
  line(6, 25, 7, 25, "#2a1c14");
  line(5, 26, 10, 26, "#2a1c14");
  line(5, 27, 11, 27, "#2a1c14");
  line(4, 28, 11, 28, "#2a1c14");
  line(4, 29, 11, 29, "#2a1c14");
  line(4, 30, 10, 30, "#2a1c14");
  line(5, 31, 10, 31, "#2a1c14");
  line(7, 32, 9, 32, "#2a1c14");
  line(6, 26, 7, 26, "#a8927c");
  line(6, 27, 10, 27, "#7f6c5c");
  line(5, 28, 10, 28, "#7f6c5c");
  line(5, 29, 10, 29, "#7f6c5c");
  line(5, 30, 9, 30, "#7f6c5c");
  line(7, 31, 9, 31, "#5a4a3e");
  line(6, 27, 8, 27, "#a8927c");
  line(5, 28, 6, 28, "#a8927c");
  line(8, 30, 9, 30, "#5a4a3e");
  set_pixel(10, 29, "#5a4a3e");
});

part("roca_e", () => {
  line(52, 24, 53, 24, "#2a1c14");
  line(52, 25, 55, 25, "#2a1c14");
  line(51, 26, 57, 26, "#2a1c14");
  line(51, 27, 58, 27, "#2a1c14");
  line(51, 28, 58, 28, "#2a1c14");
  line(50, 29, 59, 29, "#2a1c14");
  line(50, 30, 59, 30, "#2a1c14");
  line(50, 31, 59, 31, "#2a1c14");
  line(51, 32, 59, 32, "#2a1c14");
  line(52, 33, 59, 33, "#2a1c14");
  line(52, 34, 57, 34, "#2a1c14");
  line(53, 35, 55, 35, "#2a1c14");
  set_pixel(53, 25, "#a8927c");
  line(52, 26, 55, 26, "#7f6c5c");
  line(52, 27, 57, 27, "#7f6c5c");
  line(52, 28, 57, 28, "#7f6c5c");
  line(51, 29, 58, 29, "#7f6c5c");
  line(51, 30, 58, 30, "#7f6c5c");
  line(51, 31, 58, 31, "#7f6c5c");
  line(52, 32, 58, 32, "#7f6c5c");
  line(53, 33, 57, 33, "#7f6c5c");
  line(52, 26, 54, 26, "#a8927c");
  line(52, 27, 55, 27, "#a8927c");
  line(52, 28, 54, 28, "#a8927c");
  line(51, 29, 53, 29, "#a8927c");
  line(51, 30, 52, 30, "#a8927c");
  line(52, 26, 53, 26, "#c9b49c");
  set_pixel(53, 27, "#c9b49c");
  line(56, 32, 58, 32, "#5a4a3e");
  line(54, 33, 57, 33, "#5a4a3e");
  line(53, 34, 55, 34, "#5a4a3e");
  line(57, 31, 58, 31, "#5a4a3e");
  set_pixel(58, 30, "#5a4a3e");
  set_pixel(55, 30, "#4a3b31");
  set_pixel(56, 30, "#4a3b31");
  set_pixel(56, 31, "#a8927c");
});

part("roca_se", () => {
  line(42, 43, 45, 43, "#2a1c14");
  line(41, 44, 49, 44, "#2a1c14");
  line(41, 45, 52, 45, "#2a1c14");
  line(41, 46, 53, 46, "#2a1c14");
  line(40, 47, 54, 47, "#2a1c14");
  line(40, 48, 55, 48, "#2a1c14");
  line(40, 49, 55, 49, "#2a1c14");
  line(40, 50, 56, 50, "#2a1c14");
  line(40, 51, 57, 51, "#2a1c14");
  line(40, 52, 57, 52, "#2a1c14");
  line(40, 53, 56, 53, "#2a1c14");
  line(41, 54, 55, 54, "#2a1c14");
  line(42, 55, 54, 55, "#2a1c14");
  line(44, 56, 53, 56, "#2a1c14");
  line(46, 57, 52, 57, "#2a1c14");
  line(48, 58, 51, 58, "#2a1c14");
  line(42, 45, 49, 45, "#7f6c5c");
  line(42, 46, 52, 46, "#7f6c5c");
  line(41, 47, 53, 47, "#7f6c5c");
  line(41, 48, 54, 48, "#7f6c5c");
  line(41, 49, 54, 49, "#7f6c5c");
  line(41, 50, 55, 50, "#7f6c5c");
  line(41, 51, 56, 51, "#7f6c5c");
  line(41, 52, 56, 52, "#7f6c5c");
  line(41, 53, 55, 53, "#7f6c5c");
  line(42, 54, 54, 54, "#7f6c5c");
  line(42, 44, 45, 44, "#a8927c");
  line(42, 45, 48, 45, "#a8927c");
  line(42, 46, 49, 46, "#a8927c");
  line(41, 47, 47, 47, "#a8927c");
  line(41, 48, 45, 48, "#a8927c");
  line(41, 49, 43, 49, "#a8927c");
  line(41, 50, 42, 50, "#a8927c");
  line(43, 45, 46, 45, "#c9b49c");
  line(43, 46, 44, 46, "#c9b49c");
  line(55, 51, 56, 51, "#5a4a3e");
  line(53, 52, 56, 52, "#5a4a3e");
  line(50, 53, 55, 53, "#5a4a3e");
  line(47, 54, 54, 54, "#5a4a3e");
  line(44, 55, 53, 55, "#5a4a3e");
  line(46, 56, 52, 56, "#5a4a3e");
  line(48, 57, 51, 57, "#5a4a3e");
  line(54, 50, 55, 50, "#5a4a3e");
  set_pixel(54, 49, "#5a4a3e");
  set_pixel(47, 50, "#4a3b31");
  set_pixel(48, 50, "#4a3b31");
  set_pixel(47, 51, "#4a3b31");
  set_pixel(48, 51, "#a8927c");
  set_pixel(51, 47, "#4a3b31");
  set_pixel(52, 48, "#a8927c");
  set_pixel(45, 44, "transparent");
  set_pixel(41, 44, "transparent");
});

part("roca_s", () => {
  line(28, 50, 30, 50, "#2a1c14");
  line(27, 51, 34, 51, "#2a1c14");
  line(27, 52, 34, 52, "#2a1c14");
  line(27, 53, 34, 53, "#2a1c14");
  line(27, 54, 33, 54, "#2a1c14");
  line(27, 55, 33, 55, "#2a1c14");
  line(28, 56, 32, 56, "#2a1c14");
  line(30, 57, 32, 57, "#2a1c14");
  line(29, 51, 30, 51, "#a8927c");
  line(28, 52, 33, 52, "#7f6c5c");
  line(28, 53, 33, 53, "#7f6c5c");
  line(28, 54, 32, 54, "#7f6c5c");
  line(28, 55, 32, 55, "#7f6c5c");
  line(30, 56, 31, 56, "#5a4a3e");
  line(28, 52, 31, 52, "#a8927c");
  line(28, 53, 29, 53, "#a8927c");
  set_pixel(33, 53, "#5a4a3e");
  set_pixel(32, 54, "#5a4a3e");
  line(30, 55, 32, 55, "#5a4a3e");
});

part("roca_so", () => {
  line(15, 40, 17, 40, "#2a1c14");
  line(9, 41, 17, 41, "#2a1c14");
  line(8, 42, 18, 42, "#2a1c14");
  line(7, 43, 18, 43, "#2a1c14");
  line(7, 44, 18, 44, "#2a1c14");
  line(6, 45, 19, 45, "#2a1c14");
  line(6, 46, 19, 46, "#2a1c14");
  line(6, 47, 19, 47, "#2a1c14");
  line(7, 48, 18, 48, "#2a1c14");
  line(9, 49, 16, 49, "#2a1c14");
  line(11, 50, 15, 50, "#2a1c14");
  line(12, 51, 14, 51, "#2a1c14");
  line(15, 41, 16, 41, "#a8927c");
  line(9, 42, 17, 42, "#7f6c5c");
  line(8, 43, 17, 43, "#7f6c5c");
  line(8, 44, 17, 44, "#7f6c5c");
  line(7, 45, 18, 45, "#7f6c5c");
  line(7, 46, 18, 46, "#7f6c5c");
  line(7, 47, 18, 47, "#7f6c5c");
  line(9, 48, 16, 48, "#7f6c5c");
  line(12, 50, 14, 50, "#5a4a3e");
  line(9, 42, 15, 42, "#a8927c");
  line(8, 43, 13, 43, "#a8927c");
  line(8, 44, 10, 44, "#a8927c");
  line(7, 45, 8, 45, "#a8927c");
  line(10, 42, 12, 42, "#c9b49c");
  line(9, 43, 10, 43, "#c9b49c");
  line(15, 47, 18, 47, "#5a4a3e");
  line(12, 48, 16, 48, "#5a4a3e");
  line(11, 49, 15, 49, "#5a4a3e");
  line(17, 46, 18, 46, "#5a4a3e");
  set_pixel(18, 45, "#5a4a3e");
  set_pixel(12, 45, "#4a3b31");
  set_pixel(13, 45, "#4a3b31");
  set_pixel(13, 46, "#a8927c");
});

// Esquirlas diminutas.
part("chip_a", () => {
  line(37, 14, 38, 14, "#2a1c14");
  line(37, 15, 38, 15, "#2a1c14");
  set_pixel(37, 14, "#a8927c");
});
part("chip_b", () => {
  line(23, 13, 24, 13, "#2a1c14");
  set_pixel(24, 14, "#2a1c14");
  set_pixel(23, 13, "#7f6c5c");
});
part("chip_c", () => {
  line(47, 37, 48, 37, "#2a1c14");
  line(47, 38, 48, 38, "#2a1c14");
  set_pixel(47, 37, "#a8927c");
});
part("chip_d", () => {
  line(22, 50, 23, 50, "#2a1c14");
  line(22, 51, 23, 51, "#2a1c14");
  set_pixel(22, 50, "#7f6c5c");
});
part("chip_e", () => {
  set_pixel(36, 52, "#2a1c14");
  set_pixel(37, 52, "#2a1c14");
  set_pixel(36, 53, "#2a1c14");
});

// Centro aproximado de cada trozo; el estallido sale de (32, 32).
const TROZOS = {
  roca_no: [14, 11], roca_ne: [49, 9], roca_n: [31, 9], roca_o: [8, 28],
  roca_e: [55, 30], roca_se: [48, 50], roca_s: [30, 53], roca_so: [12, 45],
  chip_a: [37, 14], chip_b: [23, 13], chip_c: [47, 37], chip_d: [22, 50], chip_e: [36, 52],
};
const ROCAS = ["roca_no", "roca_ne", "roca_n", "roca_o", "roca_e", "roca_se", "roca_s", "roca_so"];
const CHIPS = ["chip_a", "chip_b", "chip_c", "chip_d", "chip_e"];

// Dibuja un trozo desplazado: k escala su distancia al centro y extra lo aleja más píxeles.
function trozo(nombre, k, extra, girar) {
  const [cx, cy] = TROZOS[nombre];
  const ox = cx - 32, oy = cy - 32, d = Math.hypot(ox, oy) || 1;
  const dx = Math.round((k - 1) * ox + (ox / d) * extra);
  const dy = Math.round((k - 1) * oy + (oy / d) * extra);
  if (girar) move(dx + 2 * cx - 63, dy, () => flip_x(() => draw(nombre)));
  else move(dx, dy, () => draw(nombre));
}

// Nube de polvo tramada: cuanto menor la densidad, más transparente se ve.
const BAYER = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
part("nube", ({ r, dens, borde, centro }) => {
  for (let y = 32 - r - 3; y <= 32 + r + 3; y++) {
    for (let x = 32 - r - 3; x <= 32 + r + 3; x++) {
      const ang = Math.atan2(y - 31.5, x - 31.5);
      const rr = r * (0.9 + 0.06 * Math.sin(ang * 3 + 1) + 0.04 * Math.sin(ang * 7 + 2));
      const q = Math.hypot(x - 31.5, y - 31.5) / rr;
      if (q > 1) continue;
      const local = dens * (1 - 0.5 * q * q);
      if ((BAYER[(y + 64) % 4][(x + 64) % 4] + 0.5) / 16 > local) continue;
      set_pixel(x, y, q < 0.6 ? centro : borde);
    }
  }
});

// Roca entera agrietándose (frame 0).
part("roca_entera", () => {
  ellipse(32, 32, 12, 11, "#5a4a3e");
  ellipse(31, 31, 10, 9, "#7f6c5c");
  ellipse(28, 28, 5, 4, "#a8927c");
  ellipse(27, 27, 2, 1, "#c9b49c");
  fill_rect(19, 29, 1, 5, "#7f6c5c");
  fill_rect(44, 30, 1, 4, "#5a4a3e");
  outline("#2a1c14");
  line(32, 32, 28, 27, "#c9b49c");
  line(28, 27, 25, 25, "#c9b49c");
  line(32, 32, 38, 28, "#c9b49c");
  line(38, 28, 42, 27, "#c9b49c");
  line(32, 32, 35, 38, "#c9b49c");
  line(35, 38, 34, 42, "#c9b49c");
  line(32, 32, 26, 36, "#c9b49c");
  line(26, 36, 22, 37, "#c9b49c");
  fill_rect(31, 31, 2, 2, "#ffffff");
});

// La pose de reposo tiene que ser idéntica al dibujo base (lo comprueba `pxcli anim check`).
rest(() => {
  draw("polvo");
  draw("destello");
  for (const r of ROCAS) draw(r);
  for (const c of CHIPS) draw(c);
});

action("romper", { frames: 8, fps: 15, loop: false }, (f, { t }) => {
  if (f === 0) {
    draw("roca_entera");
  } else if (f === 1) {
    // Bloque partido en cinco trozos casi juntos; la luz se cuela entre ellos.
    for (const r of ["roca_no", "roca_ne", "roca_e", "roca_se", "roca_so"]) trozo(r, 0.42, 0, false);
    fill_rect(31, 26, 2, 12, "#f5e9a8");
    fill_rect(26, 31, 12, 2, "#f5e9a8");
    fill_rect(30, 30, 4, 4, "#f5e9a8");
    fill_rect(31, 29, 2, 6, "#ffffff");
    fill_rect(29, 31, 6, 2, "#ffffff");
  } else if (f === 2) {
    draw("nube", { r: 12, dens: 1, borde: "#8a7e72", centro: "#a39686" });
    draw("destello");
    for (const r of ROCAS) trozo(r, 0.7, 0, false);
    for (const c of CHIPS) trozo(c, 0.7, 0, false);
  } else if (f === 3) {
    draw("polvo");
    draw("destello");
    for (const r of ROCAS) draw(r);
    for (const c of CHIPS) draw(c);
  } else if (f === 4 || f === 5) {
    const extra = f === 4 ? 4 : 7;
    if (f === 4) draw("nube", { r: 21, dens: 0.75, borde: "#8a7e72", centro: "#a39686" });
    else draw("nube", { r: 25, dens: 0.45, borde: "#a39686", centro: "#bdb1a2" });
    ROCAS.forEach((r, i) => trozo(r, 1, extra, (i + f) % 2 === 0));
    CHIPS.forEach((c, i) => trozo(c, 1, extra + 1, (i + f) % 2 === 1));
  } else if (f === 6) {
    draw("nube", { r: 28, dens: 0.18, borde: "#a39686", centro: "#bdb1a2" });
    ["roca_n", "roca_o", "roca_s"].forEach((r, i) => trozo(r, 1, 8, i % 2 === 0));
    CHIPS.forEach((c, i) => trozo(c, 1, 12, i % 2 === 0));
  } else {
    ["chip_a", "chip_b", "chip_c", "chip_d"].forEach((c, i) => trozo(c, 1, 16, i % 2 === 1));
  }
});
