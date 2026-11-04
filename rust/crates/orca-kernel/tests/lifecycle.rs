use async_trait::async_trait;
use orca_kernel::{BoxError, Context, Host, KernelError, Manifest, Plugin};
use serde_json::json;
use std::sync::{Arc, Mutex};

type Log = Arc<Mutex<Vec<String>>>;

struct Probe {
    id: &'static str,
    requires: &'static [&'static str],
    fail: bool,
    log: Log,
}

fn probe(id: &'static str, requires: &'static [&'static str], log: &Log) -> Probe {
    Probe { id, requires, fail: false, log: log.clone() }
}

#[async_trait]
impl Plugin for Probe {
    fn manifest(&self) -> Manifest {
        self.requires.iter().fold(Manifest::new(self.id, "0"), |m, r| m.requires(r))
    }
    async fn start(&self, _: &Context) -> Result<(), BoxError> {
        if self.fail {
            return Err("boom".into());
        }
        self.log.lock().unwrap().push(format!("start {}", self.id));
        Ok(())
    }
    async fn stop(&self, _: &Context) -> Result<(), BoxError> {
        self.log.lock().unwrap().push(format!("stop {}", self.id));
        Ok(())
    }
}

#[tokio::test]
async fn starts_dependencies_first_and_stops_in_reverse() {
    let log = Log::default();
    let mut host = Host::builder()
        .plugin(probe("c", &["b"], &log))
        .plugin(probe("b", &["a"], &log))
        .plugin(probe("a", &[], &log))
        .build()
        .unwrap();
    host.start().await.unwrap();
    host.shutdown().await;
    assert_eq!(
        *log.lock().unwrap(),
        ["start a", "start b", "start c", "stop c", "stop b", "stop a"]
    );
}

#[tokio::test]
async fn failed_start_rolls_back_started_plugins() {
    let log = Log::default();
    let mut bad = probe("b", &["a"], &log);
    bad.fail = true;
    let mut host = Host::builder().plugin(probe("a", &[], &log)).plugin(bad).build().unwrap();
    assert!(matches!(host.start().await, Err(KernelError::Start { .. })));
    assert_eq!(*log.lock().unwrap(), ["start a", "stop a"]);
}

#[test]
fn rejects_missing_dependency_and_cycles() {
    let log = Log::default();
    assert!(matches!(
        Host::builder().plugin(probe("a", &["ghost"], &log)).build(),
        Err(KernelError::MissingDependency { .. })
    ));
    assert!(matches!(
        Host::builder().plugin(probe("a", &["b"], &log)).plugin(probe("b", &["a"], &log)).build(),
        Err(KernelError::DependencyCycle(_))
    ));
}

struct Svc;
struct Provider;
struct Consumer(&'static [&'static str]);

#[async_trait]
impl Plugin for Provider {
    fn manifest(&self) -> Manifest {
        Manifest::new("provider", "0")
    }
    async fn start(&self, cx: &Context) -> Result<(), BoxError> {
        cx.provide_service(Arc::new(Svc));
        cx.register_command("ping", |_| async { Ok(json!("pong")) })?;
        Ok(())
    }
}

#[async_trait]
impl Plugin for Consumer {
    fn manifest(&self) -> Manifest {
        self.0.iter().fold(Manifest::new("zconsumer", "0"), |m, r| m.requires(r))
    }
    async fn start(&self, cx: &Context) -> Result<(), BoxError> {
        cx.service::<Svc>()?;
        Ok(())
    }
}

#[tokio::test]
async fn services_are_visible_only_to_declared_dependents() {
    let mut ok = Host::builder().plugin(Provider).plugin(Consumer(&["provider"])).build().unwrap();
    ok.start().await.unwrap();
    assert_eq!(ok.invoke("provider.ping", json!(null)).await.unwrap(), json!("pong"));
    ok.shutdown().await;
    assert!(matches!(ok.invoke("provider.ping", json!(null)).await, Err(KernelError::CommandNotFound(_))));

    let mut denied = Host::builder().plugin(Provider).plugin(Consumer(&[])).build().unwrap();
    let err = denied.start().await.unwrap_err().to_string();
    assert!(err.contains("may not access service"), "{err}");
}
