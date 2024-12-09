use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::error::Error;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::str::FromStr;
use std::ops::Deref;

use enum_map::{EnumMap, Enum};

#[derive(Debug, Clone, Copy, Enum, PartialEq, Eq, Hash)]
pub enum SettingCodes {
    HEADER_TABLE_SIZE = 0x1,
    ENABLE_PUSH = 0x2,
    MAX_CONCURRENT_STREAMS = 0x3,
    INITIAL_WINDOW_SIZE = 0x4,
    MAX_FRAME_SIZE = 0x5,
    MAX_HEADER_LIST_SIZE = 0x6,
    ENABLE_CONNECT_PROTOCOL = 0x7,
}

#[derive(Debug)]
pub struct InvalidSettingsValueError {
    pub setting: SettingCodes,
    pub value: u32,
    pub error_code: u32,
}

impl fmt::Display for InvalidSettingsValueError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Setting {:?} has invalid value {} (error code: {})",
            self.setting, self.value, self.error_code
        )
    }
}

impl Error for InvalidSettingsValueError {}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct ChangedSetting {
    pub setting: SettingCodes,
    pub original_value: u32,
    pub new_value: u32,
}

impl fmt::Display for ChangedSetting {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ChangedSetting(setting: {:?}, original_value: {}, new_value: {})",
            self.setting, self.original_value, self.new_value
        )
    }
}

pub struct Settings {
    settings: EnumMap<SettingCodes, VecDeque<u32>>,
}

impl Settings {
    pub fn new(client: bool, initial_values: Option<HashMap<SettingCodes, u32>>) -> Settings {
        let mut settings = EnumMap::default();

        settings[SettingCodes::HEADER_TABLE_SIZE] = VecDeque::from(vec![4096]);
        settings[SettingCodes::ENABLE_PUSH] = VecDeque::from(vec![if client { 1 } else { 0 }]);
        settings[SettingCodes::INITIAL_WINDOW_SIZE] = VecDeque::from(vec![65535]);
        settings[SettingCodes::MAX_FRAME_SIZE] = VecDeque::from(vec![16384]);
        settings[SettingCodes::ENABLE_CONNECT_PROTOCOL] = VecDeque::from(vec![0]);

        if let Some(initial) = initial_values {
            for (key, value) in initial {
                if let Err(e) = validate_setting(key, value) {
                    panic!("Invalid setting: {}", e);
                }
                settings[key] = VecDeque::from(vec![value]);
            }
        }

        Settings { settings }
    }

    pub fn acknowledge(&mut self) -> HashMap<SettingCodes, ChangedSetting> {
        let mut changed_settings = HashMap::new();
        for (key, values) in self.settings.iter_mut() {
            if values.len() > 1 {
                let old_value = values.pop_front().unwrap();
                let new_value = *values.front().unwrap();
                changed_settings.insert(
                    *key,
                    ChangedSetting {
                        setting: *key,
                        original_value: old_value,
                        new_value,
                    },
                );
            }
        }
        changed_settings
    }

    pub fn get(&self, key: SettingCodes) -> u32 {
        self.settings[&key][0]
    }

    pub fn set(&mut self, key: SettingCodes, value: u32) {
        if let Err(e) = validate_setting(key, value) {
            panic!("Invalid setting: {}", e);
        }

        self.settings.entry(key).or_default().push_back(value);
    }

    pub fn remove(&mut self, key: SettingCodes) {
        self.settings.remove(&key);
    }

    pub fn header_table_size(&self) -> u32 {
        self.get(SettingCodes::HEADER_TABLE_SIZE)
    }

    pub fn set_header_table_size(&mut self, value: u32) {
        self.set(SettingCodes::HEADER_TABLE_SIZE, value);
    }

    pub fn enable_push(&self) -> u32 {
        self.get(SettingCodes::ENABLE_PUSH)
    }

    pub fn set_enable_push(&mut self, value: u32) {
        self.set(SettingCodes::ENABLE_PUSH, value);
    }

    pub fn initial_window_size(&self) -> u32 {
        self.get(SettingCodes::INITIAL_WINDOW_SIZE)
    }

    pub fn set_initial_window_size(&mut self, value: u32) {
        self.set(SettingCodes::INITIAL_WINDOW_SIZE, value);
    }

    pub fn max_frame_size(&self) -> u32 {
        self.get(SettingCodes::MAX_FRAME_SIZE)
    }

    pub fn set_max_frame_size(&mut self, value: u32) {
        self.set(SettingCodes::MAX_FRAME_SIZE, value);
    }

    pub fn max_concurrent_streams(&self) -> u32 {
        self.get(SettingCodes::MAX_CONCURRENT_STREAMS)
    }

    pub fn set_max_concurrent_streams(&mut self, value: u32) {
        self.set(SettingCodes::MAX_CONCURRENT_STREAMS, value);
    }

    pub fn max_header_list_size(&self) -> Option<u32> {
        self.settings
            .get(&SettingCodes::MAX_HEADER_LIST_SIZE)
            .map(|v| v[0])
    }

    pub fn set_max_header_list_size(&mut self, value: u32) {
        self.set(SettingCodes::MAX_HEADER_LIST_SIZE, value);
    }

    pub fn enable_connect_protocol(&self) -> u32 {
        self.get(SettingCodes::ENABLE_CONNECT_PROTOCOL)
    }

    pub fn set_enable_connect_protocol(&mut self, value: u32) {
        self.set(SettingCodes::ENABLE_CONNECT_PROTOCOL, value);
    }
}

fn validate_setting(setting: SettingCodes, value: u32) -> Result<(), u32> {
    match setting {
        SettingCodes::ENABLE_PUSH => {
            if value != 0 && value != 1 {
                return Err(0x01); // Error code for invalid setting
            }
        }
        SettingCodes::INITIAL_WINDOW_SIZE => {
            if value > 2147483647 {
                return Err(0x02); // Flow control error code
            }
        }
        SettingCodes::MAX_FRAME_SIZE => {
            if value < 16384 || value > 16777215 {
                return Err(0x01); // Protocol error
            }
        }
        SettingCodes::MAX_HEADER_LIST_SIZE => {
            if value < 0 {
                return Err(0x01); // Protocol error
            }
        }
        SettingCodes::ENABLE_CONNECT_PROTOCOL => {
            if value != 0 && value != 1 {
                return Err(0x01); // Protocol error
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setting_changes() {
        let mut settings = Settings::new(true, None);
        let initial_value = settings.header_table_size();
        settings.set_header_table_size(8192);
        let changed = settings.acknowledge();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[&SettingCodes::HEADER_TABLE_SIZE].original_value, initial_value);
    }
}

