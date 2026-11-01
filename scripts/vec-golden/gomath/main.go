// Valores de referencia de Math en goja (el motor de pxcli) para comprobar
// crates/lume-script/src/gomath.rs: cada función con entradas especiales y
// muchas pseudoaleatorias en varios rangos, en bits. Con CONST=1 imprime
// los bits de las constantes que usan las funciones portadas.
package main

import (
	"encoding/json"
	"fmt"
	"math"
	"os"
	"strconv"

	"github.com/dop251/goja"
)

func bits(v float64) string { return fmt.Sprintf("%016x", math.Float64bits(v)) }

func main() {
	if os.Getenv("CONST") != "" {
		for _, s := range []string{
			"0.6931471805599453094172321214581766", "1.4426950408889634073599246810018920",
			"0.69314718055966295651160180568695068359375", "0.28235290563031577122588448175013436025525412068e-12",
			"7.09782712893384e+02", "1.6666666666666666667e-1", "4.1666666666666666667e-2",
			"8.3333333333333333333e-3", "1.3888888888888888889e-3", "1.9841269841269841270e-4",
			"2.4801587301587301587e-5",
			"7.07106781186547524401e-01", "6.93147180369123816490e-01", "1.90821492927058770002e-10",
			"6.666666666666735130e-01", "3.999999999940941908e-01", "2.857142874366239149e-01",
			"2.222219843214978396e-01", "1.818357216161805012e-01", "1.531383769920937332e-01",
			"1.479819860511658591e-01",
			"-8.750608600031904122785e-01", "-1.615753718733365076637e+01", "-7.500855792314704667340e+01",
			"-1.228866684490136173410e+02", "-6.485021904942025371773e+01", "+2.485846490142306297962e+01",
			"+1.650270098316988542046e+02", "+4.328810604912902668951e+02", "+4.853903996359136964868e+02",
			"+1.945506571482613964425e+02", "6.123233995736765886130e-17", "2.41421356237309504880",
		} {
			v, err := strconv.ParseFloat(s, 64)
			if err != nil {
				panic(err)
			}
			fmt.Printf("%s %s\n", s, bits(v))
		}
		fmt.Println("4/Pi", bits(4/math.Pi))
		fmt.Println("3*Pi/4", bits(3*math.Pi/4))
		fmt.Println("Pi/2", bits(math.Pi/2), bits(math.Pi)) 
		fmt.Println("Pi/4", bits(math.Pi/4))
		fmt.Println("0.5*Morebits", bits(0.5*6.123233995736765886130e-17))
		fmt.Println("SmallestNormal", bits(2.2250738585072014e-308))
		return
	}
	// Entradas deterministas.
	seed := uint64(0x9E3779B97F4A7C15)
	rnd := func() float64 {
		seed ^= seed << 13
		seed ^= seed >> 7
		seed ^= seed << 17
		return float64(seed>>11) / (1 << 53)
	}
	special := []float64{0, math.Copysign(0, -1), 1, -1, 0.5, -0.5, 2, -2, 3, 0.1, 0.7, 0.66, 2.4142135623730951,
		1e-300, 5e-324, 1e300, math.Inf(1), math.Inf(-1), math.NaN(), math.Pi, math.Pi / 2, math.Pi / 4,
		-math.Pi, 1e10, 1e20, 1e300, 536870912, 536870913, 7e8, 709.78, 709.79, -745.13, -745.14, -1000, 1000, 1e-5}
	var ranges []float64
	for _, span := range []float64{1, 4, 10, 100, 1000, 1e6, 1e9} {
		for i := 0; i < 40; i++ {
			ranges = append(ranges, (rnd()*2-1)*span)
		}
	}
	for d := 0.0; d < 720; d += 2.3 {
		ranges = append(ranges, d*math.Pi/180)
	}
	one := append(append([]float64{}, special...), ranges...)
	var two [][2]float64
	for _, a := range special {
		for _, b := range special[:14] {
			two = append(two, [2]float64{a, b})
		}
	}
	for i := 0; i < 400; i++ {
		a, b := (rnd()*2-1)*1000, (rnd()*2-1)*1000
		if i%3 == 0 {
			b = (rnd()*2 - 1) * 4
			a = rnd() * 50
		}
		if i%7 == 0 {
			b = math.Round(b)
		}
		two = append(two, [2]float64{a, b})
	}
	vm := goja.New()
	mathObj := vm.Get("Math").ToObject(vm)
	out := map[string][][]string{}
	for _, name := range []string{"sin", "cos", "tan", "atan", "asin", "acos", "exp", "log"} {
		fn, _ := goja.AssertFunction(mathObj.Get(name))
		for _, x := range one {
			arg := x
			if name == "asin" || name == "acos" {
				arg = x / 1000
				if math.Abs(x) <= 1 {
					arg = x
				}
			}
			if name == "exp" && math.Abs(arg) > 800 {
				arg = math.Mod(arg, 800)
			}
			v, err := fn(goja.Undefined(), vm.ToValue(arg))
			if err != nil {
				panic(err)
			}
			out[name] = append(out[name], []string{bits(arg), bits(v.ToFloat())})
		}
	}
	for _, name := range []string{"atan2", "pow", "hypot"} {
		fn, _ := goja.AssertFunction(mathObj.Get(name))
		for _, p := range two {
			v, err := fn(goja.Undefined(), vm.ToValue(p[0]), vm.ToValue(p[1]))
			if err != nil {
				panic(err)
			}
			out[name] = append(out[name], []string{bits(p[0]), bits(p[1]), bits(v.ToFloat())})
		}
	}
	js, _ := json.Marshal(out)
	os.Stdout.Write(append(js, '\n'))
}
