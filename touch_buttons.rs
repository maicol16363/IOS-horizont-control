#![allow(dead_code)]
//! Botones táctiles configurables con perfiles guardables ("survival", "pvp"...).

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use winit::event::{Touch, TouchPhase};
use winit::keyboard::KeyCode;

// ---------------------------------------------------------------
// Acciones: lo que un botón puede apretar (teclas y clics del mouse)
// ---------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MouseBtn {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Action {
    Key(KeyCode),
    Mouse(MouseBtn),
}

macro_rules! key_table {
    ($($n:ident),* $(,)?) => { &[$((stringify!($n), KeyCode::$n)),*] };
}

/// Teclas disponibles. El texto es igual al nombre de winit.
const KEYS: &[(&str, KeyCode)] = key_table![
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM,
    KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,
    Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    Space, ShiftLeft, ControlLeft, AltLeft, Tab, Escape, Enter, Backspace,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
];

impl Action {
    pub fn name(&self) -> String {
        match self {
            Action::Mouse(MouseBtn::Left) => "MouseLeft".to_string(),
            Action::Mouse(MouseBtn::Right) => "MouseRight".to_string(),
            Action::Mouse(MouseBtn::Middle) => "MouseMiddle".to_string(),
            Action::Key(k) => KEYS
                .iter()
                .find(|e| e.1 == *k)
                .map(|e| e.0.to_string())
                .unwrap_or_else(|| "Unknown".to_string()),
        }
    }

    pub fn from_name(s: &str) -> Option<Action> {
        match s {
            "MouseLeft" => Some(Action::Mouse(MouseBtn::Left)),
            "MouseRight" => Some(Action::Mouse(MouseBtn::Right)),
            "MouseMiddle" => Some(Action::Mouse(MouseBtn::Middle)),
            _ => KEYS.iter().find(|e| e.0 == s).map(|e| Action::Key(e.1)),
        }
    }
}

// ---------------------------------------------------------------
// Botones y perfiles
// ---------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    /// Apretado mientras el dedo esté encima.
    Hold,
    /// Un toque rápido: se aprieta y se suelta sola.
    Tap,
    /// Un toque la activa, otro la desactiva.
    Toggle,
}

impl Mode {
    fn name(&self) -> &'static str {
        match self {
            Mode::Hold => "hold",
            Mode::Tap => "tap",
            Mode::Toggle => "toggle",
        }
    }
    fn from_name(s: &str) -> Option<Mode> {
        match s {
            "hold" => Some(Mode::Hold),
            "tap" => Some(Mode::Tap),
            "toggle" => Some(Mode::Toggle),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ButtonDef {
    pub label: String,
    /// Posición y tamaño como fracción de la pantalla (0.0 a 1.0).
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Color de relleno 0xRRGGBB y opacidad.
    pub color: u32,
    pub alpha: f32,
    /// Esquinas: 0.0 = cuadrado, 1.0 = totalmente redondo.
    pub radius: f32,
    /// Color cuando el botón está apretado.
    pub pressed_color: u32,
    /// Color del texto.
    pub text_color: u32,
    pub mode: Mode,
    /// Varias acciones = combinación (todas se aprietan a la vez).
    pub actions: Vec<Action>,
}

impl ButtonDef {
    fn contains(&self, fx: f32, fy: f32) -> bool {
        fx >= self.x && fx <= self.x + self.w && fy >= self.y && fy <= self.y + self.h
    }
}

/// Un perfil: "survival", "pvp", etc.
#[derive(Clone, Debug)]
pub struct Layout {
    pub name: String,
    pub buttons: Vec<ButtonDef>,
}

impl Layout {
    pub fn to_text(&self) -> String {
        let mut s = String::from(
            "# nombre|x|y|ancho|alto|modo|color|opacidad|acciones|radio|color_apretado|color_texto\n",
        );
        for b in &self.buttons {
            let acts: Vec<String> = b.actions.iter().map(|a| a.name()).collect();
            s.push_str(&format!(
                "{}|{:.4}|{:.4}|{:.4}|{:.4}|{}|{:06x}|{:.2}|{}|{:.2}|{:06x}|{:06x}\n",
                b.label.replace('|', " "),
                b.x,
                b.y,
                b.w,
                b.h,
                b.mode.name(),
                b.color,
                b.alpha,
                acts.join("+"),
                b.radius,
                b.pressed_color,
                b.text_color
            ));
        }
        s
    }

    pub fn from_text(name: &str, text: &str) -> Layout {
        let buttons = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .filter_map(parse_button)
            .collect();
        Layout { name: name.to_string(), buttons }
    }
}

fn parse_button(line: &str) -> Option<ButtonDef> {
    let p: Vec<&str> = line.split('|').collect();
    if p.len() < 9 {
        return None;
    }
    let actions: Vec<Action> = p[8]
        .split('+')
        .filter_map(|n| Action::from_name(n.trim()))
        .collect();
    let opt_f = |i: usize, d: f32| {
        p.get(i).and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(d)
    };
    let opt_c = |i: usize, d: u32| {
        p.get(i).and_then(|v| u32::from_str_radix(v.trim(), 16).ok()).unwrap_or(d) & 0xFFFFFF
    };
    Some(ButtonDef {
        label: p[0].to_string(),
        x: p[1].trim().parse::<f32>().ok()?.clamp(0.0, 1.0),
        y: p[2].trim().parse::<f32>().ok()?.clamp(0.0, 1.0),
        w: p[3].trim().parse::<f32>().ok()?.clamp(0.03, 0.6),
        h: p[4].trim().parse::<f32>().ok()?.clamp(0.03, 0.6),
        mode: Mode::from_name(p[5].trim())?,
        color: u32::from_str_radix(p[6].trim(), 16).ok()? & 0xFFFFFF,
        alpha: p[7].trim().parse::<f32>().ok()?.clamp(0.05, 1.0),
        actions,
        radius: opt_f(9, 0.25).clamp(0.0, 1.0),
        pressed_color: opt_c(10, 0xAAAAAA),
        text_color: opt_c(11, 0xFFFFFF),
    })
}

// ---------------------------------------------------------------
// Guardado: una carpeta con un archivo .layout por perfil
// ---------------------------------------------------------------

pub struct LayoutStore {
    dir: PathBuf,
}

fn clean_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

impl LayoutStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let _ = fs::create_dir_all(&dir);
        Self { dir }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{}.layout", clean_name(name)))
    }

    pub fn list(&self) -> Vec<String> {
        let mut v: Vec<String> = fs::read_dir(&self.dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.extension()?.to_str()? == "layout" {
                    Some(p.file_stem()?.to_str()?.to_string())
                } else {
                    None
                }
            })
            .collect();
        v.sort();
        v
    }

    pub fn load(&self, name: &str) -> Option<Layout> {
        let text = fs::read_to_string(self.path(name)).ok()?;
        Some(Layout::from_text(&clean_name(name), &text))
    }

    pub fn save(&self, layout: &Layout) -> std::io::Result<()> {
        fs::write(self.path(&layout.name), layout.to_text())
    }

    pub fn delete(&self, name: &str) -> std::io::Result<()> {
        fs::remove_file(self.path(name))
    }

    pub fn active_name(&self) -> Option<String> {
        fs::read_to_string(self.dir.join("active.txt"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    pub fn set_active(&self, name: &str) -> std::io::Result<()> {
        fs::write(self.dir.join("active.txt"), clean_name(name))
    }

    /// La primera vez crea los perfiles de ejemplo.
    pub fn install_defaults(&self) {
        if self.list().is_empty() {
            let _ = self.save(&preset_survival());
            let _ = self.save(&preset_pvp());
            let _ = self.set_active("survival");
        }
    }
}

fn btn(label: &str, x: f32, y: f32, w: f32, h: f32, mode: Mode, actions: Vec<Action>) -> ButtonDef {
    ButtonDef {
        label: label.to_string(),
        x,
        y,
        w,
        h,
        color: 0xFFFFFF,
        alpha: 0.45,
        radius: 0.25,
        pressed_color: 0xAAAAAA,
        text_color: 0xFFFFFF,
        mode,
        actions,
    }
}

pub fn preset_survival() -> Layout {
    use Action::{Key, Mouse};
    Layout {
        name: "survival".to_string(),
        buttons: vec![
            btn("Saltar", 0.88, 0.55, 0.08, 0.14, Mode::Hold, vec![Key(KeyCode::Space)]),
            btn("Agachar", 0.78, 0.70, 0.08, 0.14, Mode::Toggle, vec![Key(KeyCode::ShiftLeft)]),
            btn("Correr", 0.78, 0.40, 0.08, 0.14, Mode::Toggle, vec![Key(KeyCode::ControlLeft)]),
            btn("Inv.", 0.88, 0.05, 0.08, 0.14, Mode::Tap, vec![Key(KeyCode::KeyE)]),
            btn("Soltar", 0.78, 0.05, 0.08, 0.14, Mode::Tap, vec![Key(KeyCode::KeyQ)]),
            btn("Romper", 0.66, 0.55, 0.10, 0.18, Mode::Hold, vec![Mouse(MouseBtn::Left)]),
            btn("Poner", 0.66, 0.76, 0.10, 0.18, Mode::Tap, vec![Mouse(MouseBtn::Right)]),
        ],
    }
}

pub fn preset_pvp() -> Layout {
    use Action::{Key, Mouse};
    Layout {
        name: "pvp".to_string(),
        buttons: vec![
            btn("Atacar", 0.70, 0.50, 0.14, 0.26, Mode::Hold, vec![Mouse(MouseBtn::Left)]),
            btn("Bloquear", 0.54, 0.62, 0.11, 0.20, Mode::Hold, vec![Mouse(MouseBtn::Right)]),
            btn("Saltar", 0.88, 0.50, 0.09, 0.16, Mode::Hold, vec![Key(KeyCode::Space)]),
            btn("Correr", 0.88, 0.70, 0.09, 0.16, Mode::Toggle, vec![Key(KeyCode::ControlLeft)]),
            btn("Item 1", 0.40, 0.05, 0.06, 0.12, Mode::Tap, vec![Key(KeyCode::Digit1)]),
            btn("Item 2", 0.47, 0.05, 0.06, 0.12, Mode::Tap, vec![Key(KeyCode::Digit2)]),
            btn("Item 3", 0.54, 0.05, 0.06, 0.12, Mode::Tap, vec![Key(KeyCode::Digit3)]),
        ],
    }
}

// ---------------------------------------------------------------
// Funcionamiento en vivo: qué botones están apretados ahora
// ---------------------------------------------------------------

pub struct TouchButtons {
    pub layout: Layout,
    /// (id del dedo, índice del botón) de cada dedo que está sobre un botón.
    fingers: Vec<(u64, usize)>,
    toggled: Vec<bool>,
    tap_frames: Vec<u8>,
}

impl TouchButtons {
    pub fn new(layout: Layout) -> Self {
        let n = layout.buttons.len();
        Self { layout, fingers: Vec::new(), toggled: vec![false; n], tap_frames: vec![0; n] }
    }

    /// Cambiar de perfil en vivo (por ejemplo de "survival" a "pvp").
    pub fn set_layout(&mut self, layout: Layout) {
        *self = TouchButtons::new(layout);
    }

    /// Devuelve true si este toque pertenece a un botón (el joystick debe ignorarlo).
    pub fn handle(&mut self, touch: &Touch, screen_w: f64, screen_h: f64) -> bool {
        let fx = (touch.location.x / screen_w) as f32;
        let fy = (touch.location.y / screen_h) as f32;
        match touch.phase {
            TouchPhase::Started => {
                let hit = self.layout.buttons.iter().position(|b| b.contains(fx, fy));
                match hit {
                    Some(i) => {
                        self.fingers.push((touch.id, i));
                        match self.layout.buttons[i].mode {
                            Mode::Hold => {}
                            Mode::Toggle => self.toggled[i] = !self.toggled[i],
                            Mode::Tap => self.tap_frames[i] = 2,
                        }
                        true
                    }
                    None => false,
                }
            }
            TouchPhase::Moved => self.fingers.iter().any(|(id, _)| *id == touch.id),
            TouchPhase::Ended | TouchPhase::Cancelled => {
                let before = self.fingers.len();
                self.fingers.retain(|(id, _)| *id != touch.id);
                self.fingers.len() != before
            }
        }
    }

    /// Todas las acciones apretadas en este cuadro. Llamar una vez por cuadro.
    pub fn pressed(&mut self) -> HashSet<Action> {
        let mut out = HashSet::new();
        for (_, i) in &self.fingers {
            if self.layout.buttons[*i].mode == Mode::Hold {
                out.extend(self.layout.buttons[*i].actions.iter().copied());
            }
        }
        for i in 0..self.layout.buttons.len() {
            if self.toggled[i] {
                out.extend(self.layout.buttons[i].actions.iter().copied());
            }
            if self.tap_frames[i] > 0 {
                out.extend(self.layout.buttons[i].actions.iter().copied());
                self.tap_frames[i] -= 1;
            }
        }
        out
    }
}
