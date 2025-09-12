use bevy::{prelude::*, render::view::RenderLayers};

use crate::{
    controls::{self, ControlMode},
    layers,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, (select_buttons, update_tool_button_visuals));
    }
}

#[derive(Component, Debug, PartialEq, Eq, Clone, Copy)]
enum Tool {
    Selection,
    PlaceGeometry,
}

impl Into<controls::ControlMode> for Tool {
    fn into(self) -> controls::ControlMode {
        match self {
            Tool::Selection => controls::ControlMode::Select,
            Tool::PlaceGeometry => controls::ControlMode::PlaceGeometry,
        }
    }
}

impl From<controls::ControlMode> for Tool {
    fn from(value: controls::ControlMode) -> Self {
        match value {
            controls::ControlMode::Select => Tool::Selection,
            controls::ControlMode::PlaceGeometry => Tool::PlaceGeometry,
        }
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const ACTIVE_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: layers::UI_CAMERA,
            ..default()
        },
        RenderLayers::layer(layers::UI_LAYER),
    ));
    commands.spawn(container()).with_children(|parent| {
        for &tool in &[Tool::Selection, Tool::PlaceGeometry] {
            parent.spawn((button(tool, tool == Tool::Selection), tool));
        }
    });
}

fn select_buttons(
    mut interaction_q: Query<(&Interaction, &Tool), Changed<Interaction>>,
    mut control_mode: ResMut<controls::ControlMode>,
) {
    for (interaction, tool) in interaction_q.iter_mut() {
        if *interaction == Interaction::Pressed {
            let intent: ControlMode = (*tool).into();
            if *control_mode != intent {
                info!("Tool changed to {:?}", tool);
                *control_mode = intent;
            } else {
                info!("{:?} is already selected", tool);
            }
        }
    }
}

fn update_tool_button_visuals(
    control_mode: Res<ControlMode>,
    buttons: Query<(&Tool, &mut BackgroundColor, &Children)>,
    mut texts: Query<&mut TextColor>,
) {
    for (tool, mut color, chldren) in buttons {
        let is_selected = *tool == (*control_mode).into();
        *color = button_color(is_selected);

        // Update text color
        for child in chldren.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                *text = text_color(is_selected);
            }
        }
    }
}

fn container() -> impl Bundle + use<> {
    (
        Node {
            padding: UiRect::all(Val::Px(10.0)),
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexEnd,
            ..default()
        },
        Pickable::IGNORE,
        RenderLayers::layer(layers::UI_LAYER),
    )
}

fn button(tool: Tool, is_active: bool) -> impl Bundle + use<> {
    (
        Button,
        Node {
            width: Val::Px(120.0),
            height: Val::Px(40.0),
            border: UiRect::all(Val::Px(2.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor(Color::BLACK),
        BorderRadius::all(Val::Px(5.0)),
        button_color(is_active),
        children![(
            Text::new(tool_text(&tool)),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            text_color(is_active),
        )],
        RenderLayers::layer(layers::UI_LAYER),
    )
}

fn tool_text(tool: &Tool) -> String {
    match tool {
        Tool::Selection => "Selection".to_string(),
        Tool::PlaceGeometry => "Place Box".to_string(),
    }
}

fn button_color(is_active: bool) -> BackgroundColor {
    if is_active {
        BackgroundColor(ACTIVE_BUTTON)
    } else {
        BackgroundColor(NORMAL_BUTTON)
    }
}

fn text_color(is_active: bool) -> TextColor {
    if is_active {
        TextColor(Color::srgb(0.1, 0.1, 0.1))
    } else {
        TextColor(Color::srgb(0.9, 0.9, 0.9))
    }
}
