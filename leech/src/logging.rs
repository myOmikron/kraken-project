use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log4rs::{Config, Handle};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default log pattern
///
/// **Example log**:
/// ```log
/// [2022-11-06 23:54:17 | INFO  | actix_server::builder] Starting 8 workers
/// ```
pub const LOG_PATTERN: &str = "{h([{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5} | {t}])} {m}{n}";
/// Default log pattern without target
///
/// **Example log**:
/// ```log
/// [2022-11-06 23:54:17 | INFO  ] Starting 8 workers
/// ```
pub const LOG_PATTERN_WITHOUT_TARGET: &str = "{h([{d(%Y-%m-%d %H:%M:%S)} | {({l}):5.5}])} {m}{n}";
/// Log pattern for actix-web's logging tb_middleware.
pub const LOG_PATTERN_ACTIX_NGINX_LIKE: &str = r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#;

/// Representation of a file logger
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AdditionalFileLogger {
    /// Name of the file logger
    ///
    /// This can be used as log target:
    ///
    /// ```
    /// use log::info;
    ///
    /// info!(target: "foo", "This is sent to {}", "foo");
    /// ```
    pub name: String,
    /// Path to the log file.
    pub path: String,
    /// If set to true, the log entries from the additional logger get also
    /// added to the main file and stdout logger.
    ///
    /// If None, this option is turned off.
    pub add_to_main_logger: Option<bool>,
    /// Log rotation trigger size
    ///
    /// [byte_unit::Byte] has support for serde deserialization.
    /// So "8 MB" for example, can be parsed
    pub rotation_file_size: byte_unit::Byte,
    /// Maximum number of files to rotate until the log gets deleted
    pub max_rotation_count: u32,
    /// Optional Loglevel for the specific logger
    ///
    /// If None is set, the main log level will be used.
    pub log_level: Option<LevelFilter>,
    /// Optional alternative pattern.
    ///
    /// If None, [LOG_PATTERN] will be used.
    ///
    /// See [log4rs::encode::pattern] for more information
    pub alternative_pattern: Option<String>,
}

/// The Logging configuration
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct LoggingConfig {
    /// Main log level to use
    pub log_level: LevelFilter,
    /// Path to the log file.
    pub path: String,
    /// Log rotation trigger size
    ///
    /// [byte_unit::Byte] has support for serde deserialization.
    /// E.g. the string "8 MB" can be parsed
    pub rotation_file_size: byte_unit::Byte,
    /// Maximum number of files to rotate until the log gets deleted
    pub max_rotation_count: u32,
    /// Set an alternative pattern for the stdout logger.
    ///
    /// Defaults to [LOG_PATTERN]
    pub alternative_pattern: Option<String>,
    /// Additional list of file loggers
    pub additional_file_loggers: Vec<AdditionalFileLogger>,
}

/// Sets up logging with the given parameter.
///
/// **Parameter**:
/// - `config`: [LoggingConfig]: Reference to the configuration to use for setup.
///
/// **Returns** a handle for changing the logger at runtime
pub fn setup_logging(config: &LoggingConfig) -> Result<Handle, String> {
    let stdout_uuid = Uuid::new_v4().to_string();
    let main_pattern: &str = config
        .alternative_pattern
        .as_ref()
        .map_or(LOG_PATTERN, |x| x.as_str());
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(main_pattern)))
        .build();

    let file_logger_uuid = Uuid::new_v4().to_string();
    let roller_pattern = format!("{}.{{}}.gz", &config.path);
    let roller = Box::new(
        FixedWindowRoller::builder()
            .base(1)
            .build(&roller_pattern, config.max_rotation_count)
            .map_err(|e| e.to_string())?,
    );
    let file_logger = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(main_pattern)))
        .build(
            &config.path,
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(
                    config.rotation_file_size.get_bytes() as u64
                )),
                roller,
            )),
        )
        .map_err(|e| e.to_string())?;

    let mut b = Config::builder()
        .appender(Appender::builder().build(&stdout_uuid, Box::new(stdout)))
        .appender(Appender::builder().build(&file_logger_uuid, Box::new(file_logger)));

    for x in &config.additional_file_loggers {
        let pattern = x.alternative_pattern.as_ref().map_or(LOG_PATTERN, |x| x);

        let roller_pattern = format!("{}.{{}}.gz", &x.path);
        let roller = Box::new(
            FixedWindowRoller::builder()
                .base(1)
                .build(&roller_pattern, x.max_rotation_count)
                .map_err(|e| e.to_string())?,
        );

        let ap = Box::new(
            RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new(pattern)))
                .build(
                    &x.path,
                    Box::new(CompoundPolicy::new(
                        Box::new(SizeTrigger::new(x.rotation_file_size.get_bytes() as u64)),
                        roller,
                    )),
                )
                .map_err(|e| e.to_string())?,
        );

        b = b.appender(Appender::builder().build(&x.name, ap)).logger(
            Logger::builder()
                .appender(&x.name)
                .additive(x.add_to_main_logger.unwrap_or(false))
                .build(&x.name, x.log_level.unwrap_or(config.log_level)),
        )
    }

    let logging_config = b
        .build(
            Root::builder()
                .appenders([&stdout_uuid, &file_logger_uuid])
                .build(config.log_level),
        )
        .map_err(|e| e.to_string())?;
    log4rs::init_config(logging_config).map_err(|e| e.to_string())
}
