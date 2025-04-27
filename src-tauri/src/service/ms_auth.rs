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

#[derive(Clone, Debug, Deserialize, Serialize)]
use serde::de::DeserializeOwned;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

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
use crate::tp_auth::*;

async fn ms_auth() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let client_creds = create_client("MS").await?;
    let authority_url = format!(
        "https://sts.windows.net/{}/",
        client_creds.tenant_id.unwrap()
    );

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let provider_metadata =
        CoreProviderMetadata::discover_async(client_creds.issuer_url, &http_client).await?;

    // Use CoreClient instead of AzureCoreClient for now
    // We'll need to handle the custom token response in a different way
    let client = AzureCoreClient::from_provider_metadata(
        provider_metadata,
        client_creds.client_id,
        Some(client_creds.client_secret),
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

    let (code, state) = auth_code("MS");

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
    let user_info: CoreUserInfoClaims = client
        .user_info(token_response.access_token().to_owned(), None)?
        .request_async(&http_client)
        .await
        .map_err(|err| anyhow!("Failed requesting user info: {}", err))?;

    sign_in_or_sign_up(user_info, pool);

    Ok(())
}
