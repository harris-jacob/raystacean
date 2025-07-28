use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Node {
        style: Style {
            size: Size::width(Val::Percent(100.0)),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: Color::DARK_GRAY.into(),
        ..default()
    })
    .with_children(|parent| {
        // SELECT button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(100.0), Val::Px(40.0)),
                    margin: UiRect::right(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::GRAY.into(),
                ..default()
            },
            ToolButton::Select,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Select",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ));
        });

        // PLACE BOX button
        parent.spawn((
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(100.0), Val::Px(40.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: Color::GRAY.into(),
                ..default()
            },
            ToolButton::PlaceBox,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Place Box",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 18.0,
                    color: Color::WHITE,
                },
            ));
        });
    });
}
