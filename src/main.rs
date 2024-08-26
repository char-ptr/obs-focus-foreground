use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Context;
use obws::{
    client::Client,
    requests::inputs::{Create, InputId, SetSettings},
};
use serde_json::Value;
use windows::Win32::{
    Foundation::{ERROR_LOG_SECTOR_PARITY_INVALID, HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowModuleFileNameA, GetWindowTextA, SetWindowsHookA,
        SetWindowsHookExA, EVENT_SYSTEM_FOREGROUND, WINDOWS_HOOK_ID,
    },
};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Client::connect("localhost", 4455, Some("nDZC10TiyCnrNXp6")).await?;
    let current_scene = client.scenes().current_program_scene().await?;
    let inputse = client.inputs().list(None).await?;
    let input_uuid = match inputse.iter().find(|e| e.id.name == "focused") {
        Some(inp) => inp.id.uuid,
        None => {
            client
                .inputs()
                .create(Create::<'_, ()> {
                    kind: "window_capture",
                    input: "focused",
                    settings: None,
                    scene: obws::requests::scenes::SceneId::Uuid(current_scene.id.uuid),
                    enabled: Some(true),
                })
                .await?
                .input_uuid
        }
    };
    let focused_input_id = InputId::Uuid(input_uuid);
    let inputs = client.inputs();
    let mut last_name = String::new();
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        unsafe {
            let Ok(name) = get_foreground_window_name() else {
                eprintln!("unable to get foreground");
                continue;
            };
            if (name == last_name) {
                continue;
            }
            let pathed = Path::new(&name);
            println!("namme ={name}");
            let Some(file_name) = pathed.file_name() else {
                eprintln!("unable to get fiel_name");
                continue;
            };
            let Some(file_name) = file_name.to_str() else {
                eprintln!("unable to get fiel_name str");
                continue;
            };
            let Ok(props) = inputs
                .properties_list_property_items(focused_input_id, "window")
                .await
            else {
                eprintln!("unable to get rpops");
                continue;
            };
            let Some(found) = props.iter().find(|x| x.name.contains(file_name)) else {
                eprintln!("not found");
                continue;
            };
            let mut settings = inputs.settings::<Value>(focused_input_id).await?;
            let Some(window_opts) = settings.settings.as_object_mut() else {
                continue;
            };
            let windower = window_opts
                .insert("window".to_string(), found.to_owned().value)
                .expect("window gone??!");

            last_name = name;
            inputs
                .set_settings(SetSettings::<'_, Value> {
                    input: focused_input_id,
                    overlay: None,
                    settings: &Value::Object(window_opts.to_owned()),
                })
                .await?;
        }
    }
    Ok(())
}
unsafe fn get_foreground_window_name() -> anyhow::Result<String> {
    let hwnd = GetForegroundWindow();
    let mut buf = [0; 256];
    GetWindowTextA(hwnd, &mut buf);
    let new: Vec<u8> = buf.iter().filter(|x| **x != 0).copied().collect();
    String::from_utf8(new).context("womp")
}
