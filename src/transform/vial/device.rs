use std::{collections::HashMap, io::Write};

use hidapi::{HidApi, HidDevice};
use serde_json::Value;
use vitaly::protocol;
fn load_meta(dev: &HidDevice) -> Result<Value, String> {
    let meta_data = match protocol::load_vial_meta(dev) {
        Ok(meta_data) => meta_data,
        Err(e) => {
            return Err(format!("failed to load vial meta {:?}", e));
        }
    };
    Ok(meta_data)
}

pub fn get_device(
    api: &HidApi,
    device_id: Option<u16>,
) -> Option<(HidDevice, protocol::Capabilities, Value)> {
    api.device_list().find_map(|device| {
        if let Some(id) = device_id
            && device.product_id() != id
        {
            return None;
        }

        if device.usage_page() == protocol::USAGE_PAGE && device.usage() == protocol::USAGE_ID {
            let device_path = device.path();
            let dev = api.open_path(device_path).ok()?;
            let capabilities = protocol::scan_capabilities(&dev).ok()?;
            let meta = load_meta(&dev).ok()?;
            meta["matrix"]["cols"].as_u64()?;
            meta["matrix"]["rows"].as_u64()?;
            Some((dev, capabilities, meta))
        } else {
            None
        }
    })
}

pub fn unlock_device(dev: &HidDevice, meta: &Value, unlock: bool) -> Result<(), String> {
    let mut status = protocol::get_locked_status(&dev).map_err(|e| e.to_string())?;
    if status.locked && unlock {
        println!("Starting unlock process... ");
        println!("Push marked buttons and keep then pushed to unlock...");
        let layout_options = &meta["layouts"]["labels"];
        let state = protocol::load_layout_options(&dev).map_err(|e| e.to_string())?;
        let options =
            protocol::LayoutOptions::from_json(state, layout_options).map_err(|e| e.to_string())?;
        let mut buttons = vitaly::keymap::keymap_to_buttons(&meta["layouts"]["keymap"], &options)
            .map_err(|e| e.to_string())?;
        let mut button_labels = HashMap::new();
        for (row, col) in &status.unlock_buttons {
            button_labels.insert((*row, *col), "☆☆,☆☆".to_string());
        }
        for button in &mut buttons {
            button.color = if status
                .unlock_buttons
                .contains(&(button.wire_x, button.wire_y))
            {
                Some((255, 255, 255))
            } else {
                None
            };
        }
        vitaly::keymap::render_and_dump(&buttons, Some(button_labels));
        if !status.unlock_in_progress {
            protocol::start_unlock(&dev).map_err(|e| e.to_string())?;
        }
        let sleep_duration = std::time::Duration::from_millis(100);
        let mut unlocked = false;
        let mut polls_remaining: u8;
        while !unlocked {
            std::thread::sleep(sleep_duration);
            (unlocked, polls_remaining) = protocol::unlock_poll(&dev).map_err(|e| e.to_string())?;
            print!("\r");
            print!(
                "Seconds remaining: {} keep pushing...",
                (polls_remaining as f64) / 10.0
            );
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        status = protocol::get_locked_status(&dev).map_err(|e| e.to_string())?;
        println!("\nDevice is locked: {}", status.locked);
    } else if !status.locked {
        println!("Locking keyboard...");
        protocol::set_locked(&dev).map_err(|e| e.to_string())?;
        status = protocol::get_locked_status(&dev).map_err(|e| e.to_string())?;
        println!("Device is locked: {}", status.locked);
    }

    Ok(())
}
