use bevy::prelude::*;
use rutt::{SearchContext, GraphSearch};

struct Maze(image::GrayImage);

impl Maze {
    #[inline]
    pub fn is_open(&self, x: u32, y: u32) -> bool {
        let height = self.0.height();
        let width = self.0.width();

        if x >= width || y >= height {
            return false;
        }

        self.0.get_pixel(x, y).0 == [255; 1]
    }
}

// Implement path finding in our litte maze backed by a bitmap
impl<'a> GraphSearch<'a> for Maze {
    type Vertex = (u32, u32);
    type Cost = usize;

    // A* requires a heuristic, here we use manhattan distance
    fn heuristic<'b: 'a>(&'b self, (x1, y1): Self::Vertex, (x2, y2): Self::Vertex) -> Self::Cost {
        let x1 = x1 as i32;
        let x2 = x2 as i32;

        let y1 = y1 as i32;
        let y2 = y2 as i32;

        (((x1 - x2).abs() + (y1 - y2).abs())) as usize
    }

    // Fetch neighbours in a star pattern, make sure we dont go out of bounds and that the slot is open accordning to the bitmap
    fn neighbours<'b: 'a>(
        &'b self,
        (x, y): Self::Vertex,
        neighbours: &mut Vec<(Self::Vertex, Self::Cost)>,
    ) {
        if self.is_open(x.wrapping_sub(1), y) {
            neighbours.push(((x - 1, y), 1));
        }

        if self.is_open(x + 1, y) {
            neighbours.push(((x + 1, y), 1));
        }

        if self.is_open(x, y.wrapping_sub(1)) {
            neighbours.push(((x, y - 1), 1));
        }

        if self.is_open(x, y + 1) {
            neighbours.push(((x, y + 1), 1));
        }
    }
}

struct StartPos((u32, u32));

struct GridParent(Entity);

#[derive(Default)]
struct Sprites {
    traverse: Handle<ColorMaterial>,
}

struct Traverse;
struct Goal;
struct Start;

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Find Demo".to_string(),
            width: 550.0,
            height: 550.0,
            vsync: true,
            resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(set_goal.system().label("set_goal"))
        .add_system(find_goal.system().after("set_goal").label("find_goal"))
        .add_system(set_start.system().after("find_goal"))
        .run();
}

fn set_goal(
    maze: Res<Maze>,
    mut cursor: EventReader<CursorMoved>,
    mut goal: Query<&mut Transform, With<Goal>>,
) {
    if let Some(event) = cursor.iter().last() {
        // Turn windows coord into grid coords
        let pos = (event.position - Vec2::new(0.0, 550.0)).abs();
        let pos = ((pos / 5.0).floor() - Vec2::new(5.0, 5.0)).as_u32();

        if maze.is_open(pos.x, pos.y) {
            let mut goal = goal.single_mut().unwrap();
            let translation = Vec3::new(pos.x as f32 * 5.0, pos.y as f32 * 5.0, goal.translation.z);

            if goal.translation != translation {
                // Make sure change detection dont trigger for nothing
                goal.translation = translation;
            }
        }
    }
}

fn find_goal(
    mut commands: Commands,
    maze: Res<Maze>,
    sprites: Res<Sprites>,
    grid_parent: Res<GridParent>,
    traverse: Query<Entity, With<Traverse>>,
    start: Query<&Transform, With<Start>>,
    goal: Query<&Transform, (With<Goal>, Changed<Transform>)>,

    mut context: Local<SearchContext<(u32, u32), usize>>,
    mut out_buffer: Local<Vec<(u32, u32)>>,
) {
    let goal = match goal.single() {
        Ok(goal) => goal,
        _ => return,
    };

    // Remove the old search
    for entity in traverse.iter() {
        commands.entity(entity).despawn();
    }

    let start = start.single().unwrap();
    let start = start.translation / Vec3::new(5.0, 5.0, 5.0);
    let start = (start.x as u32, start.y as u32);

    let goal = goal.translation / Vec3::new(5.0, 5.0, 5.0);
    let goal = (goal.x as u32, goal.y as u32);

    maze.find_path_with_context(&mut context, start, goal, &mut out_buffer);

    for &(x, y) in out_buffer.iter() {
        commands
            .spawn_bundle(SpriteBundle {
                material: sprites.traverse.clone(),
                transform: Transform::from_xyz(x as f32 * 5.0, y as f32 * 5.0, 100.0),
                ..Default::default()
            })
            .insert(Traverse)
            .insert(Parent(grid_parent.0));
    }
}

fn set_start(
    mut commands: Commands,
    click: Res<Input<MouseButton>>,
    mut start: Query<&mut Transform, (With<Start>, Without<Goal>)>,
    goal: Query<&Transform, With<Goal>>,
    traverse: Query<Entity, With<Traverse>>,
) {
    if click.just_released(MouseButton::Left) {
        // Remove the old search
        for entity in traverse.iter() {
            commands.entity(entity).despawn();
        }

        let mut start = start.single_mut().unwrap();
        let goal = goal.single().unwrap();

        start.translation = goal.translation;
    }
}

// This setup stuff is dirty and quick, don't take any lessions from here :)
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bitmap = image::open("assets/maze.png").unwrap();
    let bitmap = bitmap.to_luma8();

    // Build the grid
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("open_square.png");
    let open_mat = materials.add(texture_handle.into());

    let texture_handle = asset_server.load("closed_square.png");
    let closed_mat = materials.add(texture_handle.into());

    let texture_handle = asset_server.load("traverse.png");
    let traverse_mat = materials.add(texture_handle.into());

    let grid_half = 250.0;

    let mut parent_transform = Transform::from_xyz((-grid_half) + 2.5, grid_half - 2.5, 0.0);
    parent_transform.scale = Vec3::new(1.0, -1.0, 1.0);

    let parent = commands
        .spawn()
        .insert(parent_transform)
        .insert(GlobalTransform::default())
        .id();

    for (x, y, color) in bitmap.enumerate_pixels() {
        let x = x as f32;
        let y = y as f32;

        let transform = Transform::from_xyz(x * 5.0, y * 5.0, 0.0);

        if color.0 == [255; 1] {
            // Open
            commands
                .spawn_bundle(SpriteBundle {
                    material: open_mat.clone(),
                    transform,
                    ..Default::default()
                })
                .insert(Parent(parent));
        } else {
            commands
                .spawn_bundle(SpriteBundle {
                    material: closed_mat.clone(),
                    transform,
                    ..Default::default()
                })
                .insert(Parent(parent));
        }
    }

    commands
        .spawn_bundle(SpriteBundle {
            material: traverse_mat.clone(),
            transform: Transform::from_xyz(50.0 * 5.0, 50.0 * 5.0, 100.0),
            ..Default::default()
        })
        .insert(Start)
        .insert(Parent(parent));

    commands
        .spawn_bundle(SpriteBundle {
            material: traverse_mat.clone(),
            transform: Transform::from_xyz(50.0 * 5.0, 50.0 * 5.0, 100.0),
            ..Default::default()
        })
        .insert(Goal)
        .insert(Parent(parent));

    commands.insert_resource(GridParent(parent));
    commands.insert_resource(Maze(bitmap));
    commands.insert_resource(StartPos((50, 50)));
    commands.insert_resource(Sprites {
        traverse: traverse_mat,
    });
}
