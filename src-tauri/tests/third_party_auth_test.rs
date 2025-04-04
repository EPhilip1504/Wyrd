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
    helpers, EmptyExtraTokenFields, EndpointNotSet, StandardErrorResponse, StandardRevocableToken,
    StandardTokenResponse,
};
use openidconnect::core::{
    CoreAuthPrompt, CoreAuthenticationFlow, CoreClient, CoreErrorResponseType, CoreGenderClaim,
    CoreIdTokenFields, CoreProviderMetadata, CoreResponseType, CoreRevocationErrorResponse,
    CoreTokenIntrospectionResponse, CoreTokenResponse, CoreTokenType,
};
use openidconnect::{
    reqwest, AccessToken, AdditionalClaims, Client, EmptyAdditionalClaims, ExtraTokenFields,
    GenderClaim, IdTokenFields, JweContentEncryptionAlgorithm, JwsSigningAlgorithm, LanguageTag,
    TokenResponse, TokenType,
};
use openidconnect::{
    AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope,
};
use std::ops::Deref;
use std::process::exit;
use sysinfo::System;
use url::Url;

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

const html_success_response: &str = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>Authentication Successful</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', sans-serif;
                text-align: center;
                padding: 40px 20px;
                max-width: 600px;
                margin: 0 auto;
                line-height: 1.5;
            }
            h1 {
                color: #4285f4;
                margin-bottom: 20px;
            }
            p {
                font-size: 16px;
                color: #333;
            }
            .success-icon {
                font-size: 64px;
                margin-bottom: 20px;
                color: #34a853;
            }
        </style>
    </head>
    <body>
        <div class="success-icon">✓</div>
        <h1>Authentication Successful</h1>
        <p>You have successfully signed in with Google.</p>
        <p>You can close this window and return to the application.</p>
        <script>
            // Auto-close this window after a short delay (optional)
            setTimeout(function() {
                window.close();
            }, 3000);
        </script>
    </body>
    </html>
    "#;

struct ThirdPartyAuthInfo {
    first_name: String,
    last_name: Option<String>,
    user_id: String,
    email: String,
    username: String,
    profile_url: Option<String>,
}

fn handle_error<T: std::error::Error>(fail: &T, msg: &'static str) {
    let mut err_msg = format!("ERROR: {}", msg);
    let mut cur_fail: Option<&dyn std::error::Error> = Some(fail);
    while let Some(cause) = cur_fail {
        err_msg += &format!("\n    caused by: {}", cause);
        cur_fail = cause.source();
    }
    println!("{}", err_msg);
    exit(1);
}

pub async fn tp_auth(provider: &str) -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let Some((client_id, client_secret, ms_tenant_id, redirect_url)) = match provider {
        "GOOGLE" => {
            std::env::var("GOOGLE_CLIENT_ID")?;
        }
        "MS" => {}
        "GITHUB" => {}
    };
    let google_client_id =
    let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET")?;

    let ms_client_id = std::env::var("MS_CLIENT_ID")?;
    let ms_client_secret = std::env::var("MS_CLIENT_SECRET")?;
    let ms_tenant_id = std::env::var("MS_TENANT_ID")?;
    let ms_redirect_url = "http://localhost:8080";

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let issuer_url =
        IssuerUrl::new("https://accounts.google.com".to_string()).unwrap_or_else(|err| {
            handle_error(&err, "Invalid issuer URL");
            unreachable!();
        });

    let provider_metadata = GoogleProviderMetadata::discover_async(issuer_url, &http_client)
        .await?
        .set_scopes_supported(Some(vec![
            Scope::new("openid".to_string()),
            Scope::new("email".to_string()),
            Scope::new("profile".to_string()),
        ]))
        .set_claims_supported(Some(vec![
            // Providers may also define an enum instead of using CoreClaimName.
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
        ]));

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(google_client_id),
        Some(ClientSecret::new(google_client_secret)),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080".to_string()).unwrap_or_else(|err| {
            handle_error(&err, "Invalid redirect URL");
            unreachable!();
        }),
    )
    .set_revocation_url(
        RevocationUrl::new(revocation_endpoint).unwrap_or_else(|err| {
            handle_error(&err, "Invalid revocation endpoint URL");
            unreachable!();
        }),
    );

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        // Set the desired scopes.
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    println!("Browse to: {}", auth_url);

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_state`.

    // Now you can exchange it for an access token and ID token.

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

        let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                html_success_response.len(),
                html_success_response
            );

        stream.write_all(response.as_bytes()).unwrap();
        (code, state)
    };

    let token_response = client
        .exchange_code(code)
        // This needs to be done before exchange_code returns the Result
        .unwrap_or_else(|err| {
            handle_error(&err, "No user info endpoint");
            unreachable!();
        })
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to contact token endpoint");
            unreachable!();
        });

    let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
    let id_token_claims: &CoreIdTokenClaims = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token")
        .claims(&id_token_verifier, &nonce)
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to verify ID token");
            unreachable!();
        });

    println!("{:?}", id_token_claims);

    // Revoke the obtained token
    let token_to_revoke: CoreRevocableToken = match token_response.refresh_token() {
        Some(token) => token.into(),
        None => token_response.access_token().into(),
    };

    let userinfo: CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)?
        .request_async(&http_client)
        .await
        .map_err(|err| anyhow!("Failed requesting user info: {}", err))?;

    let GI = create_tp_user(userinfo).await.unwrap();

    client
        .revoke_token(token_to_revoke)
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to revoke token");
            unreachable!();
        })
        .request_async(&http_client)
        .await
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to revoke token");
            unreachable!();
        });

    Ok(())
}

use serde::de::DeserializeOwned;
use std::fmt::{Debug, Display, Formatter};

//Create a custom Token Reciever to correctly parse the 'expires_in' value
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AzureTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    access_token: AccessToken,
    #[serde(bound = "TT: TokenType")]
    #[serde(deserialize_with = "helpers::deserialize_untagged_enum_case_insensitive")]
    token_type: TT,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<RefreshToken>,
    #[serde(rename = "scope")]
    #[serde(deserialize_with = "helpers::deserialize_space_delimited_vec")]
    #[serde(serialize_with = "helpers::serialize_space_delimited_vec")]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    scopes: Option<Vec<Scope>>,
    #[serde(bound = "EF: ExtraTokenFields")]
    #[serde(flatten)]
    extra_fields: EF,
}
use std::time::Duration;

impl<EF, TT> AzureTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    pub fn new(access_token: AccessToken, token_type: TT, extra_fields: EF) -> Self {
        Self {
            access_token,
            token_type,
            expires_in: None,
            refresh_token: None,
            scopes: None,
            extra_fields,
        }
    }

    pub fn set_access_token(&mut self, access_token: AccessToken) {
        self.access_token = access_token;
    }

    pub fn set_token_type(&mut self, token_type: TT) {
        self.token_type = token_type;
    }

    pub fn set_expires_in(&mut self, expires_in: Option<&Duration>) {
        self.expires_in = expires_in.map(|duration| (duration.as_secs() as u64).to_string());
    }

    pub fn set_refresh_token(&mut self, refresh_token: Option<RefreshToken>) {
        self.refresh_token = refresh_token;
    }

    pub fn set_scopes(&mut self, scopes: Option<Vec<Scope>>) {
        self.scopes = scopes;
    }

    pub fn extra_fields(&self) -> &EF {
        &self.extra_fields
    }

    pub fn set_extra_fields(&mut self, extra_fields: EF) {
        self.extra_fields = extra_fields;
    }
}

impl<EF, TT> OAuth2TokenResponse for AzureTokenResponse<EF, TT>
where
    EF: ExtraTokenFields,
    TT: TokenType,
{
    type TokenType = TT; // Add this line to define the associated type

    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    fn token_type(&self) -> &TT {
        &self.token_type
    }

    fn expires_in(&self) -> Option<Duration> {
        self.expires_in.as_ref().map(|dur| {
            let secs = dur.as_str().parse::<u64>();
            let secs_success = match secs {
                Ok(value) => value,
                Err(error) => {
                    println!("Unsuccessfully parsed string to u64: {:?}", error);
                    0
                }
            };
            Duration::new(secs_success, 0)
        })
    }

    fn refresh_token(&self) -> Option<&RefreshToken> {
        self.refresh_token.as_ref()
    }

    fn scopes(&self) -> Option<&Vec<Scope>> {
        self.scopes.as_ref()
    }
}

impl<AC, EF, GC, JE, JS, TT> TokenResponse<AC, GC, JE, JS>
    for AzureTokenResponse<IdTokenFields<AC, EF, GC, JE, JS>, TT>
where
    AC: AdditionalClaims,
    EF: ExtraTokenFields,
    GC: GenderClaim,
    JE: JweContentEncryptionAlgorithm<KeyType = JS::KeyType>,
    JS: JwsSigningAlgorithm,
    TT: TokenType,
{
    fn id_token(&self) -> Option<&openidconnect::IdToken<AC, GC, JE, JS>> {
        self.extra_fields.id_token()
    }
}

pub type AzureCoreTokenResponse = AzureTokenResponse<CoreIdTokenFields, CoreTokenType>;

pub type AzureCoreClient<
    HasAuthUrl = EndpointNotSet,
    HasDeviceAuthUrl = EndpointNotSet,
    HasIntrospectionUrl = EndpointNotSet,
    HasRevocationUrl = EndpointNotSet,
    HasTokenUrl = EndpointNotSet,
    HasUserInfoUrl = EndpointNotSet,
> = Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    AzureCoreTokenResponse,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    HasAuthUrl,
    HasDeviceAuthUrl,
    HasIntrospectionUrl,
    HasRevocationUrl,
    HasTokenUrl,
    HasUserInfoUrl,
>;

//pub type AzureCoreToken = ;

#[tokio::test]
async fn ms_auth() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let ms_client_id = std::env::var("MS_CLIENT_ID")?;
    let ms_client_secret = std::env::var("MS_CLIENT_SECRET")?;
    let ms_tenant_id = std::env::var("MS_TENANT_ID")?;
    let ms_redirect_url = "http://localhost:8080";

    let authority_url = format!("https://sts.windows.net/{}/", ms_tenant_id);

    let issuer_url = IssuerUrl::new(authority_url).unwrap_or_else(|err| {
        handle_error(&err, "Invalid issuer URL");
        unreachable!();
    });

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, &http_client)
        .await?
        .set_scopes_supported(Some(vec![
            Scope::new("openid".to_string()),
            Scope::new("email".to_string()),
            Scope::new("profile".to_string()),
        ]))
        .set_claims_supported(Some(vec![
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
        ]));

    // Use CoreClient instead of AzureCoreClient for now
    // We'll need to handle the custom token response in a different way
    let client = AzureCoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(ms_client_id),
        Some(ClientSecret::new(ms_client_secret)),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080".to_string()).unwrap_or_else(|err| {
            handle_error(&err, "Invalid redirect URL");
            unreachable!();
        }),
    );

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token, nonce) = client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    println!("Browse to: {}", auth_url);

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

        let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                html_success_response.len(),
                html_success_response
            );

        stream.write_all(response.as_bytes()).unwrap();
        (code, state)
    };

    println!(
        "Microsoft returned the following code:\n{}\n",
        code.secret()
    );
    println!(
        "Microsoft returned the following state:\n{} (expected `{}`)\n",
        state.secret(),
        csrf_token.secret()
    );

    // Exchange the authorization code for a token
    let token_response = client
        .exchange_code(code)
        .unwrap_or_else(|err| {
            handle_error(&err, "No user info endpoint");
            unreachable!();
        })
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client)
        .await
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to contact token endpoint");
            unreachable!();
        });

    println!(
        "Microsoft returned access token:\n{}\n",
        token_response.access_token().secret()
    );
    println!("Microsoft returned scopes: {:?}", token_response.scopes());

    let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
    let id_token_claims: &CoreIdTokenClaims = token_response
        .extra_fields()
        .id_token()
        .expect("Server did not return an ID token")
        .claims(&id_token_verifier, &nonce)
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to verify ID token");
            unreachable!();
        });

    println!("{:?}", id_token_claims);

    // Get user info
    let userinfo: CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)?
        .request_async(&http_client)
        .await
        .map_err(|err| anyhow!("Failed requesting user info: {}", err))?;

    // Create user from userinfo
    let ms_user = create_tp_user(userinfo).await.unwrap();

    Ok(())
}

//#[tokio::test]
async fn github_auth() -> Result<(), anyhow::Error> {
    Ok(())
}

/*async fn generate_username() -> String {
    "Hi".to_string()
}*/
use std::cell::RefCell;
async fn create_tp_user(user_info: CoreUserInfoClaims) -> Result<(String), anyhow::Error> {
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

    let google_user = ThirdPartyAuthInfo {
        first_name,
        last_name,
        user_id,
        email,
        username,
        profile_url,
    };
    //println!("LAST NAME: {:?}", last_name);
    //println!("PICTURE URL: {}", picture_url);

    (return Ok("".to_string()));
}

async fn find_user() {}

#[tokio::test]
async fn tp_tester_function() {
    let provider = "GOOGLE";
}
