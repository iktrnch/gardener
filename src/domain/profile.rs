//! Immutable automaton configuration and complete HID snapshots.

/// Stable identifiers for profile states.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateType {
    Idle,
    Left,
    Right,
}

/// Direction of a button-held transition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

/// USB HID keyboard usage IDs needed by the temporary profile.
pub mod keycodes {
    pub const A: u8 = 4;
    pub const D: u8 = 7;
    pub const S: u8 = 22;
    pub const W: u8 = 26;
}

/// Modifier bits in a standard keyboard report.
#[allow(unused)]
pub mod modifiers {
    pub const LEFT_CTRL: u8 = 1 << 0;
    pub const LEFT_SHIFT: u8 = 1 << 1;
    pub const LEFT_ALT: u8 = 1 << 2;
    pub const LEFT_GUI: u8 = 1 << 3;
    pub const RIGHT_CTRL: u8 = 1 << 4;
    pub const RIGHT_SHIFT: u8 = 1 << 5;
    pub const RIGHT_ALT: u8 = 1 << 6;
    pub const RIGHT_GUI: u8 = 1 << 7;
}

/// Standard mouse report button bits.
#[allow(unused)]
pub mod mouse_buttons {
    pub const NONE: u8 = 0;
    pub const LEFT: u8 = 1 << 0;
    pub const RIGHT: u8 = 1 << 1;
    pub const MIDDLE: u8 = 1 << 2;
}

/// Up to six simultaneous ordinary keys and a modifier byte.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keystrokes {
    pub modifiers: u8,
    pub keycodes: [u8; 6],
}

impl Keystrokes {
    pub const NONE: Self = Self {
        modifiers: 0,
        keycodes: [0; 6],
    };

    pub const fn one(key: u8) -> Self {
        Self {
            modifiers: 0,
            keycodes: [key, 0, 0, 0, 0, 0],
        }
    }

    #[allow(unused)]
    pub const fn two(first: u8, second: u8) -> Self {
        Self {
            modifiers: 0,
            keycodes: [first, second, 0, 0, 0, 0],
        }
    }

    pub const fn report(self) -> [u8; 8] {
        [
            self.modifiers,
            0,
            self.keycodes[0],
            self.keycodes[1],
            self.keycodes[2],
            self.keycodes[3],
            self.keycodes[4],
            self.keycodes[5],
        ]
    }
}

/// Complete keyboard and mouse state, never an incremental operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HidOutput {
    pub keystrokes: Keystrokes,
    pub mouse_buttons: u8,
}

impl HidOutput {
    pub const NONE: Self = Self {
        keystrokes: Keystrokes::NONE,
        mouse_buttons: mouse_buttons::NONE,
    };

    pub const fn new(keystrokes: Keystrokes, mouse_buttons: u8) -> Self {
        Self {
            keystrokes,
            mouse_buttons,
        }
    }

    pub const fn keyboard_report(self) -> [u8; 8] {
        self.keystrokes.report()
    }

    pub const fn mouse_report(self) -> [u8; 5] {
        [self.mouse_buttons, 0, 0, 0, 0]
    }
}

/// One complete profile output state.
///
/// For transition slots, `kind` is the stable state entered when the initiating
/// button is released.
#[derive(Clone, Copy, Debug)]
pub struct State {
    pub kind: StateType,
    pub output: HidOutput,
}

impl State {
    pub const fn new(kind: StateType, output: HidOutput) -> Self {
        Self { kind, output }
    }
}

/// Fixed profile layout used by the automaton.
pub struct Profile {
    pub name: &'static str,
    pub idle: State,
    pub left: State,
    pub right: State,
    pub transition_lr: Option<State>,
    pub transition_rl: Option<State>,
}

impl Profile {
    pub const fn state(&self, kind: StateType) -> &State {
        match kind {
            StateType::Idle => &self.idle,
            StateType::Left => &self.left,
            StateType::Right => &self.right,
        }
    }

    /// Validate fixed state and transition destinations before the controller starts.
    pub fn is_valid(&self) -> bool {
        self.idle.kind == StateType::Idle
            && self.idle.output == HidOutput::NONE
            && self.left.kind == StateType::Left
            && self.right.kind == StateType::Right
            && self
                .transition_lr
                .is_none_or(|state| state.kind == StateType::Right)
            && self
                .transition_rl
                .is_none_or(|state| state.kind == StateType::Left)
    }
}
