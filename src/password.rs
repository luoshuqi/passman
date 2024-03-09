use conerror::conerror;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};

use crate::encryption::EncryptionManager;
use crate::user::User;
use crate::util::timestamp;

pub struct PasswordCreate<'a> {
    pub name: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub attachment: Option<&'a str>,
}

pub type PasswordUpdate<'a> = PasswordCreate<'a>;

#[derive(FromRow, Serialize)]
pub struct PasswordListItem {
    id: i64,
    name: String,
    updated_at: i64,
}

#[derive(Serialize)]
pub struct Password {
    id: i32,
    name: String,
    username: String,
    password: String,
    attachment: Option<String>,
}

pub struct PasswordManager {
    db: SqlitePool,
    encryption: EncryptionManager,
}

impl PasswordManager {
    pub fn new(db: SqlitePool, encryption: EncryptionManager) -> Self {
        Self { db, encryption }
    }

    #[conerror]
    pub async fn list_password(&self, user: &User) -> conerror::Result<Vec<PasswordListItem>> {
        let list = select!(
            "password",
            ["id", "name", "updated_at"],
            { "user_id" = user.id() },
            "ORDER BY updated_at DESC"
        )
        .fetch_all(&self.db)
        .await?;
        Ok(list)
    }

    #[conerror]
    pub async fn view_password(&self, user: &User, id: i64) -> conerror::Result<Option<Password>> {
        #[derive(FromRow)]
        struct Row {
            id: i32,
            name: String,
            username: Vec<u8>,
            password: Vec<u8>,
            attachment: Option<Vec<u8>>,
        }
        let password: Option<Row> = select!(
            "password",
            ["id", "name", "username", "password", "attachment"],
            {"id" = id, "user_id" = user.id()}
        )
        .fetch_optional(&self.db)
        .await?;

        match password {
            Some(password) => Ok(Some(Password {
                id: password.id,
                name: password.name,
                username: String::from_utf8(self.decrypt(user, &password.username)?)?,
                password: String::from_utf8(self.decrypt(user, &password.password)?)?,
                attachment: match password.attachment {
                    Some(v) => Some(String::from_utf8(self.decrypt(user, &v)?)?),
                    None => None,
                },
            })),
            None => Ok(None),
        }
    }

    #[conerror]
    pub async fn create_password(
        &self,
        user: &User,
        create: PasswordCreate<'_>,
    ) -> conerror::Result<()> {
        let username = self.encrypt(user, create.username.as_bytes())?;
        let password = self.encrypt(user, create.password.as_bytes())?;
        let attachment = match create.attachment {
            Some(v) => Some(self.encrypt(user, v.as_bytes())?),
            _ => None,
        };
        let now = timestamp();
        insert!("password", {
            "user_id": user.id(),
            "name": create.name,
            "username": &username,
            "password": &password,
            "attachment": &attachment,
            "updated_at": now,
            "created_at": now,
        })
        .execute(&self.db)
        .await?;
        Ok(())
    }

    #[conerror]
    pub async fn update_password(
        &self,
        user: &User,
        id: i64,
        update: PasswordUpdate<'_>,
    ) -> conerror::Result<()> {
        let username = self.encrypt(user, update.username.as_bytes())?;
        let password = self.encrypt(user, update.password.as_bytes())?;
        let attachment = match update.attachment {
            Some(v) => Some(self.encrypt(user, v.as_bytes())?),
            _ => None,
        };
        let now = timestamp();
        update!("password",
        {"name": update.name, "username": &username, "password": &password, "attachment": &attachment, "updated_at": now},
        {"id" = id, "user_id" = user.id()}).execute(&self.db).await?;
        Ok(())
    }

    #[conerror]
    pub async fn delete_password(&self, user: &User, id: i64) -> conerror::Result<()> {
        delete!("password", {"id" = id, "user_id" = user.id()})
            .execute(&self.db)
            .await?;
        Ok(())
    }

    #[conerror]
    fn encrypt(&self, user: &User, data: &[u8]) -> conerror::Result<Vec<u8>> {
        let data = self.encryption.encrypt(
            data,
            user.credential().password(),
            user.credential().salt(),
        )?;
        Ok(data)
    }

    #[conerror]
    fn decrypt(&self, user: &User, data: &[u8]) -> conerror::Result<Vec<u8>> {
        let data = self.encryption.decrypt(
            data,
            user.credential().password(),
            user.credential().salt(),
        )?;
        Ok(data)
    }
}
