mod workers;
mod constants;

// src/lib.rs
use mlua::prelude::*;
use mlua::{UserData, Value};
use serde_derive::{Deserialize, Serialize};
use crate::workers::client;


fn hello(_: &Lua, name: String) -> LuaResult<()> {
    println!("hello, {name}!");
    Ok(())
}

#[derive(Clone)]
struct A {
    name: String
}

impl ToLua<'_> for A {
    fn to_lua(self, lua: &'_ Lua) -> LuaResult<LuaValue> {
        let table = lua.create_table()?;
        table.set("name", self.name)?;
        Ok(table.to_lua(lua)?)
    }
}

fn create_a(lua: &Lua) -> LuaResult<LuaValue> {
    let a = A {
        name: "hello".to_string()
    };
    Ok(a.to_lua(lua)?)
}

#[mlua::lua_module]
fn discolua(lua: &Lua) -> LuaResult<LuaTable> {
    // TODO
    // Créer fonction pour définir le niveau de log (optionnel, par défaut: erreur ?)
    // Créer un event ?
    env_logger::builder().filter_level(log::LevelFilter::Trace).init();

    let exports = lua.create_table()?;
    exports.set("hello", lua.create_function(hello)?)?;

    exports.set("A", "test")?;

    exports.set("Client", client::build_table(lua)?)?;

    // create metadata
    {
        let metadata = lua.create_table()?;
        metadata.set("__name", "Discordia")?;

        metadata.set("Client", client::create_metatable(lua))?;

        exports.set_metatable(Some(metadata));
    }

    Ok(exports)
}
