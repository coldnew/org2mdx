use napi_derive::napi;

#[napi]
pub fn convert(input: String) -> Result<String, napi::Error> {
    crate::org_to_mdx::convert(&input).map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub fn convert_mdx_to_org(input: String) -> Result<String, napi::Error> {
    crate::mdx_to_org::convert(&input).map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub fn parse_org_to_ast(input: String) -> Result<String, napi::Error> {
    crate::org_to_ast::parse(&input)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
        .and_then(|root| {
            serde_json::to_string_pretty(&root)
                .map_err(|e| napi::Error::from_reason(e.to_string()))
        })
}

#[napi]
pub fn parse_mdx_to_ast(input: String) -> Result<String, napi::Error> {
    crate::mdx_to_ast::parse(&input)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
        .and_then(|root| {
            serde_json::to_string_pretty(&root)
                .map_err(|e| napi::Error::from_reason(e.to_string()))
        })
}
