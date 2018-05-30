use discord::gateway::Dispatcher;
use error::DResult;

use futures::prelude::*;

#[async]
pub fn start_heartbeat(_dispatcher: Dispatcher) -> DResult<()> {
    unimplemented!()
}
