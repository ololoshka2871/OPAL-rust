use core::marker;

/// Field reader.
///
/// Result of the `read` methods of fields.
pub struct FieldReader<U, T> {
    pub(crate) bits: U,
    _reg: marker::PhantomData<T>,
}

impl<U, T> FieldReader<U, T>
where
    U: Copy,
{
    /// Creates a new instance of the reader.
    #[allow(unused)]
    #[inline(always)]
    pub(crate) fn new(bits: U) -> Self {
        Self {
            bits,
            _reg: marker::PhantomData,
        }
    }

    /// Reads raw bits from field.
    #[inline(always)]
    pub fn bits(&self) -> U {
        self.bits
    }
}

impl<U, T, FI> PartialEq<FI> for FieldReader<U, T>
where
    U: PartialEq,
    FI: Copy + Into<U>,
{
    #[inline(always)]
    fn eq(&self, other: &FI) -> bool {
        self.bits.eq(&(*other).into())
    }
}

impl<FI> FieldReader<bool, FI> {
    /// Value of the field as raw bits.
    #[inline(always)]
    pub fn bit(&self) -> bool {
        self.bits
    }
    /// Returns `true` if the bit is clear (0).
    #[inline(always)]
    pub fn bit_is_clear(&self) -> bool {
        !self.bit()
    }
    /// Returns `true` if the bit is set (1).
    #[inline(always)]
    pub fn bit_is_set(&self) -> bool {
        self.bit()
    }
}
