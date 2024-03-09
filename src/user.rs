use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use conerror::conerror;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, SqlitePool};

use crate::encryption::EncryptionManager;
use crate::error::{invalid_token, msg};
use crate::util::{fill_bytes, timestamp};

pub struct User {
    id: i64,
    credential: Credential,
}

impl User {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn credential(&self) -> &Credential {
        &self.credential
    }
}

pub struct Credential(Vec<u8>);

impl Credential {
    fn generate() -> Self {
        let mut v = vec![0; 64];
        fill_bytes(&mut v);
        Self(v)
    }

    pub fn password(&self) -> &[u8] {
        &self.0[..32]
    }

    pub fn salt(&self) -> &[u8] {
        &self.0[32..]
    }
}

const MAX_LOGIN_ATTEMPT: i64 = 5;

const LOGIN_SUSPEND_DURATION: i64 = 300;

const TOKEN_IDLE_DURATION: i64 = 300;

#[derive(Clone)]
pub struct UserManager {
    db: SqlitePool,
    encryption: EncryptionManager,
}

impl UserManager {
    pub fn new(db: SqlitePool, encryption: EncryptionManager) -> Self {
        Self { db, encryption }
    }

    #[conerror]
    pub async fn login(&self, username: &str, password: &str) -> conerror::Result<User> {
        let u = match UserRow::find_by_username(&self.db, username).await? {
            Some(v) => v,
            None => return Err(msg("用户名或密码错误")),
        };
        if u.suspend > timestamp() {
            return Err(msg("请稍后再试"));
        }
        let mut user = User {
            id: u.id,
            credential: Credential(Vec::new()),
        };
        user.credential.0 =
            match self
                .encryption
                .decrypt(&u.credential, password.as_bytes(), &u.salt)
            {
                Ok(v) => {
                    update!("user", {"suspend": 0}, {"id" = u.id})
                        .execute(&self.db)
                        .await?;
                    v
                }
                Err(_) => {
                    let suspend = if u.suspend + 1 >= MAX_LOGIN_ATTEMPT {
                        timestamp() + LOGIN_SUSPEND_DURATION
                    } else {
                        u.suspend + 1
                    };
                    update!("user", {"suspend": suspend}, {"id" = u.id})
                        .execute(&self.db)
                        .await?;
                    return Err(msg("用户名或密码错误"));
                }
            };
        Ok(user)
    }

    #[conerror]
    pub async fn create_token(&self, user: &User) -> conerror::Result<String> {
        let token = Credential::generate();
        let credential =
            self.encryption
                .encrypt(&to_vec(user.credential()), token.password(), token.salt())?;
        let now = timestamp();
        let id = insert!("token", {
            "user_id": user.id,
            "credential": &credential,
            "last_active": now,
            "created_at": now,
        })
        .execute(&self.db)
        .await?
        .last_insert_rowid();

        let mut token = token.0;
        token.extend_from_slice(&id.to_le_bytes());

        let mut hasher = Sha256::new();
        hasher.update(&token);
        hasher.update(&credential);
        token.extend_from_slice(&hasher.finalize());
        Ok(BASE64_URL_SAFE_NO_PAD.encode(token))
    }

    #[conerror]
    pub async fn create_user(&self, username: &str, password: &str) -> conerror::Result<User> {
        if username.is_empty() || password.is_empty() {
            return Err(msg("参数错误"));
        }

        let mut user = User {
            id: 0,
            credential: Credential::generate(),
        };

        let mut salt = vec![0u8; 32];
        fill_bytes(&mut salt);
        let credential =
            self.encryption
                .encrypt(&to_vec(&user.credential), password.as_bytes(), &salt)?;

        let result = insert_ignore!("user", {
            "username": username,
            "salt": &salt,
            "credential": &credential,
            "created_at": timestamp(),
        })
        .execute(&self.db)
        .await?;
        if result.rows_affected() == 0 {
            return Err(msg("用户已存在"));
        }
        user.id = result.last_insert_rowid();
        Ok(user)
    }

    #[conerror]
    pub async fn find_user(&self, token: &str) -> conerror::Result<User> {
        match self.find_user_optional(token).await? {
            Some(v) => Ok(v),
            None => Err(invalid_token()),
        }
    }

    #[conerror]
    pub async fn find_user_optional(&self, token: &str) -> conerror::Result<Option<User>> {
        let token = BASE64_URL_SAFE_NO_PAD.decode(token.as_bytes())?;
        if token.len() != 104 {
            return Ok(None);
        }
        let id = i64::from_le_bytes([
            token[64], token[65], token[66], token[67], token[68], token[69], token[70], token[71],
        ]);
        let t = match TokenRow::find(&self.db, id).await? {
            Some(v) => v,
            None => return Ok(None),
        };

        let mut hasher = Sha256::new();
        hasher.update(&token[..72]);
        hasher.update(&t.credential);
        if hasher.finalize().as_slice() != &token[72..] {
            return Ok(None);
        }

        if t.last_active + TOKEN_IDLE_DURATION <= timestamp() as i64 {
            return Ok(None);
        }

        let user = match UserRow::find(&self.db, t.user_id).await? {
            Some(v) => v,
            None => return Ok(None),
        };
        let credential = self
            .encryption
            .decrypt(&t.credential, &token[..32], &token[32..64])?;

        update!("token", {"last_active": timestamp()}, {"id" = t.id})
            .execute(&self.db)
            .await?;
        Ok(Some(User {
            id: user.id,
            credential: Credential(credential),
        }))
    }
}

#[derive(FromRow)]
struct UserRow {
    id: i64,
    salt: Vec<u8>,
    credential: Vec<u8>,
    suspend: i64,
}

impl UserRow {
    #[conerror]
    async fn find(db: &SqlitePool, id: i32) -> conerror::Result<Option<UserRow>> {
        let row = select!("user", ["id", "salt", "credential", "suspend"], {
            "id" = id
        })
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    #[conerror]
    async fn find_by_username(
        db: &SqlitePool,
        username: &str,
    ) -> conerror::Result<Option<UserRow>> {
        let row = select!("user", ["id", "salt", "credential", "suspend"], {
            "username" = username
        })
        .fetch_optional(db)
        .await?;
        Ok(row)
    }
}

#[derive(FromRow)]
struct TokenRow {
    id: i32,
    user_id: i32,
    credential: Vec<u8>,
    last_active: i64,
}

impl TokenRow {
    #[conerror]
    async fn find(db: &SqlitePool, id: i64) -> conerror::Result<Option<Self>> {
        let row = select!("token", ["id", "user_id", "credential", "last_active"], {
            "id" = id
        })
        .fetch_optional(db)
        .await?;
        Ok(row)
    }
}

fn to_vec(credential: &Credential) -> Vec<u8> {
    let mut v = Vec::with_capacity(credential.password().len() + credential.salt().len());
    v.extend_from_slice(credential.password());
    v.extend_from_slice(credential.salt());
    v
}
