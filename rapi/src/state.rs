use crate::openid::OidConf;

#[derive(Clone)]
pub struct AppState<'a> {
    pub oidc: OidConf<'a>,
    pub validation: jsonwebtoken::Validation,
}
