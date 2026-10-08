use std::{fmt, str::FromStr};

use thiserror::Error;

/// Exact, unsigned fixed point with twelve decimal places. Not an exchange tick validator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Decimal(u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DecimalError {
    #[error("invalid unsigned decimal syntax")]
    Syntax,
    #[error("more than twelve fractional digits")]
    Precision,
    #[error("decimal arithmetic overflow")]
    Overflow,
    #[error("price or quantity must be positive")]
    Zero,
    #[error("price is not an exact multiple of the supplied tick")]
    OffTick,
}

impl Decimal {
    pub const SCALE: u128 = 1_000_000_000_000;
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(Self::SCALE);

    pub const fn atoms(self) -> u128 {
        self.0
    }

    pub(crate) const fn from_atoms(atoms: u128) -> Self {
        Self(atoms)
    }

    pub fn checked_product_ceil(self, rhs: Self) -> Result<Self, DecimalError> {
        let product = self.0.checked_mul(rhs.0).ok_or(DecimalError::Overflow)?;
        Ok(Self(product.div_ceil(Self::SCALE)))
    }

    pub fn abs_diff(self, other: Self) -> Self {
        Self(self.0.abs_diff(other.0))
    }
}

impl FromStr for Decimal {
    type Err = DecimalError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.is_empty() || text.len() > 52 {
            return Err(DecimalError::Syntax);
        }
        let mut mantissa = 0_u128;
        let mut fraction = None;
        let mut whole_digits = 0;
        for byte in text.bytes() {
            match byte {
                b'0'..=b'9' => {
                    mantissa = mantissa
                        .checked_mul(10)
                        .and_then(|v| v.checked_add(u128::from(byte - b'0')))
                        .ok_or(DecimalError::Overflow)?;
                    if let Some(ref mut digits) = fraction {
                        *digits += 1;
                        if *digits > 12 {
                            return Err(DecimalError::Precision);
                        }
                    } else {
                        whole_digits += 1;
                    }
                }
                b'.' if fraction.is_none() && whole_digits > 0 => fraction = Some(0_u32),
                _ => return Err(DecimalError::Syntax),
            }
        }
        if fraction == Some(0) {
            return Err(DecimalError::Syntax);
        }
        let factor = 10_u128.pow(12 - fraction.unwrap_or(0));
        Ok(Self(
            mantissa.checked_mul(factor).ok_or(DecimalError::Overflow)?,
        ))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let whole = self.0 / Self::SCALE;
        let fraction = self.0 % Self::SCALE;
        if fraction == 0 {
            write!(f, "{whole}")
        } else {
            let fraction = format!("{fraction:012}");
            write!(f, "{whole}.{}", fraction.trim_end_matches('0'))
        }
    }
}

macro_rules! positive_type {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name(Decimal);

        impl $name {
            pub fn new(value: Decimal) -> Result<Self, DecimalError> {
                if value == Decimal::ZERO {
                    Err(DecimalError::Zero)
                } else {
                    Ok(Self(value))
                }
            }

            pub const fn decimal(self) -> Decimal {
                self.0
            }
        }

        impl FromStr for $name {
            type Err = DecimalError;
            fn from_str(text: &str) -> Result<Self, Self::Err> {
                Self::new(text.parse()?)
            }
        }
    };
}

positive_type!(Price);
positive_type!(Quantity);

impl Price {
    /// Caller must obtain a valid price quantum from independently validated instrument rules.
    pub fn exact_ticks(self, quantum: Price) -> Result<u128, DecimalError> {
        let price = self.decimal().atoms();
        let step = quantum.decimal().atoms();
        if !price.is_multiple_of(step) {
            return Err(DecimalError::OffTick);
        }
        Ok(price / step)
    }
}
