mod ast;
mod block;
mod converter;
mod error;
mod frontmatter;
mod heading;
mod inline;
mod inline_parser;
mod list;
mod mdx_parser;
mod mdx_renderer;
mod org_parser;
mod org_renderer;
mod paragraph;
mod render;
mod util;

pub use converter::convert;
pub use error::{Error, Result};

// New bidirectional API
pub mod org_to_mdx {
    use crate::error::Result;
    use crate::mdx_renderer::render_mdx;
    use crate::org_parser::parse_org;

    pub fn convert(input: &str) -> Result<String> {
        let doc = parse_org(input)?;
        Ok(render_mdx(&doc))
    }
}

pub mod mdx_to_org {
    use crate::error::Result;
    use crate::mdx_parser::parse_mdx;
    use crate::org_renderer::render_org;

    pub fn convert(input: &str) -> Result<String> {
        let doc = parse_mdx(input)?;
        Ok(render_org(&doc))
    }
}
