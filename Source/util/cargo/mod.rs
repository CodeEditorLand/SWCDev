use std::{
	env,
	path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use cached::proc_macro::cached;
use cargo_metadata::MetadataCommand;

pub mod add;
pub mod upgrade;

pub fn cargo_metadata(mut cmd:MetadataCommand, from:&Path) -> Result<cargo_metadata::Metadata> {
	let from = from.to_path_buf();

	let result = cmd.current_dir(&from).exec().context("failed to execute `cargo metadata`")?;

	Ok(result)
}

/// Get all crate in cargo workspace.
///
/// Returns `(name, manifest_dir)`
pub fn get_all_crates() -> Result<Vec<(String, PathBuf)>> {
	(|| -> Result<_> {
		let mut cmd = MetadataCommand::new();
		cmd.no_deps();

		let md = cargo_metadata(cmd, &env::current_dir()?)?;
		let ws_pkgs = md.workspace_members;

		Ok(md
			.packages
			.into_iter()
			.filter_map(|p| {
				if !ws_pkgs.contains(&p.id) {
					return None;
				}

				let manifest_dir =
					p.manifest_path.parent().unwrap().to_path_buf().into_std_path_buf();
				Some((p.name, manifest_dir))
			})
			.collect())
	})()
	.context("failed to get members of the cargo workspace")
}

pub fn swc_output_dir() -> Result<PathBuf> {
	let cargo_target = cargo_target_dir()?;

	Ok(cargo_target.join(".swc"))
}

#[cached(result)]
pub fn get_cargo_manifest_path(crate_name:String) -> Result<PathBuf> {
	let from = env::current_dir().context("failed to get current dir")?;

	let cmd = MetadataCommand::new();
	let meta = cargo_metadata(cmd, &from)?;

	Ok(meta
		.packages
		.iter()
		.find(|p| p.name == crate_name)
		.context("failed to find the package")?
		.manifest_path
		.to_path_buf()
		.into_std_path_buf())
}

#[cached(result)]
pub fn cargo_target_dir() -> Result<PathBuf> {
	let from = env::current_dir().context("failed to get current dir")?;

	let mut cmd = MetadataCommand::new();
	cmd.no_deps();
	let md = cargo_metadata(cmd, &from)?;

	Ok(md.target_directory.as_std_path().to_path_buf())
}

#[cached(result)]
pub fn get_default_cargo_target() -> Result<String> {
	use std::process::Command;

	let output = Command::new("rustc")
		.arg("-vV")
		.output()
		.context("Failed to run rustc to get the host target")?;
	let output =
		String::from_utf8(output.stdout).context("`rustc -vV` didn't return utf8 output")?;

	let field = "host: ";
	let host = output
		.lines()
		.find(|l| l.starts_with(field))
		.map(|l| &l[field.len()..])
		.ok_or_else(|| {
			anyhow!("`rustc -vV` didn't have a line for `{}`, got:\n{}", field.trim(), output)
		})?
		.to_string();
	Ok(host)
}
