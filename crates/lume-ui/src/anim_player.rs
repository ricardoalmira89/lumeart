//! Reproducción de una acción: qué fotograma se ve y cuándo pasa al
//! siguiente, según la duración de cada fotograma, el bucle, la ida y vuelta
//! y la velocidad. Es lógica pura (se prueba sin ventana).

/// Estado del reproductor.
#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    /// Fotograma que se ve.
    pub frame: usize,
    pub playing: bool,
    /// Al llegar al final vuelve a empezar (si no, se para).
    pub looping: bool,
    /// Ida y vuelta: 0, 1, 2, 3, 2, 1, 0, 1…
    pub pingpong: bool,
    /// Multiplicador de velocidad (1 = tiempo real).
    pub speed: f64,
    forward: bool,
    /// Milisegundos que lleva el fotograma en pantalla.
    elapsed: f64,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            frame: 0,
            playing: false,
            looping: true,
            pingpong: false,
            speed: 1.0,
            forward: true,
            elapsed: 0.0,
        }
    }
}

impl Player {
    /// Reproductor parado en el primer fotograma, con el bucle de la acción.
    pub fn new(looping: bool) -> Player {
        Player {
            looping,
            ..Player::default()
        }
    }

    /// ¿Está parado al final de una reproducción sin bucle?
    fn at_end(&self, frames: usize) -> bool {
        if self.pingpong {
            self.frame == 0 && !self.forward
        } else {
            self.frame + 1 >= frames
        }
    }

    /// Reproduce (si estaba al final de una acción sin bucle, desde el
    /// principio) o pausa.
    pub fn toggle(&mut self, frames: usize) {
        if self.playing {
            self.playing = false;
            return;
        }
        if !self.looping && self.at_end(frames) {
            self.seek(0);
        }
        self.playing = true;
    }

    /// Va a un fotograma (y empieza a contar su duración).
    pub fn seek(&mut self, frame: usize) {
        self.frame = frame;
        self.forward = true;
        self.elapsed = 0.0;
    }

    pub fn first(&mut self) {
        self.playing = false;
        self.seek(0);
    }

    pub fn last(&mut self, frames: usize) {
        self.playing = false;
        self.seek(frames.saturating_sub(1));
    }

    pub fn next(&mut self, frames: usize) {
        self.playing = false;
        if frames > 0 {
            self.seek((self.frame + 1) % frames);
        }
    }

    pub fn prev(&mut self, frames: usize) {
        self.playing = false;
        if frames > 0 {
            self.seek((self.frame + frames - 1) % frames);
        }
    }

    /// Pasa al fotograma siguiente según el modo. `false` si se acabó (sin
    /// bucle).
    fn step(&mut self, frames: usize) -> bool {
        if frames <= 1 {
            return self.looping;
        }
        if self.pingpong {
            if self.forward {
                if self.frame + 1 < frames {
                    self.frame += 1;
                } else {
                    self.forward = false;
                    self.frame -= 1;
                }
            } else if self.frame > 0 {
                self.frame -= 1;
            } else {
                if !self.looping {
                    return false;
                }
                self.forward = true;
                self.frame = 1;
            }
        } else if self.frame + 1 < frames {
            self.frame += 1;
        } else if self.looping {
            self.frame = 0;
        } else {
            return false;
        }
        true
    }

    /// Avanza `dt_ms` milisegundos de tiempo real con estas duraciones (una
    /// por fotograma). Devuelve si cambió el fotograma.
    pub fn advance(&mut self, dt_ms: f64, durations: &[u32]) -> bool {
        let frames = durations.len();
        if !self.playing || frames == 0 {
            return false;
        }
        self.frame = self.frame.min(frames - 1);
        let start = self.frame;
        self.elapsed += dt_ms * self.speed;
        loop {
            let duration = f64::from(durations[self.frame].max(1));
            if self.elapsed < duration {
                break;
            }
            self.elapsed -= duration;
            if !self.step(frames) {
                self.playing = false;
                self.elapsed = 0.0;
                break;
            }
        }
        self.frame != start
    }

    /// Milisegundos que faltan para el siguiente cambio (para despertar la
    /// ventana justo a tiempo).
    pub fn remaining_ms(&self, durations: &[u32]) -> Option<f64> {
        let duration = f64::from(*durations.get(self.frame)?);
        self.playing
            .then(|| ((duration - self.elapsed) / self.speed).max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Los fotogramas que se ven, uno por cada `dt` ms, durante `steps` pasos.
    fn run(player: &mut Player, durations: &[u32], dt: f64, steps: usize) -> Vec<usize> {
        (0..steps)
            .map(|_| {
                player.advance(dt, durations);
                player.frame
            })
            .collect()
    }

    #[test]
    fn each_frame_lasts_its_duration() {
        // 10 fps → 100 ms; el tercero dura 300.
        let durations = [100, 100, 300, 100];
        let mut p = Player::new(true);
        p.toggle(4);
        assert_eq!(
            run(&mut p, &durations, 50.0, 14),
            [0, 1, 1, 2, 2, 2, 2, 2, 2, 3, 3, 0, 0, 1]
        );
    }

    #[test]
    fn fps_set_the_pace() {
        // 8 fps → 125 ms por fotograma: en 1 s, 8 fotogramas (vuelve al 0).
        let durations = [125; 4];
        let mut p = Player::new(true);
        p.toggle(4);
        p.advance(1000.0, &durations);
        assert_eq!(p.frame, 0);
        p.advance(250.0, &durations);
        assert_eq!(p.frame, 2);
        // Al doble de velocidad, la mitad de tiempo.
        p.speed = 2.0;
        p.advance(125.0, &durations);
        assert_eq!(p.frame, 0);
    }

    #[test]
    fn without_loop_it_stops_at_the_end_and_play_restarts() {
        let durations = [100; 3];
        let mut p = Player::new(false);
        p.toggle(3);
        assert_eq!(run(&mut p, &durations, 100.0, 4), [1, 2, 2, 2]);
        assert!(!p.playing);
        p.toggle(3);
        assert_eq!(p.frame, 0, "reproducir al final empieza otra vez");
        assert!(p.playing);
    }

    #[test]
    fn ping_pong_bounces_without_repeating_the_ends() {
        let durations = [100; 4];
        let mut p = Player::new(true);
        p.pingpong = true;
        p.toggle(4);
        assert_eq!(
            run(&mut p, &durations, 100.0, 9),
            [1, 2, 3, 2, 1, 0, 1, 2, 3]
        );
        // Sin bucle: ida y vuelta una vez y se para en el primero.
        let mut p = Player::new(false);
        p.pingpong = true;
        p.toggle(4);
        assert_eq!(run(&mut p, &durations, 100.0, 8), [1, 2, 3, 2, 1, 0, 0, 0]);
        assert!(!p.playing);
        // Con duraciones distintas, también.
        let mut p = Player::new(true);
        p.pingpong = true;
        p.toggle(3);
        assert_eq!(
            run(&mut p, &[100, 200, 100], 100.0, 7),
            [1, 1, 2, 1, 1, 0, 1]
        );
    }

    #[test]
    fn manual_steps_wrap_and_pause() {
        let mut p = Player::new(true);
        p.toggle(4);
        p.prev(4);
        assert_eq!((p.frame, p.playing), (3, false));
        p.next(4);
        assert_eq!(p.frame, 0);
        p.last(4);
        assert_eq!(p.frame, 3);
        p.first();
        assert_eq!(p.frame, 0);
        assert_eq!(p.remaining_ms(&[100; 4]), None, "parado");
        p.toggle(4);
        p.advance(30.0, &[100; 4]);
        assert_eq!(p.remaining_ms(&[100; 4]), Some(70.0));
    }

    #[test]
    fn a_single_frame_never_moves() {
        let mut p = Player::new(true);
        p.toggle(1);
        assert!(!p.advance(1000.0, &[100]));
        assert_eq!(p.frame, 0);
    }
}
