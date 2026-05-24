use anyhow::Result;
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// IPC Bus system - internal messaging between components
pub struct IpcBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<mpsc::UnboundedSender<IpcMessage>>>>>,
}

#[derive(Debug, Clone)]
pub enum IpcMessage {
    ProcessEvent { pid: i32, status: i32 },
    ContainerStart { id: String },
    ContainerStop { id: String },
    FilesystemEvent { path: String, kind: String },
    NetworkEvent { interface: String, event: String },
    UserNotification { title: String, body: String },
    Shutdown,
}

impl IpcBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, topic: &str) -> mpsc::UnboundedReceiver<IpcMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut subs = self.subscribers.lock().await;
        subs.entry(topic.to_string()).or_default().push(tx);
        rx
    }

    pub async fn publish(&self, topic: &str, msg: IpcMessage) {
        let subs = self.subscribers.lock().await;
        if let Some(txs) = subs.get(topic) {
            for tx in txs {
                let _ = tx.send(msg.clone());
            }
        }
    }
}

/// Virtual socket (vsock) communication for AVF mode
pub mod protocol {
    use anyhow::Result;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct VsockMessage {
        pub message_type: VsockMessageType,
        pub payload: Vec<u8>,
        pub timestamp: u64,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub enum VsockMessageType {
        Command,
        CommandOutput,
        FileTransfer,
        Signal,
        WindowResize,
        Clipboard,
        Audio,
        Shutdown,
        Heartbeat,
    }

    /// AIDL-like RPC over vsock for AVF
    pub mod aidl_rpc {
        use anyhow::Result;

        pub struct RpcSession {
            cid: u32,
            port: u32,
        }

        impl RpcSession {
            pub fn new(cid: u32, port: u32) -> Self {
                Self { cid, port }
            }

            pub async fn connect(&self) -> Result<()> {
                // Connect to AVF VM via vsock
                Ok(())
            }

            pub async fn call(&self, method: &str, args: &[u8]) -> Result<Vec<u8>> {
                // Make RPC call over vsock
                Ok(vec![])
            }
        }
    }
}
