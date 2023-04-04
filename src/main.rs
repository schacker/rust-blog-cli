extern crate clap;
extern crate rsw;
extern crate toml;
extern crate regex;

use std::{fs};
use std::path::Path;
use std::process;

use clap::{Arg, Command};
use regex::Regex;

use chrono::Utc;
use rsw::model::*;
use rsw::parse;
use rsw::site::Site;
use rsw::template;
use rsw::util::*;

/**
 * 解析目标项目rsw.toml配置文件
 * 解析符合规范的Site配置
 */
fn parse_toml() -> Site {
	let content = fs::read_to_string("rsw.toml").unwrap();
	let site: Site = match toml::from_str(content.as_str()) {
		Err(why) => {
			println!("- {}: {}", why.to_string(), "rsw.toml");
			process::exit(1);
		}
		Ok(site) => site,
	};
	site
}

fn copy_files(re_ignore: &Regex, target: &str, src: &str) {
	let dir = Path::new(src);
	// 遍历目录
	let entrys = match fs::read_dir(dir) {
		Err(why) => {
			println!("- {}: {}", why.to_string(), src);
			process::exit(1);
		}
		Ok(entrys) => entrys,
	};
	for entry in entrys {
		if let Ok(entry) = entry {
			let child = entry.path();
			let file_name = convert_path(child.to_str().unwrap());

			if child.is_file() {
				// 忽略匹配到的文件
				if re_ignore.is_match(file_name.as_str()) {
					continue;
				}
				// 拆分源文件名，方便后面组合成目标文件名
				let dirs: Vec<&str> = file_name.splitn(2, '/').collect();
				let new_file = format!("{}/{}", target, dirs[1]);
				// 判断是否需要跳过
				if skip(&file_name, &new_file) {
					continue;
				}
				// 将目标文件从右边按`/`拆分得到目录
				let dirs: Vec<&str> = new_file.rsplitn(2, '/').collect();
				// 如果要复制的目标目录不存在，就创建
				create_not_exists(dirs[1]);
				// 复制文件
				match fs::copy(file_name.clone(), &new_file) {
					Err(why) => panic!("{} -> {}: {}", file_name, new_file, why.to_string()),
					Ok(_) => println!("copy {} -> {}", file_name, new_file),
				}
			} else {
				// 如果是目录，则继续递归
				copy_files(re_ignore, target, &file_name);
			}
		}
	}
}

fn loop_parse(site: &Site, build: &str, public: &str, src: &str) {
	let path = Path::new(src);
	// 递归方式列出所有的源文件
	for entry in fs::read_dir(path).expect("Failed to read src directory") {
		if let Ok(entry) = entry {
			let child = entry.path(); // 获取path
			let file_name = child.to_str().unwrap(); // 获取filename
			if child.is_file() {
				let re_md_file = Regex::new(r".*\.md$").unwrap();
				if !re_md_file.is_match(file_name) {
					continue;
				}
				let md_file = parse::parse_md_file(build, &child);
				// 判断是否需要重新生成
				let target_file_name = md_file.target_file_name.clone();

				if skip(file_name, &target_file_name) {
					continue;
				}
				template::render(site, public, md_file);
			} else {
				loop_parse(site, build, public, file_name);
			}
		}
	}
}

// 加载指定配置文件，加载yaml文件转为model
// fn load_config<T>(path: &str) -> Option<T>
// where
//     T: DeserializeOwned,
// {
// 	// 1.通过std::fs读取配置文件内容
// 	// 2.通过serde_yaml解析读取到的yaml配置转换成json对象
// 	match serde_yaml::from_str::<RootSchema>(
// 		&std::fs::read_to_string(path).expect(&format!("failure read file {}", path)),
// 	) {
// 		Ok(root_schema) => {
// 			// 通过serde_json把json对象转换指定的model
// 			let data =
// 				serde_json::to_string_pretty(&root_schema).expect("failure to parse RootSchema");
// 			let config = serde_json::from_str::<T>(&*data)
// 				.expect(&format!("failure to format json str {}", &data));
// 			// 返回格式化结果
// 			Some(config)
// 		}
// 		Err(err) => {
// 			// 记录日志
// 			println!("{}", err);
// 			// 返回None
// 			None
// 		}
// 	}
// }
// 编译后的模板文件目录，将可以发布到网上
static BUILD_DIR: &str = "build";
// 资源文件，如css、js、图片
static PUBLIC_DIR: &str = "public";
// 源文件目录
static SRC_DIR: &str = "src";

fn main() {
	let start_time = Utc::now().timestamp_millis() as f64;
	// let yaml_data = export_cliyml();
	const YAML_DATA: &str = include_str!("../assets/cli.yml");
	let config: AppConfig = serde_yaml::from_str(&YAML_DATA).expect("Unable to parse YAML data");

	// Use the parsed config to set up the command-line parser with clap
	let mut app = Command::new(config.name)
		.version(config.version.to_string())
		.author(config.author.to_string())
		.about(config.about.to_string());
	// 需要根据yaml生成subcommand
	for arg in config.subcommands {
		if let Some(_) = arg.new {
			let app_arg = clap::Arg::new("new")
				.short('n')
				.long("new")
				.help(String::from("project name"))
				.action(clap::ArgAction::Set);

			app = app.arg(app_arg);
			app = app.subcommand(
				Command::new("new")
					.arg(Arg::new("project"))
					.about("new about"),
			);
		}
		if let Some(_) = arg.clean {
			let app_arg = clap::Arg::new("clean")
				.short('c')
				.long("clean")
				.help(String::from("delete the files in the build directory"))
				.action(clap::ArgAction::Set);

			app = app.arg(app_arg);
			app = app.subcommand(Command::new("clean").about("clean about"));
		}
		if let Some(_) = arg.build {
			let app_arg = clap::Arg::new("build")
				.short('b')
				.long("build")
				.help(String::from("build project"))
				.action(clap::ArgAction::Set);

			app = app.arg(app_arg);
			app = app.subcommand(Command::new("build").about("build about"));
		}
	}

	let matches = app.get_matches();
	// 新建项目
	if let Some(matches) = matches.subcommand_matches("new") {
		let project_name = match matches.get_one::<String>("project") {
			Some(p) => p,
			None => "",
		};
		init_work_space(project_name, PUBLIC_DIR, SRC_DIR);
		return;
	}

	if let Some(_) = matches.subcommand_matches("clean") {
		fs::remove_dir_all(BUILD_DIR).unwrap();
		return;
	}

	if let Some(_) = matches.subcommand_matches("build") {
		// copy public下的资源文件到build目录，但会忽略模板文件
		let re_template_file = Regex::new(r".*__.*\.html$").unwrap();
		copy_files(&re_template_file, BUILD_DIR, PUBLIC_DIR);
		let re_md_file = Regex::new(r".*\.md$").unwrap();
		// 解析rsw.toml文件
		let site = parse_toml();
		// copy src下的资源文件到build目录，但会忽略.md文件
		copy_files(&re_md_file, BUILD_DIR, SRC_DIR);
		// 解析md文件
		loop_parse(&site, BUILD_DIR, PUBLIC_DIR, SRC_DIR);
		println!("{}", "BUILD SUCCESS");
	}
	let end_time = Utc::now().timestamp_millis() as f64;
	println!("Total time: {} s", (end_time - start_time) / 1000.00);
	println!("Finished at: {}", Utc::now().to_string())
}
