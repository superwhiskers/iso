//
// iso - implementations of data types related to common iso standards
//
// copyright (c) 2021 superwhiskers <whiskerdev@protonmail.com>
// this source code form is subject to the terms of the mozilla public
// license, v. 2.0. if a copy of the mpl was not distributed with this
// file, you can obtain one at http://mozilla.org/MPL/2.0/.
//

#![allow(clippy::cognitive_complexity)]
#![warn(clippy::cargo_common_metadata)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::explicit_deref_methods)]
#![warn(clippy::filetype_is_file)]
#![warn(clippy::imprecise_flops)]
#![warn(clippy::large_stack_arrays)]
#![warn(clippy::todo)]
#![warn(clippy::unimplemented)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::doc_markdown)]
#![deny(clippy::empty_enum)]
#![deny(clippy::enum_glob_use)]
#![deny(clippy::exit)]
#![deny(clippy::explicit_into_iter_loop)]
#![deny(clippy::explicit_iter_loop)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::large_digit_groups)]
#![deny(clippy::wildcard_dependencies)]
#![deny(clippy::wildcard_imports)]
#![deny(clippy::unused_self)]
#![deny(clippy::single_match_else)]
#![deny(clippy::option_option)]
#![deny(clippy::mut_mut)]
#![feature(proc_macro_diagnostic)]

use proc_macro::{Diagnostic, Level, TokenStream};
use proc_macro2::{Literal, Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    env::var,
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    result::Result as StdResult,
};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, LitStr, Token,
};

//TODO(superwhiskers):
//   - refactor the source code to not be so repetitive
//   - give proper diagnostics and handle errors well (no `.unwrap()`)
//   - add documentation comments

/// An enumeration over the supported ISO language code formats aas well as the name of the language
#[derive(PartialEq, Eq, Hash, Clone)]
enum LanguageTableEntryKey {
    Iso639_3,
    Iso639_2b,
    Iso639_2t,
    Iso639_1,
    Name,
}

impl LanguageTableEntryKey {
    fn as_standard_code(&self) -> Option<&'static str> {
        Some(match &self {
            Self::Iso639_3 => "639-3",
            Self::Iso639_2b => "639-2b",
            Self::Iso639_2t => "639-2t",
            Self::Iso639_1 => "639-1",
            _ => return None,
        })
    }
}

impl TryFrom<String> for LanguageTableEntryKey {
    type Error = &'static str;

    fn try_from(string: String) -> StdResult<Self, Self::Error> {
        Ok(match string.to_lowercase().as_ref() {
            "iso639_3" => Self::Iso639_3,
            "iso639_2b" => Self::Iso639_2b,
            "iso639_2t" => Self::Iso639_2t,
            "iso639_1" => Self::Iso639_1,
            "name" => Self::Name,
            _ => return Err("unable to find a matching variant"),
        })
    }
}

impl TryInto<&'static str> for LanguageTableEntryKey {
    type Error = &'static str;

    fn try_into(self) -> StdResult<&'static str, Self::Error> {
        Ok(match &self {
            Self::Iso639_3 => "Iso639_3",
            Self::Iso639_2b => "Iso639_2b",
            Self::Iso639_2t => "Iso639_2t",
            Self::Iso639_1 => "Iso639_1",
            _ => return Err("unable to find a matching string"),
        })
    }
}

fn parse_language_table(table: &Path) -> Option<Vec<HashMap<LanguageTableEntryKey, String>>> {
    let table_reader = BufReader::new(match File::open(table) {
        Ok(file) => file,
        Err(e) => {
            Diagnostic::new(
                Level::Error,
                format!(
                    "Unable to load the language table, {}",
                    table.as_os_str().to_string_lossy()
                ),
            )
            .note(format!("{}", e))
            .emit();
            return None;
        }
    });

    Some(
        table_reader
            .lines()
            .skip(1)
            .filter_map(|raw_line| {
                let line = match &raw_line {
                    Ok(s) => s,
                    Err(_) => return None,
                }
                .split('\t')
                .collect::<Vec<&str>>();

                let mut entry = HashMap::new();
                entry.insert(LanguageTableEntryKey::Iso639_3, line[0].to_string());
                if line[1].len() == 3 {
                    entry.insert(LanguageTableEntryKey::Iso639_2b, line[1].to_string());
                }
                if line[2].len() == 3 {
                    entry.insert(LanguageTableEntryKey::Iso639_2t, line[2].to_string());
                }
                if line[3].len() == 2 {
                    entry.insert(LanguageTableEntryKey::Iso639_1, line[3].to_string());
                }
                entry.insert(LanguageTableEntryKey::Name, line[6].to_string());

                Some(entry)
            })
            .collect(),
    )
}

fn parse_language_table_from_environment() -> Option<Vec<HashMap<LanguageTableEntryKey, String>>> {
    let mut language_table_path = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    language_table_path.push("assets/language.tab");
    parse_language_table(&language_table_path)
}

// note: the second parameter of each tuple is `true` if a string is being generated
struct IdentifierGenerationInput {
    enumeration: Option<String>,
    match_against: Option<TokenStream2>,
    lhs: (LanguageTableEntryKey, bool),
    rhs: Option<(LanguageTableEntryKey, bool)>,
}

impl Parse for IdentifierGenerationInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let keyword = input.lookahead1();
        let enumeration = if keyword.peek(Token![enum]) {
            input.parse::<Token![enum]>()?;
            let enumeration_name = input.parse::<Ident>()?.to_string();
            input.parse::<Token![:]>()?;
            Some(enumeration_name)
        } else {
            None
        };
        let match_against = if keyword.peek(Token![match]) {
            input.parse::<Token![match]>()?;
            let match_against = input.lookahead1();
            let match_against = if match_against.peek(Token![&]) {
                input.parse::<Token![&]>()?;
                input.parse::<Token![self]>()?;
                Some(quote! { &self })
            } else if match_against.peek(Ident) {
                Some(input.parse::<Ident>()?.to_token_stream())
            } else {
                None
            };
            input.parse::<Token![:]>()?;
            match_against
        } else {
            None
        };
        let lhs = input.lookahead1();
        let lhs = if lhs.peek(Ident) {
            (
                input.parse::<Ident>()?.to_string().try_into().unwrap(),
                false,
            )
        } else if lhs.peek(LitStr) {
            (input.parse::<LitStr>()?.value().try_into().unwrap(), true)
        } else {
            return Err(lhs.error());
        };
        let token = input.lookahead1();
        let rhs = if token.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let rhs = input.lookahead1();
            Some(if rhs.peek(Ident) {
                (
                    input.parse::<Ident>()?.to_string().try_into().unwrap(),
                    false,
                )
            } else if rhs.peek(LitStr) {
                (input.parse::<LitStr>()?.value().try_into().unwrap(), true)
            } else {
                return Err(rhs.error());
            })
        } else {
            None
        };

        Ok(IdentifierGenerationInput {
            enumeration,
            match_against,
            lhs,
            rhs,
        })
    }
}

fn ascii_capitalize(string: &mut str) {
    if let Some(r) = string.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}

#[proc_macro]
pub fn identifiers_from_table(tokens: TokenStream) -> TokenStream {
    let table = parse_language_table_from_environment().unwrap();
    let IdentifierGenerationInput {
        enumeration,
        match_against,
        lhs,
        rhs,
    } = parse_macro_input!(tokens as IdentifierGenerationInput);

    let mut rows: Vec<proc_macro2::TokenStream> = Vec::new();
    for table_entry in table {
        if table_entry.get(&lhs.0).is_none() {
            continue;
        }
        match (&lhs, &rhs) {
            ((lhs_table, true), None) => {
                let lhs: Literal = Literal::string(&table_entry[lhs_table]).into();
                rows.push(quote! {
                    #lhs
                });
            }
            ((lhs_table, false), None) => {
                let mut lhs_string = table_entry[lhs_table].clone();
                ascii_capitalize(&mut lhs_string);
                let lhs: Ident = Ident::new(&lhs_string, Span::call_site()).into();
                rows.push(quote! {
                    #lhs
                });
            }
            ((lhs_table, true), Some((rhs_table, true))) => {
                let lhs = Literal::string(&table_entry[lhs_table]);
                let rhs = Literal::string(&table_entry[rhs_table]);
                rows.push(quote! {
                    #lhs => #rhs
                });
            }
            ((lhs_table, false), Some((rhs_table, true))) => {
                let mut lhs_string = table_entry[lhs_table].clone();
                ascii_capitalize(&mut lhs_string);
                let lhs = Ident::new(&lhs_string, Span::call_site());

                // while this technically isn't safe, trying to generate a literal for a name is impossible
                let lhs_path = Ident::new(lhs_table.clone().try_into().unwrap(), Span::call_site());

                let rhs = Literal::string(&table_entry[rhs_table]);
                rows.push(quote! {
                    #lhs_path::#lhs => #rhs
                })
            }
            ((lhs_table, true), Some((rhs_table, false))) => {
                let lhs = Literal::string(&table_entry[lhs_table]);
                if let Some(rhs) = table_entry.get(rhs_table) {
                    let mut rhs_string = rhs.clone();
                    ascii_capitalize(&mut rhs_string);
                    let rhs = Ident::new(&rhs_string, Span::call_site());

                    // while this technically isn't safe, trying to generate a literal for a name is impossible
                    let rhs_path =
                        Ident::new(rhs_table.clone().try_into().unwrap(), Span::call_site());
                    rows.push(quote! {
                        #lhs => Some(#rhs_path::#rhs)
                    })
                } else {
                    rows.push(quote! {
                        #lhs => None
                    })
                }
            }
            ((lhs_table, false), Some((rhs_table, false))) => {
                let mut lhs_string = table_entry[lhs_table].clone();
                ascii_capitalize(&mut lhs_string);
                let lhs = Ident::new(&lhs_string, Span::call_site());

                // while this technically isn't safe, trying to generate a literal for a name is impossible
                let lhs_path = Ident::new(lhs_table.clone().try_into().unwrap(), Span::call_site());
                if let Some(rhs) = table_entry.get(rhs_table) {
                    let mut rhs_string = rhs.clone();
                    ascii_capitalize(&mut rhs_string);
                    let rhs = Ident::new(&rhs_string, Span::call_site());

                    // while this technically isn't safe, trying to generate a literal for a name is impossible
                    let rhs_path =
                        Ident::new(rhs_table.clone().try_into().unwrap(), Span::call_site());
                    rows.push(quote! {
                        #lhs_path::#lhs => Some(#rhs_path::#rhs)
                    })
                } else {
                    rows.push(quote! {
                        #lhs_path::#lhs => None
                    })
                }
            }
        }
    }

    return TokenStream::from(if let Some(enumeration_name) = enumeration {
        let enumeration_name = Ident::new(&enumeration_name, Span::call_site());
        let iso_code = lhs.0.as_standard_code();
        if let Some(iso_code) = iso_code {
            quote! {
                /// Enumeration over all possible ISO
                #[doc = #iso_code]
                /// language codes
                #[derive(Debug, Copy, Clone, Eq, PartialEq)]
                pub enum #enumeration_name {
                    #(#rows),*
                }
            }
        } else {
            quote! {
                compile_error!("the selected table column to generate an enumeration from does not have a corresponding iso standard")
            }
        }
    } else if let Some(match_against) = match_against {
        if lhs.1 {
            quote! {
                match #match_against {
                    #(#rows),*,
                    _ => None,
                }
            }
        } else {
            quote! {
                match #match_against {
                    #(#rows),*
                }
            }
        }
    } else {
        quote! {
            compile_error!("not enough information was provided");
        }
    });
}
