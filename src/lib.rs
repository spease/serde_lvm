#![deny(missing_docs,
        missing_debug_implementations, missing_copy_implementations,
        trivial_casts, trivial_numeric_casts,
        unsafe_code,
        unstable_features,
        unused_import_braces, unused_qualifications)]
// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]
//! Tools for interacting with LabVIEW and associated file formats
extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
extern crate itertools;
#[macro_use]
extern crate log;
extern crate num;
extern crate semver;
#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate strum;
#[macro_use]
extern crate strum_macros;

/// Utilities for working with LVM data structures
pub mod lvm;
/// Internal lowlevel utilities for parsing and writing LVM files
mod lvm_format;

#[allow(missing_docs)]
mod errors {
  use itertools::Itertools;
  use super::*;
  error_chain! {
    errors {
      /// A deserialization error occurred
      Deserialize(s: String) {
        description("A deserialization error occurred")
        display("deserialization error: \"{}\"", s)
      }
      /// An invalid separator
      InvalidSeparator(c: char) {
        description("An invalid separator was used by the file")
        display("An invalid separator \"{}\" was used by the file", c)
      }
      /// An error occurred while parsing a floating-point number
      ParseFloatError(e: num::traits::ParseFloatError) {
        description("An error occurred while parsing a floating-point number")
        display("parse floating-point error: \"{:?}\"", e)
      }
      /// An error occurred parsing a line
      ParseLine(l: usize) {
        description("An error occurred parsing a line")
        display("Error parsing line {}", l)
      }
      /// An unexpected character was found when attempting to parse a separator
      ParseSeparatorExpected(c: char, s: lvm::Separator) {
        description("An unexpected character was found when attempting to parse a separator")
        display("Unexpected character \"{}\" was found when attempting to parse a {} separator", c, s.as_ref())
      }
      /// Trailing characters were found instead of the end of a line
      ParseEolExpected(s: String) {
        description("Trailing characters were found instead of the end of a line")
        display("Trailing characters \"{}\" were found instead of the end of a line", s)
      }
      /// The end of the file was encountered before parsing was finished
      ParseEofUnexpected {
        description("The end of the file was encountered before parsing was finished")
        display("The end of the file was encountered before parsing was finished")
      }
      /// the end of the line was encountered before parsing was finished
      ParseEolUnexpected {
        description("The end of the line was encountered before parsing was finished")
        display("The end of the line was encountered before parsing was finished")
      }
      /// The specified token was found when attempting to parse a specific token
      ParseTokenUnexpected(u: String, e: &'static [&'static str]) {
        description("The specified token was found when attempting to parse a specific token")
        display("\"{}\" was found instead of {}", u, e.iter().map(|s|format!("\"{}\"", s)).join(" or "))
      }
    }

    foreign_links {
      Io(std::io::Error);
      ParseIntError(std::num::ParseIntError);
    }
  }
}

pub use errors::*;

impl serde::de::Error for Error {
  fn custom<T: std::fmt::Display>(i_message: T) -> Self {
    ErrorKind::Deserialize(i_message.to_string()).into()
  }
}
