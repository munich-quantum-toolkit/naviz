use fraction::Fraction;

/// Something that can be converted into a float ([f32] or [f64]).
pub(crate) trait ToFloat {
    /// Converts this value to a [f32]
    #[allow(unused)]
    fn f32(self) -> f32;
    /// Converts this value to a [f64]
    #[allow(unused)]
    fn f64(self) -> f64;
}

/// Convert [Fraction]s to floats.
/// Will first try to use [TryInto].
/// If that does not return a valid value,
/// it will convert the [Fraction] into a [String] and parse that back into a float
/// (which should always succeed; otherwise it will panic).
impl ToFloat for Fraction {
    fn f32(self) -> f32 {
        self.try_into().unwrap_or_else(|_| {
            self.to_string()
                .parse()
                .expect("Fraction produced an invalid number representation")
        })
    }

    fn f64(self) -> f64 {
        self.try_into().unwrap_or_else(|_| {
            self.to_string()
                .parse()
                .expect("Fraction produced an invalid number representation")
        })
    }
}
