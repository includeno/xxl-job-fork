# rust-admin

`rust-admin` 是对原 `xxl-job-admin` 管理端的 Rust 版本重写，完全复用 `xxl_job` MySQL 表结构，使用 [SeaORM](https://www.sea-ql.org/SeaORM/) + [Axum](https://github.com/tokio-rs/axum) 构建。模块覆盖文档中整理的全部核心功能域，具体对照如下：

| 功能域 | 对应 API | 说明 |
| --- | --- | --- |
| 用户认证与管理 | `POST /api/auth/login`, `POST /api/auth/logout`, `GET/POST/PUT/DELETE /api/job-users` | 登录、登出、账号维护、权限/角色配置 |
| 执行器管理 | `GET/POST/PUT/DELETE /api/job-groups` | 对应执行器分页、创建、编辑、删除及自动注册地址回填 |
| 任务管理 | `GET/POST/PUT/DELETE /api/job-info`, `POST /api/job-info/{id}/start`, `POST /api/job-info/{id}/stop`, `POST /api/job-info/{id}/trigger`, `GET /api/job-info/next-trigger-time` | 任务 CRUD、启停、手动触发、调度时间预览 |
| 调度日志与报表 | `GET /api/job-logs`, `GET /api/job-logs/{id}`, `GET /api/job-logs/{id}/cat`, `POST /api/job-logs/{id}/kill`, `POST /api/job-logs/clear`, `GET /api/dashboard/summary`, `GET /api/dashboard/chart` | 日志分页、详情、终止、清理以及仪表盘统计 |
| GLUE 脚本管理 | `GET/POST /api/job-code/{jobId}`, `GET /api/job-code/{jobId}/versions` | 在线 GLUE 编辑与历史版本列表 |

> 详细功能说明参见仓库中的《[XXL-Job Admin 功能列表](../doc/xxl-job-admin-function-list.md)》。

## 环境要求

- Rust 1.74+（建议使用 `rustup` 安装最新 stable）
- 已初始化的 `xxl_job` MySQL 数据库（可直接执行 `doc/db/tables_xxl_job.sql`）

## 快速开始

1. **配置数据库连接**：

   可以修改 `config/default.toml` 中的 `database.url`，或在启动时通过环境变量覆盖：

   ```bash
   export RUST_ADMIN__DATABASE__URL="mysql://user:pass@localhost:3306/xxl_job"
   ```

2. **运行服务**：

   ```bash
   cd rust-admin
   cargo run
   ```

   默认监听 `0.0.0.0:8080`，同样可使用环境变量 `RUST_ADMIN__SERVER__HOST`、`RUST_ADMIN__SERVER__PORT` 调整。

## 主要技术栈

- **Web 框架**：Axum 0.7
- **ORM**：SeaORM 0.12（启用 MySQL、Chrono 支持）
- **配置**：`config` + `dotenvy`
- **日志与追踪**：`tracing` + `tower-http::trace`
- **其它**：`validator`（请求体验证）、`cron`（CRON 解析）

## 运行效果

所有 API 返回统一 JSON 结构，示例：

```json
{
  "records_total": 2,
  "records_filtered": 2,
  "data": [
    {
      "id": 1,
      "appname": "xxl-job-executor-sample",
      "title": "通用执行器Sample",
      "address_type": 0,
      "address_list": null,
      "update_time": "2024-01-01T00:00:00"
    }
  ]
}
```

API 鉴权通过 `Authorization: Bearer <token>`，token 在登录成功后写入 `xxl_job_user.token` 字段，可直接复用原 Admin 用户表。

## 开发者提示

- `cargo check` 会在 4s 左右完成编译校验。
- 支持 `.env` 文件或环境变量覆盖配置。
- 如需扩展自定义功能，推荐在 `routes/` 下添加模块，并复用 `state::AppState` 中的数据库连接。

