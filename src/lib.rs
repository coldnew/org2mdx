pub mod ast;
pub mod parser;
pub mod renderer;
pub mod error;
pub mod util;

pub use error::{Error, Result};

pub mod org_to_mdx {
    pub fn convert(input: &str) -> crate::error::Result<String> {
        let ast = crate::parser::org::parse_org(input)?;
        Ok(crate::renderer::mdx_renderer::render_mdx(&ast))
    }
}

pub mod mdx_to_org {
    pub fn convert(input: &str) -> crate::error::Result<String> {
        let ast = crate::parser::mdx::parse_mdx(input)?;
        Ok(crate::renderer::org_renderer::render_org(&ast))
    }
}

pub mod org_to_ast {
    use super::ast::Node;
    use crate::error::Result;

    pub fn parse(input: &str) -> Result<Node> {
        crate::parser::org::parse_org(input)
    }
}

pub mod mdx_to_ast {
    use super::ast::Node;
    use crate::error::Result;

    pub fn parse(input: &str) -> Result<Node> {
        crate::parser::mdx::parse_mdx(input)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_stub_types() {}
}
