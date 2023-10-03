// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{ops::RemAssign, collections::HashMap};

use tauri::{Manager, AppHandle};
use tokio::{self, sync::mpsc};
use windows::{core, Media::Control::{GlobalSystemMediaTransportControlsSessionManager, GlobalSystemMediaTransportControlsSession}, Foundation::TypedEventHandler, Storage::Streams::{IRandomAccessStreamReference, IRandomAccessStreamWithContentType, DataReader}};
use willhook::willhook;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Clone, serde::Serialize)]
struct Payload {
    message: String,
    event_type: EventType,
}

fn send_message<R: tauri::Runtime>(handle: &impl Manager<R>, event: &str, payload: Update) {
    handle.emit_all(event, payload).unwrap();
}

#[derive(Clone, serde::Serialize)]
pub enum EventType {
    CurrentSessionChanged,
    VolumeUp,
    VolumeDown
}


#[derive(Clone, serde::Serialize)]
struct Update {
    event_type: EventType,
    sessions: Vec<SessionData>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct SessionData {
    id: String,
    title: String,
    thumbnail: Thumbnail,
}

#[derive(Debug, serde::Serialize, Clone)]
struct Thumbnail {
    content_type: String,
    data: Vec<u8>,
}

fn get_thumbnail(stream: IRandomAccessStreamReference) -> core::Result<Thumbnail> {
    let read = stream.OpenReadAsync()?;
    let stream = read.get()?;
    let content_type = stream.ContentType()?.to_string();
    let data = read_stream(stream)?;
    let thumbnail = Thumbnail {
        content_type: content_type,
        data: data,
    };
    Ok(thumbnail)
}

fn read_stream(stream: IRandomAccessStreamWithContentType) -> core::Result<Vec<u8>> {
    let stream_len = stream.Size()? as usize;
    let mut data = vec![0u8; stream_len];
    let reader = DataReader::CreateDataReader(&stream)?;

    reader.LoadAsync(stream_len as u32)?.get().ok();
    reader.ReadBytes(&mut data).ok();

    reader.Close().ok();
    stream.Close().ok();

    Ok(data)
}

async fn get_sessions_data(manager: &GlobalSystemMediaTransportControlsSessionManager) -> core::Result<Vec<SessionData>> {
    let updated: core::Result<Vec<(String, GlobalSystemMediaTransportControlsSession)>> = manager
                        .GetSessions()?
                        .into_iter()
                        .map(|session| Ok((session.SourceAppUserModelId()?.to_string(), session)))
                        .collect();

    let updated = updated?;
    let mut sessions: Vec<SessionData> = Vec::new();

    for (id, session) in updated {
        let session_data = session.TryGetMediaPropertiesAsync()?.await?;
        
        let thumbnail_stream = session_data.Thumbnail()?;

        let thumbnail = get_thumbnail(thumbnail_stream)?;

        let session = SessionData {
            id: id,
            title: session_data.Title()?.to_string(),
            thumbnail: thumbnail,
        };

        sessions.push(session);
    }

    Ok(sessions)

}

#[tokio::main]
async fn main() -> core::Result<()> {

    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?.await?;
    
    let m = manager.clone();

    //get_sessions_data(m).await?;
    
    let (tx, mut rx) = mpsc::unbounded_channel::<EventType>();
    
    tokio::spawn(async move {

        println!("1");

        let kb_hook = willhook().unwrap();
        println!("2");

        loop {
            if let Ok(ie) = kb_hook.try_recv() {
                match ie {
                    willhook::InputEvent::Keyboard(ke) => {
                        let pressed = ke.pressed;
                        let manager_clone = manager.clone();
                        match pressed {
                            willhook::KeyPress::Up(_) => {
                                //let sessions_data = get_sessions_data(manager_clone).unwrap();
                            },
                            willhook::KeyPress::Down(_) => {
                                if ke.key.is_some() {
                                    //willhook::KeyboardKey::Other(174);
                                    let key = ke.key.unwrap();

                                    match key {
                                        willhook::KeyboardKey::Other(174) => {tx.send(EventType::VolumeDown).ok();},
                                        willhook::KeyboardKey::Other(175) => {tx.send(EventType::VolumeUp).ok();},
                                        _ => {}
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            } else {
                std::thread::yield_now();
            }
        };
    });


    let manager = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().unwrap().await.unwrap();
    let manager_clone = manager.clone();

    /*manager.CurrentSessionChanged(&TypedEventHandler::new(move |_, _| {
        tx.send(EventType::CurrentSessionChanged).ok();

        Ok(())
    })).ok();*/

    tauri::Builder::default()
        .setup(|app| {
            let app_handle = app.app_handle();
            
            tauri::async_runtime::spawn(async move {

                while let Some(event) = rx.recv().await {
                    let sessions = get_sessions_data(&manager_clone).await.unwrap();
                    let payload = Update { event_type: event, sessions };
                    send_message(&app_handle, "core://update", payload);
                    //send_message(&app_handle, "core://update", Payload {message: "Update".into(), event_type: event});
                }

                /*loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    send_message(&app_handle, "core://update", Payload {message: "yes".into()});
                }*/
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())

}
