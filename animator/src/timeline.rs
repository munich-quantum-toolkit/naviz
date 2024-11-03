//! A [Timeline] which contains multiple [Keyframe]s and allows interpolating between them.
//!
//! A [Keyframe]'s [value][Keyframe::value] will start at the [Keyframe]'s [time][Keyframe::time].
//! It will then interpolate for the [Keyframe]'s specified [duration][Keyframe::duration],
//! after which it will be held until the next [Keyframe].
//! If a new [Keyframe] starts while another one would be interpolating,
//! the new [Keyframe] will take precedence
//! (which leads to a jump to the previous [Keyframe]'s [value][Keyframe::value]
//! at the start of the new [Keyframe]).

use std::marker::PhantomData;

use ordered_float::OrderedFloat;

use crate::interpolator::InterpolationFunction;

pub type Time = OrderedFloat<f32>;

// Something which can represent a duration
pub trait Duration {
    /// Convert this duration to [f32]
    fn as_f32(&self) -> f32;
}

impl Duration for f32 {
    fn as_f32(&self) -> f32 {
        *self
    }
}

impl Duration for () {
    fn as_f32(&self) -> f32 {
        0.
    }
}

/// A single keyframe.
///
/// Keyframes are ordered only by their time.
pub struct Keyframe<T, Dur: Duration> {
    /// The start-time of this keyframe
    pub time: Time,
    /// The duration this keyframe will interpolate for
    pub duration: Dur,
    /// The value this keyframe will interpolate to
    pub value: T,
}

impl<T, Dur: Duration> Keyframe<T, Dur> {
    /// The start-time of this keyframe
    pub fn time(&self) -> &Time {
        &self.time
    }

    /// The duration this keyframe will interpolate for
    pub fn duration(&self) -> f32 {
        self.duration.as_f32()
    }

    /// The value this keyframe will interpolate to
    pub fn value(&self) -> &T {
        &self.value
    }
}

impl<T, Dur: Duration> PartialEq for Keyframe<T, Dur> {
    fn eq(&self, other: &Self) -> bool {
        self.time.eq(&other.time)
    }
}

impl<T, Dur: Duration> Eq for Keyframe<T, Dur> {}

impl<T, Dur: Duration> PartialOrd for Keyframe<T, Dur> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, Dur: Duration> Ord for Keyframe<T, Dur> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.time.cmp(&other.time)
    }
}

impl<T> From<(Time, T)> for Keyframe<T, ()> {
    fn from((time, value): (Time, T)) -> Self {
        Self {
            time,
            duration: (),
            value,
        }
    }
}

impl<T> From<(f32, T)> for Keyframe<T, ()> {
    fn from((time, value): (f32, T)) -> Self {
        Self::from((OrderedFloat::from(time), value))
    }
}

impl<T, Dur: Duration> From<(Time, Dur, T)> for Keyframe<T, Dur> {
    fn from((time, duration, value): (Time, Dur, T)) -> Self {
        Self {
            time,
            duration,
            value,
        }
    }
}

impl<T, Dur: Duration> From<(f32, Dur, T)> for Keyframe<T, Dur> {
    fn from((time, duration, value): (f32, Dur, T)) -> Self {
        Self::from((OrderedFloat::from(time), duration, value))
    }
}

/// A timeline which holds many keyframes and specifies the used interpolation function.
pub struct Timeline<T: Copy, Dur: Duration, I: InterpolationFunction<T>> {
    /// The keyframes of this functions.
    /// This vector is expected to be ordered at any time.
    keyframes: Vec<Keyframe<T, Dur>>,
    /// The default value, which is valid from `-Inf` until the first keyframe
    default: T,
    /// The used interpolation function
    interpolation_function: PhantomData<I>,
}

impl<T: Copy + Default, Dur: Duration, I: InterpolationFunction<T>> Default
    for Timeline<T, Dur, I>
{
    fn default() -> Self {
        Self {
            keyframes: Vec::new(),
            default: T::default(),
            interpolation_function: PhantomData,
        }
    }
}

impl<T: Copy, Dur: Duration, I: InterpolationFunction<T>> Timeline<T, Dur, I> {
    /// Creates a new [Timeline] with the passed `default`-value
    pub fn new(default: T) -> Self {
        Self {
            keyframes: Vec::new(),
            default,
            interpolation_function: PhantomData,
        }
    }

    /// Searches the keyframes for the index of the passed time.
    /// See [slice::binary_search] for more information on the return type.
    fn search_time(&self, time: Time) -> Result<usize, usize> {
        self.keyframes.binary_search_by_key(&&time, Keyframe::time)
    }

    /// Gets the index where a keyframe with the passed time may be inserted
    fn get_idx(&self, time: Time) -> usize {
        match self.search_time(time) {
            Ok(idx) => idx,
            Err(idx) => idx,
        }
    }

    /// Finds the index for the keyframe that is active during the passed time.
    /// This does not take the keyframe's duration into account,
    /// meaning that the keyframe may already be finished
    /// (though no other keyframe would be active in that case).
    fn find_idx(&self, time: Time) -> Option<usize> {
        match self.search_time(time) {
            Ok(idx) => Some(idx),
            Err(idx) => {
                if idx > 0 {
                    Some(idx - 1)
                } else {
                    None
                }
            }
        }
    }

    /// Gets the value at the passed time.
    /// Will interpolate the keyframe.
    pub fn get(&self, time: Time) -> T {
        let idx = self.find_idx(time);
        if let Some(idx) = idx {
            let keyframe = &self.keyframes[idx];
            let to = keyframe.value;
            let from = idx
                .checked_sub(1)
                .map(|i| &self.keyframes[i])
                .map(|k| I::ENDPOINT.get(self.default, k.value))
                .unwrap_or(self.default);
            let duration = keyframe.duration.as_f32();
            let fraction = if duration > 0. {
                ((time - keyframe.time) / (Time::from(duration))).min((1.).into())
            } else {
                (1.).into()
            };
            I::interpolate(fraction, from, to)
        } else {
            self.default
        }
    }

    /// Adds a keyframe into this [Timeline]
    pub fn add(&mut self, keyframe: impl Into<Keyframe<T, Dur>>) -> &mut Self {
        let keyframe = keyframe.into();
        self.keyframes.insert(self.get_idx(keyframe.time), keyframe);
        self
    }

    /// Adds multiple keyframes into this [Timeline]
    pub fn add_all(
        &mut self,
        keyframes: impl IntoIterator<Item = impl Into<Keyframe<T, Dur>>>,
    ) -> &mut Self {
        self.keyframes.extend(keyframes.into_iter().map(Into::into));
        self.keyframes.sort();
        self
    }
}
