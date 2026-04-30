mod ast;
mod block;
mod error;
mod heading;
mod inline_parser;
mod list;
mod mdx_parser;
mod mdx_renderer;
mod org_parser;
mod org_renderer;
mod util;

pub use error::{Error, Result};

pub mod org_to_mdx {
    use crate::error::Result;
    use crate::mdx_renderer::render_mdx;
    use crate::org_parser::parse_org;

    pub fn convert(input: &str) -> Result<String> {
        let root = parse_org(input)?;
        Ok(render_mdx(&root))
    }
}

pub mod mdx_to_org {
    use crate::error::Result;
    use crate::mdx_parser::parse_mdx;
    use crate::org_renderer::render_org;

    pub fn convert(input: &str) -> Result<String> {
        let root = parse_mdx(input)?;
        Ok(render_org(&root))
    }
}
