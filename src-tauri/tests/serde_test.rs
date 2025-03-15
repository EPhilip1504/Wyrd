#![allow(unused)]
#![allow(warnings)]

use axum::Json;
use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};

#[derive(Deserialize, Serialize, Debug)]
pub struct Theme {
    pub mode: String,
    pub rgb: (u8, u8, u8),
}

#[cfg(test)]
mod tests {
    use postgres::types::ToSql;
    use tower_sessions::cookie::time::serde::timestamp::option::deserialize;

    use super::*;

    #[test]
    fn test_theme_json() {
        let mode = "Light";
        let rgb: (u8, u8, u8) = (0, 142, 255);
        let theme: Option<Theme> = Some(Theme {
            mode: "Dark".to_string(),
            rgb: (0, 145, 255),
        });

        let encoded = bincode::serialize(&theme).unwrap();
        let decoded: Option<Theme> = bincode::deserialize(&encoded[..]).unwrap();
        let payload = Json(theme.unwrap());
        println!("{:?}", encoded);
        println!("{:?}", payload);
    }
}

fn main() {
    // Your main function can be empty if you're only running tests
}
