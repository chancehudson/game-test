use bevy::prelude::*;

// Event for incoming messages
#[derive(Event, Debug)]
pub struct InfoMessage(pub String);

#[derive(Component)]
pub struct InfoLine {
    timer: Timer,
}

impl InfoLine {
    fn new(duration_secs: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs, TimerMode::Once),
        }
    }
}

pub struct InfoTextPlugin;

impl Plugin for InfoTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InfoMessage>()
            .add_systems(FixedUpdate, (read_info, update_info_timers));
    }
}

fn read_info(
    mut info_events: EventReader<InfoMessage>,
    mut existing_messages: Query<&mut Node, With<InfoLine>>,
    mut commands: Commands,
) {
    for event in info_events.read() {
        let message = &event.0;
        for mut node in &mut existing_messages {
            node.bottom = match node.bottom {
                Val::Px(v) => Val::Px(v + 12.0),
                _ => unreachable!(),
            };
        }
        commands
            .spawn((
                InfoLine::new(3.0),
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(50.),
                    bottom: Val::Px(50.),
                    ..default()
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text(message.clone()),
                    BackgroundColor(Color::linear_rgba(0., 0., 0., 0.2)),
                    TextLayout {
                        justify: JustifyText::Right,
                        ..default()
                    },
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                ));
            });
    }
}

// System to handle auto-disappearing
fn update_info_timers(
    time: Res<Time>,
    mut commands: Commands,
    mut info_lines: Query<(Entity, &mut InfoLine)>,
) {
    for (entity, mut info_line) in info_lines.iter_mut() {
        info_line.timer.tick(time.delta());

        if info_line.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
