use bevy::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const ACTIVE_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
    ));
    commands.spawn((
        container(),
        children![
            button("Selection".to_string(), true),
            button("Box Tool".to_string(), false)
        ],
    ));
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}


fn container() -> Node {
    Node {
        padding: UiRect::all(Val::Px(10.0)),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(10.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexEnd,
        ..default()
    }
}

fn button(text: String, is_active: bool) -> impl Bundle + use<> {
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
            Text::new(text),
            TextFont {
                font_size: 18.0,
                ..default()
            },
            text_color(is_active),
        )],
    )
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
