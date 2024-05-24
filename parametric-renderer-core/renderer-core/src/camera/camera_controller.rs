use glam::Quat;
use glamour::Point3;
use winit_input_helper::WinitInputHelper;

use super::{freecam_controller::FreecamController, orbitcam_controller::OrbitcamController};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CursorCapture {
    Free,
    LockedAndHidden,
}

pub trait IsCameraController {
    fn position(&self) -> Point3;
    fn orientation(&self) -> Quat;
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
        input: &WinitInputHelper,
        delta_time: f32,
        settings: &GeneralControllerSettings,
    ) -> CursorCapture {
        match self {
            ChosenController::Orbitcam(orbitcam) => orbitcam.update(input, delta_time, settings),
            ChosenController::Freecam(freecam) => freecam.update(input, delta_time, settings),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneralController {
    pub position: Point3,
    pub orientation: Quat,
    pub distance_to_center: f32,
}

pub struct GeneralControllerSettings {
    pub fly_speed: f32,
    pub pan_speed: f32,
    pub rotation_sensitivity: f32,
}

pub struct CameraController {
    pub settings: GeneralControllerSettings,
    chosen: ChosenController,
}

// Magic number.
const FREECAM_DISTANCE_TO_CENTER: f32 = 15.;

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
        self.chosen = ChosenController::new(self.get_general_controller(), chosen_kind);
    }

    pub fn get_general_controller(&self) -> GeneralController {
        let position = self.position();
        let orientation = self.orientation();
        match &self.chosen {
            ChosenController::Orbitcam(orbitcam) => GeneralController {
                position,
                orientation,
                distance_to_center: orbitcam.distance,
            },
            ChosenController::Freecam(_freecam) => GeneralController {
                position,
                orientation,
                distance_to_center: FREECAM_DISTANCE_TO_CENTER,
            },
        }
    }

    pub fn get_chosen_kind(&self) -> ChosenKind {
        match &self.chosen {
            ChosenController::Orbitcam(_) => ChosenKind::Orbitcam,
            ChosenController::Freecam(_) => ChosenKind::Freecam,
        }
    }

    pub fn update(&mut self, input: &WinitInputHelper, delta_time: f32) -> CursorCapture {
        self.chosen.update(input, delta_time, &self.settings)
    }
}

impl IsCameraController for CameraController {
    fn position(&self) -> Point3 {
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
}
