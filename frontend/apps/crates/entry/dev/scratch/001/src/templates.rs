use simple_html_template::TemplateCache;

use std::fmt;
use web_sys::HtmlElement;

thread_local! {
    pub static TEMPLATES: Templates = Templates::new();
}

macro_rules! template_path {
    ($e:tt) => {
        concat!("../html/", $e)
    };
}

const HEADER: &str = "header";

pub fn header() -> HtmlElement {
    TEMPLATES.with(|t| t.cache.render_elem_plain(HEADER))
}

pub struct Templates {
    pub cache: TemplateCache<'static>,
}

impl fmt::Debug for Templates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.cache.templates.keys()).finish()
    }
}
impl Templates {
    pub fn new() -> Self {
        let cache = TemplateCache::new(&[(HEADER, include_str!(template_path!("header.html")))]);

        Self { cache }
    }
}
