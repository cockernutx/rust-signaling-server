use super::Signal;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

impl Serialize for Signal {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Signal::Offer(sdp_signal) => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("type", "offer")?;
                map.serialize_entry("name", &sdp_signal.name)?;
                map.serialize_entry("target", &sdp_signal.target)?;
                map.serialize_entry("sdp", &sdp_signal.sdp)?;
                map.end()
            }
            Signal::Answer(sdp_signal) => {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("type", "answer")?;
                map.serialize_entry("name", &sdp_signal.name)?;
                map.serialize_entry("target", &sdp_signal.target)?;
                map.serialize_entry("sdp", &sdp_signal.sdp)?;
                map.end()
            }
            Signal::NewIceCandidate(ice_candidate) => {
                let mut map = serializer.serialize_map(Some(3))?;
                map.serialize_entry("type", "new_ice_candidate")?;
                map.serialize_entry("target", &ice_candidate.target)?;
                map.serialize_entry("candidate", &ice_candidate.candidate)?;
                map.serialize_entry("name", &ice_candidate.name)?;
                map.end()
            }
            Signal::Assign(user_name) => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "assign")?;
                map.serialize_entry("name", &user_name)?;
                map.end()
            }
        }
    }
}

#[test]
fn test_serealizing_offer_signal() {
    use super::SessionDescriptionMessage;

    let offer_signal_struct = Signal::Offer(SessionDescriptionMessage {
        target: "4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned(),
        name: "3872379c-4743-4a7d-b2ee-79cf7368cf58".to_owned(),
        sdp: "sdp".to_owned(),
    });

    let offer_signal_text = r#"{"type":"offer","name":"3872379c-4743-4a7d-b2ee-79cf7368cf58","target":"4fe681ad-aba1-4732-89df-ee784b7d4abf","sdp":"sdp"}"#;

    assert_eq!(
        &serde_json::to_string(&offer_signal_struct).unwrap(),
        offer_signal_text
    );
}

#[test]
fn test_serializing_answer_signal() {
    use super::SessionDescriptionMessage;

    let answer_signal_struct = Signal::Answer(SessionDescriptionMessage {
        target: "4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned(),
        name: "3872379c-4743-4a7d-b2ee-79cf7368cf58".to_owned(),
        sdp: "sdp".to_owned(),
    });

    let answer_signal_text = r#"{"type":"answer","name":"3872379c-4743-4a7d-b2ee-79cf7368cf58","target":"4fe681ad-aba1-4732-89df-ee784b7d4abf","sdp":"sdp"}"#;

    assert_eq!(
        &serde_json::to_string(&answer_signal_struct).unwrap(),
        answer_signal_text
    );
}

#[test]
fn test_serializing_new_ice_candidate_signal() {
    use super::IceCandidate;

    let new_ice_candidate_struct = Signal::NewIceCandidate(IceCandidate {
        target: "4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned(),
        candidate: "candidate".to_owned(),
        name: "3872379c-4743-4a7d-b2ee-79cf7368cf58".to_owned()
    });

    let ice_candidate_text = r#"{"type":"new_ice_candidate","target":"4fe681ad-aba1-4732-89df-ee784b7d4abf","candidate":"candidate"}"#;

    assert_eq!(
        &serde_json::to_string(&new_ice_candidate_struct).unwrap(),
        ice_candidate_text
    );
}

#[test]
fn test_serializing_assign_message() {
    let assign_message_struct = Signal::Assign("4fe681ad-aba1-4732-89df-ee784b7d4abf".to_owned());

    let assign_message_text = r#"{"type":"assign","name":"4fe681ad-aba1-4732-89df-ee784b7d4abf"}"#;

    assert_eq!(
        &serde_json::to_string(&assign_message_struct).unwrap(),
        assign_message_text
    );
}
