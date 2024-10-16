use std::{
	env::current_dir,
	fs::{copy, create_dir_all},
	sync::Arc,
};

use anyhow::{anyhow, bail, Context, Error};
use indexmap::IndexSet;
use rayon::prelude::*;
use structopt::StructOpt;
use swc_node_arch::PlatformDetail;
use tracing::{debug, error, info};

use self::cargo::BaseCargoBuildCommand;
use crate::util::{
	cargo::{get_default_cargo_target, swc_output_dir},
	node::create_npm_package,
};

mod cargo;

/// Build plugin packages.
#[derive(Debug, StructOpt)]
pub struct BuildCommand {
	#[structopt(flatten)]
	pub cargo:BaseCargoBuildCommand,

	/// Create node package file named `plugin-name.platform.swc-pkg.tar.gz`.
	#[structopt(long)]
	pub package:bool,
}

impl BuildCommand {
	pub fn run(self) -> Result<(), Error> {
		let output_base = swc_output_dir()?;

		let platform = match self.cargo.target.clone() {
			Some(v) => v,
			None => get_default_cargo_target()?,
		};
		let p:PlatformDetail = platform.parse().context("failed to parse platform")?;

		let libs = self.cargo.run()?;

		let build_dir = Arc::new(output_base.join("build"));
		let pkgs_dir = Arc::new(output_base.join("pkgs"));
		create_dir_all(&*build_dir)?;

		let results = libs
			.par_iter()
			.map(|lib| -> Result<_, Error> {
				let cdylib_ext = lib.cdylib_path.extension().expect("cdylib should have extension");
				let name = format!(
					"{}.{}.{}",
					lib.crate_name,
					p.platform_arch_abi,
					cdylib_ext.to_string_lossy()
				);
				let copied_path = build_dir.join(&name);

				copy(&lib.cdylib_path, &copied_path).context("failed to copy file")?;

				debug!("Copying {} to {}", lib.cdylib_path.display(), copied_path.display());

				Ok(())
			})
			.collect::<Vec<_>>();

		let crate_names = libs.iter().map(|l| l.crate_name.clone()).collect::<IndexSet<_>>();
		let mut error = false;
		for result in results {
			match result {
				Ok(..) => {},
				Err(err) => {
					error = true;
					error!("failed to copy plugin: {:?}", err);
				},
			}
		}
		if error {
			bail!("failed to copy plugin");
		}

		info!("Built files are copied to {}", build_dir.display());

		let cur_dir = current_dir().context("failed to get current directory")?;

		if self.package {
			for crate_name in crate_names.iter() {
				let pkg_dir = super::package::create_package_for_platform(
					&pkgs_dir,
					&build_dir,
					&crate_name,
					&p,
				)
				.context("failed to create package for the built platform")?;

				let pkg_file =
					create_npm_package(&pkg_dir).context("failed to create npm package")?;

				info!("Created package file at `{}`", pkg_file.display());

				let ext = pkg_file
					.extension()
					.ok_or_else(|| {
						anyhow!("package file built by `npm pack` should have filename")
					})?
					.to_string_lossy();
				let filename = format!("{}.{}.swc-pkg.{}", crate_name, p, ext);

				copy(&pkg_file, &cur_dir.join(filename))
					.context("failed to copy npm package file")?;
			}
		}

		Ok(())
	}
}
