let id = 1

export function toast(msg) {
    emit('error', msg)
}

export async function rpc(method, params = {}) {
    if (method != "user.create" && method != 'user.login') {
        params.token = getToken()
    }

    let response = await fetch("/rpc", {
        method: "POST",
        body: JSON.stringify({
            jsonrpc: "2.0",
            method: method,
            params: params,
            id: id++,
        })
    })
    let result = await response.json()
    if (result.error) {
        if (result.error.code == -2) {
            saveToken(null)
            emit('invalid_token')
        }
        emit('error', result.error.message)
        throw result.error
    }
    return result.result
}

let token = null

export function saveToken(token_) {
    token = token_
}

export function getToken() {
    return token
}


let handler = {}

export function on(event, func) {
    handler[event] = func
}

export function emit(event, ...args) {
    let func = handler[event]
    func && (func(...args))
}