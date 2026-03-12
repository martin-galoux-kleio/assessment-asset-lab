use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct AppState {
    pub s3: Client,
    pub bucket: String,
}

impl AppState {
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}
