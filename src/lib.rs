use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use form_urlencoded;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::error::Error;
use tg_flows::{listen_to_update, update_handler, Telegram, Update, UpdateKind};

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn on_deploy() {
    let telegram_token = std::env::var("telegram_token").unwrap();
    listen_to_update(telegram_token).await;
}

#[update_handler]
async fn handler(update: Update) {
    dotenv().ok();

    logger::init();
    let telegram_token = env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());
    let msg_url = format!("https://api.telegram.org/bot{}/sendMessage", telegram_token);

    match update.kind {
        UpdateKind::ChannelPost(msg) => {
            let chat_id = msg.chat.id;
            log::info!("channel post msg: {}", chat_id);

            if let Some(t) = msg.text() {
                log::info!("echoing msg text: {}", t);
            }
            if let Some(_) = msg.video() {
                let video_file_id = msg.video().unwrap().file.id.clone();

                log::info!("video file id: {}", video_file_id.clone());
                let video_file_path = get_video_file_path(&telegram_token, &video_file_id)
                    .await
                    .expect("failed to get video file path");

                log::info!("video file path: {}", video_file_path.clone());

                let res = upload_video_to_gaianet_w_return(
                    &video_file_path,
                    &msg_url,
                    &chat_id.to_string(),
                )
                .expect("upload failed")
                .to_string();
                let _ = tele.send_message(chat_id, &res);
            }
        }

        UpdateKind::Message(msg) => {
            let chat_id = msg.chat.id;
            log::info!("channel post msg: {}", chat_id);
            if let Some(t) = msg.text() {
                log::info!("echoing msg text: {}", t);
            }

            if let Some(_) = msg.video() {
                let video_file_id = msg.video().unwrap().file.id.clone();

                log::info!("video file id: {}", video_file_id.clone());
                let video_file_path = get_video_file_path(&telegram_token, &video_file_id)
                    .await
                    .expect("failed to get video file path");

                log::info!("video file id: {}", video_file_path.clone());

                let res = upload_video_to_gaianet_w_return(
                    &video_file_path,
                    &msg_url,
                    &chat_id.to_string(),
                )
                .expect("upload failed")
                .to_string();
                let _ = tele.send_message(chat_id, &res);
            }
        }
        _ => unreachable!(),
    }
}

pub async fn get_video_file_path(token: &str, file_id: &str) -> Result<String, Box<dyn Error>> {
    let file_url = format!(
        "https://api.telegram.org/bot{}/getFile?file_id={}",
        token, file_id
    );

    let client = Client::new();

    let response = client.get(&file_url).send().await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        log::error!("Failed to get file. Status: {}, Response: {}", status, text);
        return Err(format!("Telegram API request failed with status {}", status).into());
    }

    let body = response.text().await?;
    log::info!("Response payload: {:?}", body);

    #[derive(Deserialize)]
    struct ApiResponse {
        ok: bool,
        #[serde(rename = "result")]
        inner: Inner,
    }

    #[derive(Deserialize)]
    struct Inner {
        file_path: String,
    }

    let load: ApiResponse = serde_json::from_str(&body)?;
    let file_path = load.inner.file_path;
    let path = format!("https://api.telegram.org/file/bot{}/{}", token, file_path);

    Ok(path)
}

pub fn upload_video_to_gaianet_by_url(
    video_file_path: &str,
    email: &str,
) -> anyhow::Result<String> {
    let form_data = form_urlencoded::Serializer::new(String::new())
        .append_pair("url", video_file_path)
        .append_pair("email_link", email)
        .append_pair("resultType", "1")
        .append_pair("soundId", "59cb5986671546eaa6ca8ae6f29f6d22")
        .append_pair("language", "zh")
        .finish();

    let body_bytes = form_data.as_bytes();

    let uri = Uri::try_from("https://videolangua.com /runCodeByUrl")?;

    let mut request = Request::new(&uri);
    request
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", &body_bytes.len().to_string())
        .body(body_bytes);

    let mut writer = Vec::new();

    let response = request.send(&mut writer).map_err(|e| anyhow::anyhow!(e))?;

    println!("Status: {} {}", response.status_code(), response.reason());
    println!("Headers: {}", response.headers());
    let res = String::from_utf8_lossy(&writer).to_string();
    println!("Response: {}", res);

    Ok(res)
}

pub fn upload_video_to_gaianet_w_return(
    video_file_path: &str,
    msg_url: &str,
    chat_id: &str,
) -> anyhow::Result<String> {
    let form_data = form_urlencoded::Serializer::new(String::new())
        .append_pair("url", video_file_path)
        .append_pair("msg_url", msg_url)
        .append_pair("chat_id", chat_id)
        .append_pair("resultType", "1")
        .append_pair("soundId", "59cb5986671546eaa6ca8ae6f29f6d22")
        .append_pair("language", "zh")
        .finish();

    log::info!("msg_url: {:?}", msg_url);
    log::info!("chat_id: {:?}", chat_id);

    let body_bytes = form_data.as_bytes();

    let uri = Uri::try_from("https://video-translator.gaianet.ai/runCodeByUrl")?;

    let mut request = Request::new(&uri);
    request
        .method(Method::POST)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", &body_bytes.len().to_string())
        .body(body_bytes);

    let mut writer = Vec::new();

    let response = request.send(&mut writer).map_err(|e| anyhow::anyhow!(e))?;

    println!("Status: {} {}", response.status_code(), response.reason());
    println!("Headers: {}", response.headers());
    let res = String::from_utf8_lossy(&writer).to_string();
    println!("Response: {}", res);

    Ok(res)
}
