use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use reqwest::multipart::{Form, Part};
use tokio::sync::OnceCell;
use vendrtk_server::App;

const TEST_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 9999);
const TEST_PUBLIC_DIR: &str = "file";
static CLIENT: OnceCell<TestClient> = OnceCell::const_new();

pub struct TestClient {
    base_url: String,
    http: reqwest::Client,
    _server: tokio::task::JoinHandle<()>,
}

impl TestClient {
    pub async fn get() -> &'static Self {
        CLIENT.get_or_init(init).await
    }

    pub async fn upload_file(
        &self,
        file_bytes: &[u8],
        filename: &str,
    ) -> reqwest::Result<reqwest::Response> {
        let form = Form::new().part(
            "file",
            Part::bytes(file_bytes.to_vec()).file_name(filename.to_string()),
        );

        self.http
            .post(format!("{}/api/files/upload", self.base_url))
            .multipart(form)
            .send()
            .await
    }
}

async fn init() -> TestClient {
    let app = App::build(TEST_ADDR, TEST_PUBLIC_DIR)
        .await
        .expect("failed to build test server");

    let base_url = format!("http://{TEST_ADDR}");

    let server = tokio::spawn(async move {
        app.run().await.expect("test server failed");
    });

    TestClient {
        base_url,
        http: reqwest::Client::new(),
        _server: server,
    }
}
