//
// iso - implementations of datatypes related to common iso standards
//
// copyright (c) 2021 superwhiskers <whiskerdev@protonmail.com>
// this source code form is subject to the terms of the mozilla public
// license, v. 2.0. if a copy of the mpl was not distributed with this
// file, you can obtain one at http://mozilla.org/MPL/2.0/.
//

//TODO(superwhiskers): add more tests

//! Type definitions related to the ISO 3166-1 country code standard
//!
//! Most of this module is dedicated to the code enumerations, which each enumerate over their
//! corresponding iso country code sets. Each can be converted into each other.
//!
//! # Basic usage
//!
//! ```
//! # use iso::country::{Iso3166_1_alpha_2, Iso3166_1_alpha_3, Country};
//! # use std::convert::TryInto;
//! // the variant representing the united states
//! let united_states = Iso3166_1_alpha_2::Us;
//!
//! println!(
//!     "The name of the country represented by the ISO 3166-1 alpha-3 code of {} is {}!",
//!     TryInto::<Iso3166_1_alpha_3>::try_into(united_states).unwrap().code(),
//!     united_states.name()
//! );
//!
//! assert_eq!(united_states.name(), "United States of America");
//! assert_eq!(united_states.code(), "US");
//! assert_eq!(united_states.try_into(), Ok(Iso3166_1_alpha_2::Us));
//! assert_eq!(united_states.try_into(), Ok(Iso3166_1_alpha_3::Usa));
//! ```

use core::{
    convert::TryFrom,
    fmt,
    str::{self, FromStr},
};
use iso_macro::country_identifiers_from_table;

#[cfg(feature = "std")]
use std::error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A list of all possible errors encountered while working with the country code enumerations
#[non_exhaustive]
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Error {
    /// An error returned when the provided country code is invalid
    InvalidCountryCode(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InvalidCountryCode(c) => {
                formatter.write_str("`")?;
                formatter.write_str(c)?;
                formatter.write_str("` is an invalid country code")
            }
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {}

/// An abstraction over a country code providing ways to extract information as well as convert
pub trait Country {
    /// Returns the country's name
    fn name(&self) -> &'static str;

    /// Returns the country's ISO 3166-1 numeric code
    fn numeric(&self) -> u16;

    /// Returns the country's corresponding langugae code as a `&str` based on the underlying format
    fn code(&self) -> &'static str;
}

//TODO: consider making this into a derive macro like what is said in the language file
macro country_impl($country:ident, $country_as_string:literal) {
    impl Country for $country {
        fn name(&self) -> &'static str {
            country_identifiers_from_table!(match &self: $country => "name")
        }

        fn numeric(&self) -> u16 {
            country_identifiers_from_table!(match &self: $country => "Iso3166_1_numeric")
        }

        fn code(&self) -> &'static str {
            country_identifiers_from_table!(match &self: $country => $country_as_string)
        }
    }

    impl fmt::Display for $country {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.name())
        }
    }

    impl TryFrom<u16> for $country {
        type Error = Error;

        fn try_from(c: u16) -> Result<Self, <Self as TryFrom<u16>>::Error> {
            country_identifiers_from_table!(match c: "Iso3166_1_numeric" => $country).ok_or(Error::InvalidCountryCode(c.to_string()))
        }
    }

    impl FromStr for $country {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
            country_identifiers_from_table!(match s: $country_as_string => $country).ok_or(Error::InvalidCountryCode(s.to_string()))
        }
    }
}

macro country_impl_from($from:ident, $to:ident) {
    impl From<$from> for $to {
        fn from(c: $from) -> Self {
            country_identifiers_from_table!(match c: $from => $to)
        }
    }
}

country_identifiers_from_table!(enum Iso3166_1_alpha_2: iso3166_1_alpha_2);
country_impl!(Iso3166_1_alpha_2, "Iso3166_1_alpha_2");
country_impl_from!(Iso3166_1_alpha_3, Iso3166_1_alpha_2);

country_identifiers_from_table!(enum Iso3166_1_alpha_3: iso3166_1_alpha_3);
country_impl!(Iso3166_1_alpha_3, "Iso3166_1_alpha_3");
country_impl_from!(Iso3166_1_alpha_2, Iso3166_1_alpha_3);
