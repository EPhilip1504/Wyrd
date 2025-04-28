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

#[path = "../service/third_party_auth.rs"]
mod tp_auth;
use tp_auth::*;

async fn google_auth() -> Result<(), anyhow::Error> {
    //const redirect_url: &str = "http://localhost:8080";
    //let google_additional_md = GoogleProviderMetadata::additional_metadata();
    //let cm = CoreProviderMetadata::new(issuer, authorization_endpoint, jwks_uri, response_types_supported, subject_types_supported, id_token_signing_alg_values_supported, additional_metadata)

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap_or_else(|err| {
            handle_error(&err, "Failed to build HTTP client");
            unreachable!();
        });

    let client_creds = create_client("GOOGLE").await?;

    let provider_metadata =
        GoogleProviderMetadata::discover_async(client_creds.issuer_url, &http_client)
            .await?
            .set_scopes_supported(client_creds.scopes)
            .set_claims_supported(client_creds.claims);

    let revocation_endpoint = provider_metadata
        .additional_metadata()
        .revocation_endpoint
        .clone();

    let client = CoreClient::from_provider_metadata(
        provider_metadata,
        client_creds.client_id,
        Some(client_creds.client_secret),
    )
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

    let (code, state) = auth_code("GOOGLE");

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

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    google_auth().await?;
    Ok(())
}
