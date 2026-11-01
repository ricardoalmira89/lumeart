// Ejecuta los pasos de cada caso (`steps.json`) con la CLI de animación de
// pxcli sobre una copia y escribe `pxcli_cli.json`: salida, error y archivos
// nuevos o cambiados (PNG: tamaño y hash FNV-1a de sus píxeles NRGBA; el
// resto, su texto).
package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"hash/fnv"
	"image"
	"image/color"
	"image/png"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"pxcli/internal/cli"
)

type step struct {
	Args   []string          `json:"args"`
	Stdout string            `json:"stdout"`
	Error  string            `json:"error"`
	Files  map[string]string `json:"files"`
}

func signature(path string) string {
	data, err := os.ReadFile(path)
	if err != nil {
		return "?"
	}
	if strings.HasSuffix(path, ".png") {
		img, err := png.Decode(bytes.NewReader(data))
		if err != nil {
			return "png roto"
		}
		b := img.Bounds()
		h := fnv.New64a()
		for y := b.Min.Y; y < b.Max.Y; y++ {
			for x := b.Min.X; x < b.Max.X; x++ {
				c := color.NRGBAModel.Convert(img.At(x, y)).(color.NRGBA)
				_, _ = h.Write([]byte{c.R, c.G, c.B, c.A})
			}
		}
		return fmt.Sprintf("png %dx%d %016x", b.Dx(), b.Dy(), h.Sum64())
	}
	return string(data)
}

func snapshot(dir string) map[string]string {
	out := map[string]string{}
	_ = filepath.WalkDir(dir, func(p string, d fs.DirEntry, err error) error {
		if err == nil && !d.IsDir() {
			rel, _ := filepath.Rel(dir, p)
			out[rel] = signature(p)
		}
		return nil
	})
	return out
}

func copyDir(src, dst string) {
	_ = filepath.WalkDir(src, func(p string, d fs.DirEntry, err error) error {
		rel, _ := filepath.Rel(src, p)
		if d.IsDir() {
			return os.MkdirAll(filepath.Join(dst, rel), 0o755)
		}
		if rel == "steps.json" || rel == "pxcli_cli.json" {
			return nil
		}
		data, _ := os.ReadFile(p)
		return os.WriteFile(filepath.Join(dst, rel), data, 0o644)
	})
}

var _ image.Image

func main() {
	for _, caseDir := range os.Args[1:] {
		abs, _ := filepath.Abs(caseDir)
		var plan struct {
			Steps [][]string `json:"steps"`
		}
		data, err := os.ReadFile(filepath.Join(abs, "steps.json"))
		if err != nil {
			panic(err)
		}
		if err := json.Unmarshal(data, &plan); err != nil {
			panic(err)
		}
		tmp, _ := os.MkdirTemp("", "animcli")
		copyDir(abs, tmp)
		if err := os.Chdir(tmp); err != nil {
			panic(err)
		}
		var steps []step
		before := snapshot(tmp)
		for _, args := range plan.Steps {
			cmd := cli.NewAnimCmd()
			var out, errb bytes.Buffer
			cmd.SetOut(&out)
			cmd.SetErr(&errb)
			cmd.SetArgs(args)
			s := step{Args: args, Files: map[string]string{}}
			if err := cmd.Execute(); err != nil {
				s.Error = err.Error()
			}
			s.Stdout = out.String()
			after := snapshot(tmp)
			for k, v := range after {
				if before[k] != v {
					s.Files[k] = v
				}
			}
			before = after
			steps = append(steps, s)
		}
		b, _ := json.MarshalIndent(map[string]any{"steps": steps}, "", "  ")
		_ = os.WriteFile(filepath.Join(abs, "pxcli_cli.json"), append(b, '\n'), 0o644)
		_ = os.RemoveAll(tmp)
		fmt.Println(filepath.Base(abs), len(steps), "pasos")
	}
}
