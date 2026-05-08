pub mod ast;
pub mod error;
pub mod parser;
pub mod renderer;
pub mod util;

pub use error::{Error, Result};

pub mod org_to_mdx {
    use std::path::Path;

    /// Convert Org to MDX, resolving `#+INCLUDE:` directives relative to CWD.
    pub fn convert(input: &str) -> crate::error::Result<String> {
        convert_with_base(input, Path::new("."))
    }

    /// Convert Org to MDX, resolving `#+INCLUDE:` directives relative to `base_dir`.
    pub fn convert_with_base(input: &str, base_dir: &Path) -> crate::error::Result<String> {
        let resolved = if input.contains("#+INCLUDE:") {
            crate::parser::org::include::resolve_includes(input, base_dir)?
        } else {
            input.to_string()
        };
        let ast = crate::parser::org::parse_org(&resolved)?;
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
    use std::path::Path;

    use super::ast::Node;
    use crate::error::Result;

    /// Parse Org to AST, resolving `#+INCLUDE:` directives relative to CWD.
    pub fn parse(input: &str) -> Result<Node> {
        parse_with_base(input, Path::new("."))
    }

    /// Parse Org to AST, resolving `#+INCLUDE:` directives relative to `base_dir`.
    pub fn parse_with_base(input: &str, base_dir: &Path) -> Result<Node> {
        let resolved = if input.contains("#+INCLUDE:") {
            crate::parser::org::include::resolve_includes(input, base_dir)?
        } else {
            input.to_string()
        };
        crate::parser::org::parse_org(&resolved)
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
