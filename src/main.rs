use chrono::NaiveDateTime;
use clap::Parser;
use regex::{self, Regex};
use std::{collections::HashMap, fs, usize};

/// A programe for parsing visibroker default log format
#[derive(Parser)]
#[command(version, about)]
struct CLI {
    /// Process ID you want to search for
    #[arg(long, short)]
    pid: Option<usize>,
    /// Thread ID you want to search for
    #[arg(long, short)]
    tid: Option<usize>,
    /// The name of the logger you want to search for
    #[arg(long)]
    logger: Option<String>,
    /// The time which you want to see all log messages that apear before
    #[arg(long, short)]
    before: Option<String>,
    /// The time which you want to see all log messages that apear before
    #[arg(long, short)]
    after: Option<String>,
    /// The file which you want to see log messages from optionaly you can add a ':' and a line number which you want to see
    #[arg(long, short)]
    source: Option<String>,
    /// The name of the component the message orignated from
    #[arg(long, short)]
    component: Option<String>,
    /// The level of the messages you want to filter for
    #[arg(long, short)]
    level: Option<String>,
    /// Regex that you want to grep the message for
    #[arg(long, short)]
    message: Option<String>,
    ///format of the output of the programe
    #[arg(long,short,default_value_t=String::from("{level}: {message}"))]
    fmt: String,

    ///the format of the date and time string you want to use
    #[arg(long,default_value_t=String::from("%a %b %e %H:%M:%S %Y %fus"))]
    date_fmt: String,
    /// The file you want to read
    files: Vec<String>,
}

#[derive(Debug, PartialEq, Eq)]
enum LogLevel {
    EMERG,
    ALERT,
    CRIT,
    ERROR,
    WARNING,
    INFO,
    DEBUG,
}

impl LogLevel {
    pub fn from(txt: String) -> Option<Self> {
        match txt.to_lowercase().as_str() {
            "emergency" | "ermg" | "emg" => Some(LogLevel::EMERG),
            "alert" | "alt" => Some(LogLevel::ALERT),
            "ciritcal" | "crit" | "crt" => Some(LogLevel::CRIT),
            "error" | "err" => Some(LogLevel::ERROR),
            "warning" | "warn" | "wrn" => Some(LogLevel::WARNING),
            "info" | "inf" => Some(LogLevel::INFO),
            "debug" | "dbg" => Some(LogLevel::DEBUG),
            _ => None,
        }
    }

    pub fn to_string(self) -> String {
        match self {
            LogLevel::EMERG => "EMERG".to_string(),
            LogLevel::ALERT => "ALERT".to_string(),
            LogLevel::CRIT => "CRIT".to_string(),
            LogLevel::ERROR => "ERROR".to_string(),
            LogLevel::WARNING => "WARN".to_string(),
            LogLevel::INFO => "INFO".to_string(),
            LogLevel::DEBUG => "DEBUG".to_string(),
        }
    }
}

#[derive(Debug)]
struct Log {
    pid: usize,
    time: NaiveDateTime,
    tid: usize,
    logger: String,
    component: String,
    file: String,
    line: usize,
    level: LogLevel,
    message: String,
}

impl Log {
    pub fn from(text: String) -> Log {
        let marker_size = 4;

        let tid_start = text.find("Tid#").expect(
            format!(
                "Bad formatted Visibroker Log message missing Thread ID {}",
                text
            )
            .as_str(),
        );
        let time_start = text
            .find("Tim#")
            .expect("Badly formatted Visibroker Log message missing Time of log");
        let logger_start = text
            .find("Log#")
            .expect("Bad formatted Visibroker Log message missing logger");
        let component_start = text
            .find("Src#")
            .expect("Badly formatted Visibroker Log message Component missing");
        let file_start = text
            .find("Fil#")
            .expect("Badly formatted Visibroker log message file name is missing");
        let line_start = text
            .find("Lin#")
            .expect("Badly formatted Visibroker log message missing line");
        let level_start = text
            .find("Lvl#")
            .expect("Badly formatted Visibrokr log message missing log level");
        let message_start = text
            .find("Msg#")
            .expect("Badly formatted Visibroker Log message missing message");

        let pid_str: String = text
            .chars()
            .skip(marker_size)
            .take(time_start - marker_size)
            .collect();
        let time_str: String = text
            .chars()
            .skip(time_start + marker_size)
            .take(tid_start - (time_start + marker_size))
            .collect();
        let tid_str: String = text
            .chars()
            .skip(tid_start + marker_size)
            .take(logger_start - (tid_start + marker_size))
            .collect();
        let logger_str: String = text
            .chars()
            .skip(logger_start + marker_size)
            .take(component_start - (logger_start + marker_size))
            .collect();
        let component_str: String = text
            .chars()
            .skip(component_start + marker_size)
            .take(file_start - (component_start + marker_size))
            .collect();
        let file_str: String = text
            .chars()
            .skip(file_start + marker_size)
            .take(line_start - (file_start + marker_size))
            .collect();
        let line_str: String = text
            .chars()
            .skip(line_start + marker_size)
            .take(level_start - (line_start + marker_size))
            .collect();
        let level_str: String = text
            .chars()
            .skip(level_start + marker_size)
            .take(message_start - (level_start + marker_size))
            .collect();
        let message_str: String = text
            .chars()
            .skip(message_start + marker_size)
            .take(text.len() - (message_start + marker_size))
            .collect();

        return Log {
            pid: pid_str.trim().parse().expect("Unable to parse pid"),
            time: NaiveDateTime::parse_from_str(time_str.trim(), "%a %b %e %H:%M:%S %Y %fus")
                .expect("Failed to parse date"),
            tid: tid_str.trim().parse().expect("unable to parse tid"),
            logger: logger_str.trim().to_string(),
            component: component_str.trim().to_string(),
            file: file_str.trim().to_string(),
            line: line_str
                .trim()
                .parse()
                .expect("Unabled to parse line number"),
            level: LogLevel::from(level_str.trim().to_string()).expect("unable to parse log level"),
            message: message_str.trim().to_string(),
        };
    }

    pub fn from_log(text: &String, start: usize, end: usize) -> Log {
        let log_str: String = text.chars().skip(start).take(end - start).collect();

        Log::from(log_str)
    }
}

fn tid_validator(log: &Log, args: &CLI) -> bool {
    log.tid == args.tid.unwrap()
}

fn logger_validator(log: &Log, args: &CLI) -> bool {
    log.logger == args.logger.clone().unwrap()
}

fn component_validator(log: &Log, args: &CLI) -> bool {
    log.component == args.component.clone().unwrap()
}

fn level_validator(log: &Log, args: &CLI) -> bool {
    log.level == LogLevel::from(args.level.clone().unwrap()).expect("Unable to parse log level")
}

fn message_validator(log: &Log, args: &CLI) -> bool {
    let re = Regex::new(&args.message.clone().unwrap()).unwrap();
    re.is_match(&log.message)
}

fn before_validator(log: &Log, args: &CLI) -> bool {
    let date = NaiveDateTime::parse_from_str(args.before.clone().unwrap().as_str(), &args.date_fmt)
        .unwrap();
    log.time.and_utc().timestamp() <= date.and_utc().timestamp()
}

fn after_validator(log: &Log, args: &CLI) -> bool {
    let date = NaiveDateTime::parse_from_str(args.after.clone().unwrap().as_str(), &args.date_fmt)
        .unwrap();
    log.time.and_utc().timestamp() >= date.and_utc().timestamp()
}

fn file_validator(log: &Log, args: &CLI) -> bool {
    log.file == args.source.clone().unwrap()
}

fn print_log(log: Log, format: &String) {
    let mut vars = HashMap::new();
    vars.insert("pid".to_string(), log.pid.to_string());
    vars.insert("time".to_string(), log.time.to_string());
    vars.insert("tid".to_string(), log.tid.to_string());
    vars.insert("logger".to_string(), log.logger.to_string());
    vars.insert("component".to_string(), log.component);
    vars.insert("file".to_string(), log.file);
    vars.insert("line".to_string(), log.line.to_string());
    vars.insert("level".to_string(), log.level.to_string());
    vars.insert("message".to_string(), log.message);

    println!(
        "{}",
        strfmt::strfmt(format.as_str(), &vars).expect("Failed to format output")
    )
}

fn filtered_print(log: Log, args: &CLI, filters: &Vec<&dyn Fn(&Log, &CLI) -> bool>) {
    let mut filterd = false;

    for filter_index in 0..filters.len() {
        if !(filters[filter_index])(&log, &args) {
            filterd = true;
            break;
        }
    }

    if !filterd {
        print_log(log, &args.fmt);
    }
}

fn main() {
    let args = CLI::parse();

    let mut filters: Vec<&dyn Fn(&Log, &CLI) -> bool> = Vec::new();

    if let Some(_) = args.tid {
        filters.push(&tid_validator);
    }

    if let Some(_) = args.logger.clone() {
        filters.push(&logger_validator);
    }

    if let Some(_) = args.component.clone() {
        filters.push(&component_validator);
    }

    if let Some(_) = args.level.clone() {
        filters.push(&level_validator);
    }

    if let Some(_) = args.message.clone() {
        filters.push(&message_validator);
    }

    if let Some(_) = args.before.clone() {
        filters.push(&before_validator);
    }

    if let Some(_) = args.after.clone() {
        filters.push(&after_validator);
    }

    if let Some(_) = args.source.clone() {
        filters.push(&file_validator);
    }

    let files = args.files.clone();

    for file in files {
        let contents: String = fs::read_to_string(file).expect("No such file or Directory");
        let pids: Vec<_> = contents.match_indices("Pid#").collect();

        for index in 0..pids.len() - 1 {
            let log = Log::from_log(&contents, pids[index].0, pids[index + 1].0);
            filtered_print(log, &args, &filters);
        }

        let log = Log::from_log(&contents, pids[pids.len() - 1].0, contents.len());

        filtered_print(log, &args, &filters);
    }
}
