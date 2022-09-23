extern crate regex;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::time::SystemTime;

use self::regex::Regex;
/**
 * 创建文件
 */
pub fn create_not_exists(dir: &str) {
    if !Path::new(dir).exists() {
        match fs::create_dir_all(dir) {
            Err(why) => panic!("create {}: {}", dir, why.to_string()),
            Ok(_) => println!("create {}", dir),
        };
    }
}
/**
 * 写文件操作
 */
pub fn write_file(file_name: &str, content: &str) {
    // 创建路径
    let index_md = Path::new(&file_name);
    let mut file_index_md = match File::create(index_md) {
        Err(why) => panic!("create {}: {}", file_name, why.to_string()),
        Ok(file) => file,
    };
    match file_index_md.write_all(content.as_bytes()) {
        Err(why) => panic!("write {}: {}", file_name, why.to_string()),
        Ok(_) => println!("write {}", file_name),
    };
}

static MD_STR: &str = r#"---
title: {{ project }} 
author: RustWriter
template: index
---

# {{ project }}
This is written in rust writer. Simple, free and happy."#;

static HTML_STR: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <meta name="author" content="{{ author }}" />
    <title>{{ title }}</title>
</head>
<body>
    {{ content }}
</body>
</html>"#;

pub fn init_work_space(project_name: &str, public_dir: &str, src_dir: &str) {
    // 声明项目路径
    let path = Path::new(project_name);
    if path.exists() {
        println!("{} exists", project_name);
        process::exit(0x0100);
    }
    let project_src = format!("{}/{}", project_name, src_dir);
    let project_public = format!("{}/{}", project_name, public_dir);
    create_not_exists(&project_src);
    create_not_exists(&project_public);

    let index_md_name = format!("{}/{}", &project_src, "index.md");
    let re_project = Regex::new(r"\{\{\s*project\s*\}\}").unwrap();
    // 根据{{ project }} 正则匹配替换 MD_STR
    let md_text = String::from(re_project.replace_all(MD_STR, project_name));
    // 将生成的md内容写入到index.md
    write_file(&index_md_name, &md_text);
    // public 路径下写入HTML_STR内容到__index.html
    let index_tpl_name = format!("{}/{}", &project_public, "__index.html");
    write_file(&index_tpl_name, HTML_STR);
    // 填充rsw.toml配置文件
    write_file(&format!("{}/{}", project_name, "rsw.toml"), &format!("site_name = \"{}\"\nsite_url=\"http://localhost\"", project_name));
    println!("{} created successfully", project_name);
}

pub fn convert_path(path_str: &str) -> String {
    if cfg!(target_os = "windows") {
        String::from(path_str.replace("\\", "/"))
    } else {
        String::from(path_str)
    }
}

pub fn mtime(path_str: &str) -> std::result::Result<u64, std::io::Error> {
    let file = File::open(path_str)?;
    let metadata = file.metadata()?;
    let mtime = metadata.modified()?;
    let secs = match mtime.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => 0,
    };
    Ok(secs)
}
/**
 * 根据源文件修改时间 & 目标文件修改时间，判断是否时间先后
 */
pub fn skip(src_file: &str, target_file: &str) -> bool {
    // 源文件最后修改时间
    let src_mtime = mtime(src_file).unwrap_or(0);
    // 目标文件最后修改时间
    let target_mtime = mtime(target_file).unwrap_or(0);
    // 如果目标文件 > 源文件，表示不需要重新build
    if target_mtime >= src_mtime {
        true
    } else {
        false
    }
}
pub fn md_to_html_ext(file_name: &str) -> String {
    let names: Vec<&str> = file_name.rsplitn(2, '.').collect();
    format!("{}{}", names[1], ".html")
}

