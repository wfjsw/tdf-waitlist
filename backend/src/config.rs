use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub idle_timeout: u64,
    pub connect_timeout: u64,
}

#[derive(Deserialize, Clone)]
pub struct ESIConfig {
    pub client_id: String,
    pub client_secret: String,
    pub url: String,
}

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub token_secret: String,
}

#[derive(Deserialize, Clone)]
pub struct SSEConfig {
    pub url: String,
    pub secret: String,
}

#[derive(Deserialize, Clone)]
pub struct FleetUpdaterConfig {
    pub enable: bool,
    pub min_in_fleet: usize,
}

#[derive(Deserialize, Clone)]
pub struct SkillUpdaterConfig {
    pub enable: bool,
    pub runtime: f64,
}

#[derive(Deserialize, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub app: AppConfig,
    pub esi: ESIConfig,
    pub sse: SSEConfig,
    pub fleet_updater: FleetUpdaterConfig,
    pub skill_updater: SkillUpdaterConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    // The example config parses as shipped: SRP has no switch or section any more
    #[test]
    fn example_config_parses() {
        // A Windows checkout has CRLF line endings
        let example = include_str!("../config.example.toml").replace("\r\n", "\n");
        toml::from_str::<Config>(&example).unwrap();
    }

    // A deployed config that still carries [srp] keeps working
    #[test]
    fn old_srp_section_is_ignored() {
        let example = include_str!("../config.example.toml").replace("\r\n", "\n");
        let old = format!("{}\n[srp]\nenable = true\nmanagers = [\"1\"]\n", example);
        toml::from_str::<Config>(&old).unwrap();
    }
}
