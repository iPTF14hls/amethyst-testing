use crate::utils::Array2d;
use amethyst::{
    assets::{AssetStorage, Handle, Loader},
    core::transform::Transform,
    core::Float,
    ecs::prelude::{Component, DenseVecStorage, Join, ReadStorage, System, WriteStorage},
    prelude::*,
    renderer::{Camera, ImageFormat, SpriteRender, SpriteSheet, SpriteSheetFormat, Texture},
};
use rand::Rng;

use std::f32;
pub struct MyState;

const ARENA_HEIGHT: f32 = 8192.;
const ARENA_WIDTH: f32 = 8192.;

pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

impl Default for Velocity {
    fn default() -> Velocity {
        Velocity { dx: 0., dy: 0. }
    }
}

#[derive(Clone, Copy)]
pub enum BlockState {
    On,
    Off,
}

impl Default for BlockState {
    fn default() -> BlockState {
        BlockState::Off
    }
}

pub struct Block {
    pos: (usize, usize),
}

impl Component for Block {
    type Storage = DenseVecStorage<Self>;
}

pub struct Grid {
    block_size: f32,
    pub blocks: Array2d<BlockState>,
}

impl Component for Grid {
    type Storage = DenseVecStorage<Self>;
}

pub struct GameOfLife;

impl<'s> System<'s> for GameOfLife {
    type SystemData = WriteStorage<'s, Grid>;

    fn run(&mut self, mut grids: Self::SystemData) {
        for grid in (&mut grids).join() {
            let blocks = &mut grid.blocks;
            let dim = blocks.dimensions();
            let mut block_buffer = Array2d::<BlockState>::new(dim);

            let kernel_iter = || {
                (0..9)
                    .map(|i| (i % 3, i / 3))
                    .map(|(x, y)| (x - 1, y - 1))
                    .filter(|(x, y)| (*x) != 0 || (*y) != 0)
            };

            let (w, h) = dim;
            (0..w * h).map(|i| (i % w, i / w)).for_each(|(x, y)| {
                let neighbor_count: u8 = kernel_iter()
                    .map(|(dx, dy)| (dx + (x as isize), dy + (y as isize)))
                    .filter_map(|pos| blocks.try_index((pos.0 as usize, pos.1 as usize)))
                    .map(|state| match state {
                        BlockState::Off => 0,
                        BlockState::On => 1,
                    })
                    .sum();

                match blocks[(x, y)] {
                    BlockState::Off => {
                        match neighbor_count {
                            3 => {
                                block_buffer[(x, y)] = BlockState::On;
                            }
                            _ => {
                                block_buffer[(x, y)] = BlockState::Off;
                            }
                        }
                        if neighbor_count == 3 {
                            block_buffer[(x, y)] = BlockState::On;
                        }
                    }
                    BlockState::On => match neighbor_count {
                        2 | 3 => {
                            block_buffer[(x, y)] = BlockState::On;
                        }
                        _ => {
                            block_buffer[(x, y)] = BlockState::Off;
                        }
                    },
                }
            });

            (0..w * h)
                .map(|i| (i % w, i / w))
                .for_each(|pos|{
                    blocks[pos] = block_buffer[pos];
                });
        }
    }
}

pub struct GridUpdate;

impl<'s> System<'s> for GridUpdate {
    type SystemData = (
        ReadStorage<'s, Block>,
        ReadStorage<'s, Grid>,
        WriteStorage<'s, Transform>,
        WriteStorage<'s, SpriteRender>,
    );
    //TODO: make this code more general
    fn run(&mut self, (blocks, grids, mut transforms, mut rendering): Self::SystemData) {
        //Right now we're just dealing with 1 grid.
        //The code will be refined as developemtn continues.
        let (grid, grid_zero) = {
            let (grid, zero) = (&grids, &mut transforms).join().next().unwrap();
            let mut z_t = *zero.translation();
            z_t.x += (grid.block_size / 2.).into();
            z_t.y += (grid.block_size / 2.).into();
            (grid, z_t)
        };
        for (bloc, trans, sprite) in (&blocks, &mut transforms, &mut rendering).join() {
            let state = grid.blocks[bloc.pos];

            match state {
                BlockState::On => sprite.sprite_number = 1,
                BlockState::Off => sprite.sprite_number = 0,
            }

            let t = trans.translation_mut();
            t.x = Float::from((bloc.pos.0 as f32) * grid.block_size) + grid_zero.x;
            t.y = Float::from((bloc.pos.1 as f32) * grid.block_size) + grid_zero.y;
        }
    }
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

fn initialize_grid(world: &mut World, sprite_sheet: &Handle<SpriteSheet>) {
    let mut transform = Transform::default();
    let mut rng = rand::thread_rng();
    transform.set_translation_xyz(0., 0., 0.);

    let (wid, hei) = (512, 512);
    let len = wid * hei;

    let blocks = {
        let mut bloc = Array2d::new((wid, hei));

        (0..len).map(|i| (i % wid, i / wid)).for_each(|pos| {
            bloc[pos] = match rng.gen_range(0, 2) {
                0 => BlockState::Off,
                _ => BlockState::On,
            };
            initialize_block(world, pos, sprite_sheet);
        });
        bloc
    };

    let grid = Grid {
        block_size: 16.,
        blocks,
    };

    world.create_entity().with(grid).with(transform).build();

    let len = wid * hei;
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

fn initialize_block(world: &mut World, pos: (usize, usize), sprite_sheet: &Handle<SpriteSheet>) {
    let transform = Transform::default();

    let render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(transform)
        .with(render.clone())
        .with(Block { pos })
        .build();
}

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;

        let handle = load_sprite_sheet(world);

        world.register::<Block>();
        world.register::<Grid>();

        initialize_grid(world, &handle);

        initialize_camera(world);
    }
}
