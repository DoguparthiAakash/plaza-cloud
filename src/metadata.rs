use plaza_foundation::core::PlazaResult;

pub struct MetadataProxy;

impl MetadataProxy {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn fetch_metadata(&self, _key: &str) -> PlazaResult<String> {
        Ok("{}".to_string()) // DP1 Stub
    }
}
