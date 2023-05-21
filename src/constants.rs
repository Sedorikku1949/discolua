/// The API version to use.
pub const API_VERSION: u8 = 10;
/// The base URL for the API.
pub const API_BASE_URL: &str = "https://discord.com/api/v10";

/// The base URL for the gateway.
pub const GATEWAY_URL: &str = "wss://gateway.discord.gg";
/// The version of the gateway to use.
pub const GATEWAY_VERSION: u8 = 10;

/// The user agent to use for requests to Discord.
pub const USER_AGENT: &str = concat!("DiscordBot (", env!("CARGO_PKG_HOMEPAGE"), ", ", env!("CARGO_PKG_VERSION"), ")");