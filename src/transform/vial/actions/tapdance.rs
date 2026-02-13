use super::Keycode;

#[derive(Clone)]
pub struct TapDance {
    pub tap: Keycode,
    pub hold: Keycode,
    pub double_tap: Keycode,
    pub tap_hold: Keycode,
    pub tapping_term: u16,
}
impl PartialEq for TapDance {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}
impl Eq for TapDance {}
impl std::hash::Hash for TapDance {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self).hash(state);
    }
}
impl std::fmt::Debug for TapDance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "tap-dance({:?}, {:?}, {:?}, {:?})",
            self.tap, self.hold, self.double_tap, self.tap_hold,
        )
    }
}
