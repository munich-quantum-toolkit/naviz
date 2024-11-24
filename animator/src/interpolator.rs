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

/// An interpolation function which can interpolate values of type `T`.
pub trait InterpolationFunction<T> {
    /// The [Endpoint] of this [InterpolationFunction]
    /// (i.e., whether it will loop back to the start value or not).
    const ENDPOINT: Endpoint = Endpoint::TO;

    /// Interpolate between two values based on the passed normalized `fraction`
    fn interpolate(&self, fraction: Time, from: T, to: T) -> T;
}

/// Constant interpolation
///
/// Will always be the `to`-value
#[derive(Default)]
pub struct Constant();
impl<T> InterpolationFunction<T> for Constant {
    fn interpolate(&self, _fraction: Time, _from: T, to: T) -> T {
        to
    }
}

/// Linear interpolation
///
/// Will interpolate linearly from `from` to `to`
#[derive(Default)]
pub struct Linear();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<T> for Linear {
    fn interpolate(&self, fraction: Time, from: T, to: T) -> T {
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
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<T> for Triangle {
    const ENDPOINT: Endpoint = Endpoint::FROM;

    fn interpolate(&self, fraction: Time, from: T, to: T) -> T {
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
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<T> for Cubic {
    fn interpolate(&self, fraction: Time, from: T, to: T) -> T {
        let fraction = fraction.as_f32();

        let fraction_cubic = if fraction < 0.5 {
            4. * fraction.powi(3)
        } else {
            1. - (-2. * fraction + 2.).powi(3) / 2.
        };

        Linear().interpolate(fraction_cubic.into(), from, to)
    }
}

/// An interpolation-function that is parameterized
/// to allow calculating the time it should take
/// to interpolate from `from` to `to`.
pub trait DurationCalculable<T> {
    fn duration(&self, from: T, to: T) -> f32;
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
trait ConstantJerkImpl {
    fn j0(&self, s_start: f32, s_finish: f32) -> f32;

    fn t_total(&self, s_start: f32, s_finish: f32) -> f32;

    fn s0(&self, s_start: f32, s_finish: f32) -> f32;

    fn v0(&self, s_start: f32, s_finish: f32) -> f32;
}

impl<CJ: ConstantJerkImpl> DurationCalculable<f32> for CJ {
    fn duration(&self, from: f32, to: f32) -> f32 {
        // `t_total` is the total time it takes
        self.t_total(from, to)
    }
}

impl<T: ConstantJerkImpl> InterpolationFunction<f32> for T {
    fn interpolate(&self, fraction: Time, from: f32, to: f32) -> f32 {
        if from > to {
            // from must be `less` than `to`.
            // If not, negate both, then negate result
            return -self.interpolate(fraction, -from, -to);
        }

        let j0 = self.j0(from, to);
        let v0 = self.v0(from, to);
        let s0 = self.s0(from, to);
        let t_total = self.t_total(from, to);
        let t = (fraction.0 - 0.5) * t_total;
        -j0 / 6. * t.powi(3) + v0 * t + s0
    }
}

/// A [ConstantJerkImpl] with a constant, fixed jerk.
pub struct ConstantJerk {
    jerk: f32,
}

impl ConstantJerk {
    /// Creates a new [ConstantJerk] interpolation-function with the specified `jerk`.
    pub fn new(jerk: f32) -> Self {
        Self { jerk: jerk.abs() }
    }
}

impl ConstantJerkImpl for ConstantJerk {
    fn j0(&self, _s_start: f32, _s_finish: f32) -> f32 {
        self.jerk
    }

    fn t_total(&self, s_start: f32, s_finish: f32) -> f32 {
        let j0 = self.j0(s_start, s_finish);
        ((12. * (s_finish - s_start)) / (j0)).powf(1.0 / 3.0)
    }

    fn s0(&self, s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, s_start: f32, s_finish: f32) -> f32 {
        let j0 = self.j0(s_start, s_finish);
        let t_total = self.t_total(s_start, s_finish);
        (j0 * t_total.powi(2)) / 8.
    }
}

/// A [ConstantJerkImpl] with a fixed maximum velocity.
/// The jerk value is calculated from the maximum velocity for a given interpolation.
pub struct ConstantJerkFixedMaxVelocity {
    max_velocity: f32,
}

impl ConstantJerkFixedMaxVelocity {
    pub fn new(max_velocity: f32) -> Self {
        Self { max_velocity }
    }
}

impl ConstantJerkImpl for ConstantJerkFixedMaxVelocity {
    fn j0(&self, s_start: f32, s_finish: f32) -> f32 {
        let v0 = self.v0(s_start, s_finish);
        let t_total = self.t_total(s_start, s_finish);
        v0 * 8. / t_total.powi(2)
    }

    fn t_total(&self, s_start: f32, s_finish: f32) -> f32 {
        let v0 = self.v0(s_start, s_finish);
        3. / 2. * (s_finish - s_start) / v0
    }

    fn s0(&self, s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, _s_start: f32, _s_finish: f32) -> f32 {
        self.max_velocity
    }
}

/// A [ConstantJerkImpl] with a fixed average velocity.
/// The jerk value is calculated from the average velocity for a given interpolation.
pub struct ConstantJerkFixedAverageVelocity {
    average_velocity: f32,
}

impl ConstantJerkFixedAverageVelocity {
    pub fn new(average_velocity: f32) -> Self {
        Self { average_velocity }
    }
}

impl ConstantJerkImpl for ConstantJerkFixedAverageVelocity {
    fn j0(&self, s_start: f32, s_finish: f32) -> f32 {
        let v0 = self.v0(s_start, s_finish);
        let t_total = self.t_total(s_start, s_finish);
        v0 * 8. / t_total.powi(2)
    }

    fn t_total(&self, s_start: f32, s_finish: f32) -> f32 {
        (s_finish - s_start) / self.average_velocity
    }

    fn s0(&self, s_start: f32, s_finish: f32) -> f32 {
        (s_start + s_finish) / 2.
    }

    fn v0(&self, s_start: f32, s_finish: f32) -> f32 {
        let t_total = self.t_total(s_start, s_finish);
        3. / 2. * (s_finish - s_start) / t_total
    }
}

/// Diagonal interpolator for a [Position].
/// Interpolates the direct (diagonal) connection between `from` and `to`.
pub struct Diagonal<I: InterpolationFunction<f32>>(pub I);

impl<I: InterpolationFunction<f32> + DurationCalculable<f32>> DurationCalculable<Position>
    for Diagonal<I>
{
    fn duration(&self, from: Position, to: Position) -> f32 {
        self.0.duration(
            0.,
            ((to.x - from.x).powi(2) + (to.y - from.y).powi(2)).sqrt(),
        )
    }
}

impl<I: InterpolationFunction<f32>> InterpolationFunction<Position> for Diagonal<I> {
    fn interpolate(&self, fraction: Time, from: Position, to: Position) -> Position {
        fn dst(Position { x: x0, y: y0 }: Position, Position { x: x1, y: y1 }: Position) -> f32 {
            ((x0 - x1).powi(2) + (y0 - y1).powi(2)).sqrt()
        }

        // We lay out a new 1-dimensional coordinate-system
        // where `0` is `from` and the axis points to `to`

        // This allows to calculate in 1D-space
        // and get the new position by adding a scaled unit-vector towards `to` to `from`

        // Distance between the points (1D-coordinate of `to`):
        let distance = dst(from, to);
        // Our axis-vector (unit-vector from `from` to `to`):
        let axis = ((to.x - from.x) / distance, (to.y - from.y) / distance);

        // Du to the choice of the coordinate-system, we get the following 1D-positions:
        let s_start = 0.;
        let s_finish = distance;

        // The value in our 1D-coordinate-system
        let s = self.0.interpolate(fraction, s_start, s_finish);

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
pub struct ComponentWise<I: InterpolationFunction<f32> + DurationCalculable<f32>>(pub I);

impl<I: InterpolationFunction<f32> + DurationCalculable<f32>> DurationCalculable<Position>
    for ComponentWise<I>
{
    fn duration(&self, from: Position, to: Position) -> f32 {
        let tx = self.0.duration(from.x.min(to.x), to.x.max(from.x));
        let ty = self.0.duration(from.y.min(to.y), to.y.max(from.y));
        tx.max(ty)
    }
}

impl<I: InterpolationFunction<f32> + DurationCalculable<f32>> InterpolationFunction<Position>
    for ComponentWise<I>
{
    fn interpolate(&self, fraction: Time, from: Position, to: Position) -> Position {
        // The times in x- and y- direction
        let tx = self.0.duration(from.x.min(to.x), to.x.max(from.x));
        let ty = self.0.duration(from.y.min(to.y), to.y.max(from.y));

        // Rescale the shorter times fraction (and clamp to 1)
        let (fx, fy) = if tx < ty {
            ((fraction * ty / tx).min((1.).into()), fraction)
        } else {
            (fraction, (fraction * tx / ty).min((1.).into()))
        };

        // Interpolate components
        let x = self.0.interpolate(fx, from.x, to.x);
        let y = self.0.interpolate(fy, from.y, to.y);

        Position { x, y }
    }
}
