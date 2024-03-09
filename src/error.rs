use rustic_jsonrpc::Error;

pub fn invalid_token() -> conerror::Error {
    error(-2, "登录已过期")
}

pub fn msg(msg: impl ToString) -> conerror::Error {
    error(-1, msg)
}

pub fn error(code: i32, msg: impl ToString) -> conerror::Error {
    conerror::Error::plain(Error {
        code,
        message: msg.to_string(),
        data: None,
    })
}
