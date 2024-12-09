use core::fmt;
use std::fmt::Display;

use clap_verbosity_flag::{InfoLevel, LevelFilter, Verbosity};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum LogFormat {
    Json,
    Text,
}

impl Display for LogFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(clap::Parser, Clone)]
pub struct LogConfig {
    #[command(flatten)]
    pub verbosity: Verbosity<InfoLevel>,

    #[clap(long, env, default_value_t = LogFormat::Text, help = "Logging format")]
    pub log_format: LogFormat,
}

impl LogConfig {
    pub fn init(&self) {
        let log_level = match self.verbosity.log_level_filter() {
            LevelFilter::Off => None,
            LevelFilter::Error => Some(Level::ERROR),
            LevelFilter::Warn => Some(Level::WARN),
            LevelFilter::Info => Some(Level::INFO),
            LevelFilter::Debug => Some(Level::DEBUG),
            LevelFilter::Trace => Some(Level::TRACE),
        };
        if let Some(log_level) = log_level {
            let subscriber = FmtSubscriber::builder().with_max_level(log_level);

            match self.log_format {
                LogFormat::Text => subscriber.init(),
                LogFormat::Json => subscriber.json().flatten_event(true).init(),
            };
        }
    }
}
