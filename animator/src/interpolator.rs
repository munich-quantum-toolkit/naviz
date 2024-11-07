//! [InterpolationFunction] trait and some interpolation functions.

use std::ops::{Add, Mul};

use crate::timeline::{Duration, Time};

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
    fn interpolate(fraction: Time, from: T, to: T) -> T;
}

/// Constant interpolation
///
/// Will always be the `to`-value
pub struct Constant();
impl<T> InterpolationFunction<T> for Constant {
    fn interpolate(_fraction: Time, _from: T, to: T) -> T {
        to
    }
}

/// Linear interpolation
///
/// Will interpolate linearly from `from` to `to`
pub struct Linear();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<T> for Linear {
    fn interpolate(fraction: Time, from: T, to: T) -> T {
        let fraction = fraction.0;
        from * (1. - fraction) + to * fraction
    }
}

/// Triangle interpolation
///
/// Will interpolate linearly from `from` to `to` in the first half
/// and then back from `to` to `from` in the second half.
/// This will always cycle back to the initial value.
pub struct Triangle();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<T> for Triangle {
    const ENDPOINT: Endpoint = Endpoint::FROM;

    fn interpolate(fraction: Time, from: T, to: T) -> T {
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
pub struct Cubic();
impl<T: Mul<f32, Output = I>, I: Add<Output = T>> InterpolationFunction<T> for Cubic {
    fn interpolate(fraction: Time, from: T, to: T) -> T {
        let fraction = fraction.as_f32();

        let fraction_cubic = if fraction < 0.5 {
            4. * fraction.powi(3)
        } else {
            1. - (-2. * fraction + 2.).powi(3) / 2.
        };

        Linear::interpolate(fraction_cubic.into(), from, to)
    }
}
