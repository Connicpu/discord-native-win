use discord::api;
use discord::models::gateway::GatewayResponse;
use error::DResult;

use futures::prelude::*;

#[async]
pub fn get() -> DResult<GatewayResponse> {
    await!(api::get_data("/api/gateway"))
}
