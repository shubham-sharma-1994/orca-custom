use orca_plugin_core_config::ConfigStore;
use orca_sdk::prelude::*;
use std::sync::Arc;

pub struct Worktree;

#[derive(Debug, PartialEq)]
pub struct Entry {
    pub path: String,
    pub head: String,
    pub branch: Option<String>,
}

/// Parses `git worktree list --porcelain`.
pub fn parse_porcelain(out: &str) -> Vec<Entry> {
    out.split("\n\n")
        .filter_map(|block| {
            let mut e = Entry { path: String::new(), head: String::new(), branch: None };
            for line in block.lines() {
                if let Some(v) = line.strip_prefix("worktree ") {
                    e.path = v.into();
                } else if let Some(v) = line.strip_prefix("HEAD ") {
                    e.head = v.into();
                } else if let Some(v) = line.strip_prefix("branch ") {
                    e.branch = Some(v.strip_prefix("refs/heads/").unwrap_or(v).into());
                }
            }
            (!e.path.is_empty()).then_some(e)
        })
        .collect()
}

#[async_trait]
impl Plugin for Worktree {
    fn manifest(&self) -> Manifest {
        Manifest::new("worktree", "0.1.0")
            .describe("List and inspect git worktrees")
            .requires("core.config")
    }

    async fn start(&self, cx: &Context) -> Result<(), BoxError> {
        let config: Arc<ConfigStore> = cx.service()?;
        let events = cx.clone();
        cx.register_command("list", move |args| {
            let (config, events) = (config.clone(), events.clone());
            async move {
                let repo = args["repo"]
                    .as_str()
                    .map(String::from)
                    .or_else(|| config.get("worktree.default_repo")?.as_str().map(String::from))
                    .ok_or("no `repo` argument and no `worktree.default_repo` configured")?;
                let out = tokio::process::Command::new("git")
                    .args(["-C", &repo, "worktree", "list", "--porcelain"])
                    .output()
                    .await?;
                if !out.status.success() {
                    return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned().into());
                }
                let list: Vec<Value> = parse_porcelain(&String::from_utf8_lossy(&out.stdout))
                    .into_iter()
                    .map(|e| json!({ "path": e.path, "head": e.head, "branch": e.branch }))
                    .collect();
                events.publish("worktree.listed", json!({ "repo": repo, "count": list.len() }));
                Ok(Value::Array(list))
            }
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_and_detached_entries() {
        let out = "worktree /r\nHEAD abc\nbranch refs/heads/main\n\nworktree /r-wt\nHEAD def\ndetached\n";
        assert_eq!(
            parse_porcelain(out),
            vec![
                Entry { path: "/r".into(), head: "abc".into(), branch: Some("main".into()) },
                Entry { path: "/r-wt".into(), head: "def".into(), branch: None },
            ]
        );
    }
}
