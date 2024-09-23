use wgpu::{Device, Queue, RenderPass, TextureFormat};

use crate::{
    component::{
        atoms::{AtomSpec, Atoms, AtomsSpec},
        legend::{Legend, LegendEntrySpec, LegendSpec},
        machine::{HPosition, Machine, MachineSpec, VPosition},
        primitive::rectangles::RectangleSpec,
        time::{Time, TimeSpec},
    },
    globals::Globals,
    layout::Layout,
    shaders::{create_composer, load_default_shaders},
    viewport::ViewportSource,
};

/// The specs for the [Renderer]
pub struct RendererSpec<
    'a,
    TrapIterator: IntoIterator<Item = (f32, f32)>,
    ZoneIterator: IntoIterator<Item = RectangleSpec>,
    AtomIterator,
    LegendEntryIterator,
> where
    for<'r> &'r AtomIterator: IntoIterator<Item = &'r AtomSpec<'a>>,
    for<'r> &'r LegendEntryIterator: IntoIterator<Item = &'r (&'a str, &'a [LegendEntrySpec<'a>])>,
{
    pub machine: MachineSpec<'a, TrapIterator, ZoneIterator>,
    pub atoms: AtomsSpec<'a, AtomIterator>,
    pub legend: LegendSpec<'a, LegendEntryIterator>,
    pub time: TimeSpec<'a>,
}

/// The main renderer, which renders the visualization output
pub struct Renderer {
    globals: Globals,

    machine: Machine,
    atoms: Atoms,
    legend: Legend,
    time: Time,
}

impl Renderer {
    /// Creates a new [Renderer] on the passed [Device] and for the passed [TextureFormat]
    pub fn new<
        'a,
        TrapIterator: IntoIterator<Item = (f32, f32)>,
        ZoneIterator: IntoIterator<Item = RectangleSpec>,
        AtomIterator,
        LegendEntryIterator,
    >(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        spec: RendererSpec<'a, TrapIterator, ZoneIterator, AtomIterator, LegendEntryIterator>,
    ) -> Self
    where
        for<'r> &'r AtomIterator: IntoIterator<Item = &'r AtomSpec<'a>>,
        for<'r> &'r LegendEntryIterator:
            IntoIterator<Item = &'r (&'a str, &'a [LegendEntrySpec<'a>])>,
    {
        let mut composer =
            load_default_shaders(create_composer()).expect("Failed to load default shader modules");

        let globals = Globals::new(device);

        Self {
            machine: Machine::new(device, queue, format, &globals, &mut composer, spec.machine),
            atoms: Atoms::new(device, queue, format, &globals, &mut composer, spec.atoms),
            legend: Legend::new(device, queue, format, &globals, &mut composer, spec.legend),
            time: Time::new(device, queue, format, spec.time),
            globals,
        }
    }

    /// Draws the contents of this [Renderer] to the passed [RenderPass]
    pub fn draw<'a>(&'a self, render_pass: &mut RenderPass<'a>) {
        self.rebind(render_pass);

        self.machine.draw::<true>(render_pass, self.rebind_fn());
        self.atoms.draw::<true>(render_pass, self.rebind_fn());
        self.legend.draw::<false>(render_pass, self.rebind_fn()); // No rebind: time does not need globals
        self.time.draw::<false>(render_pass, self.rebind_fn());
    }

    /// A closure which calls [Self::rebind] on `self` with the passed [RenderPass]
    #[inline]
    fn rebind_fn(&self) -> impl Fn(&mut RenderPass) + '_ {
        |r| self.rebind(r)
    }

    /// Rebinds all globals of this renderer
    #[inline]
    fn rebind(&self, render_pass: &mut RenderPass<'_>) {
        self.globals.bind(render_pass);
    }
}

// Random types as generic parameters to allow calling `RenderSpec::example` without specifying types
impl
    RendererSpec<
        'static,
        Vec<(f32, f32)>,
        Vec<RectangleSpec>,
        Vec<AtomSpec<'static>>,
        Vec<(&'static str, &'static [LegendEntrySpec<'static>])>,
    >
{
    /// An example [RendererSpec], which can be used as a default
    pub fn example(
        screen_resolution: (u32, u32),
    ) -> RendererSpec<
        'static,
        impl IntoIterator<Item = (f32, f32)>,
        impl IntoIterator<Item = RectangleSpec>,
        Vec<AtomSpec<'static>>,
        [(&'static str, &'static [LegendEntrySpec<'static>]); 3],
    > {
        let content_size = ViewportSource {
            width: 100.,
            height: 120.,
        };
        let legend_height = 1024.;
        let time_height = 64.;
        let Layout {
            content: content_viewport,
            legend: legend_viewport,
            time: time_viewport,
        } = Layout::new(
            screen_resolution,
            content_size,
            36.,
            legend_height,
            time_height,
        );

        RendererSpec {
            machine: MachineSpec {
                viewport: content_viewport,
                screen_resolution,
                grid_step: (20., 20.),
                grid_color: [127, 127, 127, 255],
                grid_line_width: 1.,
                grid_segment_length: 0.,
                grid_segment_duty: 1.,
                legend_step: (40., 40.),
                legend_font_size: 12.,
                legend_color: [16, 16, 16, 255],
                legend_font: "Fira Mono",
                legend_labels: ("x", "y"),
                legend_position: (VPosition::Bottom, HPosition::Left),
                traps: (0..=7).map(|x| x as f32 * 14.).flat_map(|x| {
                    (0..=1)
                        .map(|y| y as f32 * 17.)
                        .chain((0..=2).map(|y| y as f32 * 17. + 38.))
                        .chain((0..=1).map(|y| y as f32 * 17. + 85.))
                        .map(move |y| (x, y))
                }),
                trap_radius: 3.,
                trap_line_width: 0.5,
                trap_color: [100, 100, 130, 255],
                zones: [
                    RectangleSpec {
                        start: [-10., -10.],
                        size: [120., 36.],
                        color: [0, 122, 255, 255],
                        width: 1.,
                        segment_length: 0.,
                        duty: 1.,
                    },
                    RectangleSpec {
                        start: [-10., 30.],
                        size: [120., 46.],
                        color: [255, 122, 0, 255],
                        width: 1.,
                        segment_length: 0.,
                        duty: 1.,
                    },
                    RectangleSpec {
                        start: [-10., 80.],
                        size: [120., 36.],
                        color: [0, 122, 255, 255],
                        width: 1.,
                        segment_length: 0.,
                        duty: 1.,
                    },
                ],
            },
            atoms: AtomsSpec {
                viewport: content_viewport,
                screen_resolution,
                atoms: (0..=16)
                    .map(|i| i as f32)
                    .map(|i| (i * 2647. % 97., i * 6373. % 113., i * 5407. % 7. > 3.5))
                    .map(|(x, y, s)| AtomSpec {
                        pos: [x, y],
                        size: 3.,
                        color: [255, 128, 32, 255],
                        shuttle: s,
                        label: "5",
                    })
                    .collect::<Vec<_>>(),
                shuttle_color: [180, 180, 180, 255],
                shuttle_line_width: 1.,
                shuttle_segment_length: 10.,
                shuttle_duty: 0.5,
                label_font_size: 5.,
                label_font: "Fira Mono",
                label_color: [0, 0, 0, 255],
            },
            legend: LegendSpec {
                viewport: legend_viewport,
                screen_resolution,
                font_size: 32.,
                text_color: [0, 0, 0, 255],
                font: "Fira Mono",
                heading_skip: 80.,
                entry_skip: 64.,
                color_circle_radius: 16.,
                color_padding: 8.,
                entries: [
                    (
                        "Zones",
                        [
                            LegendEntrySpec {
                                text: "Top",
                                color: Some([0, 122, 255, 255]),
                            },
                            LegendEntrySpec {
                                text: "Middle",
                                color: Some([255, 122, 0, 255]),
                            },
                            LegendEntrySpec {
                                text: "Bottom",
                                color: Some([0, 122, 255, 255]),
                            },
                        ]
                        .as_slice(),
                    ),
                    (
                        "Atoms",
                        [LegendEntrySpec {
                            text: "Atom",
                            color: Some([255, 128, 32, 255]),
                        }]
                        .as_slice(),
                    ),
                    (
                        "Foo",
                        [LegendEntrySpec {
                            text: "Bar",
                            color: None,
                        }]
                        .as_slice(),
                    ),
                ],
            },
            time: TimeSpec {
                viewport: time_viewport,
                screen_resolution,
                font_size: 48.,
                text_color: [0, 0, 0, 255],
                font: "Fira Mono",
                text: "Time: 42 us",
            },
        }
    }
}
