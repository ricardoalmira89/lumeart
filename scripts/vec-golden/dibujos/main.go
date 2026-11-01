// Ejecuta cada dibujo.js con el motor de pxcli (goja + lib.js) y escribe su
// pxcli.json: tamaño, formas, avisos y un hash FNV-1a de 64 bits (y la
// longitud) del SVG del studio y del SVG limpio. Con FULL=1 guarda también
// los dos SVG enteros (svg.svg y clean.svg) para comparar a mano.
package main

import (
	"encoding/json"
	"fmt"
	"hash/fnv"
	"os"
	"path/filepath"

	"pxcli/internal/vec"
)

type out struct {
	Width     int         `json:"width"`
	Height    int         `json:"height"`
	Count     int         `json:"count"`
	Issues    []vec.Issue `json:"issues"`
	SVGHash   string      `json:"svg_hash"`
	SVGLen    int         `json:"svg_len"`
	CleanHash string      `json:"clean_hash"`
	CleanLen  int         `json:"clean_len"`
}

func hash(s string) string {
	h := fnv.New64a()
	_, _ = h.Write([]byte(s))
	return fmt.Sprintf("%016x", h.Sum64())
}

func main() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "uso: vecdibujos <salida> <carpeta con dibujo.js>...")
		os.Exit(2)
	}
	dest := os.Args[1]
	for _, dir := range os.Args[2:] {
		src, err := os.ReadFile(filepath.Join(dir, vec.FileName))
		check(err)
		res := vec.RenderIn(string(src), dir)
		o := out{
			Width: res.Width, Height: res.Height, Count: res.Count, Issues: res.Issues,
			SVGHash: hash(res.SVG), SVGLen: len(res.SVG),
			CleanHash: hash(res.Clean), CleanLen: len(res.Clean),
		}
		if o.Issues == nil {
			o.Issues = []vec.Issue{}
		}
		target := filepath.Join(dest, filepath.Base(dir))
		check(os.MkdirAll(target, 0o755))
		js, err := json.MarshalIndent(o, "", "  ")
		check(err)
		check(os.WriteFile(filepath.Join(target, "pxcli.json"), append(js, '\n'), 0o644))
		if os.Getenv("FULL") != "" {
			check(os.WriteFile(filepath.Join(target, "svg.svg"), []byte(res.SVG), 0o644))
			check(os.WriteFile(filepath.Join(target, "clean.svg"), []byte(res.Clean), 0o644))
		}
		fmt.Println(filepath.Base(dir), len(res.Issues))
	}
}

func check(err error) {
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
