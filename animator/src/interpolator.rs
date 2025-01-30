//! [InterpolationFunction] trait and some interpolation functions.

use std::ops::{Add, Mul};

use crate::{
    position::Position,
    timeline::{Duration, Time},
};

/// The endpoint of an interpolation function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Endpoint {
    /// The interpolation-function ends back at the from-point
    FROM,
    /// The interpolation function ends at the to-point (most interpolation functions)
    #[default]
    TO,
}

impl Endpoint {
    /// Gets the passed argument based on the value of this [Endpoint]
    #[inline]
    pub fn get<T>(self, from: T, to: T) -> T {
        match self {
            Self::FROM => from,
            Self::TO => to,
        }
    }
}

/// An interpolation function which can interpolate values of type `T`,
/// using values of type `A` as arguments.
pub trait InterpolationFunction<A, T> {
    /// The [Endpoint] of this [InterpolationFunction]
    /// (i.e., whether it will loop back to the start value or not).
    const ENDPOINT: Endpoint = Endpoint::TO;

    /// Interpolate between two values based on the passed normalized `fraction`.
    ///
    /// The passed `argument` can be used by this interpolation-functions.
    fn interpolate(&self, fraction: Time, argument: A, from: T, to: T) -> T;
}

/// Constant interpolation
///
/// Will switch from `from` to `to` at the specified point.
/// The time can be specified by the argument:
/// - [()]: Will jump at start of the interpolation (i.e., always be `true`).
/// - [ConstantTransitionPoint]: Will use the [ConstantTransitionPoint] to decide.
/// - [Time]: Will jump at the specified [Time].
#[derive(Default)]
pub struct Constant();

impl<T> InterpolationFunction<(), T> for Constant {
    fn interpolate(&self, _fraction: Time, _argument: (), _from: T, to: T) -> T {
        to
    }
}

/// Optional argument for [Constant] to describe when the constant jump should happen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstantTransitionPoint {
    /// Jump at start of interpolation
    Start,
    /// Jump at end of interpolation
    End,
}

impl<T> InterpolationFunction<ConstantTransitionPoint, T> for Constant {
    fn interpolate(
        &self,
        _fraction: Time,
        transition: ConstantTransitionPoint,
        from: T,
        to: T,
    ) -> T {
        // Depending on transition point, keep the relevant value.
        // When fraction >= 1, the interpolation should not be called anymore,
        // but instead assumes the endpoint, which will always be `to`.
        match transition {
            ConstantTransitionPoint::Start => to,
            ConstantTransitionPoint::End => from,
        }
    }
}

impl<T> InterpolationFunction<Time, T> for Constant {
    fn interpolate(&self, fraction: Time, transition: Time, from: T, to: T) -> T {
        // Change at the specified transition point.
        if fraction >= transition {
            to
        } else {
            from
        }
    }
}

/// Linear interpolation
///
/// Will interpolate linearly from `from` to `to`
#[derive(Default)]
pub struct Linear();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<(), T> for Linear {
    fn interpolate(&self, fraction: Time, _argument: (), from: T, to: T) -> T {
        let fraction = fraction.0;
        from * (1. - fraction) + to * fraction
    }
}

/// Triangle interpolation
///
/// Will interpolate linearly from `from` to `to` in the first half
/// and then back from `to` to `from` in the second half.
/// This will always cycle back to the initial value.
#[derive(Default)]
pub struct Triangle();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<(), T> for Triangle {
    const ENDPOINT: Endpoint = Endpoint::FROM;

    fn interpolate(&self, fraction: Time, _argument: (), from: T, to: T) -> T {
        let mut fraction = fraction.0;
        fraction *= 2.;
        if fraction >= 1. {
            fraction = 1. - (fraction - 1.);
        }

        from * (1. - fraction) + to * fraction
    }
}

/// A cubic interpolation
///
/// Will interpolate from `from` to `to` using cubic functions.
/// Taken from [easings.net][<https://easings.net/#easeInOutCubic>]
#[derive(Default)]
pub struct Cubic();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<(), T> for Cubic {
    fn interpolate(&self, fraction: Time, _argument: (), from: T, to: T) -> T {
        let fraction = fraction.as_f32();

        let fraction_cubic = if fraction < 0.5 {
            4. * fraction.powi(3)
        } else {
            1. - (-2. * fraction + 2.).powi(3) / 2.
        };

        Linear().interpolate(fraction_cubic.into(), (), from, to)
    }
}

/// An interpolation-function that is parameterized
/// to allow calculating the time it should take
/// to interpolate from `from` to `to`.
///
/// The passed `argument` is the same as for the [InterpolationFunction].
pub trait DurationCalculable<A, T> {
    fn duration(&self, argument: A, from: T, to: T) -> f32;
}

/// An interpolation-function which applies constant-jerk movements to [f32]s.
///
/// ## Formulas for constant-jerk
///
/// ### Inputs
///
/// - `s_start`: Starting-position
/// - `s_finish`: End-position
/// - `j0`: constant jerk
///
/// ### Functions
///
/// capitals denote the antiderivatives.
///
/// - Jerk: `j(t) = -j0`
/// - Acceleration: `a(t) = J(t) = -j0 * t`
/// - Velocity: `v(t) = A(t) + v0 = -j0/2 t^2 + v0`
/// - Position: `s(t) = V(t) + s0 = -j0/6 t^3 + v0 t + s0`
///
/// ### Intermediate values
///
/// Intermediate-values (`t_total`, `s0`, `v0`) are calculated from implementors
/// according to the functions below.
/// `t_total` is the total time for the move.
/// It is centered around `0`, meaning the move is from `-t_total/2` to `t_total/2`.
///
/// ### Conditions
///
/// - `s(-t_total/2) = s_start`
/// - `s(t_total/2) = s_finish`
/// - `v(-t_total/2) = 0` (and `v(t_total/2) = 0`, which follows due to symmetry of `v(t)`)
///
/// ## Note
///
/// All functions expect `s_start < s_finish`.
/// Implementations can assume this,
/// callers mus assure this.
trait ConstantJerkImpl<A> {
    fn j0(&self, argument: A, s_start: f32, s_finish: f32) -> f32;

    fn t_total(&self, argument: A, s_start: f32, s_finish: f32) -> f32;

    fn s0(&self, argument: A, s_start: f32, s_finish: f32) -> f32;

    fn v0(&self, argument: A, s_start: f32, s_finish: f32) -> f32;
}

impl<A, CJ: ConstantJerkImpl<A>> DurationCalculable<A, f32> for CJ {
    fn duration(&self, argument: A, from: f32, to: f32) -> f32 {
        // `t_total` is the total time it takes
        self.t_total(argument, from, to)
    }
}

impl<A: Copy, T: ConstantJerkImpl<A>> InterpolationFunction<A, f32> for T {
    fn interpolate(&self, fraction: Time, argument: A, from: f32, to: f32) -> f32 {
        if from > to {
            // from must be `less` than `to`.
            // If not, negate both, then negate result
            return -self.interpolate(fraction, argument, -from, -to);
        }
        if from == to {
            // from = to => no need to interpolate
            // would otherwise lead to division by 0 in some implementors
            return from;
        }

        let j0 = self.j0(argument, from, to);
        let v0 = self.v0(argument, from, to);
        let s0 = self.s0(argument, from, to);
        let t_total = self.t_total(argument, from, to);
        let t = (fraction.0 - 0.5) * t_total;
        -j0 / 6. * t.powi(3) + v0 * t + s0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Jerk(pub f32);

/// A [ConstantJerkImpl] with a constant jerk.
pub struct ConstantJerk();

impl ConstantJerk {
    /// Creates a new [ConstantJerk] interpolation-function with the specified fixed `jerk`.
    pub fn new_fixed(jerk: Jerk) -> FixedArgument<Jerk, Self> {
        FixedArgument {
            argument: jerk,
            interpolator: Self(),
        }
    }
}

impl ConstantJerkImpl<Jerk> for ConstantJerk {
    fn j0(&self, jerk: Jerk, _s_start: f32, _s_finish: f32) -> f32 {
        jerk.0
    }

    fn t_total(&self, jerk: Jerk, s_start: f32, s_finish: f32) -> f32 {
        let j0 = self.j0(jerk, s_start, s_finish);
        ((12. * (s_finish - s_start)) / (j0)).powf(1.0 / 3.0)
    }

    fn s0(&self, _jerk: Jerk, s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, jerk: Jerk, s_start: f32, s_finish: f32) -> f32 {
        let j0 = self.j0(jerk, s_start, s_finish);
        let t_total = self.t_total(jerk, s_start, s_finish);
        (j0 * t_total.powi(2)) / 8.
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct MaxVelocity(pub f32);

/// A [ConstantJerkImpl] with a set maximum velocity.
/// The jerk value is calculated from the maximum velocity for a given interpolation.
pub struct ConstantJerkFixedMaxVelocity();

impl ConstantJerkFixedMaxVelocity {
    pub fn new_fixed(max_velocity: MaxVelocity) -> FixedArgument<MaxVelocity, Self> {
        FixedArgument {
            argument: max_velocity,
            interpolator: Self(),
        }
    }
}

impl ConstantJerkImpl<MaxVelocity> for ConstantJerkFixedMaxVelocity {
    fn j0(&self, max_velocity: MaxVelocity, s_start: f32, s_finish: f32) -> f32 {
        let v0 = self.v0(max_velocity, s_start, s_finish);
        let t_total = self.t_total(max_velocity, s_start, s_finish);
        v0 * 8. / t_total.powi(2)
    }

    fn t_total(&self, max_velocity: MaxVelocity, s_start: f32, s_finish: f32) -> f32 {
        let v0 = self.v0(max_velocity, s_start, s_finish);
        3. / 2. * (s_finish - s_start) / v0
    }

    fn s0(&self, _max_velocity: MaxVelocity, s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, max_velocity: MaxVelocity, _s_start: f32, _s_finish: f32) -> f32 {
        max_velocity.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct AverageVelocity(pub f32);

impl AverageVelocity {
    /// Gets the average velocity for a 2d move from `source` to `destination` in `time`
    pub fn for_2d_move(source: Position, destination: Position, time: f32) -> Self {
        Self(distance(source, destination) / time)
    }
}

/// A [ConstantJerkImpl] with a set average velocity.
/// The jerk value is calculated from the average velocity for a given interpolation.
pub struct ConstantJerkFixedAverageVelocity();

impl ConstantJerkFixedAverageVelocity {
    pub fn new_fixed(average_velocity: AverageVelocity) -> FixedArgument<AverageVelocity, Self> {
        FixedArgument {
            argument: average_velocity,
            interpolator: Self(),
        }
    }

    /// Calculates the jerk for a diagonal move from `from` to `to`
    /// in the passed `duration`.
    pub fn jerk_for_diagonal_move(from: Position, to: Position, duration: f32) -> Jerk {
        let distance = distance(from, to);
        Self::jerk_for_move(0., distance, duration)
    }

    /// Calculates the jerk for a one-dimensional move from `from` to `to`
    /// in the passed `duration`.
    pub fn jerk_for_move(from: f32, to: f32, duration: f32) -> Jerk {
        let distance = (to - from).abs();
        if distance <= 0. {
            return Jerk(0.);
        }
        let average_velocity = AverageVelocity(distance / duration);
        Jerk(Self().j0(average_velocity, 0., distance))
    }
}

/// Implementation using [AverageVelocity].
/// Use this when needing the correct values..
impl ConstantJerkImpl<AverageVelocity> for ConstantJerkFixedAverageVelocity {
    fn j0(&self, average_velocity: AverageVelocity, s_start: f32, s_finish: f32) -> f32 {
        12. * average_velocity.0.powi(3) / (s_finish - s_start).powi(2)
    }

    fn t_total(&self, average_velocity: AverageVelocity, s_start: f32, s_finish: f32) -> f32 {
        (s_finish - s_start) / average_velocity.0
    }

    fn s0(&self, _average_velocity: AverageVelocity, s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, average_velocity: AverageVelocity, _s_start: f32, _s_finish: f32) -> f32 {
        3. / 2. * average_velocity.0
    }
}

/// Implementation using no argument, as the [AverageVelocity] reduces in the final interpolation.
/// Use this for interpolation in the timeline.
impl ConstantJerkImpl<()> for ConstantJerkFixedAverageVelocity {
    fn j0(&self, (): (), s_start: f32, s_finish: f32) -> f32 {
        12. / (s_finish - s_start).powi(2)
    }

    fn t_total(&self, (): (), s_start: f32, s_finish: f32) -> f32 {
        s_finish - s_start
    }

    fn s0(&self, (): (), s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, (): (), _s_start: f32, _s_finish: f32) -> f32 {
        3. / 2.
    }
}

/// Diagonal interpolator for a [Position].
/// Interpolates the direct (diagonal) connection between `from` and `to`.
///
/// Note that while it is possible to wrap any types here,
/// only wrapped [InterpolationFunction]s will make this type be an [InterpolationFunction].
pub struct Diagonal<I>(pub I);

impl<A, I: DurationCalculable<A, f32>> DurationCalculable<A, Position> for Diagonal<I> {
    fn duration(&self, argument: A, from: Position, to: Position) -> f32 {
        self.0.duration(
            argument,
            0.,
            ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt(),
        )
    }
}

impl<A, I: InterpolationFunction<A, f32>> InterpolationFunction<A, Position> for Diagonal<I> {
    fn interpolate(&self, fraction: Time, argument: A, from: Position, to: Position) -> Position {
        // We lay out a new 1-dimensional coordinate-system
        // where `0` is `from` and the axis points to `to`

        // This allows to calculate in 1D-space
        // and get the new position by adding a scaled unit-vector towards `to` to `from`

        // Distance between the points (1D-coordinate of `to`):
        let distance = distance(from, to);
        if distance <= 0. {
            // from = to
            // nothing to interpolate (and unable to construct axis)
            return from;
        }
        // Our axis-vector (unit-vector from `from` to `to`):
        let axis = ((to.x - from.x) / distance, (to.y - from.y) / distance);

        // Du to the choice of the coordinate-system, we get the following 1D-positions:
        let s_start = 0.;
        let s_finish = distance;

        // The value in our 1D-coordinate-system
        let s = self.0.interpolate(fraction, argument, s_start, s_finish);

        // The 1D-coordinate-system translated to the 2D-system using the `axis`
        let delta = (axis.0 * s, axis.1 * s);

        // Add the delta to the position
        Position {
            x: from.x + delta.0,
            y: from.y + delta.1,
        }
    }
}

/// Component-Wise interpolator for a [Position].
/// Interpolates `x`- and `y`-coordinates separately using the same passed interpolator.
/// The duration must be calculable so that each component may take its specified time.
/// The components therefore only take as long as necessary.
///
/// Note that while it is possible to wrap any types here,
/// only wrapped [InterpolationFunction]s will make this type be an [InterpolationFunction].
pub struct ComponentWiseMinTime<I>(pub I);

impl<A: Copy, I: DurationCalculable<A, f32>> DurationCalculable<A, Position>
    for ComponentWiseMinTime<I>
{
    fn duration(&self, argument: A, from: Position, to: Position) -> f32 {
        let tx = self
            .0
            .duration(argument, from.x.min(to.x), to.x.max(from.x));
        let ty = self
            .0
            .duration(argument, from.y.min(to.y), to.y.max(from.y));
        tx.max(ty)
    }
}

impl<A: Copy, I: InterpolationFunction<A, f32> + DurationCalculable<A, f32>>
    InterpolationFunction<A, Position> for ComponentWiseMinTime<I>
{
    fn interpolate(&self, fraction: Time, argument: A, from: Position, to: Position) -> Position {
        // The times in x- and y- direction
        let tx = self
            .0
            .duration(argument, from.x.min(to.x), to.x.max(from.x));
        let ty = self
            .0
            .duration(argument, from.y.min(to.y), to.y.max(from.y));

        // Rescale the shorter times fraction (and clamp to 1)
        let (fx, fy) = if tx < ty {
            ((fraction * ty / tx).min((1.).into()), fraction)
        } else {
            (fraction, (fraction * tx / ty).min((1.).into()))
        };

        // Interpolate components
        let x = self.0.interpolate(fx, argument, from.x, to.x);
        let y = self.0.interpolate(fy, argument, from.y, to.y);

        Position { x, y }
    }
}

/// Component-Wise interpolator for a [Position].
/// Interpolates `x`- and `y`-coordinates separately using the same passed interpolator.
/// The components are both interpreted in the same duration.
///
/// Note that while it is possible to wrap any types here,
/// only wrapped [InterpolationFunction]s will make this type be an [InterpolationFunction].
pub struct ComponentWise<I>(pub I);

impl<A: Copy, I: InterpolationFunction<A, f32>> InterpolationFunction<(A, A), Position>
    for ComponentWise<I>
{
    fn interpolate(
        &self,
        fraction: Time,
        (argument_x, argument_y): (A, A),
        from: Position,
        to: Position,
    ) -> Position {
        // Interpolate components
        let x = self.0.interpolate(fraction, argument_x, from.x, to.x);
        let y = self.0.interpolate(fraction, argument_y, from.y, to.y);

        Position { x, y }
    }
}

/// A wrapper around a Interpolation-Function that fixes the argument
///
/// Note that while it is possible to wrap any types here,
/// only wrapped [InterpolationFunction]s will make this type be an [InterpolationFunction]
/// and only wrapped [DurationCalculable]s will make this type be an [DurationCalculable].
pub struct FixedArgument<A: Copy, I> {
    /// The fixed argument
    pub argument: A,
    /// The interpolator
    pub interpolator: I,
}

impl<A: Copy, T, I: DurationCalculable<A, T>> DurationCalculable<(), T> for FixedArgument<A, I> {
    fn duration(&self, _argument: (), from: T, to: T) -> f32 {
        self.interpolator.duration(self.argument, from, to)
    }
}

impl<A: Copy, T, I: InterpolationFunction<A, T>> InterpolationFunction<(), T>
    for FixedArgument<A, I>
{
    const ENDPOINT: Endpoint = I::ENDPOINT;
    fn interpolate(&self, fraction: Time, _argument: (), from: T, to: T) -> T {
        self.interpolator
            .interpolate(fraction, self.argument, from, to)
    }
}

/// The distance between two positions
fn distance(Position { x: x0, y: y0 }: Position, Position { x: x1, y: y1 }: Position) -> f32 {
    ((x0 - x1).powi(2) + (y0 - y1).powi(2)).sqrt()
}
