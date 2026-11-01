// Reloj de arena: piezas, reposo y tres acciones.
config({ name: "Reloj de Arena", base: "base.png", cell: [12, 12] });
part("marco", () => { fill_rect(2, 0, 8, 1, "#7a4a2a"); fill_rect(2, 11, 8, 1, "#7a4a2a"); fill_rect(3, 1, 6, 10, "#cfe8ff"); });
part("arena", ({ nivel = 0 }) => { fill_rect(4, 2 + nivel, 4, 3 - nivel, "#e8c070"); line(6, 5, 6, 9, "#e8c070"); });
rest(() => { draw("marco"); draw("arena"); });
action("caer", { frames: 4, fps: 8 }, (f) => { draw("marco"); draw("arena", { nivel: Math.min(f, 3) }); });
action("salto", { frames: 3, loop: false }, (f, { t }) => { move(0, -Math.round(t * 4), () => { draw("marco"); }); });
action("espejo", { frames: 2 }, (f) => { flip_x(() => { set_pixel(0, 0, "#7a4a2a"); }); set_pixel(f, 5, "#00ff00"); });
