use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
  pub name: String,
  pub version: String,
	pub author: String,
	pub about: String,
	pub subcommands: Vec<Subcommand>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Subcommand {
  pub new: Option<NewArgs>,
  pub build: Option<BuildArgs>,
  pub clean: Option<CleanArgs>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewArgs {
  pub about: String,
  pub args: Vec<NewArg>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewArg {
	pub project: ProjectArg,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectArg {
  pub help: String,
	pub required: bool,
	pub index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildArgs {
  pub about: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CleanArgs {
	pub about: String,
}
