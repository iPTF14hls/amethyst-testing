use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::transform::Transform,
    ecs::prelude::{Join, Component, DenseVecStorage, System, ReadStorage, WriteStorage},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
    core::{Float, math::Vector3},
};

pub struct MyState;

const ARENA_HEIGHT: f32 = 256.;
const ARENA_WIDTH: f32 = 256.;

pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

impl Default for Velocity {
    fn default() -> Velocity {
        Velocity { dx: 0., dy: 0. }
    }
}

pub struct Spinner {
    pub center: (f32, f32),
    pub theta: f32,
    pub mag: f32,
}

impl Spinner {
    fn new(center: (f32, f32), mag: f32) -> Spinner {
        Spinner {
            center,
            mag,
            theta: 0.,
        }
    }
}

pub struct SpinnerSystem;

impl<'s> System<'s> for SpinnerSystem {
    type SystemData = (
        WriteStorage<'s, Transform>,
        WriteStorage<'s, Spinner>,
    );

    fn run(&mut self, (mut transforms, mut progress): Self::SystemData) {
        for (pos, prog) in (&mut transforms, &mut progress).join() {
            let (dx, dy) = (prog.theta.cos(), prog.theta.sin());
            let (dx, dy) = (dx*prog.mag, dy*prog.mag);
            let trans = pos.translation_mut();
            trans.x = Float::from_f32(prog.center.0 + dx);
            trans.y = Float::from_f32(prog.center.1 + dy);

            prog.theta += 0.01;
        }
    }
}

impl Component for Spinner {
    type Storage = DenseVecStorage<Self>;
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();

        loader.load(
            "textures/box_spritesheet.png",
            ImageFormat::default(),
            (),
            &texture_storage,
        )
    };

    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

    loader.load(
        "textures/box_spritesheet.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store,
    )
}

fn initialize_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5, 1.);

    world
        .create_entity()
        .with(Camera::standard_2d(ARENA_WIDTH, ARENA_HEIGHT))
        .with(transform)
        .build();
}

fn initialize_square(world: &mut World, sprite_sheet: Handle<SpriteSheet>) {
    let mut transform = Transform::default();
    let cen = (ARENA_WIDTH * 0.5, ARENA_HEIGHT * 0.5);
    transform.set_translation_xyz(cen.0 , cen.1, 0.);
    //transform.set_scale(Vector3::new(10., 10., 1.));

    let render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(render.clone())
        .with(Spinner::new(cen, 50.))
        .with(transform)
        .build();
}

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let handle = load_sprite_sheet(world);

        world.register::<Spinner>();

        initialize_square(world, handle);
        initialize_camera(world);

    }
}