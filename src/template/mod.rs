extern crate comrak;
extern crate regex;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::process;

use super::parse::{MdFile, MdHead};
use super::util::*;

use self::comrak::{ComrakOptions, markdown_to_html};
use self::regex::Regex;
use super::site::Site;

pub fn render(site: &Site, public: &str, md_file: MdFile) {
    let md_head:MdHead= match serde_yaml::from_str(md_file.yaml_str.as_str()) {
      Ok(h) => h,
        Err(e) => {
            eprintln!("- {}: {}", e, md_file.file_name);
            process::exit(1);
        }
    };
    let menu = &site.menu;

    // 支持table解析
    let options = ComrakOptions {
        ext_table: true,
        unsafe_: true,
        ..ComrakOptions::default()
    };

    if md_head.page_size.is_some() && md_head.page_size.unwrap() > 0 {
        let page_size = md_head.page_size.unwrap();
        let lines: Vec<&str> = md_file.md_str.split("\n").collect();
        let page_count = (lines.len() as f64 / page_size as f64).ceil() as usize;

        for page_index in 0..page_count {
            let mut page = String::new();
            let start = page_index * page_size;

            let mut end = start + page_size;
            if end > lines.len() {
                end = lines.len();
            }

            for i in start..end {
                page.push_str(lines[i]);
                page.push_str("\n");
            }

            let html_file = md_file.target_file_name.as_str();
            let html_file_names: Vec<&str> = html_file.rsplitn(2, ".").collect();

            let mut target_file = String::from(html_file);

            if page_index > 0 {
                target_file = format!("{}-{}.{}", html_file_names[1], page_index, html_file_names[0]);
            }

            let mut pagination = String::from("");
            let html_name: Vec<&str> = html_file_names[1].rsplitn(2, "/").collect();
            if page_index == 0 && page_count > 0 {
                let next_link = format!("{}-{}.{}", html_name[0], page_index + 1, html_file_names[0]);
                pagination.push_str(
                    format!("<ul><li>&nbsp;</li><li>第 {} 页</li><li><a href=\"{}\">下一页</a></li></ul>",
                            page_index + 1, next_link).as_str()
                );
            } else if page_index > 0 && page_index < page_count - 1 {
                let next_link = format!("{}-{}.{}", html_name[0], page_index + 1, html_file_names[0]);
                if page_index - 1 == 0 {
                    let pre_link = format!("{}.{}", html_name[0], html_file_names[0]);
                    pagination.push_str(
                        format!("<ul><li><a href=\"{}\">上一页</a></li><li>第 {} 页</li><li><a href=\"{}\">下一页</a></li></ul>",
                                pre_link, page_index + 1, next_link).as_str()
                    );
                } else {
                    let pre_link = format!("{}-{}.{}", html_name[0], page_index - 1, html_file_names[0]);
                    pagination.push_str(
                        format!("<ul><li><a href=\"{}\">上一页</a></li><li>第 {} 页</li><li><a href=\"{}\">下一页</a></li></ul>",
                                pre_link, page_index + 1, next_link).as_str()
                    );
                }
            } else if page_index > 0 && page_index == page_count - 1 {
                let pre_link = format!("{}-{}.{}", html_name[0], page_index - 1, html_file_names[0]);
                pagination.push_str(
                    format!("<ul><li><a href=\"{}\">上一页</a></li><li>第 {} 页</li><li>&nbsp;</li></ul>",
                            pre_link, page_index + 1).as_str()
                );
            }

            let html_str = String::from(markdown_to_html(page.as_str(), &options).as_str());
            // 渲染模板
            let html_content = render_template(&site,
                                               menu,
                                               public,
                                               &md_head,
                                               html_str.as_str(),
                                               md_file.page_id.as_str(),
                                               pagination.as_str(),
                                               md_file.url.as_str());

            // 生成目标文件
            generate_html(md_file.file_name.as_str(), target_file.as_str(), html_content.as_str());

        }
    } else {
        let html_str = String::from(markdown_to_html(md_file.md_str.as_str(), &options).as_str());

        // 渲染模板
        let html_content = render_template(&site,
                                           menu,
                                           public,
                                           &md_head,
                                           html_str.as_str(),
                                           md_file.page_id.as_str(),
                                           String::from("").as_str(),
                                           md_file.url.as_str());

        // 生成目标文件
        generate_html(md_file.file_name.as_str(), md_file.target_file_name.as_str(), html_content.as_str());
    }
}

fn generate_html(md_path: &str, html_path: &str, content: &str) {
    let mut html_content = content.replace("<table>", "<div class=\"table-wrap\"><table>");
    html_content = html_content.replace("</table>", "</table></div>");
    // 拆分文件名，如`build/2017/01/01/happy.html`得到的是`["happy.html", "build/2017/01/01"]`
    let dirs: Vec<&str> = html_path.rsplitn(2, '/').collect();
    create_not_exists(dirs[1]);

    let path = Path::new(html_path);
    let display = path.display();

    // 以只写模式打开文件，返回`io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("create {}: {}", display, why.to_string()),
        Ok(file) => file,
    };

    match file.write_all(html_content.as_bytes()) {
        Err(why) => panic!("write {}: {}", display, why.to_string()),
        Ok(_) => println!("build {} -> {}", md_path, display),
    };
}

fn render_template(site: &Site,
                   menu: &Option<Vec<Vec<String>>>,
                   public: &str,
                   md_head: &MdHead,
                   html_str: &str,
                   page_id: &str,
                   pagination:& str,
                   url: &str) -> String {
    // 从yaml数据中取出md文件的元数据
    let template = &md_head.template;
    let template_names: Vec<&str> = template.rsplitn(2, '/').collect();
    let mut file_name = String::new();
    if template_names.len() == 1 {
        file_name.push_str("__");
        file_name.push_str(template);
    } else {
        file_name.push_str(template_names[1]);
        file_name.push_str("/__");
        file_name.push_str(template_names[0]);
    }

    let template_file = format!("{}/{}.html", public, file_name);

    // 打开模板文件
    let path = Path::new(template_file.as_str());
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
        Ok(file) => file
    };

    let mut template_content = String::new();
    if let Err(err) = file.read_to_string(&mut template_content) {
        panic!("couldn't read {}: {}", display, err.to_string());
    }

    // 将site_name渲染到模板中
    let re_author = Regex::new(r"\{\{\s*site_name\s*\}\}").unwrap();
    template_content = String::from(re_author.replace_all(template_content.as_str(), *(&site.site_name.as_str())));

    // 将site_url渲染到模板中
    let re_site_url = Regex::new(r"\{\{\s*site_url\s*\}\}").unwrap();
    template_content = String::from(re_site_url.replace_all(template_content.as_str(), *(&site.site_url.as_str())));

    // 将url渲染到模板中
    let re_url = Regex::new(r"\{\{\s*url\s*\}\}").unwrap();
    template_content = String::from(re_url.replace_all(template_content.as_str(), url));

    // 将author渲染到模板中
    if md_head.author.is_some() {
        let re_author = Regex::new(r"\{\{\s*author\s*\}\}").unwrap();
        template_content = String::from(re_author.replace_all(template_content.as_str(), md_head.author.as_ref().unwrap().as_str()));
    } else {
        let re_author = Regex::new(r"\{\{\s*author\s*\}\}").unwrap();
        template_content = String::from(re_author.replace_all(template_content.as_str(), ""));
    }

    // 将title渲染到模板中
    if md_head.title.is_some() {
        let re_title = Regex::new(r"\{\{\s*title\s*\}\}").unwrap();
        template_content = String::from(re_title.replace_all(template_content.as_str(), md_head.title.as_ref().unwrap().as_str()));
    } else {
        let re_title = Regex::new(r"\{\{\s*title\s*\}\}").unwrap();
        template_content = String::from(re_title.replace_all(template_content.as_str(), ""));
    }

    // 将keywords渲染到模板中
    if md_head.keywords.is_some() {
        let re_keywords = Regex::new(r"\{\{\s*keywords\s*\}\}").unwrap();
        template_content = String::from(re_keywords.replace_all(template_content.as_str(), md_head.keywords.as_ref().unwrap().as_str()));
    } else {
        let re_keywords = Regex::new(r"\{\{\s*keywords\s*\}\}").unwrap();
        template_content = String::from(re_keywords.replace_all(template_content.as_str(), ""));
    }

    // 将description渲染到模板中
    if md_head.description.is_some() {
        let re_description = Regex::new(r"\{\{\s*description\s*\}\}").unwrap();
        template_content = String::from(re_description.replace_all(template_content.as_str(), md_head.description.as_ref().unwrap().as_str()));
    } else {
        let re_description = Regex::new(r"\{\{\s*description\s*\}\}").unwrap();
        template_content = String::from(re_description.replace_all(template_content.as_str(), ""));
    }

    // 将ctime渲染到模板中
    if md_head.ctime.is_some() {
        let re_ctime = Regex::new(r"\{\{\s*ctime\s*\}\}").unwrap();
        template_content = String::from(re_ctime.replace_all(template_content.as_str(), md_head.ctime.as_ref().unwrap().as_str()));
    } else {
        let re_ctime = Regex::new(r"\{\{\s*ctime\s*\}\}").unwrap();
        template_content = String::from(re_ctime.replace_all(template_content.as_str(), ""));
    }

    // 将mtime渲染到模板中
    if md_head.mtime.is_some() {
        let re_mtime = Regex::new(r"\{\{\s*mtime\s*\}\}").unwrap();
        template_content = String::from(re_mtime.replace_all(template_content.as_str(), md_head.mtime.as_ref().unwrap().as_str()));
    } else {
        let re_mtime = Regex::new(r"\{\{\s*mtime\s*\}\}").unwrap();
        template_content = String::from(re_mtime.replace_all(template_content.as_str(), ""));
    }

    // 将page_id渲染到模板中
    let re_content = Regex::new(r"\{\{\s*page_id\s*\}\}").unwrap();
    template_content = String::from(re_content.replace_all(template_content.as_str(), page_id));

    // 将content渲染到模板中
    let re_content = Regex::new(r"\{\{\s*content\s*\}\}").unwrap();
    template_content = String::from(re_content.replace_all(template_content.as_str(), html_str));

    // 将上一页渲染到模板中
    if md_head.prev.is_some() {
        let prev_page = md_head.prev.as_ref().unwrap();

        let re_prev_title = Regex::new(r"\{\{\s*prev_title\s*\}\}").unwrap();
        template_content = String::from(re_prev_title.replace_all(template_content.as_str(), prev_page.title.as_str()));

        let re_prev_url = Regex::new(r"\{\{\s*prev_url\s*\}\}").unwrap();

        let mut page_url = prev_page.url.clone();
        if page_url.ends_with(".md") {
            page_url = md_to_html_ext(page_url.as_str());
        }
        template_content = String::from(re_prev_url.replace_all(template_content.as_str(), page_url.as_str()));

        let re_class =  Regex::new(r"\{\{\s*prev_class_name\s*\}\}").unwrap();
        template_content = String::from(re_class.replace_all(template_content.as_str(), "show"));
    } else {
        let re_prev_title = Regex::new(r"\{\{\s*prev_title\s*\}\}").unwrap();
        template_content = String::from(re_prev_title.replace_all(template_content.as_str(), "暂无上一页"));
        let re_prev_url = Regex::new(r"\{\{\s*prev_url\s*\}\}").unwrap();
        template_content = String::from(re_prev_url.replace_all(template_content.as_str(), "javascript:void(0)"));
        let re_class =  Regex::new(r"\{\{\s*prev_class_name\s*\}\}").unwrap();
        template_content = String::from(re_class.replace_all(template_content.as_str(), "hide"));
    }
    // 将下一页渲染到模板中
    if md_head.next.is_some() {
        let next_page = md_head.next.as_ref().unwrap();

        let re_next_title = Regex::new(r"\{\{\s*next_title\s*\}\}").unwrap();
        template_content = String::from(re_next_title.replace_all(template_content.as_str(), next_page.title.as_str()));

        let re_next_url = Regex::new(r"\{\{\s*next_url\s*\}\}").unwrap();

        let mut page_url = next_page.url.clone();
        if page_url.ends_with(".md") {
            page_url = md_to_html_ext(page_url.as_str());
        }
        template_content = String::from(re_next_url.replace_all(template_content.as_str(), page_url.as_str()));

        let re_class =  Regex::new(r"\{\{\s*next_class_name\s*\}\}").unwrap();
        template_content = String::from(re_class.replace_all(template_content.as_str(), "show"));
    } else {
        let re_next_title = Regex::new(r"\{\{\s*next_title\s*\}\}").unwrap();
        template_content = String::from(re_next_title.replace_all(template_content.as_str(), "暂无下一页"));
        let re_next_url = Regex::new(r"\{\{\s*next_url\s*\}\}").unwrap();
        template_content = String::from(re_next_url.replace_all(template_content.as_str(), "javascript:void(0)"));
        let re_class =  Regex::new(r"\{\{\s*next_class_name\s*\}\}").unwrap();
        template_content = String::from(re_class.replace_all(template_content.as_str(), "hide"));
    }

    if let Some(menu_list) = menu {
        // 将菜单渲染到模板中
        let re_menu = Regex::new(r"\{\{\s*menu\s*\}\}").unwrap();

        let mut ul_element = String::new();
        ul_element.push_str("<ul>");

        for item in menu_list {
            let li_element = format!("<li><a href=\"{}\">{}</a></li>",
                                     item.get(1).unwrap_or(&"".to_string()),
                                     item.get(0).unwrap_or(&"".to_string()));
            ul_element.push_str(&li_element);
        }
        ul_element.push_str("</ul>");

        template_content = String::from(re_menu.replace_all(template_content.as_str(), ul_element.as_str()));
    }

    // 将分页数据渲染到模板中
    let re_pagination = Regex::new(r"\{\{\s*pagination\s*\}\}").unwrap();
    template_content = String::from(re_pagination.replace_all(template_content.as_str(), pagination));

    return template_content;
}