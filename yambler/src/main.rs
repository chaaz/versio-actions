//! Yambler is a simple command-line program that stitches together snippets of YAML.

#[macro_use]
mod errors;

use crate::errors::Result;
use clap::{crate_version, App, AppSettings, Arg};
use error_chain::bail;
use std::collections::HashMap;
use std::path::Path;
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

fn run(input_file: &str, output_file: &str, snips: &[&str]) -> Result<()> {
  let inputs: Vec<Yaml> = YamlLoader::load_from_str(&std::fs::read_to_string(input_file)?)?;

  let snips = snips
    .iter()
    .map(|snip| Ok(YamlLoader::load_from_str(&std::fs::read_to_string(snip)?)?))
    .collect::<Result<Vec<_>>>()?
    .into_iter()
    .flat_map(|docs| docs.into_iter())
    .map(|y| {
      let mut hash = y.into_hash().ok_or_else(|| bad!("Yaml snip is not a hash."))?;
      let key = hash
        .remove(&Yaml::String("key".into()))
        .and_then(|y| y.into_string())
        .ok_or_else(|| bad!("Yaml snip doesn't have a string key."))?;
      let val = hash.remove(&Yaml::String("value".into())).ok_or_else(|| bad!("Yaml snip doesn't have a value."))?;
      Result::Ok((key, val))
    })
    .collect::<Result<_>>()?;

  let mut trail = Vec::new();
  let outputs = inputs.into_iter().map(|input| replace(&mut trail, input, &snips)).collect::<Result<Vec<_>>>()?;

  let input_file_name = Path::new(input_file).file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "??".into());
  let mut out_str = format!("---\n# DO NOT EDIT\n# Created from template \"{}\".\n", input_file_name);
  let header_len = out_str.len();
  let mut emitter = YamlEmitter::new(&mut out_str);
  for output in &outputs {
    emitter.dump(output)?;
  }
  if !outputs.is_empty() {
    out_str.replace_range(header_len .. header_len + 4, "");
  }
  std::fs::write(output_file, &out_str)?;

  Ok(())
}

fn replace(trail: &mut Vec<String>, input: Yaml, snips: &HashMap<String, Yaml>) -> Result<Yaml> {
  match input {
    Yaml::String(v) if v.starts_with("SNIPPET_") => {
      let key = &v[8 ..];
      let snip = snips.get(key).ok_or_else(|| bad!("No snippet for {}", v))?;
      if trail.iter().any(|v| key == v.as_str()) {
        bail!("Circular replacement: {:?} -> {}", trail, key);
      }
      trail.push(key.into());
      let snip = replace(trail, snip.clone(), snips)?;
      trail.pop();
      Ok(snip)
    }
    Yaml::Array(a) => {
      Ok(Yaml::Array(
        a.into_iter()
          .map(|v| {
            let was_str = v.as_str().is_some();
            replace(trail, v, snips).map(|r| (was_str, r))
          })
          .collect::<Result<Vec<_>>>()?
          .into_iter()
          .flat_map(|(was_str, v)| {
            if was_str && v.as_vec().is_some() {
              v.into_vec().unwrap().into_iter()
            } else {
              vec![v].into_iter()
            }
          })
          .collect()
      ))
    }
    Yaml::Hash(h) => {
      Ok(Yaml::Hash(h.into_iter().map(|(k, v)| replace(trail, v, snips).map(|v| (k, v))).collect::<Result<_>>()?))
    }
    other => Ok(other)
  }
}

fn run_cli() -> Result<()> {
  let m = App::new("versio")
    .setting(AppSettings::UnifiedHelpMessage)
    .author("Charlie Ozinga, ozchaz@gmail.com")
    .version(crate_version!())
    .about("Combine YAML documents")
    .arg(
      Arg::with_name("input")
        .short("i")
        .long("input")
        .takes_value(true)
        .value_name("file")
        .required(true)
        .display_order(1)
        .help("The base yaml to read")
    )
    .arg(
      Arg::with_name("output")
        .short("o")
        .long("output")
        .takes_value(true)
        .value_name("file")
        .required(true)
        .display_order(1)
        .help("The output yaml to generate")
    )
    .arg(
      Arg::with_name("snips")
        .short("s")
        .long("snips")
        .takes_value(true)
        .value_name("files")
        .required(true)
        .multiple(true)
        .display_order(1)
        .help("The input yaml snippets")
    )
    .get_matches();

  let snips = m.values_of("snips").unwrap().collect::<Vec<_>>();
  let input = m.value_of("input").unwrap();
  let output = m.value_of("output").unwrap();
  run(input, output, &snips)
}

fn main() {
  if let Err(e) = run_cli() {
    use std::io::Write;
    let stderr = &mut std::io::stderr();
    let errmsg = "Error writing to stderr.";

    writeln!(stderr, "Error: {}", e).expect(errmsg);

    for e in e.iter().skip(1) {
      writeln!(stderr, "  Caused by: {}", e).expect(errmsg);
    }

    // Try running with `RUST_BACKTRACE=1` for a backtrace
    if let Some(backtrace) = e.backtrace() {
      writeln!(stderr, "Backtrace:\n{:?}", backtrace).expect(errmsg);
    }

    std::process::exit(1);
  }
}
