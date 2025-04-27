#![allow(unused)]
#![allow(warnings)]
use anyhow::anyhow;
#[cfg(test)]
use do_username::do_username;
use env_logger::fmt::Timestamp;
use log::log;
use oauth2::basic::{
    BasicErrorResponse, BasicErrorResponseType, BasicRevocationErrorResponse,
    BasicTokenIntrospectionResponse, BasicTokenType,
};
use oauth2::{
    helpers, AsyncHttpClient, AuthUrl, CodeTokenRequest, ConfigurationError, EmptyExtraTokenFields,
    EndpointNotSet, EndpointState, ErrorResponse, PkceCodeVerifier, RevocableToken,
    RevocationRequest, StandardErrorResponse, StandardRevocableToken, StandardTokenResponse,
    TokenIntrospectionResponse,
};
use openidconnect::core::{
    CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreErrorResponseType, CoreGenderClaim,
    CoreIdTokenFields, CoreJwsSigningAlgorithm, CoreProviderMetadata, CoreResponseType,
    CoreRevocationErrorResponse, CoreTokenIntrospectionResponse, CoreTokenResponse, CoreTokenType,
};
use openidconnect::{
    reqwest, AccessToken, AdditionalClaims, AuthDisplay, AuthPrompt, AuthorizationRequest, Client,
    EmptyAdditionalClaims, ExtraTokenFields, GenderClaim, IdTokenFields, IdTokenVerifier,
    JsonWebKey, JsonWebKeySetUrl, JweContentEncryptionAlgorithm, JwsSigningAlgorithm, LanguageTag,
    ResponseType, ResponseTypes, SubjectIdentifier, SubjectIdentifierType, TokenResponse,
    TokenType, UserInfoRequest,
};
use openidconnect::{
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope,
};
use sqlx::PgPool;
use std::ops::Deref;
use std::process::exit;
use std::result::Result::Ok;
use std::str::FromStr;
use sysinfo::System;
use url::{ParseError, Url};

use openidconnect::core::{
    CoreAuthDisplay, CoreClaimName, CoreClaimType, CoreClientAuthMethod, CoreGrantType,
    CoreIdTokenClaims, CoreIdTokenVerifier, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreRevocableToken, CoreSubjectIdentifierType,
    CoreUserInfoClaims,
};
use openidconnect::{AdditionalProviderMetadata, ProviderMetadata, RevocationUrl, UserInfoClaims};
use serde::{Deserialize, Serialize};

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
#[derive(Clone, Debug, Deserialize, Serialize)]

struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

pub const HTML_GOOGLE_SUCCESS_RESPONSE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication Successful</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;500&display=swap');

        body {
            font-family: 'Roboto', -apple-system, BlinkMacSystemFont, 'Segoe UI', Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 90vh;
            background-color: #f8f9fa;
            color: #5f6368;
            margin: 0;
            padding: 20px;
            box-sizing: border-box;
        }
        .container {
            background-color: #ffffff;
            padding: 40px 30px;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24);
            text-align: center;
            max-width: 450px;
            width: 100%;
        }
        .logo { /* Style for the logo image */
            display: block; /* Prevents extra space below */
            width: 48px;    /* Set desired size */
            height: 48px;
            margin: 0 auto 20px auto; /* Center logo and add space below */
        }
        h1 {
            color: #202124;
            font-weight: 500;
            font-size: 22px;
            margin-top: 0;
            margin-bottom: 15px;
        }
        p {
            font-size: 14px;
            line-height: 1.6;
            margin-bottom: 25px;
            color: #5f6368;
        }
        .close-info {
             font-size: 12px;
             color: #80868b;
        }
    </style>
</head>
<body>
    <div class="container">
        <img src="https://upload.wikimedia.org/wikipedia/commons/5/53/Google_%22G%22_logo.svg" alt="Google Logo" class="logo">

        <h1>Authentication Successful</h1>
        <p>You have successfully signed in with Google.</p>
        <p class="close-info">This window will close automatically in a few seconds.</p>
    </div>
    <script>
        // Auto-close this window after a short delay
        setTimeout(function() {
            window.close();
        }, 3000);
    </script>
</body>
</html>
"#;

pub const HTML_MS_SUCCESS_RESPONSE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication Successful</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;500&display=swap'); /* Using Roboto */
        /* Or consider Segoe UI for a more Microsoft feel:
        @import url('https://fonts.googleapis.com/css2?family=Segoe+UI:wght@400;600&display=swap');
        body { font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif; } */

        body {
            font-family: 'Roboto', -apple-system, BlinkMacSystemFont, 'Segoe UI', Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 90vh;
            background-color: #f8f9fa;
            color: #5f6368;
            margin: 0;
            padding: 20px;
            box-sizing: border-box;
        }
        .container {
            background-color: #ffffff;
            padding: 40px 30px;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24);
            text-align: center;
            max-width: 450px;
            width: 100%;
        }
        .logo { /* Style for the logo image */
            display: block;
            width: 48px;    /* Adjust size if needed */
            height: 48px;
            margin: 0 auto 20px auto;
        }
        h1 {
            color: #202124;
            font-weight: 500;
            font-size: 22px;
            margin-top: 0;
            margin-bottom: 15px;
        }
        p {
            font-size: 14px;
            line-height: 1.6;
            margin-bottom: 25px;
            color: #5f6368;
        }
        .close-info {
             font-size: 12px;
             color: #80868b;
        }
    </style>
</head>
<body>
    <div class="container">
        <img src="https://upload.wikimedia.org/wikipedia/commons/4/44/Microsoft_logo.svg" alt="Microsoft Logo" class="logo">

        <h1>Authentication Successful</h1>
        <p>You have successfully signed in with Microsoft.</p>
        <p class="close-info">This window will close automatically in a few seconds.</p>
    </div>
    <script>
        // Auto-close this window after a short delay
        setTimeout(function() {
            window.close();
        }, 3000);
    </script>
</body>
</html>
"#;

pub const HTML_GH_SUCCESS_RESPONSE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Authentication Successful</title>
    <style>
        @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;500&display=swap');

        body {
            font-family: 'Roboto', -apple-system, BlinkMacSystemFont, 'Segoe UI', Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 90vh;
            background-color: #f8f9fa;
            color: #5f6368;
            margin: 0;
            padding: 20px;
            box-sizing: border-box;
        }
        .container {
            background-color: #ffffff;
            padding: 40px 30px;
            border-radius: 8px;
            box-shadow: 0 1px 3px rgba(0,0,0,0.12), 0 1px 2px rgba(0,0,0,0.24);
            text-align: center;
            max-width: 450px;
            width: 100%;
        }
        .logo { /* Style for the logo image */
            display: block;
            width: 48px;    /* Adjust size if needed */
            height: 48px;
            margin: 0 auto 20px auto;
        }
        h1 {
            color: #202124;
            font-weight: 500;
            font-size: 22px;
            margin-top: 0;
            margin-bottom: 15px;
        }
        p {
            font-size: 14px;
            line-height: 1.6;
            margin-bottom: 25px;
            color: #5f6368;
        }
        .close-info {
             font-size: 12px;
             color: #80868b;
        }
    </style>
</head>
<body>
    <div class="container">
        <img src="https://upload.wikimedia.org/wikipedia/commons/9/91/Octicons-mark-github.svg" alt="GitHub Logo" class="logo">

        <h1>Authentication Successful</h1>
        <p>You have successfully signed in with GitHub.</p>
        <p class="close-info">This window will close automatically in a few seconds.</p>
    </div>
    <script>
        // Auto-close this window after a short delay
        setTimeout(function() {
            window.close();
        }, 3000);
    </script>
</body>
</html>
"#;

pub struct ThirdPartyAuthInfo {
    first_name: String,
    last_name: Option<String>,
    user_id: String,
    email: String,
    username: String,
    profile_url: Option<String>,
}

pub struct ClientCredentials {
    pub issuer_url: IssuerUrl,
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub tenant_id: Option<String>,
    pub scopes: Option<Vec<Scope>>,
    pub claims: Option<Vec<CoreClaimName>>,
}

pub fn handle_error<T: std::error::Error>(fail: &T, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    caused by: {}", cause);
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    exit(1);
}

pub async fn create_client(provider: &str) -> Result<(ClientCredentials), anyhow::Error> {
    dotenv::dotenv().ok();
    let scopes = Some(vec![
        Scope::new("openid".to_string()),
        Scope::new("email".to_string()),
        Scope::new("profile".to_string()),
    ]);
    let claims = Some(vec![
        CoreClaimName::new("sub".to_string()),
        CoreClaimName::new("aud".to_string()),
        CoreClaimName::new("email".to_string()),
        CoreClaimName::new("email_verified".to_string()),
        CoreClaimName::new("exp".to_string()),
        CoreClaimName::new("iat".to_string()),
        CoreClaimName::new("iss".to_string()),
        CoreClaimName::new("name".to_string()),
        CoreClaimName::new("given_name".to_string()),
        CoreClaimName::new("family_name".to_string()),
        CoreClaimName::new("picture".to_string()),
        CoreClaimName::new("locale".to_string()),
    ]);
    let client_creds: ClientCredentials = match provider {
        "GOOGLE" => ClientCredentials {
            issuer_url: IssuerUrl::new(String::from("https://accounts.google.com"))?,
            client_id: ClientId::new(std::env::var("GOOGLE_CLIENT_ID")?),
            client_secret: ClientSecret::new(std::env::var("GOOGLE_CLIENT_SECRET")?),
            tenant_id: None,
            scopes,
            claims,
        },
        "MS" => {
            let tenant_id = std::env::var("MS_TENANT_ID")?;
            ClientCredentials {
                issuer_url: IssuerUrl::new(format!("https://sts.windows.net/{}/", tenant_id))?, // Example issuer URL
                client_id: ClientId::new(std::env::var("MS_CLIENT_ID")?),
                client_secret: ClientSecret::new(std::env::var("MS_CLIENT_SECRET")?),
                tenant_id: Some(tenant_id.clone()),
                scopes,
                claims,
            }
        }
        _ => ClientCredentials {
            issuer_url: IssuerUrl::new(String::from("https://accounts.google.com"))?, // Example issuer URL
            client_id: ClientId::new(std::env::var("MS_CLIENT_ID")?),
            client_secret: ClientSecret::new(std::env::var("MS_CLIENT_SECRET")?),
            tenant_id: None,
            scopes,
            claims,
        },
    };
    return Ok(client_creds);
}

pub fn auth_code(provider: &str) -> (AuthorizationCode, CsrfToken) {
    let (code, state) = {
        // A very naive implementation of the redirect server.
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        // Accept one connection
        let (mut stream, _) = listener.accept().unwrap();

        let mut reader = BufReader::new(&stream);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();

        let redirect_url = request_line.split_whitespace().nth(1).unwrap();
        let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

        let code = url
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, code)| AuthorizationCode::new(code.into_owned()))
            .unwrap();

        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, state)| CsrfToken::new(state.into_owned()))
            .unwrap();

        let success_response = match provider {
            "GOOGLE" => HTML_GOOGLE_SUCCESS_RESPONSE,
            "MS" => HTML_MS_SUCCESS_RESPONSE,
            _ => HTML_GH_SUCCESS_RESPONSE,
        };

        let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                success_response.len(),
                success_response
            );

        stream.write_all(response.as_bytes()).unwrap();
        (code, state)
    };
    (code, state)
}



pub async fn sign_in_or_sign_up(
    user_info: CoreUserInfoClaims,
    pool: &PgPool,
) -> Result<(String), anyhow::Error> {
    /*
    Search through DB for user_id to make sure user isn't already signedup
    */

    let user_id = user_info.subject().to_string();
    let email = user_info.email().unwrap().to_string();
    let first_name = user_info
        .given_name()
        .unwrap()
        .get(None)
        .unwrap()
        .to_string();
    let username = email.split('@').collect::<Vec<&str>>()[0].to_string();
    let lang = LanguageTag::new(String::from("en"));
    if user_info.locale().is_none() {
        user_info.clone().set_locale(Some(lang.clone()));
    }

    let mut last_name = user_info
        .family_name()
        .and_then(|name| name.get(None).map(|s| s.to_string()));

    let profile_url = user_info
        .picture()
        .and_then(|pic| pic.get(None).map(|s| s.to_string()));

    let provider = "GOOGLE";
    //let username = generate_username();
    /*let new_google_user = GoogleUserInfo{
        google_id,
        first_name,
    }*/
    println!("FIRST NAME: {:?}: ", first_name);
    println!("LAST NAME: {:?}", last_name);
    println!("USER ID: {:?}: ", user_id);
    println!("EMAIL: {:?}", email);
    println!("USERNAME: {:?}", username);
    println!("PICTURE URL: {:?}", profile_url);

    let user = ThirdPartyAuthInfo {
        first_name,
        last_name,
        user_id,
        email,
        username,
        profile_url,
    };
    //println!("LAST NAME: {:?}", last_name);
    //println!("PICTURE URL: {}", picture_url);

    Ok(user.user_id)
}


