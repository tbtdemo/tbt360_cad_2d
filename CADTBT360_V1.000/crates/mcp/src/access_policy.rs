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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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
