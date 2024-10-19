use bevy_mod_stylebuilder::{StyleBuilder, StyleBuilderFont};

/// Default text style for UI.
pub fn text_default(ss: &mut StyleBuilder) {
    ss.font("embedded://bevy_reactor_obsidian/assets/fonts/Fira_Sans/FiraSans-Medium.ttf")
        .font_size(14);
}

/// When we need to emphasize a label
pub fn text_strong(ss: &mut StyleBuilder) {
    ss.font("embedded://bevy_reactor_obsidian/assets/fonts/Fira_Sans/FiraSans-Bold.ttf")
        .font_size(14);
}
