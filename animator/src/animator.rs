use std::{borrow::Cow, collections::VecDeque, sync::Arc};

use fraction::{ConstZero, Fraction};
use naviz_parser::{
    config::{
        machine::MachineConfig,
        visual::{
            LeftRightPosition, OperationConfigConfigConfig, TopBottomPosition, VisualConfig,
            ZoneConfigConfig,
        },
    },
    input::concrete::{Instructions, SetupInstruction, TimedInstruction},
};
use naviz_state::{
    config::{
        AtomsConfig, Config, FontConfig, GridConfig, GridLegendConfig, HPosition, LegendConfig,
        LegendEntry, LegendSection, LineConfig, TimeConfig, TrapConfig, VPosition, ZoneConfig,
    },
    state::{AtomState, State},
};
use regex::Regex;

use crate::{
    color::Color,
    interpolator::{Constant, Cubic, Triangle},
    position::Position,
    timeline::{Time, Timeline},
    to_float::ToFloat,
};

/// The timelines for a single atom
pub struct AtomTimelines {
    position: Timeline<Position, f32, Cubic>,
    overlay_color: Timeline<Color, f32, Triangle>,
    size: Timeline<f32, f32, Triangle>,
    shuttling: Timeline<bool, (), Constant>,
}

impl AtomTimelines {
    /// Creates new AtomTimelines from the passed default values
    pub fn new(position: Position, overlay_color: Color, size: f32, shuttling: bool) -> Self {
        Self {
            position: Timeline::new(position),
            overlay_color: Timeline::new(overlay_color),
            size: Timeline::new(size),
            shuttling: Timeline::new(shuttling),
        }
    }

    /// Gets the values of these timelines at the passed time
    pub fn get(&self, time: Time) -> (Position, Color, f32, bool) {
        (
            self.position.get(time),
            self.overlay_color.get(time),
            self.size.get(time),
            self.shuttling.get(time),
        )
    }
}

/// An atom-state in the animator
struct Atom {
    /// id of the atom
    id: String,
    /// display name (label) of the atom
    name: String,
    /// the timelines of the atom
    timelines: AtomTimelines,
}

/// The animator.
/// Contains the calculated [Atom]-states and static [Config].
///
/// Use [Animator::state] to get the animated state
/// and [Animator::config] to get the [Config].
pub struct Animator {
    atoms: Vec<Atom>,
    config: Arc<Config>,

    /// The total durations of the animations
    duration: Fraction,

    machine: MachineConfig,
    visual: VisualConfig,
}

impl Animator {
    /// Creates the [Animator]: calculates the timelines and the [Config]
    pub fn new(machine: MachineConfig, visual: VisualConfig, input: Instructions) -> Self {
        // Create the atoms
        let mut atoms: Vec<_> = input
            .setup
            .iter()
            .map(|a| match a {
                SetupInstruction::Atom { position, id } => Atom {
                    id: id.clone(),
                    name: get_name(&visual.atom.legend.name, id),
                    timelines: AtomTimelines::new(
                        (*position).into(),
                        Color::default(),
                        visual.atom.radius.f32(),
                        false,
                    ),
                },
            })
            .collect();

        // Convert the `Vec`s to `VecDeque`s to allow popping from front
        let mut instructions: VecDeque<(_, VecDeque<_>)> = input
            .instructions
            .into_iter()
            .map(|(t, i)| (t, i.into()))
            .collect();

        let mut content_size = (Fraction::ZERO, Fraction::ZERO);

        let mut duration_total = Fraction::ZERO;

        // Animate the atoms
        while let Some((time, mut instructions_)) = instructions.pop_front() {
            if let Some((_, offset, instruction)) = instructions_.pop_front() {
                let start_time = time + offset;
                let duration = get_duration(&instruction, &atoms, &machine, start_time);
                let start_time_f32 = start_time.f32();
                let duration_f32 = duration.f32();

                if let Some(position) = get_position(&instruction) {
                    content_size.0 = content_size.0.max(position.0);
                    content_size.1 = content_size.1.max(position.1);
                }

                targeted(&mut atoms, &instruction, start_time, &machine).for_each(|a| {
                    insert_animation(
                        &mut a.timelines,
                        &instruction,
                        start_time_f32,
                        duration_f32,
                        &visual,
                    )
                });

                let next_from_start = instructions_
                    .front()
                    .map(|(x, _, _)| *x)
                    .unwrap_or_default();
                duration_total = duration_total.max(start_time + duration);
                let next_time = if next_from_start {
                    start_time
                } else {
                    start_time + duration
                };
                let idx = instructions.binary_search_by_key(&&next_time, |(t, _)| t);
                let idx = match idx {
                    Ok(idx) => idx,
                    Err(idx) => idx,
                };
                instructions.insert(idx, (next_time, instructions_));
            }
        }

        // The legend entries
        let mut legend_entries = Vec::new();
        if visual.zone.legend.display {
            legend_entries.push(LegendSection {
                name: visual.zone.legend.title.clone(),
                entries: machine
                    .zone
                    .iter()
                    .filter_map(|(id, _)| {
                        get_first_match(&visual.zone.config, id)
                            .filter(|zone| !zone.name.is_empty())
                            .map(|zone| LegendEntry {
                                text: zone.name.clone(),
                                color: Some(zone.color.rgba()),
                            })
                    })
                    .collect(),
            });
        }
        if visual.operation.legend.display {
            legend_entries.push(LegendSection {
                name: visual.operation.legend.title.clone(),
                entries: [
                    (
                        &visual.operation.config.rz.name,
                        visual.operation.config.rz.color,
                    ),
                    (
                        &visual.operation.config.ry.name,
                        visual.operation.config.ry.color,
                    ),
                    (
                        &visual.operation.config.cz.name,
                        visual.operation.config.cz.color,
                    ),
                ]
                .into_iter()
                .filter(|(name, _)| !name.is_empty())
                .map(|(name, color)| LegendEntry {
                    text: name.clone(),
                    color: Some(color.rgba()),
                })
                .collect(),
            });
        }
        if visual.machine.legend.display {
            legend_entries.push(LegendSection {
                name: visual.machine.legend.title.clone(),
                entries: [
                    (&visual.machine.trap.name, visual.atom.trapped.color),
                    (&visual.machine.shuttle.name, visual.atom.shuttling.color),
                ]
                .into_iter()
                .filter(|(name, _)| !name.is_empty())
                .map(|(name, color)| LegendEntry {
                    text: name.clone(),
                    color: Some(color.rgba()),
                })
                .collect(),
            });
        }

        // Create static config
        let config = Config {
            machine: naviz_state::config::MachineConfig {
                grid: GridConfig {
                    step: (
                        visual.coordinate.tick.x.f32(),
                        visual.coordinate.tick.y.f32(),
                    ),
                    line: LineConfig {
                        width: visual.coordinate.tick.line.thickness.f32(),
                        segment_length: visual.coordinate.tick.line.dash.length.f32(),
                        duty: Into::<Fraction>::into(visual.coordinate.tick.line.dash.duty).f32(),
                        color: visual.coordinate.tick.color.rgba(),
                    },
                    legend: GridLegendConfig {
                        step: (
                            visual.coordinate.number.x.distance.f32(),
                            visual.coordinate.number.y.distance.f32(),
                        ),
                        font: FontConfig {
                            size: visual.coordinate.number.font.size.f32(),
                            color: visual.coordinate.number.font.color.rgba(),
                            family: visual.coordinate.number.font.family.to_owned(),
                        },
                        labels: (
                            visual.coordinate.axis.x.clone(),
                            visual.coordinate.axis.y.clone(),
                        ),
                        position: (
                            match visual.coordinate.number.x.position {
                                TopBottomPosition::Bottom => VPosition::Bottom,
                                TopBottomPosition::Top => VPosition::Top,
                            },
                            match visual.coordinate.number.y.position {
                                LeftRightPosition::Left => HPosition::Left,
                                LeftRightPosition::Right => HPosition::Right,
                            },
                        ),
                    },
                },
                traps: TrapConfig {
                    positions: machine
                        .trap
                        .values()
                        .map(|t| (t.position.0.f32(), t.position.1.f32()))
                        .collect(),
                    radius: visual.machine.trap.radius.f32(),
                    line_width: 1., // TODO: this is not configurable currently
                    color: visual.machine.trap.color.rgba(),
                },
                zones: machine
                    .zone
                    .iter()
                    .map(|(id, zone)| {
                        let start: (f32, f32) = (zone.from.0.f32(), zone.from.1.f32());
                        let end: (f32, f32) = (zone.to.0.f32(), zone.to.1.f32());
                        let size = (end.0 - start.0, end.1 - start.1);
                        let default_line = ZoneConfigConfig {
                            color: naviz_parser::common::color::Color {
                                r: 0,
                                g: 0,
                                b: 0,
                                a: 0,
                            },
                            line: naviz_parser::config::visual::LineConfig {
                                dash: naviz_parser::config::visual::DashConfig {
                                    length: Default::default(),
                                    duty: naviz_parser::common::percentage::Percentage(
                                        Default::default(),
                                    ),
                                },
                                thickness: Default::default(),
                            },
                            name: "".to_owned(),
                        };
                        let line =
                            get_first_match(&visual.zone.config, id).unwrap_or(&default_line);
                        ZoneConfig {
                            start,
                            size,
                            line: LineConfig {
                                width: line.line.thickness.f32(),
                                segment_length: line.line.dash.length.f32(),
                                duty: line.line.dash.duty.0.f32(),
                                color: line.color.rgba(),
                            },
                        }
                    })
                    .collect(),
            },
            atoms: AtomsConfig {
                label: FontConfig {
                    size: visual.atom.legend.font.size.f32(),
                    color: visual.atom.legend.font.color.rgba(),
                    family: visual.atom.legend.font.family.to_owned(),
                },
                shuttle: LineConfig {
                    width: visual.machine.shuttle.line.thickness.f32(),
                    segment_length: visual.machine.shuttle.line.dash.length.f32(),
                    duty: Into::<Fraction>::into(visual.machine.shuttle.line.dash.duty).f32(),
                    color: visual.machine.shuttle.color.rgba(),
                },
            },
            content_size: (content_size.0.f32(), content_size.1.f32()),
            legend: LegendConfig {
                font: FontConfig {
                    size: visual.sidebar.font.size.f32(),
                    color: visual.sidebar.font.color.rgba(),
                    family: visual.sidebar.font.family.to_owned(),
                },
                heading_skip: visual.sidebar.font.size.f32() * 1.6,
                entry_skip: visual.sidebar.font.size.f32() * 1.4,
                color_circle_radius: visual.sidebar.font.size.f32() / 2.,
                color_padding: visual.sidebar.font.size.f32() / 2.,
                entries: legend_entries,
            },
            time: TimeConfig {
                font: FontConfig {
                    size: visual.time.font.size.f32(),
                    color: visual.time.font.color.rgba(),
                    family: visual.time.font.family.clone(),
                },
            },
        };

        Self {
            atoms,
            config: Arc::new(config),
            duration: duration_total,
            machine,
            visual,
        }
    }

    /// The calculated [Config]
    pub fn config(&self) -> Arc<Config> {
        self.config.clone()
    }

    /// The total duration of the animations in this [Animator]
    pub fn duration(&self) -> Fraction {
        self.duration
    }

    /// Gets the [State] at the passed [Time]
    pub fn state(&self, time: Time) -> State {
        let time_strings = (&self.visual.time.prefix, &self.machine.time.unit);
        State {
            atoms: self
                .atoms
                .iter()
                .map(
                    |Atom {
                         id: _,
                         name,
                         timelines,
                     }| (timelines.get(time), name),
                )
                .map(
                    |((position, overlay_color, size, shuttling), name)| AtomState {
                        position: position.into(),
                        size,
                        color: overlay_color
                            .over(&if shuttling {
                                self.visual.atom.shuttling.color.into()
                            } else {
                                self.visual.atom.trapped.color.into()
                            })
                            .0,
                        shuttle: shuttling,
                        label: name.clone(),
                    },
                )
                .collect(),
            time: format!("{}{:.1} {}", time_strings.0, time, time_strings.1),
        }
    }

    /// The background color
    pub fn background(&self) -> [u8; 4] {
        self.visual.viewport.color.rgba()
    }
}

/// Extracts the position of a [TimedInstruction],
/// if the instruction has a position,
/// otherwise returns [None].
fn get_position(instruction: &TimedInstruction) -> Option<(Fraction, Fraction)> {
    match instruction {
        TimedInstruction::Load { position, .. } | TimedInstruction::Store { position, .. } => {
            *position
        }
        TimedInstruction::Move { position, .. } => Some(*position),
        _ => None,
    }
}

/// Checks whether an `atom` is in the passed `zone` at the specified `time`.
fn is_in_zone(
    atom: &Atom,
    zone: &naviz_parser::config::machine::ZoneConfig,
    time: Fraction,
) -> bool {
    let position = atom.timelines.position.get(time.f32().into());
    position.x >= zone.from.0.f32()
        && position.y >= zone.from.1.f32()
        && position.x <= zone.to.0.f32()
        && position.y <= zone.to.1.f32()
}

/// Checks whether two positions `a` and `b` are at most `max_distance` apart.
fn is_close(a: &Position, b: &Position, max_distance: Fraction) -> bool {
    let distance_sq = (a.x - b.x).powi(2) + (a.y - b.y).powi(2);
    distance_sq <= max_distance.f32().powi(2)
}

/// Filters the passed `atoms`-slice to only contain the atoms that are targeted
/// by the  passed `instruction` at the specified `start_time` (time the instruction starts)
/// and returns an iterator over all qualifying atoms.
fn targeted<'a>(
    atoms: &'a mut [Atom],
    instruction: &'a TimedInstruction,
    start_time: Fraction,
    machine: &'a MachineConfig,
) -> impl Iterator<Item = &'a mut Atom> {
    enum Match<'a> {
        Id(&'a str),
        IdOrZone {
            id: &'a str,
            zone: &'a naviz_parser::config::machine::ZoneConfig,
        },
        Index(Vec<usize>),
        None,
    }
    let m = match instruction {
        // Instructions that only target individual atoms
        TimedInstruction::Load { id, .. }
        | TimedInstruction::Store { id, .. }
        | TimedInstruction::Move { id, .. } => Match::Id(id),
        // Instructions that target atoms and zones
        TimedInstruction::Rz { id, .. } | TimedInstruction::Ry { id, .. } => machine
            .zone
            .get(id)
            .map_or_else(|| Match::Id(id), |zone| Match::IdOrZone { id, zone }),
        // Instructions that target zones and require interaction distance
        TimedInstruction::Cz { id, .. } => {
            if let Some(zone) = machine.zone.get(id) {
                // Get the position for each atom (identified by index) that is in the zone at the start_time
                let in_zone: Vec<_> = atoms
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| is_in_zone(a, zone, start_time))
                    .map(|(idx, a)| (idx, a.timelines.position.get(start_time.f32().into())))
                    .collect();

                // Generate the targeted atom indices using nested loop.
                // Assuming that at any time only clusters of two atoms exist,
                // at most all atoms will be added to this vector
                // as no atom will be added multiple times in the below loop.
                // Therefore, we preallocate this size
                let mut targeted = Vec::with_capacity(in_zone.len());
                for a in 0..in_zone.len() {
                    for b in (a + 1)..in_zone.len() {
                        let (a, a_pos) = &in_zone[a];
                        let (b, b_pos) = &in_zone[b];
                        // Two atoms are close -> add both
                        if is_close(a_pos, b_pos, machine.distance.interaction) {
                            targeted.push(*a);
                            targeted.push(*b);
                        }
                    }
                }
                Match::Index(targeted)
            } else {
                // Targeted zone does not exist
                Match::None
            }
        }
    };
    atoms
        .iter_mut()
        .enumerate()
        .filter(move |(idx, a)| match &m {
            Match::Id(id) => &a.id == id,
            Match::IdOrZone { id, zone } => &a.id == id || is_in_zone(a, zone, start_time),
            Match::Index(indices) => indices.contains(idx),
            Match::None => false,
        })
        .map(|(_, a)| a)
}

/// Gets the duration of the passed `instruction` when starting at the passed `time`,
fn get_duration(
    instruction: &TimedInstruction,
    atoms: &[Atom],
    machine: &MachineConfig,
    time: Fraction,
) -> Fraction {
    match instruction {
        TimedInstruction::Load { .. } => machine.time.load,
        TimedInstruction::Store { .. } => machine.time.store,
        TimedInstruction::Move { position, id } => {
            // fraction-crate currently has neither `pow` nor `sqrt`, therefore we calculate using `f64`s
            fn dst((x0, y0): (f64, f64), (x1, y1): (f64, f64)) -> f64 {
                ((x0 - x1).powi(2) + (y0 - y1).powi(2)).sqrt()
            }
            (|| {
                let start = atoms
                    .iter()
                    .find(|a| &a.id == id)?
                    .timelines
                    .position
                    .get(time.f32().into());
                let start = (start.x as f64, start.y as f64);
                let end = (position.0.f64(), position.1.f64());
                let distance = dst(start, end);

                let a_up = machine.movement.acceleration.up.f64();
                let a_down = machine.movement.acceleration.down.f64();
                let speed_max = machine.movement.speed.f64();

                // The time until speed_max is reached during speed-up
                // a_up * t = speed_max
                let time_until_speed_max_up = speed_max / a_up;
                // The time from speed_max was reached during speed-down
                // a_down * t = speed_max
                let time_until_speed_max_down = speed_max / a_down;

                // The intersection-time of the start and stop quadratics
                // a_up / 2 * t^2 = distance - a_down / 2 * t^2
                let t_up_down_intersect = (2. * distance / (a_up + a_down)).sqrt();

                if t_up_down_intersect <= time_until_speed_max_up
                    && t_up_down_intersect <= time_until_speed_max_down
                {
                    return Some(2. * t_up_down_intersect);
                }

                // The distance the atom travels at max speed
                let distance_at_max_speed = (distance
                    - a_down / 2. * time_until_speed_max_down.powi(2))
                    - (a_up / 2. * time_until_speed_max_up.powi(2));
                // The time the atoms travels at max speed
                let time_at_max_speed = distance_at_max_speed / speed_max;

                Some(time_until_speed_max_up + time_at_max_speed + time_until_speed_max_down)
            })()
            .map(Fraction::from)
            .unwrap_or_default()
        }
        TimedInstruction::Rz { .. } => machine.time.rz,
        TimedInstruction::Ry { .. } => machine.time.ry,
        TimedInstruction::Cz { .. } => machine.time.cz,
    }
}

/// Inserts an animation for the passed `instruction` into the passed `timelines`
fn insert_animation(
    timelines: &mut AtomTimelines,
    instruction: &TimedInstruction,
    start_time: f32,
    duration: f32,
    visual: &VisualConfig,
) {
    fn add_operation(
        timelines: &mut AtomTimelines,
        time: f32,
        duration: f32,
        config: &OperationConfigConfigConfig,
        visual: &VisualConfig,
    ) {
        timelines
            .overlay_color
            .add((time, duration, config.color.into()));
        timelines
            .size
            .add((time, duration, config.radius.get(visual.atom.radius).f32()));
    }

    fn add_load_store(
        timelines: &mut AtomTimelines,
        time: f32,
        duration: f32,
        load: bool,
        position: Option<(Fraction, Fraction)>,
    ) {
        if load {
            timelines.shuttling.add((time, true));
        } else {
            timelines.shuttling.add((time + duration, false));
        };
        if let Some(position) = position {
            timelines.position.add((time, duration, position.into()));
        }
    }

    match instruction {
        TimedInstruction::Load { position, .. } => {
            add_load_store(timelines, start_time, duration, true, *position);
        }
        TimedInstruction::Store { position, .. } => {
            add_load_store(timelines, start_time, duration, false, *position);
        }
        TimedInstruction::Move { position, .. } => {
            timelines
                .position
                .add((start_time, duration, (*position).into()));
        }
        TimedInstruction::Rz { .. } => {
            add_operation(
                timelines,
                start_time,
                duration,
                &visual.operation.config.rz,
                visual,
            );
        }
        TimedInstruction::Ry { .. } => {
            add_operation(
                timelines,
                start_time,
                duration,
                &visual.operation.config.ry,
                visual,
            );
        }
        TimedInstruction::Cz { .. } => {
            add_operation(
                timelines,
                start_time,
                duration,
                &visual.operation.config.cz,
                visual,
            );
        }
    }
}

/// Gets a name based of an id (from a regex-string-map)
fn get_name(names: &[(Regex, String)], id: &str) -> String {
    names
        .iter()
        .find_map(|(regex, replace)| match regex.replace(id, replace) {
            Cow::Borrowed(_) => None, // borrowed => original input => did not match
            Cow::Owned(n) => Some(n),
        })
        .unwrap_or_default()
}

/// Gets the first item of the passed `input`-map where the id matches the regex.
fn get_first_match<'t, T>(input: &'t [(Regex, T)], id: &str) -> Option<&'t T> {
    input.iter().find(|(r, _)| r.is_match(id)).map(|(_, t)| t)
}
