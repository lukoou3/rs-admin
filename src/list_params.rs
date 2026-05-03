use serde::Deserialize;

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_page_size")]
    pub page_size: u32,
    pub keyword: Option<String>,
}

impl ListParams {
    pub fn offset_limit(&self) -> (i64, i64) {
        let page = self.page.max(1);
        let page_size = self.page_size.clamp(1, 200);
        let offset = ((page - 1) * page_size) as i64;
        (offset, page_size as i64)
    }
}
