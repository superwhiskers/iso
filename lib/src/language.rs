//
// iso - implementations of datatypes related to common iso standards
// Copyright (c) 2021 superwhiskers <whiskerdev@protonmail.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
//

//TODO(superwhiskers): add more tests

//! Type definitions related to the ISO 639 language code standard
//!
//! Most of this module is dedicated to the code enumerations, which each enumerate over their
//! corresponding iso language code sets. Each can be converted into each other, provided that they
//! have a corresponding language code in the other set.
//!
//! # Basic usage
//!
//! ```
//! # use iso::language::{Iso639_1, Iso639_2b, Iso639_2t, Iso639_3, Language};
//! # use std::convert::TryInto;
//! // the variant representing the english language
//! let english = Iso639_1::En;
//!
//! println!(
//!     "The name of the language represented by the ISO 639-3 code of {} is {}!",
//!     TryInto::<Iso639_3>::try_into(english).unwrap().code(),
//!     english.name()
//! );
//!
//! assert_eq!(english.name(), "English");
//! assert_eq!(english.code(), "en");
//! assert_eq!(english.try_into(), Ok(Iso639_1::En));
//! assert_eq!(english.try_into(), Ok(Iso639_2b::Eng));
//! assert_eq!(english.try_into(), Ok(Iso639_2t::Eng));
//! assert_eq!(english.try_into(), Ok(Iso639_3::Eng));
//! ```

use core::{
    convert::TryFrom,
    fmt,
    str::{self, FromStr},
};
use iso_macro::language_identifiers_from_table;

#[cfg(feature = "std")]
use std::error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A list of all possible errors encountered while working with the language code enumerations
#[non_exhaustive]
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Error {
    /// An error returned when the provided language code is invalid
    InvalidLanguageCode(String),

    /// An error returned when there is no corresponding language code in the target code set
    NoCorrespondingLanguageCode(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::InvalidLanguageCode(c) => {
                formatter.write_str("`")?;
                formatter.write_str(c)?;
                formatter.write_str("` is an invalid language code")
            }
            Self::NoCorrespondingLanguageCode(c) => {
                formatter.write_str("`")?;
                formatter.write_str(c)?;
                formatter.write_str("` has no corresponding language code")
            }
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {}

/// An abstraction over a language code providing ways to extract information as well as convert
pub trait Language {
    /// Returns the language's name
    fn name(&self) -> &'static str;

    /// Returns the language's corresponding language code as a `&str` based on the underlying format
    fn code(&self) -> &'static str;
}

//TODO(superwhiskers): consider making this into a derive macro when Copy and Clone can be used within a constant context
macro_rules! language_impl {
    ($language:ident, $language_as_string:literal) =>  {
        impl Language for $language {
            fn name(&self) -> &'static str {
                language_identifiers_from_table!(match &self: $language => "name")
            }

            fn code(&self) -> &'static str {
                language_identifiers_from_table!(match &self: $language => $language_as_string)
            }
        }

        impl fmt::Display for $language {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.name())
            }
        }

        impl FromStr for $language {
            type Err = Error;

            fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
                language_identifiers_from_table!(match s: $language_as_string => $language).ok_or(Error::InvalidLanguageCode(s.to_string()))
            }
        }
    }
}

macro_rules! language_impl_try_from {
    ($from:ident, $to:ident) => {
        impl TryFrom<$from> for $to {
            type Error = Error;

            fn try_from(c: $from) -> Result<Self, <Self as TryFrom<$from>>::Error> {
                language_identifiers_from_table!(match c: $from => $to).ok_or(Error::NoCorrespondingLanguageCode(c.code()))
            }
        }
    }
}

language_identifiers_from_table!(enum Iso639_1: iso639_1);
language_impl!(Iso639_1, "Iso639_1");
language_impl_try_from!(Iso639_2b, Iso639_1);
language_impl_try_from!(Iso639_2t, Iso639_1);
language_impl_try_from!(Iso639_3, Iso639_1);

language_identifiers_from_table!(enum Iso639_2b: iso639_2b);
language_impl!(Iso639_2b, "Iso639_2b");
language_impl_try_from!(Iso639_1, Iso639_2b);
language_impl_try_from!(Iso639_2t, Iso639_2b);
language_impl_try_from!(Iso639_3, Iso639_2b);

language_identifiers_from_table!(enum Iso639_2t: iso639_2t);
language_impl!(Iso639_2t, "Iso639_2t");
language_impl_try_from!(Iso639_1, Iso639_2t);
language_impl_try_from!(Iso639_2b, Iso639_2t);
language_impl_try_from!(Iso639_3, Iso639_2t);

language_identifiers_from_table!(enum Iso639_3: iso639_3);
language_impl!(Iso639_3, "Iso639_3");
language_impl_try_from!(Iso639_1, Iso639_3);
language_impl_try_from!(Iso639_2b, Iso639_3);
language_impl_try_from!(Iso639_2t, Iso639_3);
