#[derive(Clone, Debug, Deserialize)]
pub struct GatewayResponse {
    pub url: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct GatewayBotResponse {
    pub url: String,
    pub shards: i32,
}
