use chrono::NaiveDateTime;
use clap::Parser;
use regex::{self, Regex};
use std::{collections::HashMap,str, fs, io::{stdin, Read}, process::exit, usize};

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
            "emergency" | "emerg" | "emg" => Some(LogLevel::EMERG),
            "alert" | "alt" => Some(LogLevel::ALERT),
            "critical" | "crit" | "crt" => Some(LogLevel::CRIT),
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

#[derive(Debug, PartialEq, Eq)]
struct LogError {
    cause: String,
}

#[derive(Debug, PartialEq, Eq)]
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
    pub fn from(text: String) -> Result<Log, LogError> {
        let marker_size = 4;

        let tid_start;
        match text.find("Tid#") {
            Some(txt) => tid_start = txt,
            None => {
                return Err(LogError {
                    cause: String::from(format!(
                        "Badly formatted Visibroker Log message missing Thread ID in following log: {}",
                        text
                    )),
                });
            }
        }

        let time_start;
        match text.find("Tim#") {
                Some(t) => time_start = t,
                None => return Err(LogError{
                    cause: String::from(format!("Badly formatted Visibroker Log message missing Time of log in the following log: {}",text))
                }),
        }

        let logger_start;
        match text.find("Log#") {
            Some(l) => logger_start = l,
            None => return Err(LogError{
                cause: String::from(format!("Badly formatted Visibroker Log message missing logger in the following log: {}",text))
            }),
        }

        let component_start;
        match text.find("Src#") {
            Some(i) => component_start = i,
            None => return Err(LogError{
                cause: String::from(format!("Badly formatted Visibroker Log message missing Component in the following log: {}",text))
            }),
        }

        let file_start;
        match text.find("Fil#") {
            Some(i) => file_start = i,
            None => return Err(LogError {
                cause: String::from(format!("Badly formatted Visibroker Log message missing file name in the following log: {}",text)) })
        }

        let line_start;
        match text.find("Lin#") {
            Some(i) => line_start = i,
            None => return Err(LogError{
                cause: String::from(format!("Badly formatted Visibroker Log message missing line number in the following log: {}",text))
            })
        }

        let level_start;
        match text.find("Lvl#") {
            Some(i) => level_start = i,
            None => return Err(LogError{
                cause: String::from(format!("Badly formatted Visibroker Log message missing log level in the following log: {}",text))
            }),
        }

        let message_start;
        match text.find("Msg#") {
            Some(i) => message_start = i,
            None => return Err(LogError{
                cause: String::from(format!("Badly formatted Visibroker Log message missing message text in the following log: {}",text))
            }),
        }

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

        let pid;
        match pid_str.trim().parse() {
            Ok(p) => pid = p,
            Err(_) => {
                return Err(LogError {
                    cause: String::from(format!(
                        "Unable to parse the following pid: {} in a log message",
                        pid_str
                    )),
                })
            }
        }

        let log_time;
        match NaiveDateTime::parse_from_str(time_str.trim(), "%a %b %e %H:%M:%S %Y %fus") {
            Ok(t) => log_time = t,
            Err(_) => {
                return Err(LogError {
                    cause: String::from(format!(
                    "Unable to parse the following time: {} are you sure this is a valid number",
                    pid_str
                )),
                })
            }
        }

        let tid: usize;
        match tid_str.trim().parse() {
            Ok(t) => tid = t,
            Err(_) => {
                return Err(LogError {
                    cause: String::from(format!(
                        "Unable to parse the following tid: {} in a log message",
                        tid_str
                    )),
                })
            }
        }

        let line_no;
        match line_str.trim().parse() {
            Ok(l) => line_no = l,
            Err(_) => {
                return Err(LogError {
                    cause: String::from(format!(
                        "Unable to parse the following line number: {} in a log message",
                        line_str
                    )),
                })
            }
        }

        let log_level;
        match LogLevel::from(level_str.trim().to_string()) {
            Some(level) => log_level = level,
            None => {
                return Err(LogError {
                    cause: String::from(format!(
                        "Unable to parse the following log level: {} in a log message",
                        level_str
                    )),
                })
            }
        }

        return Ok(Log {
            pid: pid,
            time: log_time,
            tid: tid,
            logger: logger_str.trim().to_string(),
            component: component_str.trim().to_string(),
            file: file_str.trim().to_string(),
            line: line_no,
            level: log_level,
            message: message_str.trim().to_string(),
        });
    }

    pub fn from_log(text: &String, start: usize, end: usize) -> Result<Log, LogError> {
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

fn print_log(log: Log, format: &String, date_fromat: &String) {
    let mut vars = HashMap::new();
    vars.insert("pid".to_string(), log.pid.to_string());
    vars.insert("time".to_string(), log.time.format(&date_fromat).to_string());
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
        print_log(log, &args.fmt,&args.date_fmt);
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
    if files.len() != 0 {
        for file in files {
            let contents: String = fs::read_to_string(file).expect("No such file or Directory");
            let pids: Vec<_> = contents.match_indices("Pid#").collect();

            for index in 0..pids.len() - 1 {
                let log;
                match Log::from_log(&contents, pids[index].0, pids[index + 1].0) {
                    Ok(data) => log = data,
                    Err(err) => {
                        eprintln!("ERROR: {}", err.cause);
                        exit(1);
                    }
                }
                filtered_print(log, &args, &filters);
            }

            let log;

            match Log::from_log(&contents, pids[pids.len() - 1].0, contents.len()) {
                Ok(data) => log = data,
                Err(err) => {
                    eprintln!("ERROR: {}", err.cause);
                    exit(1);
                }
            }

            filtered_print(log, &args, &filters);
        }
    } else {
        const BUFFER_SIZE: usize = 1024;
        let mut buffer: [u8;BUFFER_SIZE] = [0;BUFFER_SIZE];
        let mut text: String = String::new();
        loop {
            let size;
            match stdin().read(&mut buffer) {
                Ok(buf_size) => size = buf_size ,
                Err(_) => {
                    eprintln!("ERROR: Failed to read from std::in");
                    exit(1);
                },
            }

            if size == 0 {
                break;
            }

            text.push_str( match std::str::from_utf8(& buffer) {
                Ok(txt) => txt,
                Err(_) => {
                    eprint!("ERROR: data from std::in is not utf-8 formatted");
                    exit(1)
                },
            });

            let pids: Vec<_> = text.match_indices("Pid#").collect();

            for index in 0..pids.len() - 1 {
                let log;
                match Log::from_log(&text, pids[index].0, pids[index + 1].0) {
                    Ok(data) => log = data,
                    Err(err) => {
                        eprintln!("ERROR: {}", err.cause);
                        exit(1);
                    }
                }
                filtered_print(log, &args, &filters);
            }

            text = text.chars().skip(pids[pids.len()-1 ].0).take(text.len() - pids[pids.len() - 1].0).collect();
        }
        let log;
        match Log::from_log(&text, 0, text.len()) {
            Ok(data) => log = data,
            Err(err) => {
                eprintln!("ERROR: {}", err.cause);
                exit(1);
            }
        }
        filtered_print(log, &args, &filters);

    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn log_level_from() {
        let cases = vec![
            (String::from("error"), Some(LogLevel::ERROR)),
            (String::from("ERROR"), Some(LogLevel::ERROR)),
            (String::from("err"), Some(LogLevel::ERROR)),
            (String::from("ERR"), Some(LogLevel::ERROR)),
            (String::from("ERM"), None),
            (String::from("EMERGENCY"), Some(LogLevel::EMERG)),
            (String::from("emergency"), Some(LogLevel::EMERG)),
            (String::from("EMERG"), Some(LogLevel::EMERG)),
            (String::from("emerg"), Some(LogLevel::EMERG)),
            (String::from("emg"), Some(LogLevel::EMERG)),
            (String::from("EMG"), Some(LogLevel::EMERG)),
            (String::from("ALERT"), Some(LogLevel::ALERT)),
            (String::from("alert"), Some(LogLevel::ALERT)),
            (String::from("ALT"), Some(LogLevel::ALERT)),
            (String::from("alt"), Some(LogLevel::ALERT)),
            (String::from("CRITICAL"), Some(LogLevel::CRIT)),
            (String::from("critical"), Some(LogLevel::CRIT)),
            (String::from("CRIT"), Some(LogLevel::CRIT)),
            (String::from("crit"), Some(LogLevel::CRIT)),
            (String::from("CRT"), Some(LogLevel::CRIT)),
            (String::from("crt"), Some(LogLevel::CRIT)),
            (String::from("WARNING"), Some(LogLevel::WARNING)),
            (String::from("warning"), Some(LogLevel::WARNING)),
            (String::from("WARN"), Some(LogLevel::WARNING)),
            (String::from("warn"), Some(LogLevel::WARNING)),
            (String::from("WRN"), Some(LogLevel::WARNING)),
            (String::from("wrn"), Some(LogLevel::WARNING)),
            (String::from("INFO"), Some(LogLevel::INFO)),
            (String::from("info"), Some(LogLevel::INFO)),
            (String::from("INF"), Some(LogLevel::INFO)),
            (String::from("inf"), Some(LogLevel::INFO)),
            (String::from("DEBUG"), Some(LogLevel::DEBUG)),
            (String::from("debug"), Some(LogLevel::DEBUG)),
            (String::from("DBG"), Some(LogLevel::DEBUG)),
            (String::from("dbg"), Some(LogLevel::DEBUG)),
            (String::from(""), None),
        ];

        for (input, output) in cases {
            assert_eq!(
                LogLevel::from(input.clone()),
                output,
                "Checking {} becomes {:?}",
                input,
                output
            );
        }
    }

    #[test]
    fn log_from() {
        let cases = vec![
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# server Fil# vorb.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })), 
            (String::from("Pid# 999 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# server Fil# vorb.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 999,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 000000us Tid# 1 Log# default Src# server Fil# vorb.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 000000us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })), 
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 999 Log# default Src# server Fil# vorb.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 999,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# user Src# server Fil# vorb.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("user"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# csiv2 Fil# vorb.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("csiv2"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# server Fil# vdelegate.C Lin# 1 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vdelegate.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# server Fil# vorb.C Lin# 999 Lvl# INFO Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 999,
                level: LogLevel::INFO,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# server Fil# vorb.C Lin# 1 Lvl# ERROR Msg# test"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::ERROR,
                message: String::from("test") })),
            (String::from("Pid# 1 Tim# Tue Jul  9 09:09:27 2024 612542us Tid# 1 Log# default Src# server Fil# vorb.C Lin# 1 Lvl# INFO Msg# example"), 
            Ok(Log{
                pid: 1,
                time: NaiveDateTime::parse_from_str("Tue Jul  9 09:09:27 2024 612542us","%a %b %e %H:%M:%S %Y %fus").unwrap(),
                tid: 1,
                logger: String::from("default"), 
                component: String::from("server"),
                file: String::from("vorb.C"),
                line: 1,
                level: LogLevel::INFO,
                message: String::from("example") })),
        ];

        for (input, output) in cases {
            assert_eq!(
                Log::from(input.clone()),
                output,
                "Checking {} becomes {:?}",
                input,
                output
            );
        }
    }
}
