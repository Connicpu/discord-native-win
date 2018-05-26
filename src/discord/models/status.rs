use discord::models::snowflake::Snowflake;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum UserStatus {
    #[serde(rename = "online")]
    Online,
    #[serde(rename = "dnd")]
    Dnd,
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "invisible")]
    Invisible,
    #[serde(rename = "offline")]
    Offline,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Activity {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: i32,
    pub url: Option<String>,
    pub timestamps: Option<ActivityTimestamps>,
    pub application_id: Option<Snowflake>,
    pub details: Option<String>,
    pub state: Option<String>,
    pub party: Option<ActivityParty>,
    pub assets: Option<ActivityAssets>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct ActivityTimestamps {
    start: Option<i64>,
    end: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityParty {
    pub id: Option<String>,
    pub size: Option<(i32, i32)>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActivityAssets {
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
}
