use crate::app_status::StatusResult;
use simple_status::Renderer;

pub struct StatusResultRender;

impl Renderer<StatusResult> for StatusResultRender {
    type Output = String;

    fn render(&self, status_result: &StatusResult) -> String {
        match status_result {
            StatusResult::Success(msg) => match msg.is_empty() {
                true => "[SUCCESS]".to_owned(),
                false => format!("[SUCCESS] {}", msg),
            },
            StatusResult::Error(err) => {
                let mut out = String::new();

                out.push_str(err.kind().as_str());

                if let Some(summary) = err.summary() {
                    out.push_str(": ");
                    out.push_str(summary);
                }

                if let Some(reason) = err.reason() {
                    out.push('\n');
                    out.push_str(reason);
                }

                out
            }
        }
    }
}

impl std::fmt::Display for StatusResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&StatusResultRender.render(self))
    }
}
