#![allow(unused)]
#![allow(warnings)]
use anyhow::anyhow;
#[cfg(test)]
use do_username::do_username;
use env_logger::fmt::Timestamp;
use log::log;
use oauth2::basic::{
    BasicClient, BasicErrorResponse, BasicErrorResponseType, BasicRevocationErrorResponse,
    BasicTokenIntrospectionResponse, BasicTokenType,
};
use oauth2::{
    helpers, AsyncHttpClient, AuthUrl, CodeTokenRequest, ConfigurationError, EmptyExtraTokenFields,
    EndpointNotSet, EndpointState, ErrorResponse, PkceCodeVerifier, RevocableToken,
    RevocationRequest, StandardErrorResponse, StandardRevocableToken, StandardTokenResponse,
    TokenIntrospectionResponse, TokenUrl,
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

use crate::tp_auth::*;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

pub async fn github_auth() -> Result<(), anyhow::Error> {
    //const redirect_url: &str = "http://localhost:8080";
    //let google_additional_md = GoogleProviderMetadata::additional_metadata();
    //let cm = CoreProviderMetadata::new(issuer, authorization_endpoint, jwks_uri, response_types_supported, subject_types_supported, id_token_signing_alg_values_supported, additional_metadata)
    let client_creds = create_client("GITHUB").await?;
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Github OAuth2 process.
    let client = BasicClient::new(client_creds.client_id)
        .set_client_secret(client_creds.client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        // This example will be running its own server at localhost:8080.
        // See below for the server implementation.
        .set_redirect_uri(
            RedirectUrl::new("http://127.0.0.1:3000/auth/github/callback".to_string())
                .expect("Invalid redirect URL"),
        );

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the user's public repos and email.
        .add_scope(Scope::new("public_repo".to_string()))
        .add_scope(Scope::new("user:email".to_string()))
        .url();

    println!("Open this URL in your browser:\n{authorize_url}\n");

    let (code, state) = auth_code("GITHUB");

    println!("Github returned the following code:\n{}\n", code.secret());
    println!(
        "Github returned the following state:\n{} (expected `{}`)\n",
        state.secret(),
        csrf_state.secret()
    );

    // Exchange the code with a token.
    let token_res = client.exchange_code(code).request_async(&http_client).await;

    println!("Github returned the following token:\n{token_res:?}\n");

    if let Ok(token) = token_res {
        // NB: Github returns a single comma-separated "scope" parameter instead of multiple
        // space-separated scopes. Github-specific clients can parse this scope into
        // multiple scopes by splitting at the commas. Note that it's not safe for the
        // library to do this by default because RFC 6749 allows scopes to contain commas.
        let scopes = if let Some(scopes_vec) = token.scopes() {
            scopes_vec
                .iter()
                .flat_map(|comma_separated| comma_separated.split(','))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        println!("Github returned the following scopes:\n{scopes:?}\n");
    }

    Ok(())
}
