项目结构
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
        └── ssh.rs                # SSH 模块（russh + tokio）

```

# 运行
```bash
# 开发模式（启动 Tauri + Vite 热更新）
npx tauri dev
npm run tauri dev
cargo tauri dev

# 构建生产版本
npx tauri build
```

clean
```bash
# 1. 清理前端（dist、vite缓存）
npm run clean

# 2. 清理 Tauri + Rust 编译产物（最关键）
npm run tauri clean

# 3. 彻底清理 Rust 整个 target 目录（最大的缓存）
cargo clean
```