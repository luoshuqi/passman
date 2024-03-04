use rustic_jsonrpc::Error;

pub fn invalid_token() -> conerror::Error {
    conerror::Error::plain(Error {
        code: -2,
        message: "登录已过期".to_string(),
        data: None,
    })
}

pub fn msg(msg: impl ToString) -> conerror::Error {
    conerror::Error::plain(Error {
        code: -1,
        message: msg.to_string(),
        data: None,
    })
}
