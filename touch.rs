#![allow(dead_code)]
//! Controles táctiles: joystick virtual (mitad izquierda)
//! y mirar arrastrando el dedo (mitad derecha).

use winit::event::{Touch, TouchPhase};

/// Radio del joystick en píxeles físicos.
const STICK_RADIUS: f64 = 120.0;
/// Zona muerta: movimientos más chicos que esto se ignoran.
const DEAD_ZONE: f64 = 12.0;

#[derive(Default)]
pub struct TouchControls {
    stick_id: Option<u64>,
    stick_origin: (f64, f64),
    stick_now: (f64, f64),
    look_id: Option<u64>,
    look_last: (f64, f64),
    look_delta: (f64, f64),
}

impl TouchControls {
    /// Llamar cada vez que winit avisa de un evento Touch.
    pub fn handle(&mut self, touch: &Touch, screen_width: f64) {
        let pos = (touch.location.x, touch.location.y);
        match touch.phase {
            TouchPhase::Started => {
                if pos.0 < screen_width / 2.0 {
                    if self.stick_id.is_none() {
                        self.stick_id = Some(touch.id);
                        self.stick_origin = pos;
                        self.stick_now = pos;
                    }
                } else if self.look_id.is_none() {
                    self.look_id = Some(touch.id);
                    self.look_last = pos;
                }
            }
            TouchPhase::Moved => {
                if self.stick_id == Some(touch.id) {
                    self.stick_now = pos;
                } else if self.look_id == Some(touch.id) {
                    self.look_delta.0 += pos.0 - self.look_last.0;
                    self.look_delta.1 += pos.1 - self.look_last.1;
                    self.look_last = pos;
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if self.stick_id == Some(touch.id) {
                    self.stick_id = None;
                } else if self.look_id == Some(touch.id) {
                    self.look_id = None;
                }
            }
        }
    }

    /// (derecha, adelante), cada uno entre -1.0 y 1.0.
    pub fn movement(&self) -> (f32, f32) {
        if self.stick_id.is_none() {
            return (0.0, 0.0);
        }
        let dx = self.stick_now.0 - self.stick_origin.0;
        let dy = self.stick_now.1 - self.stick_origin.1;
        let len = dx.hypot(dy);
        if len < DEAD_ZONE {
            return (0.0, 0.0);
        }
        let strength = (len / STICK_RADIUS).min(1.0);
        // En pantalla "arriba" es y negativo, por eso se invierte dy.
        (((dx / len) * strength) as f32, ((-dy / len) * strength) as f32)
    }

    /// Cuánto se arrastró el dedo derecho desde la última vez que se llamó.
    pub fn take_look_delta(&mut self) -> (f64, f64) {
        std::mem::take(&mut self.look_delta)
    }
}
