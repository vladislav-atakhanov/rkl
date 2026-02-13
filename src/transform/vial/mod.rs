mod keycode;
use keycode::Keycode;
mod actions;
mod device;

use super::graph::{Node, priority_topo_sort};
use crate::layout::{Action, Layer, Layout};
use actions::{Macro, MacroAction, TapDance, VialAction};
use device::{get_device, unlock_device};
use hidapi::{HidApi, HidDevice};
use keys::keys::{Key, KeyIndex};
use parser::VialItem;
use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    io::Write,
    str::FromStr,
};
use vitaly::protocol;
impl Layout {
    fn sorted_layers(&self) -> Result<Vec<&Layer>, String> {
        let mut order = priority_topo_sort(
            &self
                .layers
                .iter()
                .map(|(_, l)| {
                    (
                        l.name.as_str(),
                        Node {
                            deps: l.get_dependencies(),
                            weight: l.index,
                        },
                    )
                })
                .collect(),
        )?;
        order.reverse();
        println!("{:?}", order);
        Ok(order
            .into_iter()
            .map(|n| self.layers.get(n).unwrap())
            .collect())
    }
    pub fn vial(&self, device_id: Option<u16>) -> Result<(), String> {
        let sorted = self.sorted_layers()?;
        let vial = self
            .keyboard
            .vial
            .ok_or("Vial is not defined".to_string())?;
        let api = HidApi::new().map_err(|e| e.to_string())?;

        let Some((device, capabilities, meta)) = get_device(&api, device_id) else {
            return Err("Device not found".to_string());
        };
        if capabilities.vial_version > 0 {
            unlock_device(&device, &meta, false)?;
            unlock_device(&device, &meta, true)?;
        }

        let version = capabilities.vial_version;
        let layers_by_name: HashMap<&str, usize> = sorted
            .iter()
            .enumerate()
            .map(|(i, l)| (l.name.as_str(), i))
            .collect();

        let mut layers: Vec<_> = sorted
            .iter()
            .map(|layer| {
                let keys: HashMap<&KeyIndex, VialAction> = layer
                    .keys
                    .iter()
                    .map(|(key_index, action)| {
                        let action = VialAction::from_action(
                            action,
                            &|name| layers_by_name.get(name).map(|x| *x),
                            version,
                        )?;
                        Ok::<_, String>((key_index, action))
                    })
                    .collect::<Result<_, _>>()?;
                let layer_index = layers_by_name
                    .get(layer.name.as_str())
                    .ok_or(format!("Layer {:?} not found", layer.name))?;
                Ok::<_, String>((*layer_index, keys))
            })
            .collect::<Result<_, _>>()?;

        layers.sort_by_key(|(n, _)| *n);
        let layers: Vec<_> = layers.into_iter().map(|(_, v)| v).collect();

        let mut macros: HashMap<Macro, u8> = Default::default();
        let mut tap_dances: HashMap<TapDance, u8> = Default::default();

        layers
            .iter()
            .enumerate()
            .try_for_each(|(layer_index, keys)| {
                let mut keys: Vec<_> = keys.iter().map(|(k, v)| (*k, v.clone())).collect();
                keys.sort_by_key(|(k, _)| *k);
                for (k, a) in keys.iter_mut() {
                    match a {
                        VialAction::Keycode(Keycode(a)) => {
                            match vial.get(k).ok_or(format!("Vial for {:?} not defined", k))? {
                                VialItem::KeyCode(row, col) => {
                                    protocol::set_keycode(
                                        &device,
                                        layer_index as u8,
                                        *row,
                                        *col,
                                        *a,
                                    )
                                    .map_err(|e| e.to_string())?;
                                }
                                VialItem::Encoder(index, direction) => protocol::set_encoder(
                                    &device,
                                    layer_index as u8,
                                    *index,
                                    *direction,
                                    *a,
                                )
                                .map_err(|e| e.to_string())?,
                            };
                        }
                        VialAction::TapDance(td) => {
                            let id = if let Some(i) = tap_dances.get(td) {
                                *i
                            } else {
                                let id = tap_dances.len() as u8;
                                tap_dances.insert(td.clone(), id);
                                id
                            };
                            *a = Keycode::from_name(format!("TD({})", id), version)
                                .map(VialAction::Keycode)?;
                        }
                        VialAction::Macro(macro_actions) => {
                            let id = if let Some(id) = macros.get(macro_actions) {
                                *id
                            } else {
                                let id = macros.len() as u8;
                                macros.insert(macro_actions.clone(), id);
                                id
                            };
                            *a = Keycode::from_name(format!("M{}", id), version)
                                .map(VialAction::Keycode)?;
                        }
                    }
                }
                println!("Layer {}", sorted.get(layer_index).unwrap().name);
                Ok::<_, String>(())
            })?;

        let mut macros: Vec<_> = macros.iter().collect();
        macros.sort_by_key(|(_, i)| *i);
        let macros: Vec<_> = macros.into_iter().map(|(x, _)| x).collect();

        protocol::set_macros(
            &device,
            &capabilities,
            &macros
                .iter()
                .enumerate()
                .map(|(i, m)| protocol::Macro {
                    index: i as u8,
                    steps: m
                        .0
                        .iter()
                        .map(|s| match s {
                            MacroAction::Down(Keycode(action)) => {
                                protocol::MacroStep::Down(*action)
                            }
                            MacroAction::Up(Keycode(action)) => protocol::MacroStep::Up(*action),
                            MacroAction::Tap(Keycode(action)) => protocol::MacroStep::Tap(*action),
                            MacroAction::Delay(d) => protocol::MacroStep::Delay(*d),
                        })
                        .collect(),
                })
                .collect(),
        )
        .map_err(|e| e.to_string())?;
        println!("Macros");

        tap_dances.iter().try_for_each(|(td, i)| {
            protocol::set_tap_dance(
                &device,
                &protocol::TapDance {
                    index: *i,
                    tap: td.tap.0,
                    hold: td.hold.0,
                    double_tap: td.double_tap.0,
                    tap_hold: td.tap_hold.0,
                    tapping_term: td.tapping_term,
                },
            )
            .map_err(|e| e.to_string())
        })?;
        println!("Tap dance");
        if capabilities.vial_version > 0 {
            unlock_device(&device, &meta, false)?;
        }
        Ok(())
    }
}
