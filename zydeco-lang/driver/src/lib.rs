#![allow(clippy::style)]
#![allow(clippy::useless_format)]

pub mod parsed;
pub mod resolved;
pub mod package;
pub mod err;

use self::{err::SurfaceError, parsed::*, resolved::*};
use package::{FileLoc, ProjectMode};
use serde::Deserialize;
use std::path::Path;
use zydeco_surface::textual::syntax::ModName;

#[derive(Deserialize)]
struct Config {
    name: String,
    mode: ProjectMode,
}

#[derive(Default)]
pub struct Driver {}

impl Driver {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn single_file(&mut self, path: impl AsRef<Path>) -> Result<(), SurfaceError> {
        let mut deps = DependencyTracker::default();

        // parse
        let mut parsed = ParsedMap::default();
        let id = parsed.add_file_parsed(parsed.parse_file(path)?);
        let std = parsed.add_file_parsed(parsed.std());
        // deps.update_dep(id, std);

        // // resolve
        // let mut resolved = ResolvedMap::new(deps);
        // resolved.resolve_one_by_one(&parsed)?;

        Ok(())
    }

    pub fn whole_project(&mut self, path: impl AsRef<Path>) -> Result<(), SurfaceError> {
        let mut deps = DependencyTracker::default();

        // locate Zydeco.toml file and find project_name/src/Module.zy file and start to parse.
        // The result should be [Toplevel::Module(modname: project_name)]
        // read Zydeco.toml file
        let project_name = path.as_ref().file_name().unwrap();
        let content = std::fs::read_to_string(path.as_ref().join(Path::new("Zydeco.toml")))
            .map_err(|_| SurfaceError::PathNotFound { path: path.as_ref().to_path_buf() })?;
        let config: Config = toml::from_str(&content).unwrap();

        // parse
        let mut parsed = ParsedMap::new(project_name.to_str().unwrap().to_string(), path.as_ref());
        // Todo: If std isn't neeeded
        let id = parsed.std_wp();
        // The first file to parse
        parsed.add_file_to_parse(FileLoc(path.as_ref().join(Path::new("src/Module.zy"))));
        loop {
            if parsed.to_parse.is_empty() {
                break;
            }
            let FileLoc(loc) = parsed.to_parse.pop().unwrap();
            let id = parsed.parse_file_wp(loc)?;
        }
        println!("{:?}", parsed.module_root);
        // update dependency
        // dependency pair should be (id, dep_fileloc)
        // if the dep hasn't been parsed, use dep_fileloc instead.
        // Once a file is parsed, look up the set of pair to update the fileloc and insert it into the map.
        // If a file is parsed, it should have a place in module tree and has its own id.
        // finally, use update_deps(id, hash_deps_id).
        // deps.update_dep(id, std);

        // resolve
        let mut resolved = ResolvedMap::new(deps);
        resolved.resolve_one_by_one(&parsed)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
