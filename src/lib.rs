use base64;
use cloud_vision_flows::text_detection;
use flowsnet_platform_sdk::logger;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use openai_flows::{chat_completion, ChatModel, ChatOptions};
use serde_json::Value;
use std::env;
use tg_flows::{listen_to_update, update_handler, Telegram, Update, UpdateKind};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    let telegram_token = std::env::var("telegram_token").unwrap();
    listen_to_update(telegram_token).await;
}

#[update_handler]
async fn handler(update: Update) {
    logger::init();
    let telegram_token = std::env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());

    if let UpdateKind::Message(msg) = update.kind {
        let chat_id = msg.chat.id;

        if let Some(text) = msg.text() {
            if text == "/start" {
                let init_message = "Hello! I am your medical lab report analyzer bot. Zoom in on where you need assistance with, take a photo and upload it as a file, or paste the photo in the chatbox to send me if you think it's clear enough.";
                let _ = tele.send_message(chat_id, init_message.to_string());
                return;
            }
        }

        let image_file_id = match (msg.document().is_some(), msg.photo().is_some()) {
            (true, false) => msg.document().unwrap().file.id.clone(),
            (false, true) => msg.photo().unwrap().last().unwrap().file.id.clone(),
            (_, _) => {
                return;
            }
        };

        match download_photo_data_base64(&telegram_token, &image_file_id) {
            Ok(data) => {
                if let Ok(ocr_text) = text_detection(data) {
                    let text = if !ocr_text.is_empty() {
                        ocr_text
                    } else {
                        "".to_string()
                    };

                    if let Some(c) = c {
                        if c.restarted {
                            // _ = tele.send_message(chat_id, "I am starting a new session. You can always type \"restart\" to terminate the current session.\n\n".to_string() + &c.choice);
                        } else {
                            _ = tele.send_message(chat_id, c.choice);
                        }
                    }
                }
            }
            Err(_e) => {
                eprintln!("Error downloading photo");
                return;
            }
        };
    }
}

pub fn download_photo_data_base64(
    token: &str,
    file_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let file_url = format!(
        "https://api.telegram.org/bot{}/getFile?file_id={}",
        token, file_id
    );
    let file_uri: Uri = Uri::try_from(file_url.as_str()).unwrap();

    let mut file_response = Vec::new();
    Request::new(&file_uri)
        .method(Method::GET)
        .send(&mut file_response)?;

    let file_info: Value = serde_json::from_slice(&file_response)?;
    let file_path = file_info["result"]["file_path"]
        .as_str()
        .ok_or("file_path missing")?;

    // Download the file using the file path
    let file_download_url = format!("https://api.telegram.org/file/bot{}/{}", token, file_path);
    let file_download_uri: Uri = Uri::try_from(file_download_url.as_str()).unwrap();

    let mut file_data = Vec::new();
    Request::new(&file_download_uri)
        .method(Method::GET)
        .send(&mut file_data)?;
    let base64_encoded = base64::encode(file_data);

    Ok(base64_encoded)
}
