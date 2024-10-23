use dotenv::dotenv;
use flowsnet_platform_sdk::logger;
use http_req::{
    request::{Method, Request},
    uri::Uri,
};
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
    dotenv().ok();

    logger::init();
    let telegram_token = env::var("telegram_token").unwrap();
    let tele = Telegram::new(telegram_token.clone());

    // if let UpdateKind::Message(msg) = update.kind {
    //     let chat_id = msg.chat.id;

    //     if let Some(text) = msg.text() {
    //         if text == "/start" {
    //             let init_message = "Hello! I am your medical lab report analyzer bot. Zoom in on where you need assistance with, take a photo and upload it as a file, or paste the photo in the chatbox to send me if you think it's clear enough.";
    //             let _ = tele.send_message(chat_id, init_message.to_string());
    //             return;
    //         }
    //     }

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
                let _ = download_video_data_and_(&telegram_token, &video_file_id).await;
            }

            let _ = tele.send_message(chat_id, "received msg in channel".to_string());
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
                let _ = download_video_data_and_(&telegram_token, &video_file_id).await;
            }
            let _ = tele.send_message(chat_id, "received msg".to_string());
        }
        _ => unreachable!(),
    }
}

pub async fn download_video_data_and_(
    token: &str,
    file_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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

    log::info!("video file downloaded, sized: {}", file_data.len());

    let _ = upload_video_to_gaianet(&file_data).await;

    Ok(())
}

pub async fn upload_video_to_gaianet_no_soundid(
    file_data: &[u8],
    video_name: &str,
) -> anyhow::Result<()> {
    use rand::{distributions::Alphanumeric, Rng}; // Import the required traits

    let boundary: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let upload_video_base_url = "https://video-translator.gaianet.ai/upload";
    let mut body = Vec::new();

    let fields = [
        ("file", video_name),
        ("email_link", "jaykchen@gmail.com"),
        ("language", "zh"),
    ];

    for (name, value) in fields.iter() {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"video.mp4\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: video/mp4\r\n\r\n");
    body.extend_from_slice(file_data);
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let content_type = format!("multipart/form-data; boundary={}", boundary);

    let mut writer = Vec::new();

    let uri: Uri = Uri::try_from(upload_video_base_url).unwrap();

    let response = Request::new(&uri)
        .method(Method::POST)
        .header("Content-Type", &content_type)
        .body(&body)
        .send(&mut writer)?;

    log::info!("Status: {}", response.status_code());
    log::info!("Response: {}", String::from_utf8_lossy(&writer));

    Ok(())
}

pub async fn upload_video_to_gaianet(file_data: &[u8]) -> anyhow::Result<()> {
    use rand::{distributions::Alphanumeric, Rng}; // Import the required traits

    let boundary: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    let upload_video_base_url = "https://video-translator.gaianet.ai/runCodeByUrl";
    let mut body = Vec::new();

    let fields = [
        ("email_link", "jaykchen@gmail.com"),
        ("resultType", "0"),
        ("soundId", "59cb5986671546eaa6ca8ae6f29f6d22"),
        ("language", "zh"),
    ];

    for (name, value) in fields.iter() {
        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name).as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }

    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"file\"; filename=\"video.mp4\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: video/mp4\r\n\r\n");
    body.extend_from_slice(file_data);
    body.extend_from_slice(b"\r\n");

    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());

    let content_type = format!("multipart/form-data; boundary={}", boundary);

    let mut writer = Vec::new();

    let uri: Uri = Uri::try_from(upload_video_base_url).unwrap();

    let response = Request::new(&uri)
        .method(Method::POST)
        .header("Content-Type", &content_type)
        .body(&body)
        .send(&mut writer)?;

    log::info!("Status: {}", response.status_code());
    log::info!("Response: {}", String::from_utf8_lossy(&writer));

    Ok(())
}
