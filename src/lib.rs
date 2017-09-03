extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate num;
#[macro_use]
extern crate serde_derive;
extern crate serde;

extern crate strum;
#[macro_use]
extern crate strum_macros;

/// Utilities for working with lvm files
pub mod lvm;

use strum::EnumMessage;

#[derive(Debug,EnumMessage)]
pub enum Error {
  Io(std::io::Error),
  IoString(String),
  Message(String),
  ParseNumberError(String),
  TrailingCharacters,
  TrailingLineCharacters,
  UnexpectedCharacter(char),
  UnexpectedEof,
  UnexpectedEol,
  UnexpectedToken(String),
  Unsupported
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    self.get_message().unwrap()
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
    f.write_str(self.get_message().unwrap())
  }
}

impl serde::de::Error for Error {
  fn custom<T: std::fmt::Display>(i_message: T) -> Self {
    Error::Message(i_message.to_string())
  }
}

type Result<T> = std::result::Result<T, Error>;
