use std::borrow::Cow;

use crate::error::msg;
use conerror::conerror;
use rustic_jsonrpc::{method, methods, Method};

use crate::password::{
    Password, PasswordCreate, PasswordListItem, PasswordManager, PasswordUpdate,
};
use crate::user::UserManager;
use crate::Opt;

#[conerror]
#[method(name = "user.login")]
async fn login<'a>(
    #[inject] user_manager: &UserManager,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
) -> conerror::Result<String> {
    let user = user_manager.login(&username, &password).await?;
    let token = user_manager.create_token(&user).await?;
    Ok(token)
}

#[conerror]
#[method(name = "user.create")]
async fn create_user<'a>(
    #[inject] user_manager: &UserManager,
    #[inject] opt: &Opt,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
) -> conerror::Result<()> {
    if !opt.allow_create_user {
        return Err(msg("create user not available"));
    }
    user_manager.create_user(&username, &password).await?;
    Ok(())
}

#[conerror]
#[method(name = "password.list")]
async fn list_password(
    #[inject] user_manager: &UserManager,
    #[inject] password_manager: &PasswordManager,
    token: &str,
) -> conerror::Result<Vec<PasswordListItem>> {
    let user = user_manager.find_user(token).await?;
    let list = password_manager.list_password(&user).await?;
    Ok(list)
}

#[conerror]
#[method(name = "password.view")]
async fn view_password(
    #[inject] user_manager: &UserManager,
    #[inject] password_manager: &PasswordManager,
    token: &str,
    id: i64,
) -> conerror::Result<Option<Password>> {
    let user = user_manager.find_user(token).await?;
    Ok(password_manager.view_password(&user, id).await?)
}

#[conerror]
#[method(name = "password.create")]
async fn create_password<'a>(
    #[inject] user_manager: &UserManager,
    #[inject] password_manager: &PasswordManager,
    token: &str,
    name: Cow<'a, str>,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
    attachment: Option<Cow<'a, str>>,
) -> conerror::Result<()> {
    let user = user_manager.find_user(token).await?;
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
#[method(name = "password.update")]
async fn update_password<'a>(
    #[inject] user_manager: &UserManager,
    #[inject] password_manager: &PasswordManager,
    token: &str,
    id: i64,
    name: Cow<'a, str>,
    username: Cow<'a, str>,
    password: Cow<'a, str>,
    attachment: Option<Cow<'a, str>>,
) -> conerror::Result<()> {
    let user = user_manager.find_user(token).await?;
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
#[method(name = "password.delete")]
async fn delete_password(
    #[inject] user_manager: &UserManager,
    #[inject] password_manager: &PasswordManager,
    token: &str,
    id: i64,
) -> conerror::Result<()> {
    let user = user_manager.find_user(token).await?;
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
