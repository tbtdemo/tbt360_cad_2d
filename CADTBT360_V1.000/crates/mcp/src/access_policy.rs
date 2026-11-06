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
            "legacy" => Self::Legacy,
            // Invalid spelling must never silently widen an external agent's rights.
            _ => Self::ReadOnly,
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
    expires_at_unix: u64,
    allowed_tools: Option<Vec<String>>,
    /// Explicit approval supplied by a trusted host after its own human review.
    /// Cannot be set from JSON-RPC tool arguments.
    approved_ai_plan: Option<ApprovedAiPlan>,
}

#[derive(Clone, Debug)]
struct ApprovedAiPlan {
    tool: String,
    text: String,
    document_epoch: u64,
    revision: u64,
}
impl HostScope {
    pub fn new(tenant_id:&str,user_id:&str,role:&str,document_uid:u64)->Result<Self,&'static str> {
        let valid=|value:&str| !value.is_empty()&&value.len()<=128&&value.bytes().all(|x|x.is_ascii_alphanumeric()||b"-_:.".contains(&x));
        if !valid(tenant_id)||!valid(user_id){return Err("MCP host scope requires trusted nonempty tenant/user identifiers");}
        let role=match role.to_ascii_lowercase().as_str() {
            "viewer"=>HostRole::Viewer,"editor"=>HostRole::Editor,"approver"=>HostRole::Approver,
            _=>return Err("Unknown host role; deny by default")
        };
        let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|_|"System clock before UNIX epoch")?.as_secs();
        Ok(Self{tenant_id:tenant_id.into(),user_id:user_id.into(),role,document_uid,
            expires_at_unix:now.saturating_add(900),allowed_tools:None,approved_ai_plan:None})
    }
    /// Trusted host sets the session expiry derived from a previously VERIFIED OIDC
    /// access token; this method does not parse or verify tokens itself.
    pub fn with_verified_expiry(mut self,expires_at_unix:u64)->Result<Self,&'static str> {
        let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|_|"System clock before UNIX epoch")?.as_secs();
        if expires_at_unix<=now || expires_at_unix>now.saturating_add(3600) {
            return Err("OIDC host-scope expiry must be in the next hour");
        }
        self.expires_at_unix=expires_at_unix;
        Ok(self)
    }
    /// Restrict the tools permitted by this trusted host session. This can only
    /// NARROW existing role permissions; it never elevates a Viewer to an Editor.
    pub fn restrict_tools(mut self,tools:&[&str])->Result<Self,&'static str> {
        if tools.len()>64 || tools.iter().any(|t|t.is_empty()||t.len()>80
            || !t.bytes().all(|b|b.is_ascii_alphanumeric()||b==b'_')) {
            return Err("Invalid MCP host tool restriction");
        }
        self.allowed_tools=Some(tools.iter().map(|t|t.to_string()).collect());
        Ok(self)
    }
    /// Trusted host only: bind an already-reviewed AI plan to a specific tool,
    /// exact user-visible text and CAD state. This is not itself a user-consent UI.
    /// The host MUST obtain approval from a real user before calling it.
    /// Advanced options applyLayers/createDoorBlock are deliberately denied on this grant.
    pub fn with_approved_ai_plan(mut self,tool:&str,text:&str,document_epoch:u64,revision:u64)->Result<Self,&'static str> {
        if !["ai_draw_plan_vietnamese","ai_draw_vietnamese"].contains(&tool)
            || text.trim().is_empty() || text.len()>4000 || text.chars().any(char::is_control) {
            return Err("Invalid trusted-host AI approval");
        }
        self.ensure_live()?;
        if self.role==HostRole::Viewer {return Err("Viewer cannot approve AI writing");}
        self.approved_ai_plan=Some(ApprovedAiPlan {
            tool:tool.to_string(),text:text.to_string(),document_epoch,revision
        });
        Ok(self)
    }
    pub fn expires_at_unix(&self)->u64 {self.expires_at_unix}
    pub fn document_uid(&self)->u64 {self.document_uid}
    pub fn tenant_id(&self)->&str {&self.tenant_id}
    pub fn user_id(&self)->&str {&self.user_id}
    pub fn role(&self)->&HostRole {&self.role}
    /// Protects MCP resource reads and discovery, not just tools/call.
    pub fn ensure_live(&self)->Result<(),&'static str> {
        let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .map_err(|_|"Cannot validate MCP scoped session clock")?.as_secs();
        if now>=self.expires_at_unix {Err("MCP scoped session expired; host must re-authenticate")}
        else {Ok(())}
    }
    pub fn resource_allowed(&self,uri:&str)->Result<(),&'static str> {
        self.ensure_live()?;
        match uri {
            "cadcraft://drawing" if self.allowed_tools.as_ref().is_none_or(|items|items.iter().any(|s|s=="inspect_drawing")) => Ok(()),
            "cadcraft://commands" if self.allowed_tools.as_ref().is_none_or(|items|items.iter().any(|s|s=="list_commands")) => Ok(()),
            _ => Err("Resource outside host-scoped MCP grants"),
        }
    }
    pub fn allowed(&self,tool:&str,args:&Value)->Result<&'static str,&'static str> {
        self.ensure_live()?;
        if self.allowed_tools.as_ref().is_some_and(|tools|!tools.iter().any(|t|t==tool)) {
            return Err("Tool is outside host-granted scope");
        }
        let checked=AccessMode::Reviewed.allowed(tool,args)?;
        if self.role==HostRole::Viewer&&
            !["ai_cad_capabilities","ai_cad_preflight","list_commands","inspect_drawing","query_entities","render","ui_inspect"].contains(&tool) {
            return Err("Host role viewer cannot perform AI drawing or preview actions");
        }
        if !matches!(self.role,HostRole::Viewer|HostRole::Editor|HostRole::Approver) {
            return Err("Unknown role denied");
        }
        if args.get("preview").and_then(Value::as_bool)==Some(false) {
            if ["ai_draw_plan_vietnamese","ai_draw_vietnamese"].contains(&tool) {
                let approval=self.approved_ai_plan.as_ref().ok_or("Host has not approved this AI plan")?;
                if approval.tool!=tool || args.get("text").and_then(Value::as_str)!=Some(approval.text.as_str())
                    || args.get("expectedDocumentEpoch").and_then(Value::as_u64)!=Some(approval.document_epoch)
                    || args.get("expectedRevision").and_then(Value::as_u64)!=Some(approval.revision)
                    || args.get("applyLayers")==Some(&Value::Bool(true))
                    || args.get("createDoorBlock")==Some(&Value::Bool(true)) {
                    return Err("AI plan does not match the trusted-host approval");
                }
            }
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
    fn stage257_trusted_host_approval_cannot_be_forged_or_reused_for_changed_text() {
        let text="vẽ hai phòng liền kề rộng 3000 và 4000 dài 4000 tại 0,0";
        let base=HostScope::new("tenant-a","editor-1","editor",42).unwrap();
        let accepted=json!({"text":text,"preview":false,"confirm":true,
            "expectedDocumentUid":42,"expectedDocumentEpoch":8,"expectedRevision":3});
        assert!(base.allowed("ai_draw_plan_vietnamese",&accepted).is_err());
        assert!(base.allowed("ai_draw_plan_vietnamese",&json!({"text":text,"preview":true})).is_ok());
        let approved=base.with_approved_ai_plan("ai_draw_plan_vietnamese",text,8,3).unwrap();
        assert!(approved.allowed("ai_draw_plan_vietnamese",&accepted).is_ok());
        let mut changed=accepted.clone();
        changed["text"]=json!("vẽ hai phòng liền kề rộng 8000 và 4000 dài 4000 tại 0,0");
        assert!(approved.allowed("ai_draw_plan_vietnamese",&changed).is_err());
        changed=accepted.clone();changed["applyLayers"]=json!(true);
        assert!(approved.allowed("ai_draw_plan_vietnamese",&changed).is_err());
        changed=accepted.clone();changed["createDoorBlock"]=json!(true);
        assert!(approved.allowed("ai_draw_plan_vietnamese",&changed).is_err());
        changed=accepted.clone();changed["expectedRevision"]=json!(4);
        assert!(approved.allowed("ai_draw_plan_vietnamese",&changed).is_err());
        changed=accepted.clone();changed["expectedDocumentEpoch"]=json!(9);
        assert!(approved.allowed("ai_draw_plan_vietnamese",&changed).is_err());
        changed=accepted.clone();changed["expectedDocumentUid"]=json!(99);
        assert!(approved.allowed("ai_draw_plan_vietnamese",&changed).is_err());
        assert!(HostScope::new("tenant-a","viewer-1","viewer",42).unwrap()
            .with_approved_ai_plan("ai_draw_plan_vietnamese",text,8,3).is_err());
        assert!(base.with_approved_ai_plan("execute",text,8,3).is_err());
    }

    #[test]
    fn stage247_scoped_resource_permissions_match_tool_grants() {
        let limited=HostScope::new("tenant-a","user-a","viewer",5).unwrap()
            .restrict_tools(&["inspect_drawing"]).unwrap();
        assert!(limited.resource_allowed("cadcraft://drawing").is_ok());
        assert!(limited.resource_allowed("cadcraft://commands").is_err());
        assert!(limited.resource_allowed("file:///etc/passwd").is_err());
        let empty=HostScope::new("tenant-a","user-a","viewer",5).unwrap()
            .restrict_tools(&[]).unwrap();
        assert!(empty.resource_allowed("cadcraft://drawing").is_err());
        assert!(empty.allowed("inspect_drawing",&json!({})).is_err());
    }

    #[test]
    fn stage246_host_scope_expires_and_restricts_tools_without_role_escalation() {
        let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let editor=HostScope::new("tenant-a","user-editor","editor",15).unwrap()
            .with_verified_expiry(now+60).unwrap()
            .restrict_tools(&["inspect_drawing","ai_draw_plan_vietnamese"]).unwrap();
        assert!(editor.allowed("inspect_drawing",&json!({})).is_ok());
        assert!(editor.allowed("list_commands",&json!({})).is_err());
        let viewer=HostScope::new("tenant-a","user-viewer","viewer",15).unwrap()
            .restrict_tools(&["ai_draw_plan_vietnamese","inspect_drawing"]).unwrap();
        assert!(viewer.allowed("inspect_drawing",&json!({})).is_ok());
        assert!(viewer.allowed("ai_draw_plan_vietnamese",&json!({})).is_err());
        assert!(HostScope::new("tenant-a","u","editor",15).unwrap()
            .with_verified_expiry(now.saturating_sub(1)).is_err());
        assert!(HostScope::new("tenant-a","u","editor",15).unwrap()
            .with_verified_expiry(now+7200).is_err());
        assert!(HostScope::new("tenant-a","u","editor",15).unwrap()
            .restrict_tools(&["execute; rm -rf"]).is_err());
    }

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
        assert!(editor.allowed("ai_draw_plan_vietnamese",&valid).is_err());
        let approved=editor.clone().with_approved_ai_plan("ai_draw_plan_vietnamese",
            "vẽ phòng",1,6).unwrap();
        let mut approved_valid=valid.clone();approved_valid["text"]=json!("vẽ phòng");
        assert!(approved.allowed("ai_draw_plan_vietnamese",&approved_valid).is_ok());
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
        assert_eq!(AccessMode::from_setting("unrecognized-mode"),AccessMode::ReadOnly);
    }
}
