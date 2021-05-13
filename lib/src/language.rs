//
// iso - implementations of datatypes related to common iso standards
//
// copyright (c) 2021 superwhiskers <whiskerdev@protonmail.com>
// this source code form is subject to the terms of the mozilla public
// license, v. 2.0. if a copy of the mpl was not distributed with this
// file, you can obtain one at http://mozilla.org/MPL/2.0/.
//

//! Type definitions related to thie ISO 639 language code standard
//!
//! Most of this module is dedicated to the [`Code`] enumeration, which enumerates over all ISO 639
//! alpha-2 codes. However, it is also possible to derive one from its alpha-3 t and alpha-3 b
//! codes, if wished.
//!
//! # Basic usage
//!
//! ```
//! # use iso::language::{Iso639_1, Iso639_2b, Iso639_2t, Iso639_3, Language};
//! # use std::convert::TryInto;
//! // the variant representing the english language
//! let english = Iso639_1::En;
//!
//! println!("The name of the language represented by the ISO 639-3 code of {:?} is {}!", TryInto::<Iso639_3>::try_into(english), english.name());
//!
//! assert_eq!(english.name(), "English");
//! assert_eq!(english.code(), "en");
//! assert_eq!(english.try_into(), Ok(Iso639_1::En));
//! assert_eq!(english.try_into(), Ok(Iso639_2b::Eng));
//! assert_eq!(english.try_into(), Ok(Iso639_2t::Eng));
//! assert_eq!(english.try_into(), Ok(Iso639_3::Eng));
//! ```
//!
//! [`Code`]: ./enum.Code.html

use core::{fmt, str};

use iso_macro::identifiers_from_table;

#[cfg(feature = "std")]
use std::error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;

/// An error that may arise while working with the [`Code`] enumeration
///
/// [`Code`]: ./enum.Code.html
#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    InvalidLanguageCode(String),
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

//TODO: consider making this into a derive macro when Copy and Clone can be used within a constant context
macro language_impl($language:ident, $language_as_string:literal) {
    impl Language for $language {
        fn name(&self) -> &'static str {
            identifiers_from_table!(match &self: $language => "name")
        }

        fn code(&self) -> &'static str {
            identifiers_from_table!(match &self: $language => $language_as_string)
        }
    }

    impl FromStr for $language {
        type Err = Error;

        fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
            identifiers_from_table!(match s: $language_as_string => $language).ok_or(Error::InvalidLanguageCode(s.to_string()))
        }
    }
}

macro language_impl_try_from($from:ident, $to:ident) {
    impl TryFrom<$from> for $to {
        type Error = Error;

        fn try_from(c: $from) -> Result<Self, <Self as TryFrom<$from>>::Error> {
            identifiers_from_table!(match c: $from => $to).ok_or(Error::NoCorrespondingLanguageCode(c.code()))
        }
    }
}

identifiers_from_table!(enum Iso639_1: iso639_1);
language_impl!(Iso639_1, "Iso639_1");
language_impl_try_from!(Iso639_2b, Iso639_1);
language_impl_try_from!(Iso639_2t, Iso639_1);
language_impl_try_from!(Iso639_3, Iso639_1);

identifiers_from_table!(enum Iso639_2b: iso639_2b);
language_impl!(Iso639_2b, "Iso639_2b");
language_impl_try_from!(Iso639_1, Iso639_2b);
language_impl_try_from!(Iso639_2t, Iso639_2b);
language_impl_try_from!(Iso639_3, Iso639_2b);

identifiers_from_table!(enum Iso639_2t: iso639_2t);
language_impl!(Iso639_2t, "Iso639_2t");
language_impl_try_from!(Iso639_1, Iso639_2t);
language_impl_try_from!(Iso639_2b, Iso639_2t);
language_impl_try_from!(Iso639_3, Iso639_2t);

identifiers_from_table!(enum Iso639_3: iso639_3);
language_impl!(Iso639_3, "Iso639_3");
language_impl_try_from!(Iso639_1, Iso639_3);
language_impl_try_from!(Iso639_2b, Iso639_3);
language_impl_try_from!(Iso639_2t, Iso639_3);
