use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct AppState {
    pub s3: Client,
    pub raw_bucket: String,
    pub video_bucket: String,
}

impl AppState {
    pub fn raw_bucket(&self) -> &str {
        &self.raw_bucket
    }

    pub fn video_bucket(&self) -> &str {
        &self.video_bucket
    }
}
