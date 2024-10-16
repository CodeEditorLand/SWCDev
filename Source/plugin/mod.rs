use anyhow::{Context, Error};
use structopt::StructOpt;

use self::{
	build::BuildCommand,
	init::InitCommand,
	package::PackageCommand,
	publish::PublishDepsCommand,
	upgrade_deps::UpgradeDepsCommand,
};

pub mod build;
pub mod init;
pub mod package;
pub mod publish;
pub mod upgrade_deps;

/// Manages the plugin. Used for developing plugins.
#[derive(Debug, StructOpt)]
pub enum PluginCommand {
	Init(InitCommand),
	Build(BuildCommand),
	Package(PackageCommand),
	PublishDeps(PublishDepsCommand),
	UpgradeDeps(UpgradeDepsCommand),
}

impl PluginCommand {
	pub fn run(self) -> Result<(), Error> {
		match self {
			PluginCommand::Init(cmd) => {
				cmd.run().context("failed to initialize a plugin project")?;
			},
			PluginCommand::Build(cmd) => {
				cmd.run()?;
			},
			PluginCommand::Package(cmd) => {
				cmd.run()?;
			},
			PluginCommand::PublishDeps(cmd) => {
				cmd.run()?;
			},
			PluginCommand::UpgradeDeps(cmd) => {
				cmd.run().context("failed to upgrade dependencies")?;
			},
		}

		Ok(())
	}
}
