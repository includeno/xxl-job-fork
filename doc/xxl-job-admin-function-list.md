# XXL-Job Admin 功能列表

本文整理 `xxl-job-admin` 模块在 Spring Boot 实现中的核心功能点，按业务域划分列出所涉及的页面/API、关键能力以及关联的数据表，便于在其他技术栈中实现功能对等的管理端。

## 1. 账户与权限

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 用户登录与登出 | 基于账号/密码的登录，支持登出、记住登录状态 | `login`、`logout` | `xxl_job_user` |
| 用户管理 | 管理调度中心用户，维护角色（管理员/普通用户）、执行器权限列表 | `/jobuser/index` 页面，`JobUserController` 中的增删改查接口 | `xxl_job_user` |
| 会话校验 | 通过 SSO Token 校验接口权限，拦截未登录或权限不足的请求 | `XxlSsoHelper` 相关过滤器 | `xxl_job_user` |

## 2. 执行器管理

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 执行器列表分页 | 根据 AppName、名称模糊查询执行器分组并分页展示 | `/jobgroup/pageList` | `xxl_job_group` |
| 新建/编辑执行器 | 支持自动注册地址模式与手动录入模式，保存执行器信息 | `/jobgroup/save`、`/jobgroup/update` | `xxl_job_group`、`xxl_job_registry` |
| 删除执行器 | 校验执行器下是否存在任务、是否唯一后允许删除 | `/jobgroup/remove` | `xxl_job_group`、`xxl_job_info` |
| 加载执行器详情 | 按 ID 获取执行器配置 | `/jobgroup/loadById` | `xxl_job_group` |

## 3. 调度任务管理

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 任务列表分页 | 支持按分组、状态、描述、处理器、负责人等条件分页查询任务 | `/jobinfo/pageList` | `xxl_job_info` |
| 新建/编辑任务 | 维护调度配置、路由、阻塞策略、GLUE、超时和子任务等信息 | `/jobinfo/add`、`/jobinfo/update` | `xxl_job_info` |
| 删除任务 | 在校验权限后删除任务信息 | `/jobinfo/remove` | `xxl_job_info` |
| 启停任务 | 将任务调度状态切换为运行/停止，维护下一次调度时间 | `/jobinfo/start`、`/jobinfo/stop` | `xxl_job_info` |
| 手动触发任务 | 支持传递执行参数、指定地址触发一次执行 | `/jobinfo/trigger` | `xxl_job_info`、`xxl_job_log` |
| 计算下次调度时间 | 根据调度类型与配置预览未来五次触发时间 | `/jobinfo/nextTriggerTime` | —— |

## 4. 日志与报表

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 日志分页查询 | 支持按执行器、任务、状态、时间范围过滤调度日志 | `/joblog/pageList` | `xxl_job_log` |
| 日志详情查看 | 展示调度结果、执行日志内容，支持分页拉取执行器日志 | `/joblog/logDetailPage`、`/joblog/logDetailCat` | `xxl_job_log` |
| 强制终止任务 | 通过执行器 API 请求终止正在运行的日志 | `/joblog/logKill` | `xxl_job_log` |
| 清理日志 | 根据条件批量删除历史日志记录 | `/joblog/clearLog` | `xxl_job_log` |
| 调度报表 | 统计每日运行中、成功、失败的任务数量，用于仪表盘展示 | `/chartInfo` 等接口 | `xxl_job_log_report`、`xxl_job_log` |

## 5. GLUE 脚本管理

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 在线编辑 GLUE | 在 Web IDE 中编辑 GLUE 脚本，保存到任务上 | `/jobcode/save` | `xxl_job_info` |
| GLUE 历史版本 | 记录每次发布的脚本版本，支持回溯 | `/jobcode/glueVersion` 等接口 | `xxl_job_logglue` |

## 6. 注册中心与执行器通信

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 执行器注册发现 | 执行器按 AppName 注册到调度中心，支持自动发现 | `RegistryController`、`XxlJobRegistryMapper` | `xxl_job_registry` |
| OpenAPI 接口 | 暴露触发任务、取消任务等 API 供第三方调用 | `/openapi/*` | 多表 |

## 7. 系统配置

| 功能 | 说明 | 相关接口/页面 | 主要数据表 |
| --- | --- | --- | --- |
| 首页仪表盘 | 展示执行器数量、任务数量、日志统计等全局概览 | `/`、`IndexController.chartInfo` | `xxl_job_group`、`xxl_job_info`、`xxl_job_log`、`xxl_job_log_report` |
| 国际化支持 | 通过 `I18nUtil` 实现多语言文案 | N/A | —— |

以上功能构成 `xxl-job-admin` 管理端的核心能力，后续的 Rust 版本实现需保证提供等价的功能点、使用同一套 `xxl_job` MySQL 数据库，并维持相同的数据模型约束。
