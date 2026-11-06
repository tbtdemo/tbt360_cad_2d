//! Opt-in MCP tool access profiles. External hosts must still authenticate
//! users; this is a server-side tool allowlist, not tenant authorization.
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessMode { Legacy, ReadOnly, Reviewed }

impl AccessMode {
    pub fn from_setting(value:&str)->Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "readonly"|"read-only" => Self::ReadOnly,
            "reviewed"|"reviewed-write" => Self::Reviewed,
            _ => Self::Legacy,
        }
    }
    pub fn as_str(self)->&'static str {
        match self {Self::Legacy=>"legacy",Self::ReadOnly=>"readonly",Self::Reviewed=>"reviewed"}
    }
    pub fn allowed(self,tool:&str,args:&Value)->Result<&'static str,&'static str> {
        if self==Self::Legacy {return Ok("legacy compatibility mode");}
        // Safety is enforced by the server BEFORE tool dispatch, not by the
        // advisory ai_cad_preflight capability.
        const READ_TOOLS:&[&str]=&["ai_cad_capabilities","ai_cad_preflight","list_commands",
            "inspect_drawing","query_entities","render","ui_inspect"];
        if READ_TOOLS.contains(&tool){return Ok("read-only MCP tool");}
        if self==Self::ReadOnly {return Err("MCP is read-only. Set CAD360_MCP_ACCESS_MODE=reviewed for reviewed AI edits.");}
        match tool {
            "ai_draw_plan_vietnamese"|"ai_draw_vietnamese" => {
                if args.get("preview").and_then(Value::as_bool).unwrap_or(true) {
                    return Ok("AI preview only");
                }
                if args.get("confirm")==Some(&Value::Bool(true))
                    && ["expectedDocumentUid","expectedDocumentEpoch","expectedRevision"]
                        .iter().all(|key|args.get(*key).and_then(Value::as_u64).is_some()) {
                    Ok("reviewed AI commit; engine must verify drawing ownership")
                } else {Err("AI write requires explicit confirm=true and document UID, epoch and revision.")}
            }
            _ => Err("MCP tool is outside reviewed allowlist; use an explicitly approved host workflow."),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostRole { Viewer, Editor, Approver }

#[derive(Clone, Debug)]
pub struct HostScope {
    /// Trusted host bootstrap only. Unverified JSON-RPC arguments cannot grant this scope.
    tenant_id: String,
    user_id: String,
    role: HostRole,
    document_uid: u64,
}
impl HostScope {
    pub fn new(tenant_id:&str,user_id:&str,role:&str,document_uid:u64)->Result<Self,&'static str> {
        let valid=|value:&str| !value.is_empty()&&value.len()<=128&&value.bytes().all(|x|x.is_ascii_alphanumeric()||b"-_:.".contains(&x));
        if !valid(tenant_id)||!valid(user_id){return Err("MCP host scope requires trusted nonempty tenant/user identifiers");}
        let role=match role.to_ascii_lowercase().as_str() {
            "viewer"=>HostRole::Viewer,"editor"=>HostRole::Editor,"approver"=>HostRole::Approver,
            _=>return Err("Unknown host role; deny by default")
        };
        Ok(Self{tenant_id:tenant_id.into(),user_id:user_id.into(),role,document_uid})
    }
    pub fn document_uid(&self)->u64 {self.document_uid}
    pub fn tenant_id(&self)->&str {&self.tenant_id}
    pub fn user_id(&self)->&str {&self.user_id}
    pub fn role(&self)->&HostRole {&self.role}
    pub fn allowed(&self,tool:&str,args:&Value)->Result<&'static str,&'static str> {
        let checked=AccessMode::Reviewed.allowed(tool,args)?;
        if self.role==HostRole::Viewer&&
            !["ai_cad_capabilities","ai_cad_preflight","list_commands","inspect_drawing","query_entities","render","ui_inspect"].contains(&tool) {
            return Err("Host role viewer cannot perform AI drawing or preview actions");
        }
        if !matches!(self.role,HostRole::Viewer|HostRole::Editor|HostRole::Approver) {
            return Err("Unknown role denied");
        }
        if args.get("preview").and_then(Value::as_bool)==Some(false) {
            if args.get("expectedDocumentUid").and_then(Value::as_u64)!=Some(self.document_uid) {
                return Err("Host-scoped drawing UID mismatch");
            }
        }
        Ok(checked)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;


    #[test]
    fn stage245_scoped_host_enforces_tenant_user_role_and_drawing_uid() {
        let bad=HostScope::new("tenant-1","user-1","owner",7);
        assert!(bad.is_err(),"unrecognized role must never become Owner implicitly");
        let editor=HostScope::new("tenant-1","user-1","editor",7).unwrap();
        let viewer=HostScope::new("tenant-1","user-2","viewer",7).unwrap();
        assert_eq!(editor.tenant_id(),"tenant-1");
        assert_eq!(editor.user_id(),"user-1");
        assert_eq!(*viewer.role(),HostRole::Viewer);
        assert!(viewer.allowed("inspect_drawing",&json!({})).is_ok());
        assert!(viewer.allowed("ai_draw_plan_vietnamese",&json!({})).is_err());
        assert!(editor.allowed("ai_draw_plan_vietnamese",&json!({})).is_ok());
        assert!(editor.allowed("script",&json!({})).is_err());
        let valid=json!({"preview":false,"confirm":true,"expectedDocumentUid":7,"expectedDocumentEpoch":1,"expectedRevision":6});
        assert!(editor.allowed("ai_draw_plan_vietnamese",&valid).is_ok());
        let forged=json!({"preview":false,"confirm":true,"expectedDocumentUid":8,"expectedDocumentEpoch":1,"expectedRevision":6});
        assert!(editor.allowed("ai_draw_plan_vietnamese",&forged).is_err());
        assert!(HostScope::new("","user","editor",7).is_err());
        assert!(HostScope::new("tenant","../../etc/passwd","editor",7).is_err());
    }
    #[test]
    fn stage244_policy_profiles_enforce_read_only_and_reviewed_write_without_backend_calls() {
        let empty=json!({});
        assert!(AccessMode::ReadOnly.allowed("inspect_drawing",&empty).is_ok());
        assert!(AccessMode::ReadOnly.allowed("ai_draw_plan_vietnamese",&empty).is_err());
        assert!(AccessMode::Reviewed.allowed("script",&empty).is_err());
        assert!(AccessMode::Reviewed.allowed("execute",&empty).is_err());
        assert!(AccessMode::Reviewed.allowed("ai_draw_plan_vietnamese",&empty).is_ok());
        assert!(AccessMode::Reviewed.allowed("ai_draw_plan_vietnamese",&json!({"preview":false,"confirm":true})).is_err());
        assert!(AccessMode::Reviewed.allowed("ai_draw_plan_vietnamese",
            &json!({"preview":false,"confirm":true,"expectedDocumentUid":1,"expectedDocumentEpoch":2,"expectedRevision":3})).is_ok());
        assert_eq!(AccessMode::from_setting("readonly"),AccessMode::ReadOnly);
        assert_eq!(AccessMode::from_setting("reviewed"),AccessMode::Reviewed);
        assert_eq!(AccessMode::from_setting("legacy"),AccessMode::Legacy);
    }
}
