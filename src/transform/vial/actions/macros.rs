use super::Keycode;

#[derive(Debug, Clone)]
pub enum MacroAction {
    Down(Keycode),
    Up(Keycode),
    Tap(Keycode),
    Delay(u16),
}

#[derive(Clone)]
pub struct Macro(pub Vec<MacroAction>);

impl std::fmt::Debug for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "macro(")?;
        self.0.iter().try_for_each(|a| write!(f, "{:?},", a))?;
        write!(f, ")")
    }
}
impl PartialEq for Macro {
    fn eq(&self, other: &Self) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}
impl Eq for Macro {}
impl std::hash::Hash for Macro {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        format!("{:?}", self).hash(state);
    }
}
