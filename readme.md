# Terminalz

一个基于 Tauri 和 Russh 的跨平台 SSH 远程终端客户端。

## 项目结构
```bash
terminalz/
├── Cargo.toml                    # workspace 根配置
├── package.json                  # 前端依赖
├── vite.config.js                # Vite 构建配置
├── index.html                    # 入口 HTML
├── src/                          # Vue3 前端
│   ├── main.js                   # Vue 入口
│   ├── App.vue                   # 根组件
│   └── components/
│       └── Terminal.vue          # 终端组件（连接表单 + xterm.js）
└── src-tauri/                    # Tauri + Rust 后端
    ├── Cargo.toml                # Rust 依赖
    ├── tauri.conf.json           # Tauri 配置
    ├── capabilities/default.json # 权限配置
    └── src/
        ├── main.rs               # Rust 入口
        ├── lib.rs                # Tauri 命令注册
        ├── ssh.rs                # SSH 模块（russh + tokio）
        └── db.rs                 # SQLite 数据库模块

```

## 运行
```bash
# 开发模式（启动 Tauri + Vite 热更新）
npx tauri dev
npm run tauri dev
cargo tauri dev

# 构建生产版本
npx tauri build
```

## 清理
```bash
# 1. 清理前端（dist、vite缓存）
npm run clean

# 2. 清理 Tauri + Rust 编译产物（最关键）
npm run tauri clean

# 3. 彻底清理 Rust 整个 target 目录（最大的缓存）
cargo clean
```

## 许可证

本项目采用**双重许可**模式：

### 开源版本

本项目的源代码以 **GNU General Public License v3.0 only（GPL-3.0-only）** 授权。你可以自由地使用、修改和分发本项目，但必须遵守 GPL-3.0-only 的条款，包括将修改后的版本同样以 GPL-3.0-only 开源。

详见 [LICENSE](./LICENSE) 文件。

### 商业版本

如需在闭源或专有产品中使用本项目，或需要商业支持，请联系项目维护者获取商业许可。

商业许可将授予你：
- 在闭源产品中集成和使用本项目的权利
- 不受 GPL 传染性条款的约束
- 优先技术支持和维护服务

联系方式：[项目维护者邮箱]

### 贡献者许可

向本项目提交代码、文档或其他贡献，即表示你同意 [贡献者许可协议（CLA）](./CLA.md) 的全部条款。通过 CLA，你授权项目维护者以 GPL-3.0-only 和商业许可两种模式使用你的贡献。
