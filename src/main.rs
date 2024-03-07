mod mail;

use axum::{
    response::IntoResponse,
    routing::{get, post},
};
use std::io::Write;

#[tokio::main]
async fn main() {
    let tcp_listen = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    let handler = || async { "Hello World 👋" };
    let app = axum::Router::new()
        .route("/", get(handler))
        .route(
            "/webhook/project-a",
            post(|| async { handle_webhook("project-a").await }),
        )
        .route(
            "/webhook/project-b",
            post(|| async { handle_webhook("project-b").await }),
        );
    axum::serve(tcp_listen, app).await.unwrap()
}

async fn handle_webhook(project_name: &str) -> impl axum::response::IntoResponse {
    let script_path = if let Some(val) = get_script_path(project_name) {
        val
    } else {
        return (axum::http::StatusCode::BAD_REQUEST, "无法获取部署脚本").into_response();
    };
    let project_name = project_name.to_string();
    tokio::spawn(async move {
        deploy(&project_name, &script_path).await;
    });

    (axum::http::StatusCode::OK, "开始部署项目...").into_response()
}

// 获取项目的部署脚本路径
fn get_script_path(project_name: &str) -> Option<String> {
    let cwd = std::env::current_dir().unwrap().display().to_string();
    match project_name {
        "project-a" => Some(cwd + "/scripts/deploy.sh"),
        "project-b" => Some(cwd + "/scripts/deploy.zx.mjs"),
        _ => None,
    }
}

async fn deploy(project_name: &str, script_path: &str) {
    // 每次记录日志时，都带上当前时间
    fn write_log(owner: &mut String, content: &str) {
        let time = chrono::Local::now().format("%H:%M:%S").to_string();
        owner.push_str(format!("⏰ {}: {}\n", time, content).as_str())
    }

    let mut log = String::new();
    let executer = if script_path.ends_with(".zx.mjs") {
        // 请确保已经安装了 zx 依赖
        "zx"
    } else {
        "sh"
    };
    let output = std::process::Command::new(executer)
        .arg(script_path)
        .output()
        .expect("❌ 脚本执行失败");
    let mut email_payload = mail::EmailPayload {
        subject: Some("Deploy Failed".into()),
        content: "".into(),
    };
    let deploy_success = output.status.success();
    // 将脚本的输出写入变量 log
    write_log(&mut log, "构建输出");
    if deploy_success {
        write_log(&mut log, &String::from_utf8_lossy(&output.stdout));
        email_payload.subject = Some(format!("✅ 应用[{}]部署成功", project_name));
    } else {
        write_log(&mut log, &String::from_utf8_lossy(&output.stderr));
        email_payload.subject = Some(format!("❌ 应用[{}]部署失败", project_name));
    };
    write_log(
        &mut log,
        &format!(
            "构建结果: {res}",
            res = if deploy_success { "成功" } else { "失败" },
        ),
    );
    email_payload.content = log.replace("\n", "<br>");
    // 发送邮件
    // if let Err(e) = mail::send_email_to_myself(email_payload).await {
    //     write_log(&mut log, &format!("邮件发送失败: {}", e.to_string()));
    // } else {
    //     write_log(&mut log, "邮件已成功发送");
    // }
    // 将日志输出到本地文件
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("./log.txt")
        .unwrap();
    file.write_all(format!("\n{}\n", log).as_bytes())
        .expect("本地文件写入失败");
}

// 控制台中运行 `$ cargo test auto_deploy_tests` 自动执行测试用例
// 如果环境、依赖没什么异常的话，测试用例应该可以通过
#[cfg(test)]
mod auto_deploy_tests {
    use crate::handle_webhook;
    use axum::response::IntoResponse;

    #[test]
    fn cwd() {
        let cwd = std::env::current_dir().unwrap().display().to_string();
        println!("{}", cwd);
    }

    #[test]
    fn deploy_project_a() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = handle_webhook("project-b").await;
            println!("{:?}", res.into_response().status());
        });
    }

    #[test]
    fn deploy_project_b() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let res = handle_webhook("project-b").await;
            println!("{:?}", res.into_response().status());
        });
    }
}
