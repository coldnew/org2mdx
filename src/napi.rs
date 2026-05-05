use napi_derive::napi;

#[napi]
pub fn convert(input: String) -> Result<String, napi::Error> {
    crate::org_to_mdx::convert(&input).map_err(|e| napi::Error::from_reason(e.to_string()))
}
