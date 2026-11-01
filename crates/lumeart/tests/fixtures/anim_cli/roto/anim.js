config({ base: "base.png", cell: [14, 14], origin: [1, 2], colors: ["#123456"] });
rest(() => { base(); set_pixel(0, 0, "#7a4a2a"); });
let n = 0;
action("quieto", { frames: 3 }, () => { base(); });
action("salta", { frames: 4 }, (f) => { if (f < 3) { set_pixel(f, 0, "#7a4a2a"); } else { fill_rect(0, 0, 12, 12, "#cfe8ff"); } });
action("vacio", { frames: 2, fps: 4, loop: false }, (f) => { if (f == 1) { base(); } });
action("estado", { frames: 3 }, (f) => { n++; set_pixel(n % 12, 1, "#123456"); outline("#7a4a2a"); });
action("ovalo", { frames: 2 }, (f) => { ellipse(6, 6, 3 + f, 2, "#e8c070"); clear(); ellipse(6, 6, 2, 3, "#e8c070"); });
