use glam::Quat;
use glamour::Point3;

pub trait CameraController {
    fn position(&self) -> Point3;
    fn orientation(&self) -> Quat;
}
