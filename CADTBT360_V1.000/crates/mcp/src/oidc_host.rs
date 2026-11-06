//! OIDC verified-host admission contract. This module DOES NOT parse or validate JWT
//! signatures. A trusted host must verify the access token cryptographically against
//! issuer JWKS (alg/kid/signature), validate issuer, audience, exp, nbf, revocation,
//! and map role/tenant claims before constructing a scoped CAD MCP session.
//! Never accept identity fields directly from JSON-RPC tool arguments.
use crate::access_policy::HostScope;

/// Token verifier implemented in the authenticated outer host (OpenIddict / OAuth2).
/// It must NOT return claims for unsigned, expired or untrusted access tokens.
pub trait VerifiedTokenProvider {
    fn verify_access_token(&self,token:&str)->Result<VerifiedClaims,String>;
}

#[derive(Clone,Debug)]
pub struct VerifiedClaims {
    pub issuer:String,
    pub audience:String,
    pub subject:String,
    pub tenant:String,
    pub role:String,
    pub expires_at_unix:u64,
    pub not_before_unix:u64,
}

pub struct OidcTrustPolicy {
    expected_issuer:String,
    expected_audience:String,
    max_session_seconds:u64,
}
impl OidcTrustPolicy {
    pub fn new(issuer:&str,audience:&str)->Result<Self,&'static str> {
        if !issuer.starts_with("https://")||issuer.len()>512||issuer.contains('#')
            ||audience.is_empty()||audience.len()>200 {
            return Err("OIDC trust policy needs a fixed HTTPS issuer and audience");
        }
        Ok(Self{expected_issuer:issuer.trim_end_matches('/').into(),
            expected_audience:audience.into(),max_session_seconds:900})
    }
    /// Only call with a trusted token verifier. This routine checks verified
    /// claim binding; it cannot replace signature/JWKS validation in the host.
    pub fn admit(&self,verifier:&dyn VerifiedTokenProvider,token:&str,document_uid:u64,
        tools:&[&str])->Result<HostScope,String> {
        if token.is_empty()||token.len()>16384 {
            return Err("Missing or invalid-size OIDC token".into());
        }
        let verified=verifier.verify_access_token(token)?;
        if verified.issuer.trim_end_matches('/')!=self.expected_issuer
            ||verified.audience!=self.expected_audience {
            return Err("Issuer or audience rejected by CAD 360 trust policy".into());
        }
        let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|_|"System clock invalid")?.as_secs();
        if verified.not_before_unix>now || verified.expires_at_unix<=now {
            return Err("OIDC token not currently valid".into());
        }
        let until=verified.expires_at_unix.min(now.saturating_add(self.max_session_seconds));
        HostScope::new(&verified.tenant,&verified.subject,&verified.role,document_uid)
            .map_err(|e|e.to_string())?
            .with_verified_expiry(until).map_err(|e|e.to_string())?
            .restrict_tools(tools).map_err(|e|e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Stub{claims:VerifiedClaims,success:bool}
    impl VerifiedTokenProvider for Stub {
        fn verify_access_token(&self,_:&str)->Result<VerifiedClaims,String> {
            if self.success {Ok(self.claims.clone())}
            else {Err("Signature verification failed".into())}
        }
    }
    fn fixture()->(OidcTrustPolicy,Stub) {
        let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let policy=OidcTrustPolicy::new("https://auth.example.test","cad360-mcp").unwrap();
        let verifier=Stub{claims:VerifiedClaims{
            issuer:"https://auth.example.test".into(),
            audience:"cad360-mcp".into(),subject:"user-123".into(),
            tenant:"tenant-abc".into(),role:"viewer".into(),
            expires_at_unix:now+300,not_before_unix:now-10},success:true};
        (policy,verifier)
    }
    #[test]
    fn stage248_oidc_host_rejects_wrong_audience_issuer_expiry_and_invalid_role() {
        let (policy,mut host)=fixture();
        let ok=policy.admit(&host,"verified-token",42,&["inspect_drawing"]).unwrap();
        assert_eq!(ok.user_id(),"user-123");
        assert_eq!(ok.tenant_id(),"tenant-abc");
        assert_eq!(ok.document_uid(),42);
        assert!(ok.allowed("inspect_drawing",&serde_json::json!({})).is_ok());
        assert!(ok.allowed("ai_draw_plan_vietnamese",&serde_json::json!({})).is_err());
        host.claims.audience="other-app".into();
        assert!(policy.admit(&host,"verified-token",42,&[]).is_err());
        host.claims.audience="cad360-mcp".into();
        host.claims.issuer="https://evil.test".into();
        assert!(policy.admit(&host,"verified-token",42,&[]).is_err());
        host.claims.issuer="https://auth.example.test".into();
        host.claims.expires_at_unix=0;
        assert!(policy.admit(&host,"verified-token",42,&[]).is_err());
        host.claims.expires_at_unix=std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()+100;
        host.claims.role="root".into();
        assert!(policy.admit(&host,"verified-token",42,&[]).is_err());
        host.success=false;
        assert!(policy.admit(&host,"verified-token",42,&[]).is_err());
    }
    #[test]
    fn stage248_oidc_host_never_accepts_unverified_provider_or_unsafe_issuer() {
        let (policy,mut host)=fixture();
        host.success=false;
        assert!(policy.admit(&host,"not-a-real-token",42,&[]).is_err());
        assert!(OidcTrustPolicy::new("http://localhost","cad360-mcp").is_err());
        assert!(OidcTrustPolicy::new("https://auth.example.test","").is_err());
        assert!(policy.admit(&host,"",42,&[]).is_err());
    }
}
