use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use serenity::all::ChannelId;

use crate::config::{ChannelConfig, Config, MediaBackupConfig};

/// Thread-safe wrapper around Config for clean state management.
#[derive(Clone)]
pub struct ConfigStore {
    inner: Arc<Mutex<Config>>,
}

impl ConfigStore {
    pub fn new(config: Config) -> Self {
        Self {
            inner: Arc::new(Mutex::new(config)),
        }
    }

    /// Returns the schedule interval in seconds.
    pub fn schedule_interval_seconds(&self) -> NonZeroU32 {
        self.inner.lock().unwrap().schedule_interval_seconds
    }

    /// Returns a list of all enabled channels with their resolved retention policies.
    pub fn enabled_channels(&self) -> Vec<(ChannelId, NonZeroU32)> {
        self.inner.lock().unwrap().enabled_channels()
    }

    /// Returns the media backup configuration.
    pub fn media_backup_config(&self) -> MediaBackupConfig {
        self.inner.lock().unwrap().media_backup.clone()
    }

    /// Adds or updates a channel configuration.
    /// Returns the resolved policy days for the channel.
    pub fn add_channel(&self, channel_id: ChannelId, config: ChannelConfig) -> Result<NonZeroU32> {
        self.inner
            .lock()
            .unwrap()
            .add_channel_config(channel_id, config)
    }

    /// Removes a channel from the configuration.
    pub fn remove_channel(&self, channel_id: ChannelId) -> Result<()> {
        self.inner.lock().unwrap().remove_channel(channel_id)
    }

    /// Gets the pagination cursor for a channel.
    pub fn get_pagination_cursor(&self, channel_id: ChannelId) -> Option<u64> {
        self.inner.lock().unwrap().get_pagination_cursor(channel_id)
    }

    /// Sets the pagination cursor for a channel.
    pub fn set_pagination_cursor(&self, channel_id: ChannelId, cursor: Option<u64>) -> Result<()> {
        self.inner
            .lock()
            .unwrap()
            .set_pagination_cursor(channel_id, cursor)
    }
}
