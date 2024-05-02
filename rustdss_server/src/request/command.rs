// Parses commands -- Maybe move this into rustdss_data or transport?
use rustdss_data::{Command, RespData};

pub trait ParseCommand {
    fn from_resp(input: RespData) -> Result<Command, String>;
}
fn string_arg(data: Option<RespData>) -> Option<String> {
    data.and_then(|inner_data| match inner_data {
        RespData::BulkStr(string) => Some(string),
        RespData::SimpleStr(string) => Some(string),
        _ => None,
    })
}

fn numerical_arg(data: Option<RespData>) -> Option<i64> {
    data.clone() // ew gross
        .and_then(|val| match val {
            RespData::Number(i) => Some(i),
            // Try and coerce the string arg
            _ => None,
        })
        .or_else(|| {
            let number: Option<i64> = string_arg(data).and_then(|s| s.parse().ok());
            number
        })
}

impl ParseCommand for Command {
    fn from_resp(input: RespData) -> Result<Self, String> {
        if let RespData::List(data) = input {
            let mut data = data.into_iter();
            if let Some(RespData::BulkStr(cmd_string)) = data.next() {
                match cmd_string.to_lowercase().as_str() {
                    "ping" => Ok(Command::Ping),
                    "echo" => {
                        if let Some(arg0) = data.next() {
                            Ok(Command::Echo(arg0.clone()))
                        } else {
                            Err("too few args".into())
                        }
                    }
                    "get" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Get(arg0))
                        } else {
                            Err("too few args".into())
                        }
                    }
                    "set" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            if let Some(arg1) = data.next() {
                                Ok(Command::Set(arg0, arg1.clone()))
                            } else {
                                Err("Not enough args".into())
                            }
                        } else {
                            Err("nope".into())
                        }
                    }
                    "flushall" => Ok(Command::FlushAll),
                    "decr" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Decr(arg0, None))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "decrby" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Decr(arg0, numerical_arg(data.next())))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "incr" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Incr(arg0, None))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "incrby" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Incr(arg0, numerical_arg(data.next())))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "lpush" => {
                        if let (Some(arg0), Some(arg1)) = (string_arg(data.next()), data.next()) {
                            Ok(Command::Lpush(arg0, arg1))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "rpush" => {
                        if let (Some(arg0), Some(arg1)) = (string_arg(data.next()), data.next()) {
                            Ok(Command::Rpush(arg0, arg1))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "lpop" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Lpop(arg0))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "rpop" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Rpop(arg0))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "llen" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Llen(arg0))
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "lrange" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            if let (Some(arg1), Some(arg2)) =
                                (numerical_arg(data.next()), numerical_arg(data.next()))
                            {
                                Ok(Command::Lrange(arg0, arg1, arg2))
                            } else {
                                Err("Not enough args".into())
                            }
                        } else {
                            Err("Not enough args".into())
                        }
                    }
                    "dump" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Dump(arg0))
                        } else {
                            Err("Not enough args".into())
                        }
                    }

                    "keys" => Ok(Command::Keys),
                    "info" => Ok(Command::Info),
                    "select" => {
                        if let Some(arg0) = string_arg(data.next()) {
                            Ok(Command::Select(arg0))
                        } else {
                            Err("too few args".into())
                        }
                    }
                    _ => Err("unknown command".into()),
                }
            } else {
                Err("invalid command format".into())
            }
        } else {
            Err("invalid command format".into())
        }
    }
}
