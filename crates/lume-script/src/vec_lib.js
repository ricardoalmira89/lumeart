// Drawing library for dibujo.js. Every call adds SVG to a tree; __render()
// turns the tree into one SVG document. Coordinates are drawing units: the
// viewBox is 0 0 width height, origin top left, y grows downwards.

var __W = 1024, __H = 1024, __bg = null, __bgLine = 0;
var __root = { tag: "g", attrs: {}, children: [] };
var __stack = [__root];
var __defs = {}, __defOrder = [];
var __count = 0;
var __MAX = 50000;

function __err(msg) { throw new Error(msg); }
function __top() { return __stack[__stack.length - 1]; }
// Where a shape comes from: its line and, inside helper functions, the calls around it.
function __where(node) {
  var ch = __chain();
  node.line = ch.length ? ch[0] : 0;
  if (ch.length > 1) node.chain = ch.join(",");
  return node;
}
function __num(v, what, fn) {
  if (typeof v !== "number" || !isFinite(v)) __err(fn + ": " + what + " tiene que ser un número (es " + v + ")");
  return v;
}
function __f(v) { var r = Math.round(v * 100) / 100; return String(r === 0 ? 0 : r); }
function __f3(v) { var r = Math.round(v * 1000) / 1000; return String(r === 0 ? 0 : r); }
function __escA(s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;"); }
function __escT(s) { return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;"); }
function __hash(s) {
  var h = 0x811c9dc5;
  for (var i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 0x01000193) >>> 0; }
  return h.toString(36);
}
function __def(prefix, body) {
  // ids come from the content, so an unchanged shape keeps the same markup
  var id = prefix + __hash(body);
  if (!__defs[id]) { __defs[id] = body.replace("__ID__", id); __defOrder.push(id); }
  return id;
}

/* ---------------- numbers, random, points ---------------- */

function lerp(a, b, t) { return a + (b - a) * t; }
function clamp(v, lo, hi) { return Math.min(hi, Math.max(lo, v)); }
function range(n) { var out = []; for (var i = 0; i < n; i++) out.push(i); return out; }
function smoothstep(a, b, x) { var t = clamp((x - a) / (b - a), 0, 1); return t * t * (3 - 2 * t); }

var __rs = 1;
function seed(n) { __rs = (Math.floor(n) >>> 0) || 1; }
function __rand() {
  __rs = (__rs + 0x6D2B79F5) >>> 0;
  var t = __rs;
  t = Math.imul(t ^ (t >>> 15), t | 1);
  t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
}
function rand(a, b) {
  var r = __rand();
  if (a === undefined) return r;
  if (b === undefined) return r * a;
  return a + r * (b - a);
}
function pick(list) { return list[Math.floor(__rand() * list.length)]; }

function __pts(p, fn, min) {
  if (!Array.isArray(p)) __err(fn + ": los puntos tienen que ser una lista [[x, y], ...]");
  var out = [], i;
  if (p.length && typeof p[0] === "number") {
    if (p.length % 2) __err(fn + ": la lista plana de coordenadas tiene un número impar de valores");
    for (i = 0; i < p.length; i += 2) out.push([__num(p[i], "x", fn), __num(p[i + 1], "y", fn), 1]);
  } else {
    for (i = 0; i < p.length; i++) {
      var q = p[i];
      if (!Array.isArray(q) || q.length < 2) __err(fn + ": el punto " + i + " no es [x, y]");
      out.push([__num(q[0], "la x del punto " + i, fn), __num(q[1], "la y del punto " + i, fn),
        q.length > 2 ? clamp(__num(q[2], "la suavidad del punto " + i, fn), 0, 2) : 1]);
    }
  }
  if (out.length < (min || 2)) __err(fn + ": hacen falta al menos " + (min || 2) + " puntos");
  return out;
}
function polar(cx, cy, r, deg) { var a = deg * Math.PI / 180; return [cx + r * Math.cos(a), cy + r * Math.sin(a)]; }
function ring(cx, cy, rx, ry, n, rot) {
  if (ry === undefined) ry = rx;
  n = n || 8; rot = rot || 0;
  var out = [], c = Math.cos(rot * Math.PI / 180), s = Math.sin(rot * Math.PI / 180);
  for (var i = 0; i < n; i++) {
    var a = -Math.PI / 2 + i * 2 * Math.PI / n, x = rx * Math.cos(a), y = ry * Math.sin(a);
    out.push([cx + x * c - y * s, cy + x * s + y * c]);
  }
  return out;
}
function arc(cx, cy, r, a0, a1, n) {
  n = n || 8;
  var out = [];
  for (var i = 0; i < n; i++) out.push(polar(cx, cy, r, lerp(a0, a1, n === 1 ? 0 : i / (n - 1))));
  return out;
}
function __copy(points, fn) { return __pts(points, fn, 1).map(function (p, i) { return Array.isArray(points[0]) && points[i].length > 2 ? [p[0], p[1], p[2]] : [p[0], p[1]]; }); }
function center(points) {
  var p = __pts(points, "center", 1), x = 0, y = 0;
  for (var i = 0; i < p.length; i++) { x += p[i][0]; y += p[i][1]; }
  return [x / p.length, y / p.length];
}
function shift(points, dx, dy) { return __copy(points, "shift").map(function (p) { p[0] += dx; p[1] += dy; return p; }); }
function grow(points, k, cx, cy) {
  if (cx === undefined) { var c = center(points); cx = c[0]; cy = c[1]; }
  var ky = Array.isArray(k) ? k[1] : k, kx = Array.isArray(k) ? k[0] : k;
  return __copy(points, "grow").map(function (p) { p[0] = cx + (p[0] - cx) * kx; p[1] = cy + (p[1] - cy) * ky; return p; });
}
function turn(points, deg, cx, cy) {
  if (cx === undefined) { var c = center(points); cx = c[0]; cy = c[1]; }
  var a = deg * Math.PI / 180, co = Math.cos(a), si = Math.sin(a);
  return __copy(points, "turn").map(function (p) {
    var x = p[0] - cx, y = p[1] - cy; p[0] = cx + x * co - y * si; p[1] = cy + x * si + y * co; return p;
  });
}
function jitter(points, amount) {
  return __copy(points, "jitter").map(function (p) { p[0] += (__rand() * 2 - 1) * amount; p[1] += (__rand() * 2 - 1) * amount; return p; });
}

/* ---------------- colors ---------------- */

function __rgba(c, fn) {
  if (typeof c !== "string") __err(fn + ": color no válido " + c);
  var m = /^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.exec(c);
  if (!m) __err(fn + ": color no válido \"" + c + "\" (usa #rrggbb o #rrggbbaa)");
  var h = m[1];
  if (h.length <= 4) h = h.split("").map(function (x) { return x + x; }).join("");
  return [parseInt(h.slice(0, 2), 16), parseInt(h.slice(2, 4), 16), parseInt(h.slice(4, 6), 16), h.length === 8 ? parseInt(h.slice(6, 8), 16) : 255];
}
function __hex(r, g, b, a) {
  var s = "#" + [r, g, b].map(function (v) { return ("0" + clamp(Math.round(v), 0, 255).toString(16)).slice(-2); }).join("");
  if (a !== undefined && Math.round(a) < 255) s += ("0" + clamp(Math.round(a), 0, 255).toString(16)).slice(-2);
  return s;
}
function __toHsl(c) {
  var r = c[0] / 255, g = c[1] / 255, b = c[2] / 255, mx = Math.max(r, g, b), mn = Math.min(r, g, b), l = (mx + mn) / 2, h = 0, s = 0;
  if (mx !== mn) {
    var d = mx - mn;
    s = l > 0.5 ? d / (2 - mx - mn) : d / (mx + mn);
    if (mx === r) h = (g - b) / d + (g < b ? 6 : 0); else if (mx === g) h = (b - r) / d + 2; else h = (r - g) / d + 4;
    h *= 60;
  }
  return [h, s, l];
}
function __fromHsl(h, s, l, a) {
  h = ((h % 360) + 360) % 360; s = clamp(s, 0, 1); l = clamp(l, 0, 1);
  var q = l < 0.5 ? l * (1 + s) : l + s - l * s, p = 2 * l - q;
  function ch(t) {
    t = ((t % 1) + 1) % 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  }
  return __hex(ch(h / 360 + 1 / 3) * 255, ch(h / 360) * 255, ch(h / 360 - 1 / 3) * 255, a);
}
function hsl(h, s, l, a) { return __fromHsl(h, s / 100, l / 100, a === undefined ? undefined : a * 255); }
function mix(a, b, t) {
  var x = __rgba(a, "mix"), y = __rgba(b, "mix");
  if (t === undefined) t = 0.5;
  return __hex(lerp(x[0], y[0], t), lerp(x[1], y[1], t), lerp(x[2], y[2], t), lerp(x[3], y[3], t));
}
function alpha(c, a) { var x = __rgba(c, "alpha"); return __hex(x[0], x[1], x[2], clamp(a, 0, 1) * 255); }
function __toward(h, target, amount) {
  var d = ((target - h + 540) % 360) - 180;
  return h + Math.sign(d) * Math.min(Math.abs(d), amount);
}
// shade(c, k): k < 0 darkens towards cool blue-violet, k > 0 lightens towards warm yellow.
function shade(c, k) {
  var x = __rgba(c, "shade"), hs = __toHsl(x), h = hs[0], s = hs[1], l = hs[2];
  k = clamp(k, -1, 1);
  if (k < 0) { l = l * (1 + k * 0.85); h = __toward(h, 250, -k * 28); s = s + (-k) * 0.08; }
  else { l = l + (1 - l) * k * 0.85; h = __toward(h, 55, k * 20); s = s - k * 0.06; }
  return __fromHsl(h, s, l, x[3]);
}

/* ---------------- paints: gradients ---------------- */

function __paint(v, fn) {
  if (v === null || v === "none" || v === "transparent") return "none";
  if (typeof v !== "string") __err(fn + ": color no válido " + v);
  if (/^url\(#[\w-]+\)$/.test(v)) return v;
  if (/^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/i.test(v)) return v.toLowerCase();
  if (/^(rgba?|hsla?)\([^)]*\)$/i.test(v) || /^[a-z]+$/i.test(v)) return v;
  __err(fn + ": color no válido \"" + v + "\" (usa #rrggbb, #rrggbbaa o un degradado)");
}
function __stops(stops, fn) {
  if (!Array.isArray(stops) || stops.length < 2) __err(fn + ": hacen falta al menos dos colores");
  return stops.map(function (s, i) {
    var off, col;
    if (Array.isArray(s)) { off = __num(s[0], "la posición del color " + i, fn); col = s[1]; }
    else { off = i / (stops.length - 1); col = s; }
    var c = __rgba(col, fn);
    return '<stop offset="' + __f3(clamp(off, 0, 1)) + '" stop-color="' + __hex(c[0], c[1], c[2]) + '"' +
      (c[3] < 255 ? ' stop-opacity="' + __f3(c[3] / 255) + '"' : "") + "/>";
  }).join("");
}
function linear(x1, y1, x2, y2, stops) {
  var fn = "linear";
  var body = '<linearGradient id="__ID__" gradientUnits="userSpaceOnUse" x1="' + __f(__num(x1, "x1", fn)) + '" y1="' + __f(__num(y1, "y1", fn)) +
    '" x2="' + __f(__num(x2, "x2", fn)) + '" y2="' + __f(__num(y2, "y2", fn)) + '">' + __stops(stops, fn) + "</linearGradient>";
  return "url(#" + __def("lg", body) + ")";
}
function radial(cx, cy, r, stops, o) {
  var fn = "radial";
  o = o || {};
  var foc = (o.fx !== undefined ? ' fx="' + __f(__num(o.fx, "fx", fn)) + '"' : "") + (o.fy !== undefined ? ' fy="' + __f(__num(o.fy, "fy", fn)) + '"' : "");
  var body = '<radialGradient id="__ID__" gradientUnits="userSpaceOnUse" cx="' + __f(__num(cx, "cx", fn)) + '" cy="' + __f(__num(cy, "cy", fn)) +
    '" r="' + __f(__num(r, "r", fn)) + '"' + foc + ">" + __stops(stops, fn) + "</radialGradient>";
  return "url(#" + __def("rg", body) + ")";
}

/* ---------------- style ---------------- */

var __BLENDS = { multiply: 1, screen: 1, overlay: 1, darken: 1, lighten: 1, "color-dodge": 1, "color-burn": 1, "hard-light": 1,
  "soft-light": 1, difference: 1, exclusion: 1, hue: 1, saturation: 1, color: 1, luminosity: 1, normal: 1 };

function __filter(s, fn) {
  var prims = [], n = 0;
  // A drop shadow written with SVG 1.1 primitives (exactly what feDropShadow is defined as), because
  // Illustrator and Inkscape don't read feDropShadow. Each one shadows the result of the step before.
  function shadowPrim(o, glow) {
    if (typeof o !== "object") __err(fn + ": " + (glow ? "glow" : "shadow") + " tiene que ser un objeto { x, y, blur, color }");
    var c = __rgba(o.color || (glow ? "#ffffff" : "#00000066"), fn);
    var src = n ? "s" + n : "SourceGraphic", k = ++n;
    prims.push('<feGaussianBlur in="' + src + '" stdDeviation="' + __f(o.blur === undefined ? (glow ? 12 : 6) : o.blur) + '"/>' +
      '<feOffset dx="' + __f(glow ? 0 : (o.x || 0)) + '" dy="' + __f(glow ? 0 : (o.y === undefined ? 6 : o.y)) + '" result="b' + k + '"/>' +
      '<feFlood flood-color="' + __hex(c[0], c[1], c[2]) + '" flood-opacity="' + __f3(c[3] / 255) + '"/>' +
      '<feComposite in2="b' + k + '" operator="in" result="c' + k + '"/>' +
      '<feMerge result="s' + k + '"><feMergeNode in="c' + k + '"/><feMergeNode in="' + src + '"/></feMerge>');
  }
  if (s.shadow) shadowPrim(s.shadow, false);
  if (s.glow) shadowPrim(s.glow, true);
  if (s.blur) prims.push('<feGaussianBlur stdDeviation="' + __f(__num(s.blur, "blur", fn)) + '"/>');
  if (!prims.length) return null;
  var body = '<filter id="__ID__" x="-50%" y="-50%" width="200%" height="200%" color-interpolation-filters="sRGB">' + prims.join("") + "</filter>";
  return "url(#" + __def("fx", body) + ")";
}
function __common(s, a, fn) {
  var css = [];
  if (s.opacity !== undefined) a.opacity = __f3(clamp(__num(s.opacity, "opacity", fn), 0, 1));
  if (s.blend && s.blend !== "normal") {
    if (!__BLENDS[s.blend]) __err(fn + ": blend \"" + s.blend + "\" no existe (multiply, screen, overlay, soft-light, color-dodge…)");
    css.push("mix-blend-mode:" + s.blend);
  }
  if (s.isolate) css.push("isolation:isolate");
  var f = __filter(s, fn);
  if (f) a.filter = f;
  if (css.length) a.style = css.join(";");
}
// kind: "closed" (filled by default) or "open" (stroked by default)
function __style(s, kind, fn, a) {
  if (typeof s !== "object" || Array.isArray(s)) __err(fn + ": el estilo tiene que ser un objeto { fill, stroke, width, ... }");
  var open = kind === "open";
  var fill = s.fill !== undefined ? s.fill : (!open && s.color !== undefined ? s.color : undefined);
  var stroke = s.stroke !== undefined ? s.stroke : (open && s.color !== undefined ? s.color : undefined);
  if (open) {
    a.fill = fill !== undefined ? __paint(fill, fn) : "none";
    a.stroke = __paint(stroke !== undefined ? stroke : "#000000", fn);
  } else {
    a.fill = fill !== undefined ? __paint(fill, fn) : (stroke !== undefined ? "none" : "#000000");
    if (stroke !== undefined) a.stroke = __paint(stroke, fn);
  }
  if (a.stroke && a.stroke !== "none") {
    a["stroke-width"] = __f(__num(s.width !== undefined ? s.width : (open ? 4 : 2), "width", fn));
    a["stroke-linecap"] = s.cap || "round";
    a["stroke-linejoin"] = s.join || "round";
    if (s.dash) a["stroke-dasharray"] = Array.isArray(s.dash) ? s.dash.join(" ") : String(s.dash);
    if (s.behind) a["paint-order"] = "stroke";
  }
  __common(s, a, fn);
}

/* ---------------- shapes ---------------- */

function __leaf(tag, geom, kind, s, fn, cx, cy) {
  s = s == null ? {} : s;
  if (__count >= __MAX) __err(fn + ": demasiadas formas (máximo " + __MAX + ")");
  var node = __where({ tag: tag, attrs: geom, leaf: true });
  if (s && typeof s === "object" && s.rotate) geom.transform = "rotate(" + __f(__num(s.rotate, "rotate", fn)) + " " + __f(cx) + " " + __f(cy) + ")";
  __style(s, kind, fn, node.attrs);
  if (!s.hidden) { node.parent = __top(); __top().children.push(node); __count++; }
  return node;
}
function rect(x, y, w, h, s) {
  var fn = "rect", g = { x: __f(__num(x, "x", fn)), y: __f(__num(y, "y", fn)), width: __f(Math.abs(__num(w, "el ancho", fn))), height: __f(Math.abs(__num(h, "el alto", fn))) };
  if (s && s.r) { g.rx = __f(__num(s.r, "r", fn)); }
  return __leaf("rect", g, "closed", s, fn, x + w / 2, y + h / 2);
}
function circle(cx, cy, r, s) {
  var fn = "circle";
  return __leaf("circle", { cx: __f(__num(cx, "cx", fn)), cy: __f(__num(cy, "cy", fn)), r: __f(Math.abs(__num(r, "r", fn))) }, "closed", s, fn, cx, cy);
}
function ellipse(cx, cy, rx, ry, s) {
  var fn = "ellipse";
  return __leaf("ellipse", { cx: __f(__num(cx, "cx", fn)), cy: __f(__num(cy, "cy", fn)), rx: __f(Math.abs(__num(rx, "rx", fn))), ry: __f(Math.abs(__num(ry, "ry", fn))) }, "closed", s, fn, cx, cy);
}
function line(x1, y1, x2, y2, s) {
  var fn = "line";
  return __leaf("line", { x1: __f(__num(x1, "x1", fn)), y1: __f(__num(y1, "y1", fn)), x2: __f(__num(x2, "x2", fn)), y2: __f(__num(y2, "y2", fn)) },
    "open", s, fn, (x1 + x2) / 2, (y1 + y2) / 2);
}
function __ptsAttr(p) { return p.map(function (q) { return __f(q[0]) + "," + __f(q[1]); }).join(" "); }
function poly(points, s) {
  var p = __pts(points, "poly", 3), c = center(p);
  return __leaf("polygon", { points: __ptsAttr(p) }, "closed", s, "poly", c[0], c[1]);
}
function polyline(points, s) {
  var p = __pts(points, "polyline", 2), c = center(p);
  return __leaf("polyline", { points: __ptsAttr(p) }, "open", s, "polyline", c[0], c[1]);
}
function path(d, s) {
  if (typeof d !== "string" || !/^\s*[Mm]/.test(d)) __err("path: el trazado tiene que ser un texto SVG que empiece por M");
  if (/NaN|Infinity|undefined/.test(d)) __err("path: el trazado contiene valores no válidos (NaN o undefined)");
  return __leaf("path", { d: d.replace(/\s+/g, " ").trim() }, s && s.open ? "open" : "closed", s, "path", 0, 0);
}

// Centripetal Catmull-Rom through the points, as cubic Bézier segments.
// The third value of a point is its smoothness: 0 makes a sharp corner.
function __segs(p, closed, k) {
  var n = p.length, segs = [];
  function P(i) { return closed ? p[((i % n) + n) % n] : p[Math.max(0, Math.min(n - 1, i))]; }
  function d(a, b) { return Math.sqrt(Math.sqrt((a[0] - b[0]) * (a[0] - b[0]) + (a[1] - b[1]) * (a[1] - b[1]))); }
  var count = closed ? n : n - 1;
  for (var i = 0; i < count; i++) {
    var p0 = P(i - 1), p1 = P(i), p2 = P(i + 1), p3 = P(i + 2);
    var d2 = d(p1, p2), d1 = d(p0, p1), d3 = d(p2, p3);
    var c1 = [p1[0], p1[1]], c2 = [p2[0], p2[1]];
    if (d2 > 1e-6) {
      if (d1 < 1e-6) d1 = d2;
      if (d3 < 1e-6) d3 = d2;
      for (var j = 0; j < 2; j++) {
        c1[j] = (d1 * d1 * p2[j] - d2 * d2 * p0[j] + (2 * d1 * d1 + 3 * d1 * d2 + d2 * d2) * p1[j]) / (3 * d1 * (d1 + d2));
        c2[j] = (d3 * d3 * p1[j] - d2 * d2 * p3[j] + (2 * d3 * d3 + 3 * d3 * d2 + d2 * d2) * p2[j]) / (3 * d3 * (d3 + d2));
      }
      var s1 = k * p1[2], s2 = k * p2[2];
      c1 = [p1[0] + (c1[0] - p1[0]) * s1, p1[1] + (c1[1] - p1[1]) * s1];
      c2 = [p2[0] + (c2[0] - p2[0]) * s2, p2[1] + (c2[1] - p2[1]) * s2];
    }
    segs.push([p1, c1, c2, p2]);
  }
  return segs;
}
function __segsD(segs, closed) {
  var d = "M" + __f(segs[0][0][0]) + " " + __f(segs[0][0][1]);
  for (var i = 0; i < segs.length; i++) {
    var s = segs[i];
    d += "C" + __f(s[1][0]) + " " + __f(s[1][1]) + " " + __f(s[2][0]) + " " + __f(s[2][1]) + " " + __f(s[3][0]) + " " + __f(s[3][1]);
  }
  return closed ? d + "Z" : d;
}
function __tension(s, fn) { return s && s.tension !== undefined ? clamp(__num(s.tension, "tension", fn), 0, 2) : 1; }
function blob(points, s) {
  var p = __pts(points, "blob", 3), c = center(p);
  return __leaf("path", { d: __segsD(__segs(p, true, __tension(s, "blob")), true) }, "closed", s, "blob", c[0], c[1]);
}
function curve(points, s) {
  var p = __pts(points, "curve", 2), c = center(p);
  return __leaf("path", { d: __segsD(__segs(p, false, __tension(s, "curve")), false) }, "open", s, "curve", c[0], c[1]);
}

// brush: a stroke whose width changes along the way, drawn as a filled shape.
function __sample(segs) {
  var out = [];
  for (var i = 0; i < segs.length; i++) {
    var s = segs[i], len = Math.hypot(s[3][0] - s[0][0], s[3][1] - s[0][1]) + Math.hypot(s[1][0] - s[0][0], s[1][1] - s[0][1]) + Math.hypot(s[3][0] - s[2][0], s[3][1] - s[2][1]);
    var n = clamp(Math.ceil(len / 3), 4, 60);
    for (var j = i === 0 ? 0 : 1; j <= n; j++) {
      var t = j / n, u = 1 - t;
      out.push([
        u * u * u * s[0][0] + 3 * u * u * t * s[1][0] + 3 * u * t * t * s[2][0] + t * t * t * s[3][0],
        u * u * u * s[0][1] + 3 * u * u * t * s[1][1] + 3 * u * t * t * s[2][1] + t * t * t * s[3][1]]);
    }
  }
  return out;
}
function __width(spec, u, fn) {
  if (typeof spec === "number") return spec;
  if (typeof spec === "function") return __num(spec(u), "width(u)", fn);
  if (Array.isArray(spec) && spec.length) {
    if (spec.length === 1) return spec[0];
    var x = u * (spec.length - 1), i = Math.min(spec.length - 2, Math.floor(x));
    return lerp(__num(spec[i], "width", fn), __num(spec[i + 1], "width", fn), x - i);
  }
  __err(fn + ": width tiene que ser un número, una lista de grosores o una función (u) => grosor");
}
function brush(points, s) {
  var fn = "brush";
  s = s || {};
  var pts = __pts(points, fn, 2), sm = __sample(__segs(pts, false, __tension(s, fn)));
  var L = [0], i;
  for (i = 1; i < sm.length; i++) L.push(L[i - 1] + Math.hypot(sm[i][0] - sm[i - 1][0], sm[i][1] - sm[i - 1][1]));
  var total = L[L.length - 1] || 1;
  var taper = s.taper === undefined ? [0.3, 0.4] : (typeof s.taper === "number" ? [s.taper, s.taper] : s.taper);
  var spec = s.width === undefined ? 8 : s.width;
  var left = [], right = [], ws = [];
  for (i = 0; i < sm.length; i++) {
    var u = L[i] / total, w = __width(spec, u, fn);
    var a = taper[0] > 0 ? Math.sin(Math.PI / 2 * clamp(u / taper[0], 0, 1)) : 1;
    var b = taper[1] > 0 ? Math.sin(Math.PI / 2 * clamp((1 - u) / taper[1], 0, 1)) : 1;
    w = Math.max(0, w * a * b) / 2;
    var p = sm[Math.max(0, i - 1)], q = sm[Math.min(sm.length - 1, i + 1)];
    var tx = q[0] - p[0], ty = q[1] - p[1], tl = Math.hypot(tx, ty) || 1;
    var nx = -ty / tl, ny = tx / tl;
    left.push([sm[i][0] + nx * w, sm[i][1] + ny * w]);
    right.push([sm[i][0] - nx * w, sm[i][1] - ny * w]);
    ws.push(w);
  }
  function cap(c, w, from, n) { // half circle around an end
    var pts = [], a0 = Math.atan2(from[1] - c[1], from[0] - c[0]);
    for (var k = 1; k < n; k++) { var a = a0 - Math.PI * k / n; pts.push([c[0] + Math.cos(a) * w, c[1] + Math.sin(a) * w]); }
    return pts;
  }
  var outline = left.slice();
  var last = sm.length - 1;
  if (ws[last] > 0.4) outline = outline.concat(cap(sm[last], ws[last], left[last], 8));
  outline = outline.concat(right.slice().reverse());
  if (ws[0] > 0.4) outline = outline.concat(cap(sm[0], ws[0], right[0], 8));
  var d = "M" + outline.map(function (q) { return __f(q[0]) + " " + __f(q[1]); }).join("L") + "Z";
  var st = {};
  for (var k in s) if (k !== "width" && k !== "taper" && k !== "tension" && k !== "stroke") st[k] = s[k];
  if (st.fill === undefined && st.color === undefined) st.fill = s.stroke !== undefined ? s.stroke : "#000000";
  var c = center(pts);
  return __leaf("path", { d: d }, "closed", st, fn, c[0], c[1]);
}

function text(x, y, str, s) {
  var fn = "text";
  s = s || {};
  var g = { x: __f(__num(x, "x", fn)), y: __f(__num(y, "y", fn)), "font-family": s.font || "Lato, 'DejaVu Sans', sans-serif",
    "font-size": __f(s.size || 48) };
  if (s.weight) g["font-weight"] = String(s.weight);
  if (s.anchor) g["text-anchor"] = s.anchor === "center" ? "middle" : s.anchor;
  if (s.spacing) g["letter-spacing"] = __f(s.spacing);
  var st = {};
  for (var k in s) if (k !== "font" && k !== "size" && k !== "weight" && k !== "anchor" && k !== "spacing") st[k] = s[k];
  var node = __leaf("text", g, "closed", st, fn, x, y);
  node.text = __escT(str);
  return node;
}

/* ---------------- structure ---------------- */

function __group(attrs, fn, name) {
  if (typeof fn !== "function") __err(name + ": falta la función con lo que se dibuja dentro, por ejemplo " + name + "(..., () => { ... })");
  var node = { tag: "g", attrs: attrs, children: [], line: __line(), parent: __top() };
  __top().children.push(node);
  __stack.push(node);
  try { fn(); } finally { __stack.pop(); }
  return node;
}
function group(s, fn) {
  if (typeof s === "function") { fn = s; s = {}; }
  s = s || {};
  var a = {}, t = [], name = "group";
  if (s.name) a["data-name"] = String(s.name);
  if (s.move) t.push("translate(" + __f(__num(s.move[0], "move[0]", name)) + " " + __f(__num(s.move[1], "move[1]", name)) + ")");
  if (s.rotate || s.scale !== undefined) {
    var pv = s.around || [__W / 2, __H / 2];
    t.push("translate(" + __f(pv[0]) + " " + __f(pv[1]) + ")");
    if (s.rotate) t.push("rotate(" + __f(__num(s.rotate, "rotate", name)) + ")");
    if (s.scale !== undefined) t.push(Array.isArray(s.scale) ? "scale(" + __f3(s.scale[0]) + " " + __f3(s.scale[1]) + ")" : "scale(" + __f3(__num(s.scale, "scale", name)) + ")");
    t.push("translate(" + __f(-pv[0]) + " " + __f(-pv[1]) + ")");
  }
  if (t.length) a.transform = t.join(" ");
  __common(s, a, name);
  return __group(a, fn, name);
}
// layer(name, { parallax }, () => { ... }): a named layer. parallax is how much it moves with the
// camera in the game (1 = with the world, 0.2 = far away, 1.3 = in front): a number is horizontal
// only (vertically it moves with the world, so bands that meet at a line stay together), and
// [x, y] sets both. pxcli vec godot makes each layer a Parallax2D with it.
function layer(name, o, fn) {
  if (typeof o === "function") { fn = o; o = {}; }
  o = o || {};
  var g = group({ name: name }, fn);
  if (Array.isArray(o.parallax)) g.parallax = __xy(o.parallax, "parallax", "layer");
  else if (o.parallax !== undefined) g.parallax = [__num(o.parallax, "parallax", "layer"), 1];
  return g;
}
// element(name, { pivot: [x, y], parallax }, () => { ... }): a piece of a scene that the game moves or
// animates on its own (a cloud, the sun, a creature, a seaweed). pxcli vec godot exports it as its own
// sprite, with its origin at pivot (default: its centre), so it can rotate and scale around that point.
// parallax, like layer()'s, makes it move with the camera unlike its layer: the sun's reflection on
// the sea stays under the sun.
var __elementNames = {};
function element(name, o, fn) {
  if (typeof o === "function") { fn = o; o = {}; }
  o = o || {};
  if (typeof name !== "string" || !name.trim()) __err("element: hace falta un nombre, por ejemplo element(\"sol\", () => { ... })");
  if (__elementNames[name]) __err("element: ya hay un elemento llamado " + name + "; usa nombres distintos (nube_1, nube_2…)");
  __elementNames[name] = true;
  var g = __group({ "data-element": name }, fn, "element");
  g.element = name;
  if (o.pivot) g.pivot = __xy(o.pivot, "pivot", "element");
  if (Array.isArray(o.parallax)) g.parallax = __xy(o.parallax, "parallax", "element");
  else if (o.parallax !== undefined) g.parallax = [__num(o.parallax, "parallax", "element"), 1];
  return g;
}

var __GEOM = { x: 1, y: 1, width: 1, height: 1, rx: 1, ry: 1, cx: 1, cy: 1, r: 1, x1: 1, y1: 1, x2: 1, y2: 1, points: 1, d: 1, transform: 1,
  "font-family": 1, "font-size": 1, "font-weight": 1, "text-anchor": 1, "letter-spacing": 1 };
function __geom(n, fn) {
  if (!n || !n.leaf) __err(fn + ": hace falta una forma (lo que devuelven rect, circle, blob, path…)");
  if (n.tag === "g" && !n.clipD) __err(fn + ": un objeto 3D de VectorCraft no sirve como forma de recorte");
  if (n.clipD) return '<path d="' + n.clipD + '"' + (n.attrs.transform ? ' transform="' + __escA(n.attrs.transform) + '"' : "") + "/>";
  var s = "<" + n.tag;
  for (var k in n.attrs) if (__GEOM[k]) s += " " + k + '="' + __escA(n.attrs[k]) + '"';
  return n.text != null ? s + ">" + n.text + "</" + n.tag + ">" : s + "/>";
}
// clip(shape | [shapes], () => { ... }): what is drawn inside only shows within the shape(s).
function clip(shape, fn) {
  var shapes = Array.isArray(shape) ? shape : [shape];
  var inner = shapes.map(function (n) { return __geom(n, "clip"); }).join("");
  var id = __def("cp", '<clipPath id="__ID__">' + inner + "</clipPath>");
  return __group({ "clip-path": "url(#" + id + ")" }, fn, "clip");
}
function __axis(axis, fn) {
  if (typeof axis === "number") return "matrix(-1 0 0 1 " + __f(2 * axis) + " 0)";
  if (axis && typeof axis.x === "number") return "matrix(-1 0 0 1 " + __f(2 * axis.x) + " 0)";
  if (axis && typeof axis.y === "number") return "matrix(1 0 0 -1 0 " + __f(2 * axis.y) + ")";
  __err(fn + ": el eje tiene que ser una x (número), { x } o { y }");
}
// mirror(x, () => { ... }): draws the shapes and their mirror image around x (or { y }).
function mirror(axis, fn) {
  var t = __axis(axis, "mirror");
  var outer = __group({}, function () { __group({}, fn, "mirror"); }, "mirror");
  var a = outer.children[0];
  outer.children.push({ tag: "g", attrs: { transform: t }, children: a.children, line: a.line, parent: outer });
  __count += __countLeaves(a);
  return outer;
}
function __countLeaves(n) { var c = 0; (n.children || []).forEach(function (ch) { c += ch.leaf ? 1 : __countLeaves(ch); }); return c; }
// flip(x, () => { ... }): draws only the mirror image (reflections).
function flip(axis, fn) { return __group({ transform: __axis(axis, "flip") }, fn, "flip"); }

// grain(): film grain over everything drawn so far (or over [x, y, w, h]).
function grain(o) {
  var fn = "grain";
  o = o || {};
  var size = o.size || 1, sd = o.seed || 7;
  var id = __def("gn", '<filter id="__ID__" x="0" y="0" width="100%" height="100%" color-interpolation-filters="sRGB">' +
    '<feTurbulence type="fractalNoise" baseFrequency="' + __f3(0.9 / size) + '" numOctaves="3" seed="' + sd + '" stitchTiles="stitch"/>' +
    '<feColorMatrix type="saturate" values="0"/><feComponentTransfer><feFuncA type="linear" slope="0" intercept="1"/></feComponentTransfer></filter>');
  var r = o.rect || [0, 0, __W, __H];
  var node = __leaf("rect", { x: __f(r[0]), y: __f(r[1]), width: __f(r[2]), height: __f(r[3]) }, "closed",
    { fill: "#808080", opacity: o.opacity === undefined ? 0.12 : o.opacity, blend: o.blend || "overlay" }, fn, 0, 0);
  node.attrs.filter = "url(#" + id + ")";
  node.attrs["pointer-events"] = "none";
  return node;
}

/* ---------------- shape tools (ideas from VectorCraft that SVG can express) ---------------- */

function __replace(old, neu) {
  var p = old.parent, i = p ? p.children.indexOf(old) : -1;
  if (i < 0) return false;
  p.children[i] = neu; neu.parent = p;
  return true;
}
function __insert(ref, neu, after) {
  var p = ref && ref.parent, i = p ? p.children.indexOf(ref) : -1;
  if (i < 0) { neu.parent = __top(); __top().children.push(neu); return; }
  p.children.splice(after ? i + 1 : i, 0, neu); neu.parent = p;
}
function __isShape(n) { return n && typeof n === "object" && n.leaf === true; }
function __needShape(n, fn) {
  if (!__isShape(n)) __err(fn + ": hace falta una forma (lo que devuelven rect, circle, blob, path…)");
  return n;
}
function __nums(str) { return (String(str).match(/-?\d*\.?\d+(?:e[-+]?\d+)?/gi) || []).map(parseFloat); }

// Outline of a shape as a polygon (curves sampled), in its own coordinates.
function __outline(n, fn) {
  if (Array.isArray(n)) return __pts(n, fn, 2).map(function (p) { return [p[0], p[1]]; });
  __needShape(n, fn);
  var a = n.attrs, f = function (k) { return parseFloat(a[k]) || 0; };
  if (n.clipD) return __dPts(n.clipD) || __boxPts(n.box);
  switch (n.tag) {
    case "rect": return [[f("x"), f("y")], [f("x") + f("width"), f("y")], [f("x") + f("width"), f("y") + f("height")], [f("x"), f("y") + f("height")]];
    case "circle": return ring(f("cx"), f("cy"), f("r"), f("r"), 48).map(function (p) { return [p[0], p[1]]; });
    case "ellipse": return ring(f("cx"), f("cy"), f("rx"), f("ry"), 48).map(function (p) { return [p[0], p[1]]; });
    case "line": return [[f("x1"), f("y1")], [f("x2"), f("y2")]];
    case "polygon": case "polyline": {
      var v = __nums(a.points), out = [];
      for (var i = 0; i + 1 < v.length; i += 2) out.push([v[i], v[i + 1]]);
      return out;
    }
    case "path": return __dPts(a.d) || __boxPts(__bboxNums(__nums(a.d)));
    case "text": {
      var size = f("font-size") || 48, w = size * 0.55 * (n.text || "").length, x = f("x");
      if (a["text-anchor"] === "middle") x -= w / 2; else if (a["text-anchor"] === "end") x -= w;
      return __boxPts([x, f("y") - size * 0.8, w, size]);
    }
  }
  __err(fn + ": no sé medir esa forma");
}
function __boxPts(b) { return [[b[0], b[1]], [b[0] + b[2], b[1]], [b[0] + b[2], b[1] + b[3]], [b[0], b[1] + b[3]]]; }
function __bboxNums(v) {
  var x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  for (var i = 0; i + 1 < v.length; i += 2) { x0 = Math.min(x0, v[i]); x1 = Math.max(x1, v[i]); y0 = Math.min(y0, v[i + 1]); y1 = Math.max(y1, v[i + 1]); }
  return [x0, y0, x1 - x0, y1 - y0];
}
// Absolute M/L/H/V/C/Q/Z path data to points (curves sampled). null when it uses anything else.
function __dPts(d) {
  if (/[a-ln-zA-BD-GI-KN-PR-UW-Y]/.test(d.replace(/e[-+]?\d/gi, ""))) return null;
  var toks = d.match(/[MLHVCQZ]|-?\d*\.?\d+(?:e[-+]?\d+)?/gi), out = [], i = 0, cmd = "", x = 0, y = 0, sx = 0, sy = 0;
  function num() { return parseFloat(toks[i++]); }
  while (i < toks.length) {
    if (/[A-Za-z]/.test(toks[i])) cmd = toks[i++];
    if (cmd === "Z" || cmd === "z") { x = sx; y = sy; continue; }
    if (cmd === "M") { x = num(); y = num(); sx = x; sy = y; out.push([x, y]); cmd = "L"; }
    else if (cmd === "L") { x = num(); y = num(); out.push([x, y]); }
    else if (cmd === "H") { x = num(); out.push([x, y]); }
    else if (cmd === "V") { y = num(); out.push([x, y]); }
    else if (cmd === "C" || cmd === "Q") {
      var c = cmd === "C" ? [num(), num(), num(), num()] : [num(), num()], ex = num(), ey = num();
      for (var k = 1; k <= 8; k++) {
        var t = k / 8, u = 1 - t;
        if (cmd === "C") out.push([u * u * u * x + 3 * u * u * t * c[0] + 3 * u * t * t * c[2] + t * t * t * ex, u * u * u * y + 3 * u * u * t * c[1] + 3 * u * t * t * c[3] + t * t * t * ey]);
        else out.push([u * u * x + 2 * u * t * c[0] + t * t * ex, u * u * y + 2 * u * t * c[1] + t * t * ey]);
      }
      x = ex; y = ey;
    } else return null;
  }
  return out.length ? out : null;
}
function bbox(shape) {
  var p = __outline(shape, "bbox"), b = __bboxNums([].concat.apply([], p));
  return { x: b[0], y: b[1], w: b[2], h: b[3], cx: b[0] + b[2] / 2, cy: b[1] + b[3] / 2 };
}

// cut(shape, cutters): takes the cutters' area out of a drawn shape (moon crescents, holes, bites).
function cut(shape, cutters) {
  var fn = "cut";
  __needShape(shape, fn);
  if (!shape.parent) __err(fn + ": la forma que recortas tiene que estar dibujada (sin hidden)");
  var list = Array.isArray(cutters) ? cutters : [cutters];
  var big = Math.max(__W, __H) * 4;
  var id = __def("ct", '<mask id="__ID__" maskUnits="userSpaceOnUse" x="' + (-big) + '" y="' + (-big) + '" width="' + (3 * big) + '" height="' + (3 * big) + '">' +
    '<rect x="' + (-big) + '" y="' + (-big) + '" width="' + (3 * big) + '" height="' + (3 * big) + '" fill="#ffffff"/>' +
    list.map(function (n) { return __geom(n, fn); }).join("") + "</mask>");
  var w = { tag: "g", attrs: { mask: "url(#" + id + ")" }, children: [], line: __line() };
  __replace(shape, w);
  w.children.push(shape); shape.parent = w;
  return shape;
}

function __shadowCopy(n, fn) {
  __needShape(n, fn);
  var a = {};
  if (n.clipD) { a.d = n.clipD; if (n.attrs.transform) a.transform = n.attrs.transform; return __where({ tag: "path", attrs: a, leaf: true }); }
  for (var k in n.attrs) if (__GEOM[k]) a[k] = n.attrs[k];
  return __where({ tag: n.tag, attrs: a, leaf: true, text: n.text });
}
// outline(shapes, distance, color): distance > 0 draws a border around the shapes, behind them
// (sticker edge, halo); distance < 0 draws a band just inside each shape's edge, on top (rim light, inner line).
function outline(shapes, d, color, s) {
  var fn = "outline", list = Array.isArray(shapes) ? shapes : [shapes];
  __num(d, "la distancia", fn);
  var c = __paint(color === undefined ? "#ffffff" : color, fn);
  s = s || {};
  if (d >= 0) {
    var g = { tag: "g", attrs: {}, children: [], line: __line() };
    __common(s, g.attrs, fn);
    list.forEach(function (n) {
      var k = __shadowCopy(n, fn);
      k.attrs.fill = c; k.attrs.stroke = c; k.attrs["stroke-width"] = __f(2 * d); k.attrs["stroke-linejoin"] = "round";
      k.parent = g; g.children.push(k); __count++;
    });
    __insert(list[0], g, false);
    return g;
  }
  var last = null;
  list.forEach(function (n) {
    var id = __def("cp", '<clipPath id="__ID__">' + __geom(n, fn) + "</clipPath>");
    var g = { tag: "g", attrs: { "clip-path": "url(#" + id + ")" }, children: [], line: __line() };
    var k = __shadowCopy(n, fn);
    k.attrs.fill = "none"; k.attrs.stroke = c; k.attrs["stroke-width"] = __f(-2 * d); k.attrs["stroke-linejoin"] = "round";
    __common(s, k.attrs, fn);
    k.parent = g; g.children.push(k); __count++;
    __insert(n, g, true);
    last = g;
  });
  return last;
}

function __perimeter(p, closed) {
  var L = 0;
  for (var i = 1; i < p.length; i++) L += Math.hypot(p[i][0] - p[i - 1][0], p[i][1] - p[i - 1][1]);
  if (closed) L += Math.hypot(p[0][0] - p[p.length - 1][0], p[0][1] - p[p.length - 1][1]);
  return L;
}
// subdivide(points, step): adds points every `step` units along each side (closed by default).
function subdivide(points, step, closed) {
  var p = __pts(points, "subdivide", 2), out = [];
  if (closed === undefined) closed = true;
  step = step || 20;
  for (var i = 0; i < p.length; i++) {
    out.push(p[i].slice());
    if (!closed && i === p.length - 1) break;
    var q = p[(i + 1) % p.length], n = Math.floor(Math.hypot(q[0] - p[i][0], q[1] - p[i][1]) / step);
    for (var k = 1; k <= n; k++) {
      var t = k / (n + 1);
      out.push([lerp(p[i][0], q[0], t), lerp(p[i][1], q[1], t), 1]);
    }
  }
  return out;
}
function box(x, y, w, h) { return [[x, y, 0], [x + w, y, 0], [x + w, y + h, 0], [x, y + h, 0]]; }
// roughen(points, size, { detail, sharp }): an irregular edge (torn paper, rock, fur, foliage).
function roughen(points, size, o) {
  o = o || {};
  var p = __pts(points, "roughen", 2), detail = o.detail || 8;
  var out = subdivide(p, 100 / detail, o.open ? false : true);
  return out.map(function (q) { return [q[0] + (__rand() * 2 - 1) * size, q[1] + (__rand() * 2 - 1) * size, o.sharp ? 0 : q[2]]; });
}
// warp(points, kind, amount): bends point lists like VectorCraft's Warp effects.
// "arc" (amount: fraction of the height, + bends the middle up), "wave" (amount: units; o.length),
// "flag" (a wave that grows to the right), "bulge" (+ inflates, - pinches), "twist" (degrees),
// or a function (x, y, u, v) => [x, y] with u, v from 0 to 1 across the bounds.
function warp(points, kind, amount, o) {
  var fn = "warp";
  o = o || {};
  var p = __pts(points, fn, 2), b = __bboxNums([].concat.apply([], p.map(function (q) { return [q[0], q[1]]; })));
  if (o.densify !== false) p = subdivide(p, o.step || Math.max(b[2], b[3]) / 30, !o.open);
  var x0 = b[0], y0 = b[1], w = b[2] || 1, h = b[3] || 1, cx = x0 + w / 2, cy = y0 + h / 2, R = Math.hypot(w, h) / 2 || 1;
  amount = amount === undefined ? 0.3 : amount;
  return p.map(function (q) {
    var x = q[0], y = q[1], u = (x - x0) / w, v = (y - y0) / h, r = Math.hypot(x - cx, y - cy) / R, out;
    if (typeof kind === "function") out = kind(x, y, u, v);
    else if (kind === "arc") out = [x, y - amount * h * (1 - (2 * u - 1) * (2 * u - 1))];
    else if (kind === "wave") out = [x, y + amount * Math.sin(2 * Math.PI * (x - x0) / (o.length || w / 2) + (o.phase || 0))];
    else if (kind === "flag") out = [x, y + amount * u * Math.sin(2 * Math.PI * (x - x0) / (o.length || w / 1.5) + (o.phase || 0))];
    else if (kind === "bulge") { var k = 1 + amount * (1 - r * r); out = [cx + (x - cx) * k, cy + (y - cy) * k]; }
    else if (kind === "twist") { var a = amount * (1 - r) * Math.PI / 180, co = Math.cos(a), si = Math.sin(a); out = [cx + (x - cx) * co - (y - cy) * si, cy + (x - cx) * si + (y - cy) * co]; }
    else __err(fn + ": tipo \"" + kind + "\" no existe (arc, wave, flag, bulge, twist o una función)");
    if (!Array.isArray(out)) __err(fn + ": la función tiene que devolver [x, y]");
    return [out[0], out[1], q[2]];
  });
}

function __signedArea(p) { var a = 0; for (var i = 0; i < p.length; i++) { var q = p[(i + 1) % p.length]; a += p[i][0] * q[1] - q[0] * p[i][1]; } return a / 2; }
function __resample(p, n) {
  var sm = __sample(__segs(p, true, 1)), L = [0], i;
  for (i = 1; i < sm.length; i++) L.push(L[i - 1] + Math.hypot(sm[i][0] - sm[i - 1][0], sm[i][1] - sm[i - 1][1]));
  var total = L[L.length - 1], out = [], j = 0;
  for (var k = 0; k < n; k++) {
    var t = total * k / n;
    while (j < L.length - 2 && L[j + 1] < t) j++;
    var f = (t - L[j]) / ((L[j + 1] - L[j]) || 1);
    out.push([lerp(sm[j][0], sm[j + 1][0], f), lerp(sm[j][1], sm[j + 1][1], f), 1]);
  }
  return out;
}
function __ease(t, e) {
  if (e === "in") return t * t;
  if (e === "out") return 1 - (1 - t) * (1 - t);
  if (e === "inout") return t * t * (3 - 2 * t);
  return t;
}
// blend(a, b, steps, style): steps shapes morphing from points a (drawn first) to points b,
// colours and opacity blended too: fill: ["#from", "#to"], opacity: [from, to], ease: "in"|"out"|"inout".
function blend(a, b, steps, s) {
  var fn = "blend";
  s = s || {};
  var pa = __pts(a, fn, 3), pb = __pts(b, fn, 3);
  if (pa.length !== pb.length) {
    var n = Math.max(pa.length, pb.length, 32);
    pa = __resample(pa, n); pb = __resample(pb, n);
    if ((__signedArea(pa) > 0) !== (__signedArea(pb) > 0)) pb = pb.reverse();
    var best = 0, bestD = Infinity;
    for (var k = 0; k < n; k++) {
      var dd = 0;
      for (var i = 0; i < n; i++) { var q = pb[(i + k) % n]; dd += (pa[i][0] - q[0]) * (pa[i][0] - q[0]) + (pa[i][1] - q[1]) * (pa[i][1] - q[1]); }
      if (dd < bestD) { bestD = dd; best = k; }
    }
    pb = pb.slice(best).concat(pb.slice(0, best));
  }
  steps = Math.max(2, Math.round(__num(steps, "steps", fn)));
  function pair(v, d) { return Array.isArray(v) ? v : [v === undefined ? d : v, v === undefined ? d : v]; }
  var fills = pair(s.fill !== undefined ? s.fill : s.color, "#000000"), ops = pair(s.opacity, 1), strokes = s.stroke !== undefined ? pair(s.stroke) : null;
  var out = [];
  for (var st = 0; st < steps; st++) {
    var t = __ease(st / (steps - 1), s.ease), pts = [];
    for (var i2 = 0; i2 < pa.length; i2++) pts.push([lerp(pa[i2][0], pb[i2][0], t), lerp(pa[i2][1], pb[i2][1], t), lerp(pa[i2][2], pb[i2][2], t)]);
    var style = {};
    for (var key in s) if (["fill", "color", "opacity", "stroke", "ease", "shape"].indexOf(key) < 0) style[key] = s[key];
    style.fill = fills[0] === "none" ? "none" : mix(fills[0], fills[1], t);
    if (ops[0] !== 1 || ops[1] !== 1) style.opacity = lerp(ops[0], ops[1], t);
    if (strokes) style.stroke = mix(strokes[0], strokes[1], t);
    out.push(s.shape === "poly" ? poly(pts, style) : blob(pts, style));
  }
  return out;
}

// Straight lines across a shape's bounds, `gap` apart, at `angle` degrees.
function __scan(b, angle, gap) {
  var cx = b[0] + b[2] / 2, cy = b[1] + b[3] / 2, R = Math.hypot(b[2], b[3]) / 2 + gap;
  var a = angle * Math.PI / 180, dx = Math.cos(a), dy = Math.sin(a), nx = -dy, ny = dx, out = [];
  for (var t = -R; t <= R; t += gap) out.push([[cx + nx * t - dx * R, cy + ny * t - dy * R], [cx + nx * t + dx * R, cy + ny * t + dy * R]]);
  return out;
}
// Where a line enters and leaves a polygon (first and last crossing).
function __span(seg, poly) {
  var ax = seg[0][0], ay = seg[0][1], bx = seg[1][0], by = seg[1][1], ts = [];
  for (var i = 0; i < poly.length; i++) {
    var p = poly[i], q = poly[(i + 1) % poly.length];
    var ex = q[0] - p[0], ey = q[1] - p[1], den = (bx - ax) * ey - (by - ay) * ex;
    if (Math.abs(den) < 1e-9) continue;
    var t = ((p[0] - ax) * ey - (p[1] - ay) * ex) / den, u = ((p[0] - ax) * (by - ay) - (p[1] - ay) * (bx - ax)) / den;
    if (t >= 0 && t <= 1 && u >= 0 && u <= 1) ts.push(t);
  }
  if (ts.length < 2) return null;
  var t0 = Math.min.apply(null, ts), t1 = Math.max.apply(null, ts);
  return [[lerp(ax, bx, t0), lerp(ay, by, t0)], [lerp(ax, bx, t1), lerp(ay, by, t1)]];
}
// hatch(shape, { angle, gap, width, color, cross, wobble, opacity, blend }): parallel ink lines inside a shape.
function hatch(shape, o) {
  var fn = "hatch";
  o = o || {};
  var poly = __outline(shape, fn), b = __bboxNums([].concat.apply([], poly));
  var angle = o.angle === undefined ? 45 : o.angle, gap = o.gap || 12, wob = o.wobble || 0, d = "";
  var angles = [angle];
  if (o.cross) angles.push(typeof o.cross === "number" ? o.cross : angle + 90);
  angles.forEach(function (ang) {
    __scan(b, ang, gap).forEach(function (seg) {
      var sp = __span(seg, poly);
      if (!sp) return;
      if (!wob) { d += "M" + __f(sp[0][0]) + " " + __f(sp[0][1]) + "L" + __f(sp[1][0]) + " " + __f(sp[1][1]); return; }
      var n = Math.max(2, Math.round(Math.hypot(sp[1][0] - sp[0][0], sp[1][1] - sp[0][1]) / (gap * 3))), pts = [];
      for (var k = 0; k <= n; k++) pts.push([lerp(sp[0][0], sp[1][0], k / n) + (__rand() * 2 - 1) * wob, lerp(sp[0][1], sp[1][1], k / n) + (__rand() * 2 - 1) * wob, 1]);
      d += __segsD(__segs(pts, false, 1), false);
    });
  });
  if (!d) return null;
  var node = null;
  var st = { open: true, stroke: o.color || "#000000", width: o.width || 2, cap: o.cap || "round" };
  ["opacity", "blend", "blur"].forEach(function (k) { if (o[k] !== undefined) st[k] = o[k]; });
  if (__isShape(shape)) clip(shape, function () { node = path(d, st); }); else node = path(d, st);
  return node;
}
// scribble(shape, { angle, gap, width, color, loose }): a loose back-and-forth pen fill, as if drawn by hand.
function scribble(shape, o) {
  var fn = "scribble";
  o = o || {};
  var poly = __outline(shape, fn), b = __bboxNums([].concat.apply([], poly));
  var gap = o.gap || 10, loose = o.loose === undefined ? 0.35 : o.loose, pts = [], flip = false;
  __scan(b, o.angle === undefined ? 30 : o.angle, gap).forEach(function (seg) {
    var sp = __span(seg, poly);
    if (!sp) return;
    var a = flip ? sp[1] : sp[0], z = flip ? sp[0] : sp[1], j = gap * loose;
    pts.push([a[0] + (__rand() * 2 - 1) * j, a[1] + (__rand() * 2 - 1) * j, 1]);
    pts.push([z[0] + (__rand() * 2 - 1) * j, z[1] + (__rand() * 2 - 1) * j, 1]);
    flip = !flip;
  });
  if (pts.length < 2) return null;
  var st = { open: true, stroke: o.color || "#000000", width: o.width || 2.5 };
  ["opacity", "blend", "blur"].forEach(function (k) { if (o[k] !== undefined) st[k] = o[k]; });
  var d = __segsD(__segs(pts, false, o.tension === undefined ? 0.6 : o.tension), false), node = null;
  if (__isShape(shape) && o.clip !== false) clip(shape, function () { node = path(d, st); }); else node = path(d, st);
  return node;
}
// repeat_radial(cx, cy, n, (i, t) => { ... }): draws the same thing n times around a centre (petals, rays, gears).
function repeat_radial(cx, cy, n, fn) {
  if (typeof fn !== "function") __err("repeat_radial: falta la función que dibuja cada copia");
  var out = [];
  for (var i = 0; i < n; i++) (function (i) { out.push(group({ rotate: i * 360 / n, around: [cx, cy] }, function () { fn(i, i / n); })); })(i);
  return out;
}

/* ---------------- VectorCraft: 3D, freeform gradients and its effects ---------------- */

function __vcColor(c, fn) { var x = __rgba(c, fn); return { hex: __hex(x[0], x[1], x[2]), opacity: x[3] / 255 }; }
function __ellipseD(cx, cy, rx, ry) {
  var k = 0.5523;
  return "M" + __f(cx) + " " + __f(cy - ry) +
    "C" + __f(cx + rx * k) + " " + __f(cy - ry) + " " + __f(cx + rx) + " " + __f(cy - ry * k) + " " + __f(cx + rx) + " " + __f(cy) +
    "C" + __f(cx + rx) + " " + __f(cy + ry * k) + " " + __f(cx + rx * k) + " " + __f(cy + ry) + " " + __f(cx) + " " + __f(cy + ry) +
    "C" + __f(cx - rx * k) + " " + __f(cy + ry) + " " + __f(cx - rx) + " " + __f(cy + ry * k) + " " + __f(cx - rx) + " " + __f(cy) +
    "C" + __f(cx - rx) + " " + __f(cy - ry * k) + " " + __f(cx - rx * k) + " " + __f(cy - ry) + " " + __f(cx) + " " + __f(cy - ry) + "Z";
}
// Path data of a shape (or of a point list, as a blob), for VectorCraft.
function __shapeD(n, fn) {
  if (Array.isArray(n)) return __segsD(__segs(__pts(n, fn, 3), true, 1), true);
  __needShape(n, fn);
  if (n.clipD) return n.clipD;
  var a = n.attrs, f = function (k) { return parseFloat(a[k]) || 0; };
  switch (n.tag) {
    case "path": return a.d;
    case "circle": return __ellipseD(f("cx"), f("cy"), f("r"), f("r"));
    case "ellipse": return __ellipseD(f("cx"), f("cy"), f("rx"), f("ry"));
    case "polygon": return "M" + a.points.trim().split(/\s+/).join("L") + "Z";
    case "rect": {
      var x = f("x"), y = f("y"), w = f("width"), h = f("height"), r = Math.min(f("rx"), w / 2, h / 2);
      if (!r) return "M" + __f(x) + " " + __f(y) + "H" + __f(x + w) + "V" + __f(y + h) + "H" + __f(x) + "Z";
      var k = 0.5523 * r, F = __f;
      return "M" + F(x + r) + " " + F(y) + "H" + F(x + w - r) + "C" + F(x + w - r + k) + " " + F(y) + " " + F(x + w) + " " + F(y + r - k) + " " + F(x + w) + " " + F(y + r) +
        "V" + F(y + h - r) + "C" + F(x + w) + " " + F(y + h - r + k) + " " + F(x + w - r + k) + " " + F(y + h) + " " + F(x + w - r) + " " + F(y + h) +
        "H" + F(x + r) + "C" + F(x + r - k) + " " + F(y + h) + " " + F(x) + " " + F(y + h - r + k) + " " + F(x) + " " + F(y + h - r) +
        "V" + F(y + r) + "C" + F(x) + " " + F(y + r - k) + " " + F(x + r - k) + " " + F(y) + " " + F(x + r) + " " + F(y) + "Z";
    }
  }
  __err(fn + ": esa forma no se puede pasar a VectorCraft (usa blob, path, rect, circle, ellipse o poly)");
}
function __shapeFill(n, o, fn) {
  var c = o && (o.color || o.fill);
  if (!c && __isShape(n) && /^#/.test(n.attrs.fill || "")) c = n.attrs.fill;
  if (!c) __err(fn + ": falta el color (color: \"#rrggbb\")");
  return __vcColor(c, fn);
}
function __vcRun(steps, fn) {
  if (!__hasVC) __err(fn + ": hace falta VectorCraft (vectorcraft-cli) instalado");
  return __vc(JSON.stringify({ w: __W, h: __H, steps: steps }));
}
// The result replaces the shape it came from (same place in the drawing), or is drawn on top.
function __vcNode(shape, steps, fn, s, d) {
  var markup = __vcRun(steps, fn);
  var node = __where({ tag: "g", attrs: {}, leaf: true, text: markup, clipD: d || null });
  if (__isShape(shape) && shape.attrs.transform) node.attrs.transform = shape.attrs.transform;
  __common(s || {}, node.attrs, fn);
  if (!(__isShape(shape) && shape.parent && __replace(shape, node))) {
    node.parent = __top(); __top().children.push(node); __count++;
  }
  return node;
}
function __base(d, fill, closed) {
  return [{ command: "path.create", params: { d: d, closed: closed !== false } },
    { command: "paint.setStroke", params: { none: true } },
    { command: "paint.setFill", params: { color: fill.hex } }];
}
// 3D options shared by revolve, extrude and inflate.
function __3d(o, fn, def) {
  var p = {}, k;
  for (k in def) p[k] = def[k];
  if (o.rotate) { p.rotationX = __num(o.rotate[0], "rotate[0]", fn); p.rotationY = __num(o.rotate[1] || 0, "rotate[1]", fn); p.rotationZ = __num(o.rotate[2] || 0, "rotate[2]", fn); }
  if (o.perspective !== undefined) p.perspectiveAngle = o.perspective;
  if (o.ambient !== undefined) p.ambient = o.ambient;
  if (o.shade !== undefined) p.shade = !!o.shade;
  var L = o.light;
  if (L) {
    if (L.from !== undefined) p.lightRotation = L.from;
    if (L.height !== undefined) p.lightHeight = L.height;
    if (L.intensity !== undefined) p.lightIntensity = L.intensity;
    if (L.softness !== undefined) p.lightSoftness = L.softness;
    if (L.color) p.lightColor = __vcColor(L.color, fn).hex;
    if (L.behind !== undefined) p.lightBehind = !!L.behind;
  }
  if (o.lights) p.lights = o.lights.map(function (l) {
    return { color: __vcColor(l.color || "#ffffff", fn).hex, intensity: l.intensity === undefined ? 80 : l.intensity,
      rotation: l.from === undefined ? 0 : l.from, height: l.height === undefined ? 30 : l.height, softness: l.softness || 0, behind: !!l.behind };
  });
  if (o.params) for (k in o.params) p[k] = o.params[k];
  return p;
}
function __finish(steps, o, rasterDefault) {
  if (o.raster === undefined ? rasterDefault : o.raster) steps.push({ command: "effect.apply", params: { effect: "rasterize", params: {} } });
  return steps;
}

// freeform(shape, [[x, y, "#color", spread], ...]): fills the shape with a smooth colour field that
// passes through each coloured point, like soft paint (spread 0..1 widens a point's solid colour).
function freeform(shape, points, s) {
  var fn = "freeform", d = __shapeD(shape, fn);
  if (!Array.isArray(points) || !points.length) __err(fn + ": hacen falta puntos de color [[x, y, \"#color\"], ...]");
  var steps = __base(d, { hex: "#808080" }).slice(0, 2);
  steps.push({ command: "paint.setFill", params: { gradient: { kind: "freeform" } } });
  points.forEach(function (p, i) {
    if (!Array.isArray(p) || p.length < 3) __err(fn + ": el punto " + i + " tiene que ser [x, y, \"#color\"] o [x, y, \"#color\", spread]");
    var c = __vcColor(p[2], fn);
    steps.push({ command: "paint.freeform.addPoint", params: { at: [__num(p[0], "x", fn), __num(p[1], "y", fn)], color: c.hex, opacity: c.opacity, spread: clamp(p[3] || 0, 0, 1) } });
  });
  for (var k = 0; k < 4; k++) steps.push({ command: "paint.freeform.deletePoint", params: { index: 0 } });
  return __vcNode(shape, steps, fn, s, d);
}
// revolve(cx, profile, opts): spins a profile into a lit 3D object around the vertical axis x = cx:
// vases, bottles, cups, lamps, planets, chess pieces. profile: [[radius, y], ...] from top to bottom.
function revolve(cx, profile, o) {
  var fn = "revolve";
  if (Array.isArray(cx)) __err(fn + ": el primer argumento es la x del eje: revolve(cx, [[radio, y], ...], { color })");
  __num(cx, "cx (el eje)", fn);
  o = o || {};
  var p = __pts(profile, fn, 2), rmin = Infinity;
  p.forEach(function (q) { if (q[0] < 0) __err(fn + ": los radios del perfil no pueden ser negativos"); rmin = Math.min(rmin, q[0]); });
  var abs = p.map(function (q) { return [cx + q[0], q[1], q[2]]; });
  var d = o.straight ? "M" + abs.map(function (q) { return __f(q[0]) + " " + __f(q[1]); }).join("L") : __segsD(__segs(abs, false, 1), false);
  var steps = __base(d, __shapeFill(null, o, fn), false);
  steps.push({ command: "effect.apply", params: { effect: "threeD.revolve", params: __3d(o, fn, { rotationX: -10, angle: o.angle === undefined ? 360 : o.angle, offset: rmin }) } });
  return __vcNode(null, __finish(steps, o, true), fn, o.style, null);
}
// extrude(shape, opts): gives a flat shape depth (signs, letters, boxes, buttons); bevel for rounded edges.
function extrude(shape, o) {
  var fn = "extrude";
  o = o || {};
  var d = __shapeD(shape, fn), def = { depth: o.depth === undefined ? 50 : o.depth };
  if (o.bevel) { def.bevel = o.bevel === true ? "rounded" : o.bevel; def.bevelWidth = o.bevelWidth || 6; def.bevelHeight = o.bevelHeight || 6; }
  var steps = __base(d, __shapeFill(shape, o, fn));
  steps.push({ command: "effect.apply", params: { effect: "threeD.extrude", params: __3d(o, fn, def) } });
  return __vcNode(shape, __finish(steps, o, false), fn, o.style, null);
}
// inflate(shape, opts): puffs a flat shape up like a balloon or a cushion (volume 0..100).
function inflate(shape, o) {
  var fn = "inflate";
  o = o || {};
  var d = __shapeD(shape, fn);
  var steps = __base(d, __shapeFill(shape, o, fn));
  steps.push({ command: "effect.apply", params: { effect: "threeD.inflate", params: __3d(o, fn, { rotationX: 0, rotationY: 0, rotationZ: 0, volume: o.volume === undefined ? 60 : o.volume, depth: o.depth || 0 }) } });
  return __vcNode(shape, __finish(steps, o, true), fn, o.style, d);
}
// fx(shape, "effect.id", params): any VectorCraft live effect on a shape: "distort.roughen",
// "stylize.scribble", "warp.flag", "distort.puckerBloat", "artistic.watercolor", "sketch.charcoal"…
function fx(shape, effect, params, o) {
  var fn = "fx";
  o = o || {};
  if (typeof effect !== "string" || !/^[a-zA-Z]+\.[a-zA-Z.]+$/.test(effect)) __err(fn + ": el efecto tiene que ser un id como \"distort.roughen\"");
  var d = __shapeD(shape, fn), fill = __shapeFill(shape, o, fn);
  var steps = __base(d, fill);
  if (o.stroke) {
    steps[1] = { command: "paint.setStroke", params: { color: __vcColor(o.stroke, fn).hex } };
    steps.push({ command: "stroke.set", params: { width: o.width || 2 } });
  }
  steps.push({ command: "effect.apply", params: { effect: effect, params: params || {} } });
  return __vcNode(shape, __finish(steps, o, false), fn, o.style, d);
}

/* ---------------- 3D construction: camera, boxes, houses, cylinders ---------------- */
// World axes: x to the right, y up, z away from the viewer (think in metres). Geometry is
// computed, so faces, roofs and ellipses always agree with one point of view and one light.

function __v(a, b) { return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]; }
function __add(a, b) { return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]; }
function __mul(a, k) { return [a[0] * k, a[1] * k, a[2] * k]; }
function __dot(a, b) { return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]; }
function __cross(a, b) { return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]; }
function __norm(a) { var l = Math.sqrt(__dot(a, a)) || 1; return [a[0] / l, a[1] / l, a[2] / l]; }
function __vec3(v, what, fn) {
  if (!Array.isArray(v) || v.length !== 3) __err(fn + ": " + what + " tiene que ser [x, y, z]");
  return [__num(v[0], what + "[0]", fn), __num(v[1], what + "[1]", fn), __num(v[2], what + "[2]", fn)];
}

// camera({ eye, look, fov, center, light }): eye and look are [x, y, z]; `look` appears at
// `center` on the canvas (default: its middle); fov is the horizontal angle in degrees;
// light is the direction the light comes FROM, e.g. [-1, 1.5, -0.6] (left, high, front).
function camera(o) {
  var fn = "camera";
  o = o || {};
  var eye = __vec3(o.eye || [0, 1.6, -20], "eye", fn), look = __vec3(o.look || [0, 1.6, 0], "look", fn);
  var up = [0, 1, 0], f = __norm(__v(look, eye));
  if (Math.abs(f[1]) > 0.999) __err(fn + ": no mires totalmente en vertical");
  var r = __norm(__cross(up, f)), u = __cross(f, r);
  var fov = o.fov === undefined ? 40 : clamp(__num(o.fov, "fov", fn), 5, 150);
  var fpx = (__W / 2) / Math.tan(fov * Math.PI / 360);
  var c = o.center || [__W / 2, __H / 2];
  var light = __norm(o.light ? __vec3(o.light, "light", fn) : [-1, 1.4, -0.7]);
  var cam = { eye: eye, light: light, fpx: fpx, f: f, r: r, u: u, cx: c[0], cy: c[1] };
  cam.p = function (q) {
    var d = __v(q, eye), z = __dot(d, f);
    if (z < 0.01) z = 0.01; // behind the eye: pinned to the near plane
    return [c[0] + fpx * __dot(d, r) / z, c[1] - fpx * __dot(d, u) / z];
  };
  cam.depth = function (q) { return __dot(__v(q, eye), f); };
  cam.dist = function (q) { var d = __v(q, eye); return Math.sqrt(__dot(d, d)); };
  var hz = __norm([f[0], 0, f[2]]);
  cam.horizon = c[1] - fpx * __dot(hz, u) / __dot(hz, f);
  cam.poly = function (pts, s) { return poly(pts.map(cam.p), s); };
  cam.line = function (a, b, s) { var p = cam.p(a), q = cam.p(b); return line(p[0], p[1], q[0], q[1], s); };
  cam.polyline = function (pts, s) { return polyline(pts.map(cam.p), s); };
  cam.curve = function (pts, s) { return curve(pts.map(cam.p), s); };
  // A horizontal circle (projected ellipse) of radius rad around [x, y, z].
  cam.circle = function (ctr, rad, s, n) {
    var pts = [];
    n = n || 48;
    for (var i = 0; i < n; i++) { var a = 2 * Math.PI * i / n; pts.push(cam.p([ctr[0] + rad * Math.cos(a), ctr[1], ctr[2] + rad * Math.sin(a)])); }
    return poly(pts, s);
  };
  // Light on a surface with normal n: from -1 (facing away) to 1 (facing the light).
  cam.lit = function (n) { return __dot(__norm(n), light); };
  return cam;
}

function __rotY(p, deg) {
  var a = deg * Math.PI / 180, c = Math.cos(a), s = Math.sin(a);
  return [p[0] * c - p[2] * s, p[1], p[0] * s + p[2] * c];
}
// A solid's placement: local points -> world (rotated about the vertical through `at`).
function __place(o, fn) {
  var at = __vec3(o.at || [0, 0, 0], "at", fn), rot = o.rotate ? __num(o.rotate, "rotate", fn) : 0;
  return { pt: function (p) { return __add(__rotY(p, rot), at); }, dir: function (v) { return __rotY(v, rot); } };
}
// Default colour of a face: its base colour lit by the camera's light.
function __faceStyle(cam, n, base, given, k) {
  if (given) return given;
  if (!base) return { fill: "#999999" };
  var l = cam.lit(n), amt = l >= 0 ? l * 0.35 : l * 0.55;
  return { fill: shade(base, clamp(amt * (k || 1), -0.8, 0.6)) };
}
// edge: { color, width } adds an ink outline to every face.
function __edged(style, edge) {
  if (!edge) return style;
  var out = {};
  for (var k in style) out[k] = style[k];
  out.stroke = edge.color || "#1a1a1a"; out.width = edge.width || 2; out.join = "round";
  return out;
}
// Draws faces far to near, skipping the ones that look away; returns name -> face helpers.
function __drawFaces(cam, faces, edge) {
  var out = {}, vis = [];
  faces.forEach(function (fc) {
    var ctr = [0, 0, 0];
    fc.pts.forEach(function (q) { ctr = __add(ctr, q); });
    ctr = __mul(ctr, 1 / fc.pts.length);
    fc.visible = __dot(fc.n, __v(cam.eye, ctr)) > 1e-6;
    fc.d = cam.dist(ctr);
    out[fc.name] = __faceApi(cam, fc);
    if (fc.visible) vis.push(fc);
  });
  // walls before roofs, wings after the main block; within each, far to near
  vis.sort(function (a, b) { return (a.layer || 0) - (b.layer || 0) || b.d - a.d; });
  vis.forEach(function (fc) { out[fc.name].shape = cam.poly(fc.pts, __edged(fc.style, edge)); });
  return out;
}
// Helpers to put things on a face: u across it (as seen from outside), v up it, in world units.
function __faceApi(cam, fc) {
  var api = { visible: fc.visible, shape: null, normal: fc.n };
  api.point3 = function (u, v) { return __add(fc.o, __add(__mul(fc.U, u), __mul(fc.V, v))); };
  api.at = function (u, v) { return cam.p(api.point3(u, v)); };
  api.quad = function (u, v, w, h, s) {
    if (!fc.visible && !(s && s.force)) return null;
    var st = {};
    for (var k in (s || {})) if (k !== "force") st[k] = s[k];
    return poly([api.at(u, v), api.at(u + w, v), api.at(u + w, v + h), api.at(u, v + h)], st);
  };
  api.points = function (list) { return list.map(function (q) { return api.at(q[0], q[1]); }); };
  return api;
}
function __face(name, pts, n, o, U, V, style, layer) {
  return { name: name, pts: pts, n: __norm(n), o: o, U: __norm(U), V: __norm(V), style: style, layer: layer || 0 };
}
function __worldFace(P, fc) {
  fc.pts = fc.pts.map(P.pt); fc.n = __norm(P.dir(fc.n)); fc.o = P.pt(fc.o); fc.U = P.dir(fc.U); fc.V = P.dir(fc.V);
  return fc;
}

// box3d(cam, { at, size: [w, h, d], rotate, color, faces: { front, back, left, right, top } }):
// a box whose front-left-bottom corner is `at`. Returns { front, back, left, right, top, bottom }
// with .visible, .shape, .at(u, v) and .quad(u, v, w, h, style) to place doors, windows, signs.
function box3d(cam, o) {
  var fn = "box3d";
  o = o || {};
  var sz = __vec3(o.size || [1, 1, 1], "size", fn), w = sz[0], h = sz[1], d = sz[2], P = __place(o, fn), fs = o.faces || {}, base = o.color;
  var F = [
    __face("front", [[0, 0, 0], [w, 0, 0], [w, h, 0], [0, h, 0]], [0, 0, -1], [0, 0, 0], [1, 0, 0], [0, 1, 0]),
    __face("back", [[w, 0, d], [0, 0, d], [0, h, d], [w, h, d]], [0, 0, 1], [w, 0, d], [-1, 0, 0], [0, 1, 0]),
    __face("left", [[0, 0, d], [0, 0, 0], [0, h, 0], [0, h, d]], [-1, 0, 0], [0, 0, d], [0, 0, -1], [0, 1, 0]),
    __face("right", [[w, 0, 0], [w, 0, d], [w, h, d], [w, h, 0]], [1, 0, 0], [w, 0, 0], [0, 0, 1], [0, 1, 0]),
    __face("top", [[0, h, 0], [w, h, 0], [w, h, d], [0, h, d]], [0, 1, 0], [0, h, 0], [1, 0, 0], [0, 0, 1]),
    __face("bottom", [[0, 0, d], [w, 0, d], [w, 0, 0], [0, 0, 0]], [0, -1, 0], [0, 0, d], [1, 0, 0], [0, 0, -1])
  ].map(function (fc) { __worldFace(P, fc); fc.style = __faceStyle(cam, fc.n, base, fs[fc.name] || o.style); return fc; });
  return __drawFaces(cam, F, o.edge);
}

// house3d(cam, spec): walls and a gabled roof, built like a real one.
//   at: front-left-bottom corner; size: [width, depth]; wall: wall height; roof: rise of the ridge
//   above the walls; overhang: eaves; ridge: "x" (ridge along the width, gables on the left and
//   right ends) or "z" (gable facing the front); rotate: degrees (+ turns the front to face right,
//   showing the left side; - turns it to face left, showing the right side); edge: ink outlines;
//   color: { wall, roof } or styles: { wall, roof, gable }.
//   cross: { u, width, depth, wall, roof, overhang } adds a front wing with its own gable (only with
//   ridge "x"): u is its centre along the front; its roof meets the main one along real valleys.
// Returns faces: front, back, left, right, roofFront, roofBack (or roofLeft, roofRight) and, with a
// cross wing, crossFront, crossLeft, crossRight, crossRoofLeft, crossRoofRight. Each has .at/.quad.
function house3d(cam, o) {
  var fn = "house3d";
  o = o || {};
  var sz = o.size || [8, 6];
  var w = __num(sz[0], "size[0]", fn), d = __num(sz[1], "size[1]", fn), h = o.wall === undefined ? 3 : __num(o.wall, "wall", fn);
  var R = o.roof === undefined ? Math.min(w, d) * 0.35 : __num(o.roof, "roof", fn), ov = o.overhang === undefined ? 0.35 : o.overhang;
  var P = __place(o, fn), col = o.color || {}, st = o.styles || {};
  var wallC = typeof col === "string" ? col : col.wall, roofC = typeof col === "string" ? shade(col, -0.3) : (col.roof || "#7a7f86");
  function ws(n) { return __faceStyle(cam, n, wallC, st.wall); }
  function rs(n) { return __faceStyle(cam, n, roofC, st.roof, 1.2); }
  var F = [], ye, s;
  if (o.ridge === "z") {
    s = R / (w / 2); ye = h - ov * s;
    F.push(__face("front", [[0, 0, 0], [w, 0, 0], [w, h, 0], [w / 2, h + R, 0], [0, h, 0]], [0, 0, -1], [0, 0, 0], [1, 0, 0], [0, 1, 0]));
    F.push(__face("back", [[w, 0, d], [0, 0, d], [0, h, d], [w / 2, h + R, d], [w, h, d]], [0, 0, 1], [w, 0, d], [-1, 0, 0], [0, 1, 0]));
    F.push(__face("left", [[0, 0, d], [0, 0, 0], [0, h, 0], [0, h, d]], [-1, 0, 0], [0, 0, d], [0, 0, -1], [0, 1, 0]));
    F.push(__face("right", [[w, 0, 0], [w, 0, d], [w, h, d], [w, h, 0]], [1, 0, 0], [w, 0, 0], [0, 0, 1], [0, 1, 0]));
    F.forEach(function (fc) { fc.style = ws(fc.n); });
    var nl = [-R, w / 2, 0], nr = [R, w / 2, 0];
    F.push(__face("roofLeft", [[-ov, ye, d + ov], [-ov, ye, -ov], [w / 2, h + R, -ov], [w / 2, h + R, d + ov]], nl, [-ov, ye, -ov], [0, 0, 1], [w / 2 + ov, R + ov * s, 0], rs(nl), 1));
    F.push(__face("roofRight", [[w + ov, ye, -ov], [w + ov, ye, d + ov], [w / 2, h + R, d + ov], [w / 2, h + R, -ov]], nr, [w + ov, ye, -ov], [0, 0, 1], [-(w / 2 + ov), R + ov * s, 0], rs(nr), 1));
  } else {
    s = R / (d / 2); ye = h - ov * s;
    F.push(__face("front", [[0, 0, 0], [w, 0, 0], [w, h, 0], [0, h, 0]], [0, 0, -1], [0, 0, 0], [1, 0, 0], [0, 1, 0]));
    F.push(__face("back", [[w, 0, d], [0, 0, d], [0, h, d], [w, h, d]], [0, 0, 1], [w, 0, d], [-1, 0, 0], [0, 1, 0]));
    F.push(__face("left", [[0, 0, d], [0, 0, 0], [0, h, 0], [0, h + R, d / 2], [0, h, d]], [-1, 0, 0], [0, 0, d], [0, 0, -1], [0, 1, 0]));
    F.push(__face("right", [[w, 0, 0], [w, 0, d], [w, h, d], [w, h + R, d / 2], [w, h, 0]], [1, 0, 0], [w, 0, 0], [0, 0, 1], [0, 1, 0]));
    F.forEach(function (fc) { fc.style = ws(fc.n); });
    var nf = [0, d / 2, -R], nb = [0, d / 2, R];
    F.push(__face("roofFront", [[-ov, ye, -ov], [w + ov, ye, -ov], [w + ov, h + R, d / 2], [-ov, h + R, d / 2]], nf, [-ov, ye, -ov], [1, 0, 0], [0, R + ov * s, d / 2 + ov], rs(nf), 1));
    F.push(__face("roofBack", [[w + ov, ye, d + ov], [-ov, ye, d + ov], [-ov, h + R, d / 2], [w + ov, h + R, d / 2]], nb, [w + ov, ye, d + ov], [-1, 0, 0], [0, R + ov * s, -(d / 2 + ov)], rs(nb), 1));
    if (o.cross) {
      var c = o.cross, cw = __num(c.width || w / 3, "cross.width", fn), cd = __num(c.depth || 2, "cross.depth", fn);
      var cu = c.u === undefined ? w / 2 : __num(c.u, "cross.u", fn), ch = c.wall === undefined ? h : c.wall;
      var cR = c.roof === undefined ? (cw / 2) * s : c.roof, co = c.overhang === undefined ? ov : c.overhang;
      var x0 = cu - cw / 2, x1 = cu + cw / 2, cs = cR / (cw / 2), cye = ch - co * cs;
      var Yr = Math.min(ch + cR, h + R - 0.01);
      var zr = d / 2 - (h + R - Yr) / s, ze = d / 2 - (h + R - cye) / s; // where ridge and eaves meet the main roof
      var F2 = [
        __face("crossFront", [[x0, 0, -cd], [x1, 0, -cd], [x1, ch, -cd], [cu, ch + cR, -cd], [x0, ch, -cd]], [0, 0, -1], [x0, 0, -cd], [1, 0, 0], [0, 1, 0]),
        __face("crossLeft", [[x0, 0, 0], [x0, 0, -cd], [x0, ch, -cd], [x0, ch, 0]], [-1, 0, 0], [x0, 0, 0], [0, 0, -1], [0, 1, 0]),
        __face("crossRight", [[x1, 0, -cd], [x1, 0, 0], [x1, ch, 0], [x1, ch, -cd]], [1, 0, 0], [x1, 0, -cd], [0, 0, 1], [0, 1, 0])
      ];
      F2.forEach(function (fc) { fc.style = ws(fc.n); fc.layer = 2; });
      var ncl = [-cR, cw / 2, 0], ncr = [cR, cw / 2, 0];
      F2.push(__face("crossRoofLeft", [[x0 - co, cye, -cd - co], [cu, Yr, -cd - co], [cu, Yr, zr], [x0 - co, cye, ze]], ncl, [x0 - co, cye, -cd - co], [0, 0, 1], [cw / 2 + co, Yr - cye, 0], rs(ncl), 3));
      F2.push(__face("crossRoofRight", [[cu, Yr, -cd - co], [x1 + co, cye, -cd - co], [x1 + co, cye, ze], [cu, Yr, zr]], ncr, [x1 + co, cye, -cd - co], [0, 0, 1], [-(cw / 2 + co), Yr - cye, 0], rs(ncr), 3));
      F = F.concat(F2);
    }
  }
  // styles once the faces are in the world, so the light falls on them as they really face
  F.forEach(function (fc) {
    __worldFace(P, fc);
    fc.style = /oof/.test(fc.name) ? rs(fc.n) : (fc.pts.length === 5 && st.gable ? st.gable : ws(fc.n));
  });
  return __drawFaces(cam, F, o.edge);
}

function __hull(pts) { // Andrew's monotone chain
  var p = pts.slice().sort(function (a, b) { return a[0] - b[0] || a[1] - b[1]; });
  if (p.length < 3) return p;
  function cr(o, a, b) { return (a[0] - o[0]) * (b[1] - o[1]) - (a[1] - o[1]) * (b[0] - o[0]); }
  var lo = [], hi = [];
  p.forEach(function (q) { while (lo.length >= 2 && cr(lo[lo.length - 2], lo[lo.length - 1], q) <= 0) lo.pop(); lo.push(q); });
  for (var i = p.length - 1; i >= 0; i--) { var q = p[i]; while (hi.length >= 2 && cr(hi[hi.length - 2], hi[hi.length - 1], q) <= 0) hi.pop(); hi.push(q); }
  return lo.slice(0, -1).concat(hi.slice(0, -1));
}
// cylinder3d(cam, { at, r, h, rTop, color, style: { body, top } }): a vertical cylinder (or a
// cone or frustum with rTop) standing on `at` = the centre of its base: towers, lighthouses,
// chimneys, cups, trunks. The body is shaded across by the scene light. Returns:
//   body, top (shapes); at(angle, v): a point on the surface, angle 0 facing the viewer, + to the
//   right; visible(angle); ring(v): the visible front arc at height v (bands, railings, gallery
//   edges); quad(angle, v, w, h, style): a window or door on the surface; disk(v, radius, style).
function cylinder3d(cam, o) {
  var fn = "cylinder3d";
  o = o || {};
  var at = __vec3(o.at || [0, 0, 0], "at", fn), r = __num(o.r === undefined ? 1 : o.r, "r", fn), h = __num(o.h === undefined ? 3 : o.h, "h", fn);
  var rt = o.rTop === undefined ? r : __num(o.rTop, "rTop", fn), n = o.segments || 64, st = o.style || {}, base = o.color || "#999999";
  // angle 0 faces the viewer; positive angles go to the right of the screen
  var toEye = __norm([cam.eye[0] - at[0], 0, cam.eye[2] - at[2]]), side = __cross(toEye, [0, 1, 0]);
  function rad(v) { return lerp(r, rt, clamp(v / h, 0, 1)); }
  function p3(ang, v) {
    var a = ang * Math.PI / 180, dir = __add(__mul(toEye, Math.cos(a)), __mul(side, Math.sin(a)));
    return __add(at, __add(__mul(dir, rad(v)), [0, v, 0]));
  }
  var all = [];
  for (var i = 0; i < n; i++) { var a = 360 * i / n; all.push(cam.p(p3(a, 0))); all.push(cam.p(p3(a, h))); }
  var hull = __hull(all);
  // shading across the body, from its left edge to its right edge as seen
  var xs = hull.map(function (q) { return q[0]; }), xl = Math.min.apply(null, xs), xr = Math.max.apply(null, xs);
  var stops = [];
  for (var k = 0; k <= 8; k++) {
    var ang = -90 + 180 * k / 8, nrm = __add(__mul(toEye, Math.cos(ang * Math.PI / 180)), __mul(side, Math.sin(ang * Math.PI / 180)));
    var l = cam.lit(nrm), amt = l >= 0 ? l * 0.4 : l * 0.6;
    stops.push([k / 8, shade(base, clamp(amt - 0.05 * Math.abs(Math.cos(ang * Math.PI / 180) - 1), -0.8, 0.6))]);
  }
  var ymid = hull.reduce(function (acc, q) { return acc + q[1]; }, 0) / hull.length;
  var bodyStyle = __edged(st.body || { fill: linear(xl, ymid, xr, ymid, stops) }, o.edge);
  var out = { body: poly(hull, bodyStyle), top: null };
  out.at = function (ang, v) { return cam.p(p3(ang, v)); };
  out.visible = function (ang) {
    var q = p3(ang, h / 2), nrm = __norm([q[0] - at[0], 0, q[2] - at[2]]);
    return __dot(nrm, __v(cam.eye, q)) > 0;
  };
  out.ring = function (v, from, to) {
    var pts = [];
    from = from === undefined ? -90 : from; to = to === undefined ? 90 : to;
    for (var a2 = from; a2 <= to + 1e-6; a2 += 4) if (out.visible(a2)) pts.push(out.at(a2, v));
    return pts;
  };
  out.quad = function (ang, v, w, hh, s) {
    var half = (w / 2) / Math.max(rad(v), 0.01) * 180 / Math.PI;
    if (!out.visible(ang)) return null;
    return poly([out.at(ang - half, v), out.at(ang + half, v), out.at(ang + half, v + hh), out.at(ang - half, v + hh)], s);
  };
  out.disk = function (v, radius, s) { return cam.circle([at[0], at[1] + v, at[2]], radius, s); };
  if (cam.eye[1] > at[1] + h && rt > 0.001) out.top = cam.circle([at[0], at[1] + h, at[2]], rt, __edged(st.top || { fill: shade(base, 0.25) }, o.edge));
  return out;
}

/* ---------------- tracing the reference ---------------- */

function __colorList(v, fn) {
  if (v === undefined || v === null) return [];
  var list = Array.isArray(v) ? v : (typeof v === "object" ? Object.keys(v).map(function (k) { return v[k]; }) : [v]);
  return list.filter(function (c) { return typeof c === "string" && /^#[0-9a-f]{6}/i.test(c); }).map(function (c) { return c.slice(0, 7).toLowerCase(); });
}
// trace({ colors, detail, corners, noise, drop, tolerance, palette, mode, threshold, ink, ref }):
// the reference image traced into vector paths on this canvas (same coordinates as pxcli vec ref).
// mode "color" (default) gives flat colour regions with the exact shapes of the reference; mode
// "lines" keeps only the dark ink, painted with `ink`. drop removes colours (e.g. a noisy sky you
// paint yourself), palette snaps every colour to the nearest of yours. Draw on top to finish it.
function trace(o) {
  var fn = "trace";
  o = o || {};
  // LumeArt: el calco no necesita VectorCraft si el motor trae el suyo (__hasTrace).
  if (!__hasVC && !__hasTrace) __err(fn + ": hace falta VectorCraft (vectorcraft-cli) instalado");
  var job = {
    w: __W, h: __H, ref: o.ref || "", mode: o.mode === "lines" ? "lines" : "color",
    colors: clamp(Math.round(o.colors || 16), 2, 64), paths: clamp(o.detail === undefined ? 60 : o.detail, 0, 100),
    corners: clamp(o.corners === undefined ? 50 : o.corners, 0, 100), noise: Math.max(0, o.noise === undefined ? 6 : o.noise),
    threshold: clamp(Math.round(o.threshold === undefined ? 110 : o.threshold), 0, 255),
    drop: __colorList(o.drop, fn), tolerance: o.tolerance || 40, palette: __colorList(o.palette, fn),
    ink: __colorList(o.ink || "#1b1b1b", fn)[0] || "#1b1b1b"
  };
  var node = __where({ tag: "g", attrs: {}, leaf: true, text: __trace(JSON.stringify(job)), clipD: null });
  __common(o, node.attrs, fn);
  node.parent = __top(); __top().children.push(node); __count++;
  return node;
}

/* ---------------- finishing: ink, cel shading, textures ---------------- */

var __light2 = __norm2([-0.6, -1]);
function __norm2(v) { var l = Math.hypot(v[0], v[1]) || 1; return [v[0] / l, v[1] / l]; }
// setLight(x, y): where the light comes FROM, as a direction on the canvas ([-1, -1] = top left).
function setLight(x, y) { __light2 = __norm2([__num(x, "x", "setLight"), __num(y, "y", "setLight")]); }
function __lightOf(o) { return o && o.light ? __norm2(o.light) : __light2; }
function __signedArea2(p) { var a = 0; for (var i = 0; i < p.length; i++) { var q = p[(i + 1) % p.length]; a += p[i][0] * q[1] - q[0] * p[i][1]; } return a / 2; }
function __fillOf(shape, o, fn) {
  var c = o && (o.base || o.fill);
  if (!c && __isShape(shape) && /^#[0-9a-f]{6}/i.test(shape.attrs.fill || "")) c = shape.attrs.fill;
  if (!c) __err(fn + ": no sé el color de la forma: pasa base: \"#rrggbb\"");
  return c.slice(0, 7);
}

// ink(shape, { width, weight, light, color, breaks }): an ink contour whose weight follows the
// light: thin on the lit side, up to `weight` times thicker in the shadow. With breaks: true the
// line fades out where the light hits hardest, as comic inkers do.
function ink(shape, o) {
  var fn = "ink";
  o = o || {};
  var pts = __outline(shape, fn);
  if (pts.length < 2) return null;
  if (__isShape(shape) && shape.attrs.transform) __err(fn + ": la forma tiene rotate; entinta una copia sin girar o usa brush");
  var closed = !(__isShape(shape) && (shape.tag === "line" || shape.tag === "polyline" || (shape.tag === "path" && !/Z\s*$/i.test(shape.attrs.d || ""))));
  var L = __lightOf(o), base = o.width || 3, weight = o.weight === undefined ? 2.2 : o.weight;
  var sgn = __signedArea2(pts) > 0 ? 1 : -1, n = pts.length, w = [];
  for (var i = 0; i < n; i++) {
    var a = pts[(i - 1 + n) % n], b = pts[(i + 1) % n];
    if (!closed) { a = pts[Math.max(0, i - 1)]; b = pts[Math.min(n - 1, i + 1)]; }
    var tx = b[0] - a[0], ty = b[1] - a[1], tl = Math.hypot(tx, ty) || 1;
    var nx = sgn * ty / tl, ny = -sgn * tx / tl;
    var d = nx * L[0] + ny * L[1]; // 1: faces the light, -1: away from it
    var k = 1 + (weight - 1) * clamp(-d, 0, 1);
    if (o.breaks && d > 0.55) k *= clamp((0.9 - d) / 0.35, 0, 1);
    w.push(base * k * (0.75 + 0.5 * (d < 0 ? 1 : 1 - d)) / 1.25);
  }
  var st = { color: o.color || "#1b1b1b" };
  ["opacity", "blend"].forEach(function (k) { if (o[k] !== undefined) st[k] = o[k]; });
  var tension = __isShape(shape) && (shape.tag === "rect" || shape.tag === "polygon" || shape.tag === "line" || shape.tag === "polyline") ? 0 : 1;
  var out = [];
  function stroke(idx, taper) {
    if (idx.length < 2) return;
    out.push(brush(idx.map(function (j) { return pts[j]; }), { width: idx.map(function (j) { return Math.max(w[j], 0.01); }), taper: taper, tension: tension, color: st.color, opacity: st.opacity, blend: st.blend }));
  }
  var g = group({}, function () {
    if (!o.breaks) {
      var all = range(n);
      if (closed) all.push(0);
      stroke(all, closed ? 0 : [0.15, 0.15]);
      return;
    }
    // split where the line fades out
    var start = -1;
    for (var s = 0; s < n; s++) if (w[s] < base * 0.15) { start = s; break; }
    if (start < 0) { var all2 = range(n); if (closed) all2.push(0); stroke(all2, 0); return; }
    var run = [];
    for (var t = 1; t <= n; t++) {
      var j = (start + t) % n;
      if (w[j] >= base * 0.15) run.push(j);
      else { if (run.length > 1) stroke(run, [0.2, 0.2]); run = []; }
    }
    if (run.length > 1) stroke(run, [0.2, 0.2]);
  });
  return g;
}

// cel(shape, { light, depth, color, opacity, blend, blur, highlight, hatch }): flat comic shading:
// the part of the shape that turns away from the light gets the shadow colour (shade(base, -0.4)
// unless given), as a crescent `depth` (fraction of the shape's size) wide. highlight: true or a
// colour adds the lit crescent too; hatch: { gap, width, angle, color } draws the shadow as ink lines.
function cel(shape, o) {
  var fn = "cel";
  o = o || {};
  __needShape(shape, fn);
  var L = __lightOf(o), b = bbox(shape), size = Math.max(b.w, b.h), depth = (o.depth === undefined ? 0.18 : o.depth) * size;
  var base = o.hatch ? (o.base || "#000000") : __fillOf(shape, o, fn);
  function shifted(k) {
    var c = __shadowCopy(shape, fn);
    c.attrs.transform = "translate(" + __f(L[0] * depth * k) + " " + __f(L[1] * depth * k) + ")" + (c.attrs.transform ? " " + c.attrs.transform : "");
    return c;
  }
  var out = { shadow: null, highlight: null };
  clip(shape, function () {
    if (o.hatch) {
      var h = o.hatch === true ? {} : o.hatch;
      var copy = __shadowCopy(shape, fn); copy.attrs.fill = "none";
      out.shadow = hatch(copy, { gap: h.gap || 6, width: h.width || 1.2, angle: h.angle === undefined ? 45 : h.angle, color: h.color || "#1b1b1b", wobble: h.wobble, cross: h.cross });
    } else {
      var sc = __shadowCopy(shape, fn);
      sc.attrs.fill = __paint(o.color || shade(base, -0.4), fn);
      __common({ opacity: o.opacity, blend: o.blend, blur: o.blur }, sc.attrs, fn);
      sc.parent = __top(); __top().children.push(sc); __count++;
      out.shadow = sc;
    }
    if (out.shadow) cut(out.shadow, shifted(1));
    if (o.highlight) {
      var hc = __shadowCopy(shape, fn);
      hc.attrs.fill = __paint(typeof o.highlight === "string" ? o.highlight : shade(base, 0.45), fn);
      hc.attrs.opacity = "0.75";
      hc.parent = __top(); __top().children.push(hc); __count++;
      cut(hc, shifted(-0.45));
      out.highlight = hc;
    }
  });
  return out;
}

// bricks(x, y, w, h, { rows, cols, colors, mortar, gap, jitter, bevel }): a brick wall with
// staggered rows, colour variation and slightly irregular edges. Returns its group.
function bricks(x, y, w, h, o) {
  var fn = "bricks";
  o = o || {};
  var rows = Math.max(1, Math.round(o.rows || Math.max(1, h / 24))), bh = h / rows;
  var cols = Math.max(1, Math.round(o.cols || Math.max(1, w / (bh * 2.4)))), bw = w / cols;
  var gap = o.gap === undefined ? Math.max(1.5, bh * 0.12) : o.gap, colors = __colorList(o.colors || ["#b5533c", "#a64a35", "#c0644a"], fn);
  var jit = o.jitter === undefined ? bh * 0.06 : o.jitter;
  return group({}, function () {
    rect(x, y, w, h, { fill: o.mortar || "#d9cbb3" });
    for (var r = 0; r < rows; r++) {
      var off = (r % 2) ? -bw / 2 : 0;
      for (var c = 0; c <= cols; c++) {
        var x0 = Math.max(x, x + off + c * bw + gap / 2), x1 = Math.min(x + w, x + off + (c + 1) * bw - gap / 2);
        if (x1 - x0 < gap * 2) continue;
        var y0 = y + r * bh + gap / 2, y1 = y + (r + 1) * bh - gap / 2;
        var col = mix(pick(colors), pick(colors), rand());
        var pts = [[x0, y0, 0], [x1, y0, 0], [x1, y1, 0], [x0, y1, 0]];
        pts = subdivide(pts, Math.max(6, (x1 - x0) / 4)).map(function (p) { return [p[0] + rand(-jit, jit), p[1] + rand(-jit, jit), p[2]]; });
        var br = poly(pts, { fill: col });
        if (o.bevel !== false) {
          line(x0 + 1, y1 - 0.5, x1 - 1, y1 - 0.5, { stroke: shade(col, -0.45), width: Math.max(1, gap * 0.5), opacity: 0.6 });
          line(x0 + 1, y0 + 0.8, x1 - 1, y0 + 0.8, { stroke: shade(col, 0.4), width: Math.max(0.8, gap * 0.35), opacity: 0.5 });
        }
      }
    }
  });
}

// cracks(region, { count, length, color, width, branch }): branching cracks inside a shape or an
// [x, y, w, h] box (old plaster, stone, dry earth, ice).
function cracks(region, o) {
  var fn = "cracks";
  o = o || {};
  var box = Array.isArray(region) && region.length === 4 && typeof region[0] === "number" ? region : null;
  var b = box ? { x: box[0], y: box[1], w: box[2], h: box[3] } : bbox(region);
  var len = o.length || Math.max(b.w, b.h) * 0.25, col = o.color || "#3a3128", wid = o.width || 1.6;
  function one(x, y, ang, L, wd, depth) {
    var pts = [[x, y]], seg = Math.max(4, L / 7);
    for (var i = 0; i < 7; i++) {
      ang += rand(-0.6, 0.6);
      x += Math.cos(ang) * seg; y += Math.sin(ang) * seg;
      pts.push([x, y]);
      if (depth < 2 && rand() < (o.branch === undefined ? 0.25 : o.branch)) one(x, y, ang + pick([-1, 1]) * rand(0.5, 1.1), L * 0.5, wd * 0.6, depth + 1);
    }
    brush(pts, { width: wd, taper: [0.1, 0.9], color: col, opacity: o.opacity === undefined ? 0.85 : o.opacity });
  }
  function draw() { for (var k = 0; k < (o.count || 5); k++) one(rand(b.x, b.x + b.w), rand(b.y, b.y + b.h), rand(0, Math.PI * 2), len, wid, 0); }
  return box ? clip(rect(b.x, b.y, b.w, b.h, { hidden: true }), draw) : clip(region, draw);
}

// stipple(shape, { count, size, color, side, light }): ink dots inside a shape; side: "shadow"
// puts them where the shape turns away from the light (texture and tone without hatching).
function stipple(shape, o) {
  var fn = "stipple";
  o = o || {};
  var poly2 = __outline(shape, fn), b = bbox(shape), L = __lightOf(o), n = Math.min(o.count || 300, 4000), r = o.size || 1.2;
  function inside(x, y) {
    var c = false;
    for (var i = 0, j = poly2.length - 1; i < poly2.length; j = i++) {
      var xi = poly2[i][0], yi = poly2[i][1], xj = poly2[j][0], yj = poly2[j][1];
      if ((yi > y) !== (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi) c = !c;
    }
    return c;
  }
  var d = "", placed = 0, tries = 0;
  while (placed < n && tries < n * 20) {
    tries++;
    var x = rand(b.x, b.x + b.w), y = rand(b.y, b.y + b.h);
    if (!inside(x, y)) continue;
    if (o.side === "shadow") {
      var t = ((x - b.cx) * -L[0] + (y - b.cy) * -L[1]) / (Math.max(b.w, b.h) / 2); // 1 at the far side
      if (rand() > clamp((t + 0.3) / 1.3, 0, 1)) continue;
    }
    var rr = r * rand(0.6, 1.3);
    d += "M" + __f(x - rr) + " " + __f(y) + "a" + __f(rr) + " " + __f(rr) + " 0 1 0 " + __f(2 * rr) + " 0a" + __f(rr) + " " + __f(rr) + " 0 1 0 " + __f(-2 * rr) + " 0";
    placed++;
  }
  if (!d) return null;
  return path(d, { fill: o.color || "#1b1b1b", opacity: o.opacity });
}

/* ---------------- rig: bones, parts and actions, for skeletal animation in Godot ---------------- */
// The character is drawn in its rest pose. bone() declares the skeleton (each bone turns around
// its joint `at`), part() draws the pieces that ride on a bone, back to front, and action()
// gives the poses of each animation. pxcli vec godot exports pieces + Skeleton2D + AnimationPlayer.

var __bones = [], __boneMap = {}, __parts = [], __actions = [], __rigOrigin = null;
function __xy(v, what, fn) {
  if (!Array.isArray(v) || v.length < 2) __err(fn + ": " + what + " tiene que ser [x, y]");
  return [__num(v[0], what + "[0]", fn), __num(v[1], what + "[1]", fn)];
}
// rig({ origin: [x, y] }): the point that stays on the ground (between the feet); in Godot the
// character's origin. Default: the bottom centre of all its parts.
function rig(o) {
  o = o || {};
  if (o.origin) __rigOrigin = __xy(o.origin, "origin", "rig");
}
// bone(name, { parent, at: [x, y] }): a bone whose joint is `at`, in rest-pose coordinates.
function bone(name, o) {
  var fn = "bone";
  if (typeof name !== "string" || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) __err(fn + ": el nombre del hueso tiene que ser una palabra sin espacios ni acentos (brazo_der, cabeza…)");
  if (__boneMap[name]) __err(fn + ": el hueso " + name + " ya existe");
  o = o || {};
  if (o.parent && !__boneMap[o.parent]) __err(fn + ": el padre " + o.parent + " no existe: declara antes el hueso padre");
  var b = { name: name, parent: o.parent || "", at: __xy(o.at || [0, 0], "at", fn), line: __line(), children: 0 };
  if (o.parent) __boneMap[o.parent].children++;
  __bones.push(b); __boneMap[name] = b;
  return b;
}
// part(bone, () => { ... }): shapes that move with that bone. Parts stack in call order, so the
// far arm can go behind the torso and the near one in front.
function part(boneName, fn) {
  var f = "part";
  if (!__boneMap[boneName]) __err(f + ": el hueso " + boneName + " no existe (decláralo con bone)");
  var n = 0;
  __parts.forEach(function (p) { if (p.bone === boneName) n++; });
  var g = __group({ "data-bone": boneName, "data-part": boneName + (n ? "_" + (n + 1) : "") }, fn, f);
  __parts.push({ bone: boneName, name: g.attrs["data-part"], node: g, line: g.line });
  return g;
}
// A pose value: degrees, or { rot, x, y, scale, alpha } (x, y move the bone in its parent's
// space; scale, a number or [sx, sy], grows or squashes it around its joint; alpha, from 0 to 1,
// fades it; both carry on to its children). sx and sy alone also work.
function __poseVal(v, bn, fn) {
  if (typeof v === "number") return { rot: __num(v, bn, fn), x: 0, y: 0 };
  if (v && typeof v === "object") {
    var out = { rot: v.rot || 0, x: v.x || 0, y: v.y || 0 };
    var sx = v.sx, sy = v.sy;
    if (v.scale !== undefined) {
      if (Array.isArray(v.scale)) { sx = v.scale[0]; sy = v.scale[1]; }
      else { sx = v.scale; sy = v.scale; }
    }
    if (sx !== undefined && sx !== 1) out.sx = __num(sx, bn + ".scale", fn);
    if (sy !== undefined && sy !== 1) out.sy = __num(sy, bn + ".scale", fn);
    if (v.alpha !== undefined && v.alpha !== 1) out.alpha = clamp(__num(v.alpha, bn + ".alpha", fn), 0, 1);
    return out;
  }
  __err(fn + ": la pose de " + bn + " tiene que ser grados o { rot, x, y, scale, alpha }");
}
// action(name, { duration, loop, move }, [[t, pose], ...]): t from 0 to 1 along the action; a
// pose is { bone: degrees or { rot, x, y, scale, alpha } }. Bones a key leaves out are interpolated from the
// keys around it; a loop's last key goes back to its first. move is how fast the character
// advances while it plays (units per second, so the feet do not slide); the game reads it.
function action(name, o, keys) {
  var fn = "action";
  if (typeof name !== "string" || !/^[A-Za-z_][A-Za-z0-9_]*$/.test(name)) __err(fn + ": el nombre de la acción tiene que ser una palabra sin espacios ni acentos");
  o = o || {};
  if (!Array.isArray(keys) || !keys.length) __err(fn + ": hacen falta poses: [[0, { ... }], [0.5, { ... }]]");
  var ks = keys.map(function (k, i) {
    if (!Array.isArray(k) || k.length < 2) __err(fn + ": la pose " + i + " tiene que ser [t, { hueso: grados }]");
    var t = clamp(__num(k[0], "t", fn), 0, 1), pose = {};
    for (var bn in k[1]) {
      if (!__boneMap[bn]) __err(fn + ": la pose usa el hueso " + bn + ", que no existe");
      pose[bn] = __poseVal(k[1][bn], bn, fn);
    }
    return { t: t, pose: pose };
  }).sort(function (a, b) { return a.t - b.t; });
  var duration = __num(o.duration === undefined ? 1 : o.duration, "duration", fn);
  if (!(duration > 0)) __err(fn + ": duration son los segundos que dura la acción, mayor que 0");
  if (__actions.some(function (a) { return a.name === name; })) __err(fn + ": la acción " + name + " ya existe");
  var move = o.move === undefined ? 0 : __num(o.move, "move", fn);
  __actions.push({ name: name, duration: duration, loop: o.loop !== false, move: move, keys: ks, line: __line() });
}
// swapSides(pose): the same pose with left and right bones swapped (_izq <-> _der), for the
// second half of a walk or run cycle.
function swapSides(pose, a, b) {
  a = a || "_izq"; b = b || "_der";
  var out = {};
  for (var k in pose) {
    var j = k.slice(-a.length) === a ? k.slice(0, -a.length) + b : k.slice(-b.length) === b ? k.slice(0, -b.length) + a : k;
    out[j] = pose[k];
  }
  return out;
}

function __leafBox(n) {
  try {
    var p = __outline(n, "rig");
    var b = __bboxNums([].concat.apply([], p)), pad = 3;
    var sw = parseFloat(n.attrs["stroke-width"]) || 0;
    pad += sw / 2;
    return [b[0] - pad, b[1] - pad, b[2] + 2 * pad, b[3] + 2 * pad];
  } catch (e) { return null; }
}
function __partBox(node) {
  var box = null, unknown = false;
  (function walk(n, blur) {
    var bl = blur;
    if (n.attrs && n.attrs.filter) bl += 24; // blur, glow or shadow spill over the outline
    if (n.leaf) {
      if (n.tag === "g" && !n.clipD) { unknown = true; return; }
      var b = __leafBox(n);
      if (!b) { unknown = true; return; }
      b = [b[0] - bl, b[1] - bl, b[2] + 2 * bl, b[3] + 2 * bl];
      if (!box) box = b;
      else { var x0 = Math.min(box[0], b[0]), y0 = Math.min(box[1], b[1]); box = [x0, y0, Math.max(box[0] + box[2], b[0] + b[2]) - x0, Math.max(box[1] + box[3], b[1] + b[3]) - y0]; }
      return;
    }
    (n.children || []).forEach(function (c) { walk(c, bl); });
  })(node, 0);
  if (unknown || !box) return [0, 0, __W, __H];
  return box;
}
function __serialize(n) {
  if (n.leaf) return "<" + n.tag + __attrs(n.attrs) + (n.text != null ? ">" + n.text + "</" + n.tag + ">" : "/>");
  var s = "<g" + __attrs(n.attrs) + ">";
  for (var i = 0; i < n.children.length; i++) s += __serialize(n.children[i]);
  return s + "</g>";
}
// What pxcli needs to preview and export the rig.
function __rigInfo() {
  if (!__bones.length) return "";
  var parts = __parts.map(function (p, i) {
    var inner = "";
    for (var k = 0; k < p.node.children.length; k++) inner += __serialize(p.node.children[k]);
    return { name: p.name, bone: p.bone, z: i, box: __partBox(p.node), markup: inner };
  });
  var origin = __rigOrigin;
  if (!origin) {
    var x0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    parts.forEach(function (p) { x0 = Math.min(x0, p.box[0]); x1 = Math.max(x1, p.box[0] + p.box[2]); y1 = Math.max(y1, p.box[1] + p.box[3]); });
    origin = parts.length ? [(x0 + x1) / 2, y1] : [__W / 2, __H];
  }
  return JSON.stringify({
    bones: __bones.map(function (b) { return { name: b.name, parent: b.parent, at: b.at, line: b.line }; }),
    actions: __actions, origin: origin, width: __W, height: __H,
    defs: __defOrder.map(function (id) { return __defs[id]; }).join(""), parts: parts
  });
}

// What pxcli needs to export a scene to a game: its layers in order, each split into pieces that
// keep the stacking order (the layer's own shapes between elements, and every element on its own),
// each piece with the groups around it (clips, moves) so it draws exactly as in the drawing.
function __sceneInfo() {
  function plainAttrs(a) { var r = {}; for (var k in a) if (k.indexOf("data-") !== 0) r[k] = a[k]; return r; }
  function hasElement(n) { return !n.leaf && (!!n.element || n.children.some(hasElement)); }
  var layers = [];
  function split(children, L, wrap) {
    var buf = "";
    function flush() { if (buf) { L.pieces.push({ markup: wrap(buf) }); buf = ""; } }
    children.forEach(function (c) {
      if (c.element) {
        flush();
        var inner = "";
        for (var i = 0; i < c.children.length; i++) inner += __serialize(c.children[i]);
        L.pieces.push({ name: c.element, element: true, pivot: c.pivot || null, parallax: c.parallax || null, line: c.line,
          markup: wrap("<g" + __attrs(plainAttrs(c.attrs)) + ">" + inner + "</g>") });
      } else if (hasElement(c)) {
        flush();
        var open = "<g" + __attrs(plainAttrs(c.attrs)) + ">";
        split(c.children, L, function (m) { return wrap(open + m + "</g>"); });
      } else buf += __serialize(c);
    });
    flush();
  }
  var loose = [];
  function flushLoose() {
    if (!loose.length) return;
    var L = { name: layers.length ? "formas sueltas" : "lienzo", parallax: null, line: loose[0].line || 0, pieces: [] };
    split(loose, L, function (m) { return m; });
    layers.push(L);
    loose = [];
  }
  __root.children.forEach(function (c) {
    if (!c.leaf && !c.element && c.attrs["data-name"]) {
      flushLoose();
      var L = { name: c.attrs["data-name"], parallax: c.parallax || null, line: c.line, pieces: [] };
      var open = "<g" + __attrs(plainAttrs(c.attrs)) + ">";
      split(c.children, L, function (m) { return open + m + "</g>"; });
      layers.push(L);
    } else loose.push(c);
  });
  flushLoose();
  var bg = null;
  if (__bg && __bg !== "none") bg = { color: /^#[0-9a-fA-F]{6}$/.test(__bg) ? __bg : "", markup: '<rect width="' + __W + '" height="' + __H + '" fill="' + __escA(__bg) + '"/>' };
  return JSON.stringify({ width: __W, height: __H, defs: __defOrder.map(function (id) { return __defs[id]; }).join(""), background: bg, layers: layers });
}

/* ---------------- canvas and output ---------------- */

function canvas(o) {
  o = o || {};
  if (o.width !== undefined) __W = Math.round(clamp(__num(o.width, "width", "canvas"), 16, 4096));
  if (o.height !== undefined) __H = Math.round(clamp(__num(o.height, "height", "canvas"), 16, 4096));
  if (o.background !== undefined) { __bg = o.background === null ? null : __paint(o.background, "canvas"); __bgLine = __line(); }
}

function __attrs(a) {
  var s = "";
  for (var k in a) s += " " + k + '="' + __escA(a[k]) + '"';
  return s;
}
// __portable(a): the attributes with every #rrggbbaa (or #rgba) colour split into #rrggbb plus its
// opacity attribute, because Illustrator and Inkscape read 8-digit colours as black.
var __OPACITY_OF = { fill: "fill-opacity", stroke: "stroke-opacity", "stop-color": "stop-opacity", "flood-color": "flood-opacity" };
function __portable(a) {
  var r = {}, k;
  for (k in a) r[k] = a[k];
  for (k in __OPACITY_OF) {
    var v = r[k];
    if (typeof v !== "string") continue;
    if (/^#[0-9a-f]{4}$/i.test(v)) v = "#" + v[1] + v[1] + v[2] + v[2] + v[3] + v[3] + v[4] + v[4];
    var m = /^#([0-9a-f]{6})([0-9a-f]{2})$/i.exec(v);
    if (!m) continue;
    var op = __OPACITY_OF[k], o = parseInt(m[2], 16) / 255 * (r[op] !== undefined ? Number(r[op]) : 1);
    r[k] = "#" + m[1];
    if (o < 1) r[op] = __f3(o); else delete r[op];
  }
  return r;
}
// __render(true) adds data-i (order), data-l (line in dibujo.js) and data-h
// (a hash of the shape and its groups) to every shape, for the studio.
// __render(false) is the file to keep and share: every layer() becomes a
// top-level group with an id (VectorCraft opens them as layers; Illustrator opens
// them as named groups in a single layer), shapes outside any layer go to a
// "Lienzo" layer, every shape is named after the code that drew it (its variable,
// or its function and line), and colours are written so other editors read them.
function __render(withData) {
  var out = [], idx = 0, used = {};
  __defOrder.forEach(function (id) { used[id] = true; });
  function plain(s) {
    var map = { "á": "a", "é": "e", "í": "i", "ó": "o", "ú": "u", "ü": "u", "ñ": "n", "Á": "A", "É": "E", "Í": "I", "Ó": "O", "Ú": "U", "Ñ": "N" };
    return String(s).replace(/[áéíóúüñÁÉÍÓÚÑ]/g, function (c) { return map[c]; });
  }
  function idFor(label) {
    var a = plain(label).replace(/[^A-Za-z0-9_-]+/g, "_").replace(/^_+|_+$/g, "");
    if (!/^[A-Za-z_]/.test(a)) a = "n_" + a;
    var k = a, n = 2;
    while (used[k]) k = a + "_" + (n++);
    used[k] = true;
    return k;
  }
  var KW = { "for": 1, "if": 1, "while": 1, "switch": 1, "function": 1, "return": 1, "catch": 1 };
  function nameOf(tag, line) {
    var src = typeof __src !== "undefined" && line ? (__src[line - 1] || "") : "";
    var m = /\b(?:const|let|var)\s+([A-Za-z_$][\w$]*)\s*=\s*[A-Za-z_$][\w$.]*\s*\(/.exec(src);
    if (m) return m[1];
    var re = /\b([A-Za-z_$][\w$]*)\s*\(/g, f;
    while ((f = re.exec(src))) if (!KW[f[1]]) return f[1] + " línea " + line;
    return tag + (line ? " línea " + line : "");
  }
  function named(label) { return ' id="' + idFor(label) + '" data-name="' + __escA(label) + '"'; }
  function leaf(tag, a, text, line, ctx, chain) {
    if (!withData) a = __portable(a);
    var core = "<" + tag + __attrs(a) + (text != null ? ">" + text + "</" + tag + ">" : "/>");
    if (withData) core = "<" + tag + ' data-i="' + idx + '" data-l="' + (line || 0) + '"' + (chain ? ' data-c="' + chain + '"' : "") +
      ' data-h="' + __hash(ctx + core) + '"' + core.slice(tag.length + 1);
    else core = "<" + tag + named(nameOf(tag, line)) + core.slice(tag.length + 1);
    idx++;
    out.push(core);
  }
  function walk(n, ctx) {
    if (n.leaf) return leaf(n.tag, n.attrs, n.text, n.line, ctx, n.chain);
    var a = n.attrs, open;
    if (!withData && (a["data-name"] || a["data-part"] || a["data-element"])) {
      var rest = {};
      for (var k in a) if (k !== "data-name" && k !== "data-part" && k !== "data-bone" && k !== "data-element") rest[k] = a[k];
      open = "<g" + named(a["data-name"] || a["data-part"] || a["data-element"]) + __attrs(__portable(rest)) + ">";
    } else if (!withData) open = "<g" + named(nameOf("grupo", n.line)) + __attrs(__portable(a)) + ">";
    else open = "<g" + __attrs(a) + ">";
    out.push(open);
    for (var i = 0; i < n.children.length; i++) walk(n.children[i], ctx + open);
    out.push("</g>");
  }
  var loose = 0, wrapped = false;
  function openLoose() { if (!withData && !wrapped) { out.push("<g" + named(loose++ ? "Formas sueltas" : "Lienzo") + ">"); wrapped = true; } }
  function closeLoose() { if (wrapped) { out.push("</g>"); wrapped = false; } }
  if (__bg && __bg !== "none") { openLoose(); leaf("rect", { width: __W, height: __H, fill: __bg }, null, __bgLine, ""); }
  for (var i = 0; i < __root.children.length; i++) {
    var c = __root.children[i];
    if (!withData && !c.leaf && (c.attrs["data-name"] || c.attrs["data-part"] || c.attrs["data-element"])) closeLoose(); else openLoose();
    walk(c, "");
  }
  closeLoose();
  var defs = __defOrder.map(function (id) { return __defs[id]; }).join("");
  return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ' + __W + " " + __H + '" width="' + __W + '" height="' + __H + '">' +
    (defs ? "<defs>" + defs + "</defs>" : "") + out.join("") + "</svg>";
}
