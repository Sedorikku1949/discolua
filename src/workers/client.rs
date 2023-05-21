use std::future::Future;
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use chrono::{DateTime, Utc};
use mlua::prelude::*;
use mlua::{AnyUserData, Function, MetaMethod, Nil, Table, ThreadStatus, UserData, UserDataFields, UserDataMetatable, UserDataMethods};
use crate::workers::cache::CacheOptions;
use crate::workers::ws::{WebSocketWorker};

/// Options for the client.
#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// The intents to use for the client.
    /// Reference:
    /// https://discord.com/developers/docs/topics/gateway#gateway-intents
    pub intents: u64,
    /// The amount of times to retry a request before giving up.
    /// By default, this is 3.
    pub retry_limit: u8,
    /// The options for the cache.
    pub cache: CacheOptions,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            intents: 0,
            retry_limit: 3,
            cache: CacheOptions::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Client {
    /// The shard ID of the client.
    shard: Option<u64>,
    /// The time since the client has been ready.
    ready_since: Option<DateTime<Utc>>,
    /// The options for the client.
    options: ClientOptions,
    /// The token to use for the client.
    token: Option<String>,
    /// The WebSocket connection to use for the client.
    ws: Option<WebSocketWorker>,
    ws_thread: Option<Arc<thread::JoinHandle<()>>>,
}

impl Client {
    fn login(&mut self, _: &Lua, token: String) {
        self.ws = Some(WebSocketWorker::init().expect("Failed to initialize WebSocket worker"));
        self.token = Some(token);

        sleep(Duration::from_secs(60));
    }
}

impl UserData for Client {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("shard", |_, this| Ok(this.shard));
        fields.add_field_method_get("ready_since", |_, this| {
            if let Some(ready_since) = this.ready_since {
                Ok(ready_since.timestamp())
            } else {
                Ok(0)
            }
        });
    }

    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::ToString, |lua, this, _: ()| client_to_string(lua, this));

        methods.add_method("get_shard", |_: &Lua, this, _: ()| Ok(this.shard));

        //methods.add_async_method("login", |lua, mut this, (token): (String)| async move {
        //    this.login(token).await;
        //    Ok(())
        //});

        methods.add_method_mut("login", |lua, mut this, (token): (String)| {
            this.login(&lua, token);
            Ok(())
        });
    }
}


// add lua metadata to client
impl Default for Client {
    fn default() -> Self {
        Self {
            shard: None,
            ready_since: None,
            options: ClientOptions::default(),
            token: None,
            ws: None,
            ws_thread: None,
        }
    }
}

fn create_client(_: &Lua, _: ()) -> LuaResult<Client> {
    Ok(Client::default())
}

fn client_to_string(_: &Lua, this: &Client) -> LuaResult<String> {
    Ok(format!("{:?}", this))
}

pub fn build_table(lua: &Lua) -> LuaResult<LuaTable> {
    let exports = lua.create_table().unwrap();
    exports.set("new", lua.create_function(create_client)?)?;

    exports.set_metatable(Some(create_metatable(lua)));

    Ok(exports)
}

pub fn create_metatable(lua: &Lua) -> Table {
    let client_meta = lua.create_table().unwrap();
    client_meta.set("__name", "Client").unwrap();
    client_meta.set("__tostring", lua.create_function(|lua: &Lua, this: Client| { client_to_string(lua, &this) }).unwrap()).unwrap();

    client_meta
}