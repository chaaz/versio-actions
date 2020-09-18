//! Yambler is a simple command-line program that stitches together snippets of YAML.

#[macro_use]
mod errors;

use crate::errors::Result;
use clap::{crate_version, App, AppSettings, Arg};
use error_chain::bail;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use yaml_rust::{Yaml, YamlEmitter, YamlLoader};

fn run(input_file: &str, output_file: &str, snips: &[&str]) -> Result<()> {
  let inputs = read_inputs(input_file, output_file)?;
  let snips = read_snips(snips)?;
  convert_and_write(inputs, &snips)
}

fn convert_and_write(inputs: Vec<(PathBuf, PathBuf, Vec<Yaml>)>, snips: &HashMap<String, Yaml>) -> Result<()> {
  for (input_file, output_file, inputs) in inputs {
    let mut trail = Vec::new();
    let outputs = inputs.into_iter().map(|input| replace(&mut trail, input, snips)).collect::<Result<Vec<_>>>()?;

    let input_file_name = input_file.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "??".into());
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
  }

  Ok(())
}

/// Read the input file and output file, and build a list of entries to work on.
///
/// Each entry has the form (input_file_path, output_file_path, list_of_yamls_in_input_file). If the input file
/// is a single file, the output file must also be a single file, and the list of entries has a length of one.
/// If the input file is a directory, the output file must also be a directory, and the list of entries
/// corresponds to each YAML file in the input directory.
fn read_inputs(input_file: &str, output_file: &str) -> Result<Vec<(PathBuf, PathBuf, Vec<Yaml>)>> {
  let input_file = Path::new(input_file);
  let output_file = Path::new(output_file);

  if input_file.is_dir() {
    if !output_file.is_dir() {
      bail!("If input is a directory, output must also be a directory.");
    }
    input_file
      .read_dir()?
      .filter(|entry| {
        entry.as_ref().map(|entry| {
          let n = entry.file_name().to_string_lossy().to_string();
          n.ends_with(".yml") || n.ends_with(".yaml")
        }).unwrap_or(false)
      })
      .map(|entry| {
        let entry = entry?;
        let output = output_file.join(entry.file_name());
        Ok((entry.path(), output, YamlLoader::load_from_str(&std::fs::read_to_string(entry.path())?)?))
      })
      .collect::<Result<Vec<_>>>()
  } else {
    if output_file.is_dir() {
      bail!("If input is a file, output must also be a file.");
    }
    Ok(vec![(
      input_file.to_path_buf(),
      output_file.to_path_buf(),
      YamlLoader::load_from_str(&std::fs::read_to_string(input_file)?)?
    )])
  }
}

/// Read the snippet files into a hash map.
///
/// The input list of files can be multiple YAML files, or can be a single directory, in which case all YAML
/// files in the directory are counted. The resulting hashmap are the keyed YAML snippets which are found in all
/// files.
fn read_snips(snips: &[&str]) -> Result<HashMap<String, Yaml>> {
  let snips = snips.iter().map(|s| Path::new(s)).collect::<Vec<_>>();
  let snips: Vec<_> = if snips.len() == 1 && snips[0].is_dir() {
    snips[0]
      .read_dir()?
      .filter_map(|entry| {
        entry.ok().and_then(|entry| {
          let n = entry.file_name().to_string_lossy().to_string();
          if n.ends_with(".yml") || n.ends_with(".yaml") {
            Some(entry.path())
          } else {
            None
          }
        })
      })
      .collect()
  } else {
    snips.into_iter().map(|s| s.to_path_buf()).collect()
  };

  snips
    .into_iter()
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
    .collect::<Result<_>>()
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
