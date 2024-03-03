use std::borrow::Cow;

use conerror::conerror;
use rustic_jsonrpc::{method, methods, Error, Method};

use crate::password::{
    Password, PasswordCreate, PasswordListItem, PasswordManager, PasswordUpdate,
};
use crate::user::{User, UserManager};
use crate::Opt;

const ERR_GENERAL: i32 = -1;
const ERR_INVALID_TOKEN: i32 = -2;

fn error(error: conerror::Error) -> conerror::Error {
    if error.location().is_some() {
        return error;
    }
    conerror::Error::plain(Error::new(ERR_GENERAL, error.to_string(), None))
}

#[conerror]
async fn find_user(user_manager: &UserManager, token: &str) -> conerror::Result<impl User> {
    match user_manager.find_user(token).await? {
        Some(v) => Ok(v),
        None => Err(conerror::Error::plain(Error::new(
            ERR_INVALID_TOKEN,
            "token 无效",
            None,
        ))),
    }
}

#[conerror]
#[method(name = "user.login", inject(user_manager))]
async fn login<'a>(
    user_manager: &UserManager,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
) -> conerror::Result<String> {
    let user = user_manager
        .login(&username, &password)
        .await
        .map_err(error)?;
    let token = user_manager.create_token(&user).await.map_err(error)?;
    Ok(token)
}

#[conerror]
#[method(name = "user.create", inject(user_manager, opt))]
async fn create_user<'a>(
    user_manager: &UserManager,
    opt: &Opt,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
) -> conerror::Result<()> {
    if !opt.allow_create_user {
        return Err(conerror::Error::plain("not available")).map_err(error);
    }
    user_manager
        .create_user(&username, &password)
        .await
        .map_err(error)?;
    Ok(())
}

#[conerror]
#[method(name = "password.list", inject(user_manager, password_manager))]
async fn list_password(
    user_manager: &UserManager,
    password_manager: &PasswordManager,
    token: &str,
) -> conerror::Result<Vec<PasswordListItem>> {
    let user = find_user(user_manager, token).await?;
    let list = password_manager.list_password(&user).await?;
    Ok(list)
}

#[conerror]
#[method(name = "password.view", inject(user_manager, password_manager))]
async fn view_password(
    user_manager: &UserManager,
    password_manager: &PasswordManager,
    token: &str,
    id: i64,
) -> conerror::Result<Option<Password>> {
    let user = find_user(user_manager, token).await?;
    Ok(password_manager.view_password(&user, id).await?)
}

#[conerror]
#[method(name = "password.create", inject(user_manager, password_manager))]
async fn create_password<'a>(
    user_manager: &UserManager,
    password_manager: &PasswordManager,
    token: &str,
    name: Cow<'a, str>,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
    attachment: Option<Cow<'a, str>>,
) -> conerror::Result<()> {
    let user = find_user(user_manager, token).await?;
    let create = PasswordCreate {
        name: &name,
        username: &username,
        password: &password,
        attachment: attachment.as_ref().map(|v| &**v),
    };
    password_manager.create_password(&user, create).await?;
    Ok(())
}

#[conerror]
#[method(name = "password.update", inject(user_manager, password_manager))]
async fn update_password<'a>(
    user_manager: &UserManager,
    password_manager: &PasswordManager,
    token: &str,
    id: i64,
    name: Cow<'a, str>,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
    attachment: Option<Cow<'a, str>>,
) -> conerror::Result<()> {
    let user = find_user(user_manager, token).await?;
    let update = PasswordUpdate {
        name: &name,
        username: &username,
        password: &password,
        attachment: attachment.as_ref().map(|v| &**v),
    };
    password_manager.update_password(&user, id, update).await?;
    Ok(())
}

#[conerror]
#[method(name = "password.delete", inject(user_manager, password_manager))]
async fn delete_password(
    user_manager: &UserManager,
    password_manager: &PasswordManager,
    token: &str,
    id: i64,
) -> conerror::Result<()> {
    let user = find_user(user_manager, token).await?;
    password_manager.delete_password(&user, id).await?;
    Ok(())
}

pub const fn methods() -> &'static [Method] {
    methods!(
        login,
        create_user,
        list_password,
        view_password,
        create_password,
        update_password,
        delete_password
    )
}
