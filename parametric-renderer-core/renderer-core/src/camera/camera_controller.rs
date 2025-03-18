use glam::{Quat, Vec3};

use crate::input::{CursorCaptureRequest, WindowInputs};

use super::{freecam_controller::FreecamController, orbitcam_controller::OrbitcamController};

pub trait IsCameraController {
    fn position(&self) -> Vec3;
    fn orientation(&self) -> Quat;
    fn general_controller(&self) -> GeneralController;
}

pub enum ChosenKind {
    Orbitcam,
    Freecam,
}

enum ChosenController {
    Orbitcam(OrbitcamController),
    Freecam(FreecamController),
}

impl ChosenController {
    fn new(controller: GeneralController, kind: ChosenKind) -> Self {
        match kind {
            ChosenKind::Orbitcam => ChosenController::Orbitcam(OrbitcamController::new(controller)),
            ChosenKind::Freecam => ChosenController::Freecam(FreecamController::new(controller)),
        }
    }

    fn update(
        &mut self,
        input: &WindowInputs,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) -> CursorCaptureRequest {
        match self {
            ChosenController::Orbitcam(orbitcam) => orbitcam.update(input, delta_time, settings),
            ChosenController::Freecam(freecam) => freecam.update(input, delta_time, settings),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneralController {
    pub position: Vec3,
    pub orientation: Quat,
    pub distance_to_center: f32,
}

#[derive(Debug, Clone)]
pub struct GeneralControllerSettings {
    pub fly_speed: f32,
    pub pan_speed: f32,
    pub rotation_sensitivity: f32,
}

pub struct CameraController {
    pub settings: GeneralControllerSettings,
    chosen: ChosenController,
}

impl CameraController {
    pub fn new(
        controller: GeneralController,
        settings: GeneralControllerSettings,
        chosen_kind: ChosenKind,
    ) -> Self {
        let chosen = ChosenController::new(controller, chosen_kind);
        Self { settings, chosen }
    }

    pub fn switch_to(&mut self, chosen_kind: ChosenKind) {
        self.chosen = ChosenController::new(self.general_controller(), chosen_kind);
    }

    pub fn get_chosen_kind(&self) -> ChosenKind {
        match &self.chosen {
            ChosenController::Orbitcam(_) => ChosenKind::Orbitcam,
            ChosenController::Freecam(_) => ChosenKind::Freecam,
        }
    }

    pub fn update(&mut self, input: &WindowInputs, delta_time: f32) -> CursorCaptureRequest {
        self.chosen.update(input, delta_time, &self.settings)
    }

    /// Only does something when the orbit cam is in use
    pub fn focus_on(&mut self, position: Vec3) {
        match &mut self.chosen {
            ChosenController::Orbitcam(orbitcam_controller) => {
                orbitcam_controller.center = position;
            }
            ChosenController::Freecam(_freecam_controller) => {
                // I could do the look-at math here I guess
            }
        }
    }
}

impl IsCameraController for CameraController {
    fn position(&self) -> Vec3 {
        match &self.chosen {
            ChosenController::Orbitcam(v) => v.position(),
            ChosenController::Freecam(v) => v.position(),
        }
    }

    fn orientation(&self) -> Quat {
        match &self.chosen {
            ChosenController::Orbitcam(v) => v.orientation(),
            ChosenController::Freecam(v) => v.orientation(),
        }
    }

    fn general_controller(&self) -> GeneralController {
        match &self.chosen {
            ChosenController::Orbitcam(v) => v.general_controller(),
            ChosenController::Freecam(v) => v.general_controller(),
        }
    }
}
