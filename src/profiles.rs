use crate::domain::{HidOutput, Keystrokes, Profile, State, StateType, keycodes, mouse_buttons};

/// Moonflower, Sunflower, Wild Rose, Sugar Cane
const ECLIPSE_ROSE_CANE: Profile = Profile {
    name: "Eclipse/Rose/Cane",
    idle: State::new(StateType::Idle, HidOutput::NONE),
    left: State::new(
        StateType::Left,
        HidOutput::new(Keystrokes::one(keycodes::A), mouse_buttons::LEFT),
    ),
    right: State::new(
        StateType::Right,
        HidOutput::new(Keystrokes::one(keycodes::S), mouse_buttons::LEFT),
    ),
    transition_lr: Some(State::new(
        StateType::Right,
        HidOutput::new(Keystrokes::one(keycodes::W), mouse_buttons::NONE),
    )),
    transition_rl: Some(State::new(
        StateType::Left,
        HidOutput::new(Keystrokes::one(keycodes::D), mouse_buttons::NONE),
    )),
};

/// Nether Wart, Wheat, Potato, Carrot
const WARTS_AND_CROPS: Profile = Profile {
    name: "Warts/Crops",
    idle: State::new(StateType::Idle, HidOutput::NONE),
    left: State::new(
        StateType::Left,
        HidOutput::new(Keystrokes::one(keycodes::A), mouse_buttons::LEFT),
    ),
    right: State::new(
        StateType::Right,
        HidOutput::new(Keystrokes::one(keycodes::D), mouse_buttons::LEFT),
    ),
    transition_lr: None,
    transition_rl: None,
};

/// Mushroom
const MUSHROOM: Profile = Profile {
    name: "Shrooms",
    idle: State::new(StateType::Idle, HidOutput::NONE),
    left: State::new(
        StateType::Left,
        HidOutput::new(Keystrokes::one(keycodes::A), mouse_buttons::LEFT),
    ),
    right: State::new(
        StateType::Right,
        HidOutput::new(Keystrokes::one(keycodes::D), mouse_buttons::LEFT),
    ),
    transition_lr: Some(State::new(
        StateType::Right,
        HidOutput::new(Keystrokes::one(keycodes::S), mouse_buttons::LEFT),
    )),
    transition_rl: None,
};

/// Immutable profile configuration. Mutable runtime state lives only in the controller.
pub static PROFILES: [Profile; 3] = [ECLIPSE_ROSE_CANE, WARTS_AND_CROPS, MUSHROOM];
