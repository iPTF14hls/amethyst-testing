use amethyst::{
    assets::{AssetStorage, Handle, Loader, Processor},
    ecs::prelude::{Component, DenseVecStorage, ReadExpect, Resources, SystemData},
    core::transform::{Transform, TransformBundle},
    prelude::*,
    renderer::{
        pass::DrawShadedDesc,
        SpriteRender,
        rendy::{
            factory::Factory,
            graph::{
                render::{RenderGroupDesc, SubpassBuilder},
                GraphBuilder,
            },
            hal::{format::Format, image},
        },
        types::DefaultBackend,
        Camera, GraphCreator, ImageFormat, RenderingSystem, SpriteSheet, SpriteSheetFormat,
        Texture,
    },
    utils::application_root_dir,
    window::{ScreenDimensions, Window, WindowBundle},
};

struct MyState;

pub const ARENA_HEIGHT: f32 = 100.;
pub const ARENA_WIDTH: f32 = 100.;

struct Square {
    size: f32,
}

impl Component for Square {
    type Storage = DenseVecStorage<Self>;
}

fn load_sprite_sheet(world: &mut World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();

        loader.load(
            "textures/box.png",
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
    transform.set_translation_xyz(10., 10., 0.);

    let render = SpriteRender {
        sprite_sheet: sprite_sheet.clone(),
        sprite_number: 0,
    };

    world
        .create_entity()
        .with(render.clone())
        .with(Square { size: 10. })
        .with(transform)
        .build();
}

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.register::<Square>();

        let handle = load_sprite_sheet(world);
        initialize_camera(world);
        initialize_square(world, handle);

    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources_dir = app_root.join("resources");
    let display_config_path = resources_dir.join("display_config.ron");

    let game_data = GameDataBuilder::default()
        .with(Processor::<SpriteSheet>::new(),
        "sprite_sheet_processor",
        &[],)
        .with_bundle(WindowBundle::from_config_path(display_config_path))?
        .with_bundle(TransformBundle::new())?
        .with_thread_local(RenderingSystem::<DefaultBackend, _>::new(
            RenderingGraph::default(),
        ));

    let mut game = Application::new(resources_dir, MyState, game_data)?;
    game.run();

    Ok(())
}

#[derive(Default)]
struct RenderingGraph {
    dimensions: Option<ScreenDimensions>,
    surface_format: Option<Format>,
    dirty: bool,
}

impl GraphCreator<DefaultBackend> for RenderingGraph {
    fn rebuild(&mut self, res: &Resources) -> bool {
        // Rebuild when dimensions change, but wait until at least two frames have the same.
        let new_dimensions = res.try_fetch::<ScreenDimensions>();
        use std::ops::Deref;
        if self.dimensions.as_ref() != new_dimensions.as_ref().map(|d| d.deref()) {
            self.dirty = true;
            self.dimensions = new_dimensions.map(|d| d.clone());
            return false;
        }

        self.dirty
    }

    fn builder(
        &mut self,
        factory: &mut Factory<DefaultBackend>,
        res: &Resources,
    ) -> GraphBuilder<DefaultBackend, Resources> {
        use amethyst::renderer::rendy::{
            graph::present::PresentNode,
            hal::command::{ClearDepthStencil, ClearValue},
        };

        self.dirty = false;
        let window = <ReadExpect<'_, Window>>::fetch(res);
        let surface = factory.create_surface(&window);
        // cache surface format to speed things up
        let surface_format = *self
            .surface_format
            .get_or_insert_with(|| factory.get_surface_format(&surface));
        let dimensions = self.dimensions.as_ref().unwrap();
        let window_kind =
            image::Kind::D2(dimensions.width() as u32, dimensions.height() as u32, 1, 1);

        let mut graph_builder = GraphBuilder::new();
        let color = graph_builder.create_image(
            window_kind,
            1,
            surface_format,
            Some(ClearValue::Color([0.34, 0.36, 0.52, 1.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            Format::D32Sfloat,
            Some(ClearValue::DepthStencil(ClearDepthStencil(1.0, 0))),
        );

        let opaque = graph_builder.add_node(
            SubpassBuilder::new()
                .with_group(DrawShadedDesc::new().builder())
                .with_color(color)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        let _present = graph_builder
            .add_node(PresentNode::builder(factory, surface, color).with_dependency(opaque));

        graph_builder
    }
}
