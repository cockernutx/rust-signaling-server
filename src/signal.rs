use super::error::Error;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum Signal {
    Offer(SessionDescriptionMessage),
    Answer(SessionDescriptionMessage),
    NewIceCandidate(IceCandidate),
    Assign(AssignName),
    Error(Error),
    ConnectedList(Vec<String>)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssignName {
    pub name: String
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionDescriptionMessage {
    pub target: String,
    pub name: String,
    sdp: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IceCandidate {
    pub target: String,
    candidate: String,
    pub name: String,
}

impl Signal {
    pub fn assign(user_name: String) -> Signal {
        Signal::Assign(AssignName { name: user_name })
    }

    /*pub fn from_string(json_string: &str) -> Result<Signal, Error> {
        let json_value: serde_json::Value = serde_json::from_str(&json_string)?;
        let binding = json_value["type"].to_string().replace("\"", "");
        let req_type = binding.as_str();

        match req_type{
            "offer" => {
                let res: SessionDescriptionMessage = serde_json::from_str(&json_string)?;
                return Ok(Signal::Offer(res))
            }
            "answer" => {
                let res: SessionDescriptionMessage = serde_json::from_str(&json_string)?;
                return Ok(Signal::Answer(res))
            }
            "new_ice_candidate" => {
                let res: IceCandidate = serde_json::from_str(&json_string)?;
                return Ok(Signal::NewIceCandidate(res))
            }
            _ => {
                return Err(Error::ParseError(serde::de::Error::invalid_value(serde::de::Unexpected::Str(req_type), &"offer | answer | new_ice_candidate")))
            }
        };
    }*/
}