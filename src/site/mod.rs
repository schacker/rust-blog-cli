use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Site {
    pub site_name: String,
    pub site_url: String,
    pub menu: Option<Vec<Vec<String>>>
}

impl Site {
    pub fn new(site_name: String, site_url: String, menu: Option<Vec<Vec<String>>>) -> Self {
        Site {
            site_name,
            site_url,
            menu
        }
    }
}