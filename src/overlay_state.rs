use std::collections::HashSet;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum DisplayKey {
    W,
    A,
    S,
    D,
    Space,
    Shift,
    Ctrl,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct KeySnapshot {
    pressed: HashSet<DisplayKey>,
}

impl KeySnapshot {
    pub fn from_pressed<const N: usize>(keys: [DisplayKey; N]) -> Self {
        Self {
            pressed: keys.into_iter().collect(),
        }
    }

    pub fn is_pressed(&self, key: DisplayKey) -> bool {
        self.pressed.contains(&key)
    }
}

#[derive(Clone, Debug)]
pub struct OverlayState {
    visible: bool,
    toggle_was_down: bool,
    keys: KeySnapshot,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayState {
    pub fn new() -> Self {
        Self {
            visible: true,
            toggle_was_down: false,
            keys: KeySnapshot::default(),
        }
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn update_toggle_key(&mut self, is_down: bool) {
        if is_down && !self.toggle_was_down {
            self.visible = !self.visible;
        }
        self.toggle_was_down = is_down;
    }

    pub fn update_keys(&mut self, snapshot: KeySnapshot) {
        self.keys = snapshot;
    }

    pub fn is_pressed(&self, key: DisplayKey) -> bool {
        self.keys.is_pressed(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn updates_pressed_keys_from_snapshot() {
        let mut overlay = OverlayState::new();

        overlay.update_keys(KeySnapshot::from_pressed([DisplayKey::W, DisplayKey::Space]));

        assert!(overlay.is_pressed(DisplayKey::W));
        assert!(overlay.is_pressed(DisplayKey::Space));
        assert!(!overlay.is_pressed(DisplayKey::A));
    }

    #[test]
    fn toggles_visibility_once_per_u_press() {
        let mut overlay = OverlayState::new();

        overlay.update_toggle_key(true);
        overlay.update_toggle_key(true);
        assert!(!overlay.visible());

        overlay.update_toggle_key(false);
        overlay.update_toggle_key(true);
        assert!(overlay.visible());
    }
}
