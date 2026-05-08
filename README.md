# rs-admin

一个自用的笔记记录后台，主要用来保存 Shell 片段、执行脚本、SQL 数据源、SQL 查询、字典和一些小工具。这个项目是从原来的 Go + Vue 简化迁移到 Rust + Vue 的版本，去掉了复杂权限等个人使用不需要的功能。

## 技术栈

- 后端：Rust、Axum、SQLx、SQLite
- 前端：Vue 3、Vite、Element Plus、CodeMirror
- 数据库：SQLite，复用原 gin-vue-admin 简化项目的表结构和数据

## 功能

- Shell 笔记
- 执行脚本
- SQL 数据源
- SQL 查询
- SQL 格式化
- Cron 表达式工具
- HTML 转 Markdown
- 字典管理
- 用户管理
- 操作历史
- 清除软删除数据

## 环境要求

- Rust toolchain
- Node.js 和 npm
- SQLite 数据库文件

后端默认读取 `.env`，也可以直接使用系统环境变量。

## 配置

常用环境变量：

```env
DATABASE_URL=sqlite:./rs-admin.db
LISTEN=0.0.0.0:8080
JWT_SECRET=rs-admin-dev-change-me-in-production
STATIC_DIR=web/dist
```

说明：

- `DATABASE_URL`：SQLite 数据库路径。默认是应用目录下的 `rs-admin.db`；开发时通常是项目根目录，安装成服务后会回落到 exe 目录。
- `LISTEN`：后端监听地址，默认 `0.0.0.0:8080`。
- `JWT_SECRET`：登录 token 签名密钥，个人本地用默认值也可以，部署到机器上建议改掉。
- `STATIC_DIR`：前端静态文件目录，默认 `web/dist`，同样按应用目录解析。

如果使用已有 Go 项目的数据库，只要把 `DATABASE_URL` 指向原来的 `sqlite.db` 即可。当前项目会自动创建 `exec_script` 表，其他表依赖原数据库已有结构。

也可以用命令行参数覆盖监听和数据库：

```powershell
cargo run -- --listen 0.0.0.0:8080 --database ./rs-admin.db
```

支持的参数：

- `--listen` 或 `-l`
- `--database`、`--db` 或 `-d`
- `--worker-threads` 或 `--threads`：Tokio worker 线程数，默认 `4`，最小值 `1`。
- `--thread-stack-size` 或 `--stack-size`：Tokio worker 线程栈大小，默认 `1m`，支持纯字节数、`k/kb/kib`、`m/mb/mib`，范围 `64k` 到 `16m`。

例如测试不同线程和栈配置：

```powershell
cargo run --release -- --worker-threads 4 --thread-stack-size 1m
cargo run --release -- --threads 8 --stack-size 512k
```

## 开发

开发时前后端分开跑。

终端 A：项目根目录启动后端：

```powershell
cargo run
```

默认绑定：

```text
0.0.0.0:8080
```

浏览器里一般访问 `http://127.0.0.1:8080`，或者这台机器的局域网 IP。

终端 B：启动前端：

```powershell
cd web
npm install
npm run dev
```

Vite 默认地址：

```text
http://localhost:5173
```

开发模式下，`web/vite.config.ts` 会把 `/api` 和 `/health` 代理到 `http://127.0.0.1:8080`，所以浏览器打开 Vite 提示的地址即可。

## 部署

这个项目是个人笔记项目，部署不需要 nginx、pm2 或复杂网关。推荐直接用后端单端口服务前端静态文件。

1. 构建前端：

```powershell
cd web
npm install
npm run build
cd ..
```

构建结果会生成到：

```text
web/dist
```

2. 启动后端：

```powershell
cargo run
```

浏览器访问：

```text
http://127.0.0.1:8080
```

后端会自动读取 `web/dist` 并提供前端页面。前端路由例如 `/shell`、`/datasource`、`/tools/html2md` 会 fallback 到 `index.html`，刷新页面也能正常打开。

如果要换端口：

```powershell
cargo run -- --listen 0.0.0.0:8090
```

如果静态目录不在默认位置：

```powershell
$env:STATIC_DIR="D:\path\to\dist"
cargo run
```

如果数据库不在默认位置：

```powershell
cargo run -- --database D:\path\to\sqlite.db
```

## Windows Service

如果是在 Windows 上长期后台运行，建议直接安装成 Windows Service。

推荐的部署目录结构：

```text
D:\apps\rs-admin\
  rs_admin.exe
  rs-admin.db
  logs\
  web\
    dist\
```

把 `target\release\rs_admin.exe` 复制到这个目录，再把前端构建产物 `web\dist` 放到 `D:\apps\rs-admin\web\dist`，数据库文件放到 `D:\apps\rs-admin\rs-admin.db`。

先构建 release：

```powershell
cargo build --release
```

然后进入部署目录安装服务：

```powershell
cd D:\apps\rs-admin
.\rs_admin.exe service install
```

启动、停止和卸载：

```powershell
.\rs_admin.exe service start
.\rs_admin.exe service stop
.\rs_admin.exe service uninstall
```

如果安装时带了参数，服务会保留这些参数。例如：

```powershell
.\rs_admin.exe service install --listen 0.0.0.0:8080 --database D:\apps\rs-admin\rs-admin.db
```

这套方式下，更新应用的流程就是：

1. 停止服务
2. 替换 `D:\apps\rs-admin\rs_admin.exe`
3. 再启动服务

## 发布二进制

如果不想在部署机器上每次 `cargo run`，可以先构建 release：

```powershell
cargo build --release
```

然后在项目根目录运行：

```powershell
.\target\release\rs_admin.exe
```

注意：默认 `STATIC_DIR=web/dist` 会按应用目录解析。开发时在项目根目录运行，部署时把 `web/dist` 和 `rs_admin.exe` 放在同一套目录结构下即可。

## 常用命令

后端检查：

```powershell
cargo check
```

前端构建：

```powershell
cd web
npm run build
```

前端开发服务：

```powershell
cd web
npm run dev
```

## 登录和用户

登录用户来自 SQLite 数据库里的 `sys_users` 表。如果复用旧项目数据库，使用旧项目已有用户登录即可。

登录后可以在用户管理里新增用户、修改用户信息或重置密码。

## 备注

- 这是个人使用的工具，不建议直接暴露到公网。
- 如果需要外网访问，至少修改 `JWT_SECRET`，并放在可信网络或反向代理后面。
- SQLite 数据库文件就是核心数据，部署前后注意备份。
