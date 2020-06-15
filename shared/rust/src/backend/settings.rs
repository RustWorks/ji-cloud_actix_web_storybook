use jsonwebtoken::EncodingKey;
use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use super::google::{get_secret, get_access_token_and_project_id};
use once_cell::sync::OnceCell;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
#[derive(Debug, PartialEq, Eq)]
pub enum RemoteTarget {
    Local,
    Sandbox,
    Release,
}

pub static SETTINGS:OnceCell<Settings> = OnceCell::new();

pub struct Settings {
    pub auth_target: RemoteTarget,
    pub db_target: RemoteTarget,
    pub media_url_base: &'static str,
    pub local_insecure: bool,
    pub port: u16,
    pub epoch: Duration,
    pub jwt_encoding_key: EncodingKey,
    //TODO see: https://github.com/Keats/jsonwebtoken/issues/120#issuecomment-634096881
    //Keeping a string is a stop-gap measure for now, not ideal
    pub jwt_decoding_key:String,
    pub inter_server_secret:String,
    pub db_connection_string:String,
}

pub async fn init(target:RemoteTarget) {
    let (token, project_id) = get_access_token_and_project_id().await.unwrap();

    let jwt_secret = get_secret(token.as_ref(), &project_id, "JWT_SECRET").await;
    let db_pass = get_secret(token.as_ref(), &project_id, "DB_PASS").await;
    let inter_server_secret = get_secret(token.as_ref(), &project_id, "INTER_SERVER").await;


    let jwt_encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());

    //TODO see: https://github.com/Keats/jsonwebtoken/issues/120#issuecomment-634096881
    //Keeping a string is a stop-gap measure for now, not ideal

    SETTINGS.set(match target {
        RemoteTarget::Local => Settings::new_local(jwt_encoding_key, jwt_secret, inter_server_secret, db_pass),
        RemoteTarget::Sandbox => Settings::new_sandbox(jwt_encoding_key, jwt_secret, inter_server_secret, db_pass),
        RemoteTarget::Release => Settings::new_release(jwt_encoding_key, jwt_secret, inter_server_secret, db_pass),
    }).unwrap();
}

fn db_connection_string(db_pass:&str, db_target:RemoteTarget) -> String {
    match db_target {
        RemoteTarget::Local => format!("postgres://postgres:{}@localhost:3306/jicloud", db_pass),
        _ => {
            let instance_connection = std::env::var("INSTANCE_CONNECTION_NAME").unwrap_or(
                match db_target {
                    RemoteTarget::Sandbox => "ji-cloud-developer-sandbox:europe-west1:ji-cloud-003-sandbox",
                    RemoteTarget::Release => "ji-cloud:europe-west1:ji-cloud-002",
                    _ => ""
                }.to_string()
            );
            let socket_path = std::env::var("DB_SOCKET_PATH").unwrap_or("/cloudsql".to_string());

            let full_socket_path = utf8_percent_encode(&format!("{}/{}", socket_path, instance_connection), NON_ALPHANUMERIC).to_string();

            let db_user = "postgres";
            let db_name = "jicloud";
            let connection_string = format!("postgres://{}:{}@{}/{}", db_user, db_pass, full_socket_path, db_name);

            connection_string
        }
    }
}
pub static CORS_ORIGINS:[&'static str;1] = ["https://jicloud.org"];
pub const MAX_SIGNIN_COOKIE:&'static str = "1209600"; // 2 weeks
pub const JSON_BODY_LIMIT:u64 = 16384; //1024 * 16
pub const HANDLEBARS_PATH:&'static str = "./handlebars";

impl Settings {
    pub fn js_api(&self) -> &'static str {
        match self.auth_target {
            RemoteTarget::Local => "http://localhost:8082",
            RemoteTarget::Sandbox=> "https://sandbox.api-js.jicloud.org",
            RemoteTarget::Release=> "https://api-js.jicloud.org",
        }
    }
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "auth_target is [{:?}] and db_target is [{:?}]. port is [{}]", self.auth_target, self.db_target, self.port)
    }
}
fn get_epoch() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
}



    //SETTINGS.set(Settings::new(jwt_encoding_key, jwt_secret, inter_server_secret, db_pass));
impl Settings {
    pub fn new_local(jwt_encoding_key:EncodingKey, jwt_decoding_key: String, inter_server_secret:String, db_pass:String) -> Self {
        Self {
            auth_target: RemoteTarget::Local,
            db_target: RemoteTarget::Local,
            media_url_base: "http://localhost:4102",
            local_insecure: true,
            port: 8081,
            epoch: get_epoch(),
            jwt_encoding_key,
            jwt_decoding_key,
            inter_server_secret,
            db_connection_string: db_connection_string(&db_pass, RemoteTarget::Local),
        }
    }
    pub fn new_sandbox(jwt_encoding_key:EncodingKey, jwt_decoding_key: String, inter_server_secret:String, db_pass:String) -> Self {
        Self {
            auth_target: RemoteTarget::Sandbox,
            db_target: RemoteTarget::Sandbox,
            media_url_base: "https://storage.googleapis.com/ji-cloud-eu",
            port: 8080,
            local_insecure: false,
            epoch: get_epoch(),
            jwt_encoding_key,
            jwt_decoding_key,
            inter_server_secret,
            db_connection_string: db_connection_string(&db_pass, RemoteTarget::Sandbox),
        }
    }
    pub fn new_release(jwt_encoding_key:EncodingKey, jwt_decoding_key: String, inter_server_secret:String, db_pass:String) -> Self {
        Self {
            auth_target: RemoteTarget::Release,
            db_target: RemoteTarget::Release,
            media_url_base: "https://storage.googleapis.com/ji-cloud-eu",
            port: 8080,
            local_insecure: false,
            epoch: get_epoch(),
            jwt_encoding_key,
            jwt_decoding_key,
            inter_server_secret,
            db_connection_string: db_connection_string(&db_pass, RemoteTarget::Release),
        }
    }

    pub fn spa_url(&self, app:&str, path:&str) -> String {
        format!("{}/spa/{}/{}", self.media_url_base, app, path)
    }
    
}