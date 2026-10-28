use url::Url;

use crate::ConfigError;

pub const ACCOUNT_MCP_PATH: &str = "/account/mcp";

/// Explicit deployment identity for the authenticated account resource.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountSettings {
    issuer: String,
    resource: String,
}

impl AccountSettings {
    pub fn new(issuer: &str, resource: &str) -> Result<Self, ConfigError> {
        let issuer = Url::parse(issuer).map_err(|_| ConfigError::InvalidAccountSettings)?;
        let resource = Url::parse(resource).map_err(|_| ConfigError::InvalidAccountSettings)?;
        let local_issuer = matches!(issuer.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
        if !valid_url(&issuer)
            || !(issuer.scheme() == "https" || (issuer.scheme() == "http" && local_issuer))
            || issuer.path() != "/"
            || !valid_url(&resource)
            || resource.scheme() != "https"
            || resource.path() != ACCOUNT_MCP_PATH
        {
            return Err(ConfigError::InvalidAccountSettings);
        }
        Ok(Self {
            issuer: issuer.origin().ascii_serialization(),
            resource: resource.to_string(),
        })
    }

    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    pub fn resource(&self) -> &str {
        &self.resource
    }

    pub fn metadata_url(&self) -> String {
        format!(
            "{}/.well-known/oauth-protected-resource{ACCOUNT_MCP_PATH}",
            self.resource.trim_end_matches(ACCOUNT_MCP_PATH)
        )
    }
}

fn valid_url(url: &Url) -> bool {
    url.as_str().len() <= 2_048
        && url.host_str().is_some_and(|host| !host.contains('*'))
        && url.username().is_empty()
        && url.password().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_identity_is_explicit_and_resource_metadata_is_canonical() {
        let settings = AccountSettings::new(
            "https://api.example.test",
            "https://mcp.example.test/account/mcp",
        )
        .unwrap_or_else(|error| panic!("synthetic account identity: {error}"));
        assert_eq!(settings.issuer(), "https://api.example.test");
        assert_eq!(
            settings.metadata_url(),
            "https://mcp.example.test/.well-known/oauth-protected-resource/account/mcp"
        );
        for issuer in [
            "https://user:secret@api.example.test",
            "https://api.example.test/path",
            "https://api.example.test?token=synthetic",
            "http://api.example.test",
        ] {
            assert!(AccountSettings::new(issuer, settings.resource()).is_err());
        }
        for resource in [
            "http://mcp.example.test/account/mcp",
            "https://mcp.example.test/mcp",
            "https://mcp.example.test/account/mcp?token=synthetic",
            "https://mcp.example.test/account/mcp#fragment",
        ] {
            assert!(AccountSettings::new(settings.issuer(), resource).is_err());
        }
    }

    #[test]
    fn account_access_cannot_be_enabled_on_stdio() {
        let account = AccountSettings::new(
            "https://api.example.test",
            "https://mcp.example.test/account/mcp",
        )
        .unwrap_or_else(|error| panic!("synthetic account identity: {error}"));
        let settings = crate::Settings::new(
            None,
            crate::Transport::Stdio,
            std::time::Duration::from_secs(30),
        )
        .unwrap_or_else(|error| panic!("default settings: {error}"));
        assert_eq!(
            settings.with_account(account),
            Err(ConfigError::AccountRequiresHttp)
        );
        assert!(!crate::ToolPolicy::PublicExplore.allows(crate::ToolCapability::AccountRead));
        assert!(!crate::ToolPolicy::AccountRead.allows(crate::ToolCapability::PublicExplore));
    }
}
