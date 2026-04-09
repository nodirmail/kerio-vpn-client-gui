use serde::{Deserialize, Serialize};
use std::process::Command;
use std::fs;
use std::sync::Mutex;
use tauri::{Manager, AppHandle, State, Wry};
use tauri::tray::{TrayIconBuilder, MouseButton, MouseButtonState, TrayIcon};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItemBuilder};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VpnConfig {
    pub server: String,
    pub username: String,
    pub password: Option<String>,
    pub save_password: bool,
    pub persistent: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub config: VpnConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VpnStatus {
    pub state: String, // "connected" or "disconnected"
    pub active_profile_id: Option<String>,
}

pub struct AppState {
    pub profiles: Mutex<Vec<Profile>>,
    pub active_profile_id: Mutex<Option<String>>,
}

fn get_profiles_path(app: &AppHandle) -> std::path::PathBuf {
    let mut path = app.path().app_config_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    fs::create_dir_all(&path).ok();
    path.push("profiles.json");
    path
}

fn load_profiles_from_disk(app: &AppHandle) -> Vec<Profile> {
    let path = get_profiles_path(app);
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(profiles) = serde_json::from_str(&data) {
            return profiles;
        }
    }
    Vec::new()
}

fn get_active_profile_from_system(profiles: &[Profile]) -> Option<String> {
    let output = Command::new("systemctl")
        .arg("is-active")
        .arg("kerio-kvc")
        .output()
        .ok()?;
        
    let is_active = String::from_utf8_lossy(&output.stdout).trim() == "active";
    if !is_active {
        return None;
    }

    let config_output = Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg("cat /etc/kerio-kvc.conf")
        .output()
        .ok()?;

    if !config_output.status.success() {
        return None;
    }

    let config_xml = String::from_utf8_lossy(&config_output.stdout);
    if let Some(start) = config_xml.find("<server>") {
        if let Some(end) = config_xml[start..].find("</server>") {
            let server_val = &config_xml[start + 8..start + end];
            for p in profiles {
                if p.config.server.starts_with(server_val) {
                    return Some(p.id.clone());
                }
            }
        }
    }
    None
}

fn save_profiles_to_disk(app: &AppHandle, profiles: &[Profile]) {
    let path = get_profiles_path(app);
    let data = serde_json::to_string_pretty(profiles).unwrap_or_default();
    fs::write(path, data).ok();
}

#[tauri::command]
fn get_profiles(state: State<'_, AppState>) -> Vec<Profile> {
    state.profiles.lock().unwrap().clone()
}

#[tauri::command]
fn save_profile(app: AppHandle, state: State<'_, AppState>, profile: Profile) -> Result<(), String> {
    let mut profiles = state.profiles.lock().unwrap();
    if let Some(p) = profiles.iter_mut().find(|p| p.id == profile.id) {
        *p = profile;
    } else {
        profiles.push(profile);
    }
    save_profiles_to_disk(&app, &profiles);
    let active_id = state.active_profile_id.lock().unwrap();
    update_tray_menu(&app, &profiles, &active_id)?;
    Ok(())
}

#[tauri::command]
fn delete_profile(app: AppHandle, state: State<'_, AppState>, id: String) -> Result<(), String> {
    let mut profiles = state.profiles.lock().unwrap();
    profiles.retain(|p| p.id != id);
    save_profiles_to_disk(&app, &profiles);
    let active_id = state.active_profile_id.lock().unwrap();
    update_tray_menu(&app, &profiles, &active_id)?;
    Ok(())
}

fn update_tray_menu(app: &AppHandle, profiles: &[Profile], active_id: &Option<String>) -> Result<(), String> {
    let tray = app.tray_by_id("main").ok_or("Tray not found")?;
    
    let mut menu_items: Vec<Box<dyn tauri::menu::IsMenuItem<Wry>>> = Vec::new();

    for profile in profiles {
        let is_active = active_id.as_ref() == Some(&profile.id);
        let item = CheckMenuItemBuilder::new(&profile.name)
            .id(format!("prof_{}", profile.id))
            .checked(is_active)
            .build(app)
            .map_err(|e| e.to_string())?;
        menu_items.push(Box::new(item));
    }

    if !profiles.is_empty() {
        menu_items.push(Box::new(PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?));
    }

    let disconnect_i = MenuItem::with_id(app, "disconnect", "Disconnect", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(disconnect_i));

    menu_items.push(Box::new(PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?));

    let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(settings_i));

    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu_items.push(Box::new(quit_i));

    let items_refs: Vec<&dyn tauri::menu::IsMenuItem<Wry>> = menu_items.iter().map(|i| i.as_ref()).collect();
    let menu = Menu::with_items(app, &items_refs).map_err(|e| e.to_string())?;
    
    tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_config() -> Result<VpnConfig, String> {
    Ok(VpnConfig {
        server: "vpn.example.com".to_string(),
        username: "user".to_string(),
        password: Some("".to_string()),
        save_password: true,
        persistent: false,
    })
}

fn generate_xml(config: &VpnConfig) -> Result<String, String> {
    let mut parts = config.server.split(':');
    let host = parts.next().unwrap_or(&config.server);
    let port = parts.next().unwrap_or("4090");
    let server_addr = format!("{}:{}", host, port);

    let openssl_cmd = format!(
        "openssl s_client -connect {} -showcerts </dev/null 2>/dev/null | openssl x509 -fingerprint -noout -md5",
        server_addr
    );
    let fp_output = Command::new("sh")
        .arg("-c")
        .arg(&openssl_cmd)
        .output()
        .map_err(|e| format!("Failed to get fingerprint: {}", e))?;

    let fp_str = String::from_utf8_lossy(&fp_output.stdout);
    let fingerprint = fp_str.replace("md5 Fingerprint=", "").replace("MD5 Fingerprint=", "").trim().to_string();

    if fingerprint.is_empty() {
        return Err("Cannot fetch server certificate fingerprint. Is the URL correct?".into());
    }

    Ok(format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<config>
    <connections>
        <connection type="persistent">
            <server>{0}</server>
            <port>{1}</port>
            <username>{2}</username>
            <password>{3}</password>
            <fingerprint>{4}</fingerprint>
            <active>1</active>
        </connection>
    </connections>
</config>"#,
        host, port, config.username, config.password.as_deref().unwrap_or_default(), fingerprint
    ))
}

#[tauri::command]
fn save_config(config: VpnConfig) -> Result<(), String> {
    let xml = generate_xml(&config)?;
    let tmp_path = "/tmp/kerio-gui-config.xml";
    fs::write(tmp_path, xml).map_err(|e| e.to_string())?;

    let output = Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg(format!("cp {} /etc/kerio-kvc.conf && chmod 0600 /etc/kerio-kvc.conf", tmp_path))
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to save config".into());
    }
    Ok(())
}

#[tauri::command]
fn toggle_vpn(connect: bool) -> Result<(), String> {
    let action = if connect { "start" } else { "stop" };
    let output = Command::new("pkexec")
        .arg("systemctl")
        .arg(action)
        .arg("kerio-kvc")
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(format!("Failed to {} VPN", action));
    }
    Ok(())
}

async fn switch_to_profile(app: AppHandle, profile: Profile) -> Result<(), String> {
    let xml = generate_xml(&profile.config)?;
    let tmp_path = format!("/tmp/kerio-gui-config-{}.xml", profile.id);
    fs::write(&tmp_path, xml).map_err(|e| e.to_string())?;

    let combined_cmd = format!(
        "systemctl stop kerio-kvc; cp {} /etc/kerio-kvc.conf; chmod 0600 /etc/kerio-kvc.conf; systemctl start kerio-kvc",
        tmp_path
    );

    let output = Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg(combined_cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err("Failed to switch profile".into());
    }

    // Update active profile ID and tray menu
    let state: State<AppState> = app.state();
    {
        let mut active_id = state.active_profile_id.lock().unwrap();
        *active_id = Some(profile.id);
        let profiles = state.profiles.lock().unwrap();
        let _ = update_tray_menu(&app, &profiles, &active_id);
    }

    Ok(())
}

#[tauri::command]
fn get_status(state: State<'_, AppState>) -> Result<VpnStatus, String> {
    let output = Command::new("systemctl")
        .arg("is-active")
        .arg("kerio-kvc")
        .output()
        .ok();
        
    let is_active = if let Some(out) = output {
        String::from_utf8_lossy(&out.stdout).trim() == "active"
    } else {
        false
    };
    
    // Automatically reset active_id if disconnected
    let mut current_active = state.active_profile_id.lock().unwrap();
    if !is_active && current_active.is_some() {
        *current_active = None;
    }

    Ok(VpnStatus {
        state: if is_active { "connected".into() } else { "disconnected".into() },
        active_profile_id: current_active.clone(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let profiles = load_profiles_from_disk(app.handle());
            
            let mut init_active_id = None;
            if let Some(active_id) = get_active_profile_from_system(&profiles) {
                init_active_id = Some(active_id);
            }

            app.manage(AppState {
                profiles: Mutex::new(profiles.clone()),
                active_profile_id: Mutex::new(init_active_id.clone()),
            });

            TrayIconBuilder::with_id("main")
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app_handle: &AppHandle, event| {
                    let id = event.id.as_ref();
                    if id == "quit" {
                        let _ = toggle_vpn(false);
                        std::process::exit(0);
                    } else if id == "settings" {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    } else if id == "disconnect" {
                        let _ = toggle_vpn(false);
                        let state: State<AppState> = app_handle.state();
                        let mut active_id = state.active_profile_id.lock().unwrap();
                        *active_id = None;
                        let profiles = state.profiles.lock().unwrap();
                        let _ = update_tray_menu(app_handle, &profiles, &active_id);
                    } else if id.starts_with("prof_") {
                        let prof_id = id[5..].to_string();
                        let app = app_handle.clone();
                        let state: State<AppState> = app.state();
                        let profile = {
                            let profiles = state.profiles.lock().unwrap();
                            profiles.iter().find(|p| p.id == prof_id).cloned()
                        };

                        if let Some(profile) = profile {
                            tauri::async_runtime::spawn(async move {
                                let _ = switch_to_profile(app, profile).await;
                            });
                        }
                    }
                })
                .on_tray_icon_event(|tray: &TrayIcon<Wry>, event| {
                    if let tauri::tray::TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            let _ = update_tray_menu(app.handle(), &profiles, &init_active_id);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            toggle_vpn,
            get_status,
            get_profiles,
            save_profile,
            delete_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
