use serde::{Deserialize, Serialize};
use std::env;

use crate::state::State;

#[derive(Serialize, Deserialize, Debug)]
struct OnesignalMessage {
    en: String,
    de: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct OnesignalPostBody {
    app_id: String,
    contents: OnesignalMessage,
    included_segments: Vec<String>,
}

pub async fn notify_onesignal(state: State) {
    let onesignal_token = env::var("ONESIGNAL_AUTH_TOKEN");
    let onesignal_app_id = env::var("ONESIGNAL_APP_ID");

    if onesignal_token.is_err() {
        log::warn!("No Onesignal auth token provided!");
        return;
    }

    if onesignal_app_id.is_err() {
        log::warn!("No Onesignal app id provided!");
        return;
    }

    log::warn!("hhh");

    let onesignal_token = onesignal_token.unwrap();
    let onesignal_app_id = onesignal_app_id.unwrap();

    let body = OnesignalPostBody {
        app_id: onesignal_app_id,
        contents: OnesignalMessage {
            en: format!("The presence state changed to: {}", state.to_str()),
            de: format!(
                "Der Anwesenheitsstatus wurde auf {} geändert.",
                state.to_str()
            ),
        },
        included_segments: vec!["Subscribed Users".to_string()],
    };

    let client = reqwest::Client::new();

    log::info!("Sending!");

    let res = client
        .post("https://onesignal.com/api/v1/notifications")
        // .post("https://httpbin.org/anything")
        .header(
            reqwest::header::AUTHORIZATION,
            format!("Basic {}", onesignal_token),
        )
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .json(&body)
        .send()
        .await;

    log::info!("{:?}", res);

    if res.is_err() {
        log::error!(
            "Error while trying to send post request to onesignal! Message: {:?}",
            res.err().unwrap()
        )
    }
}
