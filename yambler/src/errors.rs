//! Error handling is all based on `error-chain`.

use error_chain::error_chain;

error_chain! {
  foreign_links {
    Io(std::io::Error);
    Yaml(yaml_rust::ScanError);
    EmitYaml(yaml_rust::EmitError);
  }
}

#[macro_export]
macro_rules! err {
  ($($arg:tt)*) => (
    std::result::Result::Err($crate::errors::Error::from_kind($crate::errors::ErrorKind::Msg(format!($($arg)*))))
  )
}

#[macro_export]
macro_rules! bad {
  ($($arg:tt)*) => ($crate::errors::Error::from_kind($crate::errors::ErrorKind::Msg(format!($($arg)*))))
}
