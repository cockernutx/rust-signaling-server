use serde::de::{Error, MapAccess, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};

use super::{IceCandidate, SessionDescriptionMessage, Signal};

impl<'de> Deserialize<'de> for Signal {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(SignalVisitor)
    }
}

pub struct SignalVisitor;
impl<'de> Visitor<'de> for SignalVisitor {
    type Value = Signal;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "couldn't parse Signal type")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        while let Some((key, value)) = map.next_entry()? as Option<(String, &'de str)> {
            if key == "type" {
                return match value {
                    "offer" => Ok(Signal::Offer(SessionDescriptionVisitor.visit_map(map)?)),
                    "answer" => Ok(Signal::Answer(SessionDescriptionVisitor.visit_map(map)?)),
                    "new_ice_candidate" => {
                        Ok(Signal::NewIceCandidate(IceCandidateVisitor.visit_map(map)?))
                    }
                    "assign" => Ok(Signal::assign(
                        AssignedNameVisitor.visit_map(map)?.to_owned(),
                    )),
                    others => Err(M::Error::invalid_value(
                        Unexpected::Str(others),
                        &"offer, answer, new_ice_candidate, assign",
                    )),
                };
            }
        }

        Err(M::Error::missing_field("type"))
    }
}

pub struct SessionDescriptionVisitor;
impl<'de> Visitor<'de> for SessionDescriptionVisitor {
    type Value = SessionDescriptionMessage;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "couldn't parse Signal type")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut target: Result<String, M::Error>  = Err(M::Error::missing_field("target"));
        let mut name: Result<String, M::Error> = Err(M::Error::missing_field("name"));
        let mut sdp: Result<String, M::Error>  = Err(M::Error::missing_field("sdp"));

        while let Some((key, value)) = map.next_entry()? {
            match key {
                "target" => target = Ok(value),
                "name" => name = Ok(value),
                "sdp" => sdp = Ok(value),
                _ => continue,
            }
        }

        Ok(SessionDescriptionMessage {
            name: name?.to_owned(),
            target: target?.to_owned(),
            sdp: sdp?.to_owned(),
        })
    }
}

pub struct IceCandidateVisitor;
impl<'de> Visitor<'de> for IceCandidateVisitor {
    type Value = IceCandidate;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "couldn't parse Signal type")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut target: Result<String, M::Error> = Err(M::Error::missing_field("target"));
        let mut candidate: Result<String, M::Error> = Err(M::Error::missing_field("candidate"));
        let mut name: Result<String, M::Error> = Err(M::Error::missing_field("name"));

        while let Some((key, value)) = map.next_entry()? {
            match key {
                "target" => target = Ok(value),
                "candidate" => candidate = Ok(value),
                "name" => name = Ok(value),
                _ => continue,
            }
        }

        Ok(IceCandidate {
            target: target?.to_owned(),
            candidate: candidate?.to_owned(),
            name: name?.to_owned()
        })
    }
}

pub struct AssignedNameVisitor;
impl<'de> Visitor<'de> for AssignedNameVisitor {
    type Value = &'de str;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "couldn't parse Signal type")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        while let Some((key, value)) = map.next_entry()? as Option<(&'de str, &'de str)> {
            if key == "name" {
                return Ok(value);
            }
        }

        Err(M::Error::missing_field("name"))
    }
}

#[test]
fn test_deserealizing_offer_signal() {
    use super::SessionDescriptionMessage;

    let offer_signal_text = r#"{"type":"offer","name":"3872379c-4743-4a7d-b2ee-79cf7368cf58","target":"4fe681ad-aba1-4732-89df-ee784b7d4abf","sdp":"sdp"}"#;

    let offer_signal_struct = Signal::Offer(SessionDescriptionMessage {
        target: "4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned(),
        name: "3872379c-4743-4a7d-b2ee-79cf7368cf58".to_owned(),
        sdp: "sdp".to_owned(),
    });

    assert_eq!(
        serde_json::from_str::<Signal>(offer_signal_text).unwrap(),
        offer_signal_struct
    );
}

#[test]
fn test_deserializing_answer_signal() {
    use super::SessionDescriptionMessage;

    let answer_signal_text = r#"{"type":"answer","name":"3872379c-4743-4a7d-b2ee-79cf7368cf58","target":"4fe681ad-aba1-4732-89df-ee784b7d4abf","sdp":"sdp"}"#;

    let answer_signal_struct = Signal::Answer(SessionDescriptionMessage {
        target: "4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned(),
        name: "3872379c-4743-4a7d-b2ee-79cf7368cf58".to_owned(),
        sdp: "sdp".to_owned(),
    });

    assert_eq!(
        serde_json::from_str::<Signal>(&answer_signal_text).unwrap(),
        answer_signal_struct
    );
}

#[test]
fn test_deserializing_new_ice_candidate_signal() {
    use super::IceCandidate;

    let new_ice_candidate_text = r#"{"type":"new_ice_candidate","name":"3872379c-4743-4a7d-b2ee-79cf7368cf58","target":"4fe681ad-aba1-4732-89df-ee784b7d4abf","candidate":"candidate"}"#;

    let new_ice_candidate_struct = Signal::NewIceCandidate(IceCandidate {
        target: "4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned(),
        candidate: "candidate".to_owned(),
        name: "3872379c-4743-4a7d-b2ee-79cf7368cf58".to_owned()
    });

    assert_eq!(
        serde_json::from_str::<Signal>(&new_ice_candidate_text).unwrap(),
        new_ice_candidate_struct
    );
}

#[test]
fn test_serializing_assign_message() {
    let assign_message_text = r#"{"type":"assign","name":"4fe681ad-aba1-4732-89df-ee784b7d4abf"}"#;

    let assign_message_struct = Signal::Assign("4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned());

    assert_eq!(
        serde_json::from_str::<Signal>(&assign_message_text).unwrap(),
        assign_message_struct
    );
}
