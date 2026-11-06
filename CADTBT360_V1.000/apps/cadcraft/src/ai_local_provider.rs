//! Optional local LLM adapter for AI Studio. No network credentials or cloud API keys.
 //! Ollama CLI output is untrusted text: it is NEVER executed as a CAD command here.
use std::io::Read as _;
use std::process::{Command,Stdio};
use std::time::{Duration,Instant,SystemTime,UNIX_EPOCH};

const MAX_OUTPUT: u64=16*1024;
const TIMEOUT: Duration=Duration::from_secs(90);

pub fn validate_model(model:&str)->Result<(),String> {
    let model=model.trim();
    if model.is_empty() || model.len()>96 || model.contains("..") || model.starts_with(['/','-']) || !model.bytes().all(|b|b.is_ascii_alphanumeric() || b"._:/-".contains(&b)) {
        return Err("Tên mô hình Ollama chỉ được gồm chữ, số, '.', '_', ':', '/', '-' và tối đa 96 ký tự.".into());
    }
    Ok(())
}

fn safe_input(request:&str)->Result<String,String> {
    let request=request.trim();
    if request.is_empty() || request.chars().count()>1200 || request.chars().any(char::is_control) {
        return Err("Yêu cầu AI cần từ 1 đến 1200 ký tự, không chứa ký tự điều khiển.".into());
    }
    Ok(format!(
        "Bạn là bộ phiên dịch yêu cầu thành một lệnh CAD 2D. CHỈ xuất ra đúng MỘT DÒNG lệnh, không Markdown, không giải thích. \
Chỉ dùng một trong các mẫu sau: \
vẽ phòng rộng 6000 dài 4000 tại 0,0; \
vẽ đường thẳng từ 0,0 đến 100,0; \
vẽ hình chữ nhật từ 0,0 đến 100,50; \
vẽ hình tròn tâm 0,0 bán kính 50; \
vẽ polyline qua 0,0;100,0;100,50; \
vẽ cung tâm 0,0 bán kính 100 từ 0 đến 90; \
vẽ tường từ 0,0 đến 6000,0 dày 200. \
Nếu yêu cầu không tạo ra được một lệnh hợp lệ, trả lời KHONG_HO_TRO. \
Không bao giờ tạo lệnh xóa, ghi file, chạy hệ thống, gọi mạng hoặc thay đổi bảo mật. \
YÊU CẦU NGƯỜI DÙNG: {request}"
    ))
}

/// Call a separately installed local Ollama executable, using arguments (never a shell).
/// The child is terminated at timeout; stdout is stored in a bounded temporary file.
pub fn generate(model:&str,request:&str)->Result<String,String> {
    validate_model(model)?;
    let prompt=safe_input(request)?;
    let stamp=SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e|e.to_string())?.as_nanos();
    let output=std::env::temp_dir().join(format!("cad360_ollama_{}_{}.txt",std::process::id(),stamp));
    let file=std::fs::File::create_new(&output).map_err(|e|format!("Không tạo được tệp kết quả AI: {e}"))?;
    let result=(||{
        let mut child=Command::new("ollama").arg("run").arg(model.trim()).arg(prompt)
            .stdin(Stdio::null()).stdout(Stdio::from(file)).stderr(Stdio::null()).spawn()
            .map_err(|e|format!("Không chạy được Ollama ({e}). Cài Ollama, tải mô hình và thử lại."))?;
        let start=Instant::now();
        let status=loop {
            match child.try_wait() {
                Ok(Some(status))=>break status,
                Ok(None) if start.elapsed()<TIMEOUT=>std::thread::sleep(Duration::from_millis(150)),
                Ok(None)=>{let _=child.kill();let _=child.wait();return Err("AI cục bộ xử lý quá 90 giây, đã dừng tiến trình.".into());},
                Err(e)=>{let _=child.kill();let _=child.wait();return Err(format!("Không theo dõi được Ollama: {e}"));}
            }
        };
        if !status.success() {return Err(format!("Ollama thất bại ({status}). Kiểm tra mô hình đã tải và dịch vụ Ollama đang hoạt động."));}
        let size=std::fs::metadata(&output).map_err(|e|e.to_string())?.len();
        if size==0 || size>MAX_OUTPUT {return Err("Ollama trả về kết quả rỗng hoặc dài quá 16KB.".into());}
        let mut bytes=Vec::new();
        std::fs::File::open(&output).map_err(|e|e.to_string())?.take(MAX_OUTPUT).read_to_end(&mut bytes).map_err(|e|e.to_string())?;
        let response=String::from_utf8(bytes).map_err(|_|"Mô hình không trả về UTF-8 hợp lệ.".to_owned())?;
        let command=response.trim();
        if command.is_empty() || command=="KHONG_HO_TRO" {return Err("Mô hình không chuyển được yêu cầu thành lệnh CAD được hỗ trợ.".into());}
        if command.lines().count()!=1 || command.len()>600 {return Err("Mô hình không trả về đúng một dòng lệnh CAD an toàn.".into());}
        Ok(command.to_owned())
    })();
    let _=std::fs::remove_file(&output);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_id_refuses_shell_injection_and_path_tricks() {
        for model in ["", "foo;rm", "abc&&dir", "../q", "model name", "a\nb"] {
            assert!(validate_model(model).is_err(),"{model}");
        }
        assert!(validate_model("qwen2.5:7b").is_ok());
        assert!(validate_model("llama3.2:latest").is_ok());
    }
    #[test]
    fn local_model_request_is_bounded_and_not_executed() {
        assert!(safe_input("").is_err());
        assert!(safe_input("a\nb").is_err());
        assert!(safe_input(&"a".repeat(1201)).is_err());
        let p=safe_input("vẽ tường dày 200").unwrap();
        assert!(p.contains("YÊU CẦU NGƯỜI DÙNG: vẽ tường dày 200"));
        assert!(p.contains("KHONG_HO_TRO"));
    }
}
