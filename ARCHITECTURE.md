# 一、任务系统架构

### 核心思想：All Are Task

**任务类型**

- 主要任务
  - 阻塞式：全局唯一同类任务
    - 可排队（带锁任务）
    - 不可排队（锁定任务，不允许创建新任务）
  - 并发式：全局可并行同类任务
    - 无限制并发
    - 限制并发
- 子任务：主要任务的唯一组成
  - 顺序队列，支持并行分组（可选并发度/聚合策略）
  - 支持条件执行与重试
  - 失败导致任务链失败

**核心特性**

- 优先级提高与取消
- 实例锁（资源锁定）
- 异步执行器 + 锁管理器 + 任务注册表

**典型任务**

- 账户登录（阻塞式，不可排队）
- 版本下载（并发式，限制并发）
- 游戏启动（阻塞式，可排队）

---

# 二、应用架构设计

### 1. 数据持久化策略

**登录凭据存储**

- 平台 keyring（Windows: Credential Manager, Linux: Secret Service, macOS: Keychain）
- 存储：OAuth token、refresh_token（密钥需安全存储）

**启动器设置**

- 格式：YAML
- 位置：`{用户配置目录}/hako/config.yaml`
- 内容：主题、默认行为、窗口状态、全局游戏设置

**游戏实例设置**

- 位置：`.minecraft/versions/{版本名称}/Hako/settings.yaml`
- 策略：版本扫描，无需索引
- 读取：启动时扫描 `.minecraft/versions/` 目录

**缓存管理**

- 位置：系统 TMP 目录
- 命名：`hako_cache_{hash}`
- 清理：应用退出时可选清理

### 2. 配置系统

**配置层级**

```
游戏实例设置（最高优先级）
    ↓ 覆盖
启动器默认全局设置（包含全局游戏设置）
```

**配置加载**

1. 启动时加载启动器设置（包含全局游戏设置）
2. 扫描 `.minecraft/versions/` 发现可用版本
3. 编辑页面按需读取对应版本的实例设置文件

**配置热重载**

- 仅启动器设置支持热重载
- 游戏设置：编辑时读取 → 修改 → 即时写入

### 3. 资源管理

- 编译时嵌入（`include_bytes!` 或 `rust-embed`）
- 资源：图标、主题文件、默认配置

### 4. 网络层设计

**通用 HTTP 客户端**

- API 请求（OAuth 认证、版本列表等）
- 特性：重试、超时、代理支持

**下载 HTTP 客户端**

- 文件下载（游戏文件、资源包等）
- 特性：多线程下载、断点续传、进度追踪、并发控制

### 5. 安全性设计

- OAuth 认证，不存储密码
- 平台 keyring 存储 token（密钥与密码同等重要）
- 配置文件格式验证
- 下载文件哈希校验（SHA-1/SHA-256）
- TLS 证书验证

---

# 三、项目结构

```
Hako/
├── Cargo.toml
├── README.md
├── resources/              # 编译时嵌入的资源
│   ├── icons/
│   ├── themes/
│   └── defaults/
│       └── launcher.yaml
├── src/
│   ├── main.rs            # 应用入口
│   ├── core/              # 应用核心
│   ├── task/              # 任务系统
│   ├── storage/           # 数据持久化
│   ├── net/           # 网络层
│   │   ├── api.rs         # 通用 HTTP 客户端
│   │   ├── download.rs    # 下载客户端
│   │   └── error.rs       # 网络错误
│   ├── platform/          # 平台抽象
│   │   ├── paths.rs       # 路径管理（跨平台）
│   │   └── keyring.rs     # keyring 抽象
│   ├── account/           # 账户系统
│   │   ├── auth.rs        # 认证逻辑
│   │   ├── providers.rs   # 登录提供商（微软、Mojang等）
│   │   └── models.rs      # 账户模型
│   ├── game/              # 游戏管理
│   │   ├── instance.rs    # 实例管理（版本扫描）
│   │   ├── version.rs     # 版本管理
│   │   ├── launcher.rs    # 游戏启动
│   │   └── config.rs      # 游戏配置（读取/写入实例设置）
│   └── ui/                # GPUI 界面
│       ├── app.rs         # 主应用视图
│       ├── views/         # 各个视图
│       │   ├── home.rs
│       │   ├── instances.rs
│       │   ├── settings.rs
│       │   └── account.rs
│       └── components/    # 可复用组件
│           ├── task_list.rs
│           └── progress.rs
```

---

# 四、代码风格

### 原则

- 紧凑高效易读，充分利用 rust 2024 特性，在必要处采用最现代、最短、最高性能写法
- 逻辑树简短，避免冗长 elif 等非必要逻辑
- 命名简短标准
- 注释只用于吐槽/笔记，不解释代码，依靠代码自解释

### 命名规范

- 函数/变量：`snake_case`
- 类型/模块：`PascalCase`
- 常量：`UPPER_SNAKE_CASE`
- 私有模块：`_internal` 或 `mod.rs` 内部

### 代码组织

- 优先使用 `match` 而非长 `if-else`
- 使用 `?` 操作符简化错误处理
- 避免深层嵌套（>3 层）
- 函数保持简短（<50 行，特殊情况 <100 行）

---

# 五、关键设计要点

1. **最小化影响原则**：所有数据在用户目录，删除即卸载
2. **版本扫描策略**：扫描 `.minecraft/versions` 自动发现实例，无需维护索引
3. **配置层级简化**：启动器设置包含全局游戏设置，实例设置覆盖全局设置
4. **按需读取**：游戏设置仅在编辑时读取，修改后即时写入
5. **任务系统整合**：task 模块内包含具体任务实现，结构更紧凑
6. **依赖管理**：按需安装，使用最新版本

---

# 六、依赖清单（按需安装，可以使用其他现代化包替代）

只在用上的时候通过 cargo add 安装，并在这之后编辑 toml 修改所需 features

```toml
# UI
gpui = "0.2.2"

# 异步运行时
tokio = { version = "1", features = ["full"] }

# 网络
reqwest = { version = "0.11", features = ["json", "multipart"] }

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"

# 凭据存储
keyring = "2.0"

# 路径处理
dirs = "5"

# 错误处理
thiserror = "1"
anyhow = "1"

# 工具
uuid = { version = "1", features = ["v4"] }
sha2 = "0.10"
```
