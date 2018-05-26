use discord::models::status::Activity;

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

#[macro_use]
mod macros;

//--------------------
// Packet structs

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Heartbeat(pub Option<i32>);
packet_payload!(Heartbeat, op: 1);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Identify<'a> {
    pub token: Cow<'a, str>,
    pub properties: IdentifyProperties<'a>,
    pub compress: bool,
    pub large_threshold: Option<i32>,
    pub shard: Option<(i32, i32)>,
    pub presence: Option<UpdateStatus<'a>>, // TODO: Prescence object
}
packet_payload!(Identify<'a>, op: 2);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateStatus<'a> {
    pub since: Option<i64>,
    pub game: Option<Activity>,
    pub status: Cow<'a, str>,
    pub afk: bool,
}

#[derive(Copy, Clone, Debug, Default, Serialize)]
pub struct HeartbeatAck;
packet_payload!(HeartbeatAck, op: 11, skip: true);

//--------------------
// Support structs

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentifyProperties<'a> {
    #[serde(rename = "$os")]
    pub os: Cow<'a, str>,
    #[serde(rename = "$browser")]
    pub browser: Cow<'a, str>,
    #[serde(rename = "$device")]
    pub device: Cow<'a, str>,
}

//--------------------
// Packet helpers

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "T: PacketData")]
pub struct Packet<T>
where
    T: PacketData,
{
    #[serde(rename = "op")]
    pub opcode: u32,

    #[serde(skip_serializing_if = "PacketData::skip_ser")]
    #[serde(rename = "d")]
    pub payload: T,

    #[serde(rename = "s")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sequence: Option<i32>,

    #[serde(rename = "e")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<Cow<'static, str>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "T: PacketData")]
pub struct DataOnlyPacket<T>
where
    T: PacketData,
{
    #[serde(skip_serializing_if = "PacketData::skip_ser")]
    #[serde(rename = "d")]
    pub payload: T,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PartialPacket<'a> {
    #[serde(rename = "op")]
    pub opcode: u32,

    #[serde(rename = "s")]
    pub sequence: Option<i32>,

    #[serde(rename = "e")]
    #[serde(borrow)]
    pub event: Option<Cow<'a, str>>,
}

pub trait PacketData: Serialize + for<'de> Deserialize<'de> + Sized {
    const OPCODE: u32;
    const EVENT: Option<&'static str> = None;

    fn skip_ser(&self) -> bool {
        false
    }
}
