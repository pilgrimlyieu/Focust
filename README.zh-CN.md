# Focust

<div align="center">

<!-- ![Focust Logo](docs/images/logo.png) -->

**现代化、跨平台的休息提醒应用**

[![MIT 许可证](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2.9-blue?logo=tauri)](https://tauri.app/)
[![Vue 3](https://img.shields.io/badge/Vue-3.5-green?logo=vue.js)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)](https://www.rust-lang.org/)

通过可自定义的时间表、精美的主题和智能提醒，定期休息、保护眼睛、保持健康的工作习惯。

[快速开始](#-快速开始) • [功能特性](#-功能特性) • [安装](#-安装) • [从源码构建](#-从源码构建) • [贡献](#-贡献) • [文档](#-文档)

**[简体中文](README.zh-CN.md)** | **[English](README.md)**

</div>

---

> [!WARNING]
>
> 文档中的图片尚未添加，我会尽快更新。

> [!CAUTION]
> **早期开发阶段提示**
> 
> Focust 目前处于活跃的早期开发和快速迭代阶段。这是我的第一个 Rust 项目，我在开发中不断学习。您可能会遇到 bug、不完整的功能或破坏性更改。非常感谢您的耐心、反馈和贡献！🙏
>
> **平台支持**：虽然 Focust 被设计为跨平台应用（Windows、macOS、Linux），但目前仅在 **Windows** 上进行了测试。如果您使用 macOS 或 Linux，请试用并报告您遇到的任何问题。特别欢迎平台特定的贡献！

---

## 📋 快速开始

查看 [QUICKSTART.md](docs/QUICKSTART.zh-CN.md) 获取详细的入门指南。

## ✨ 功能特性

### 🕐 智能休息调度

- **灵活的休息类型**：配置短休息（20 秒短暂暂停）和长休息（5 分钟休息时段）
- **可自定义间隔**：设置您自己的休息频率 — 每 20 分钟短休息，4 次短休息后长休息
- **基于时间的时间表**：为工作时间创建不同的时间表，限制在工作日休息，或设置自定义时间范围
- **智能检测**：当您离开电脑时自动暂停休息
- **推迟支持**：需要多 5 分钟？使用全局快捷键或按钮点击推迟休息

![休息时间表设置](docs/images/screenshot-break-schedules.png)
*配置多个具有自定义时间范围和活动日期的休息计划*

### 🎨 精美且可自定义的主题

- **丰富的背景**：从纯色、单张图片或文件夹中的随机图片中选择
- **完全自定义**：调整文本颜色、模糊效果、不透明度、字体大小和字体系列
- **视觉反馈**：以可配置的大小（50%-100% 或全屏）查看完整的休息窗口
- **多显示器支持**：同时在所有显示器上显示休息提醒

![Break Window](docs/images/screenshot-break-window.png)
*具有可自定义主题和建议的沉浸式休息窗口*

### 🔔 定时提醒（注意系统）

- **闹钟式提醒**：设置特定时间来提醒自己重要任务
- **非侵入性**：与休息提醒不同，这些不会阻止您的工作 — 只是温和的通知，可以立刻结束
- **多个时间**：在一天中添加所需数量的提醒时间
- **完美适用于**：喝水、服药、站立或任何时间敏感的任务

![定时提醒](docs/images/screenshot-attentions.png)
*为健康习惯设置定时提醒*

### 💡 休息建议

- **激励性消息**：在休息期间获得有用的建议（伸展运动、眼部运动、补水提醒）
- **多语言支持**：内置英文和中文建议，易于自定义
- **基于 TOML 的配置**：添加您自己的自定义建议列表
- **随机选择**：每次休息都有新鲜的建议，保持趣味性

![建议设置](docs/images/screenshot-suggestions.png)
*自定义休息建议或使用内置精选列表*

### 🔊 音频通知

- **内置声音**：从 4 个精心挑选的通知声音中选择
  - 温和的铃声
  - 柔和的锣声
  - 明亮的通知
  - 简单的通知
- **自定义音频**：使用您自己的 MP3、WAV、OGG 或 FLAC 文件
- **音量控制**：根据您的喜好调整音频音量
- **预览**：在保存之前测试声音

### ⚙️ 高级功能

- **系统托盘集成**：最小化到托盘、快速暂停/恢复、状态指示器
- **全局快捷键**：无需切换窗口即可推迟休息
- **严格模式**：在您真正需要时强制休息（无法跳过）
- **休息通知**：在休息开始前获得通知（可配置提前时间）
- **自动暂停**：检测系统空闲时间并暂停调度器
- **勿扰模式检测**（待实现）：计划中的功能，用于检测勿扰模式
- **主题模式**：浅色、深色或基于系统的设置界面主题
- **详细日志**：用于故障排除的调试日志

![通用设置](docs/images/screenshot-general.png)
*具有语言、主题和行为选项的通用设置*

### 🌍 国际化

- 目前支持：英语 (en-US)、简体中文 (zh-CN)
- 基于系统区域设置的自动语言检测
- 易于添加新语言（欢迎贡献！）

### ⚡ 性能与效率

Focust 以性能为设计理念，提供卓越的资源效率：

- **超低内存占用**：后台运行时仅使用约 5MB，得益于动态窗口创建和延迟加载的 Vue 组件
- **原生性能**：由 Tauri 的轻量级 WebView 提供支持，而不是捆绑的 Chromium
- **快速启动**：冷启动时间不到 2 秒
- **最小 CPU 使用**：空闲时 <1% CPU，休息窗口期间 <5%
- **小二进制大小**：比基于 Electron 的替代方案小得多

*非常适合希望获得休息提醒而不牺牲系统资源的用户。*

---

## 📥 安装

### 下载预构建二进制文件

> [!WARNING]
> 由于项目仍处于早期开发阶段，预构建二进制文件尚不可用。目前请[从源码构建](#从源码构建)。

一旦可用，请为您的平台下载最新版本：

- **Windows**：`.msi` 安装程序
- **macOS**：`.dmg` 安装程序
- **Linux**：`.AppImage`

### 从源码构建

请参阅下面的[从源码构建](#-从源码构建)部分获取详细说明。

---

## 🚀 从源码构建

### 前置要求

在开始之前，请确保已安装以下内容：

1. **Node.js**（v18 或更高版本）或 **Bun**（推荐）
2. **Rust**（最新稳定版） - 通过 [rustup](https://rustup.rs/) 安装
3. **系统依赖项**（特定于平台）：

   **Windows:**
   - [Microsoft Visual C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - [WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)（通常在 Windows 10/11 上预装）

   **macOS:**
   ```bash
   xcode-select --install
   ```

   **Linux (Debian/Ubuntu):**
   ```bash
   sudo apt update
   sudo apt install libwebkit2gtk-4.1-dev \
     build-essential \
     curl \
     wget \
     file \
     libxdo-dev \
     libssl-dev \
     libayatana-appindicator3-dev \
     librsvg2-dev
   ```

4. **Just**（可选但推荐）- 命令运行器
   ```bash
   cargo install just
   ```

### 克隆和设置

```bash
# 克隆仓库
git clone https://github.com/pilgrimlyieu/Focust.git
cd Focust

# 安装依赖项
bun install  # 或：npm install / yarn install

# 设置 Rust 依赖项
cd src-tauri
cargo build
cd ..
```

或使用 Just:
```bash
just setup
```

### 开发

```bash
# 启动带有热重载的开发服务器
bun run tauri dev  # 或：npm run tauri dev

# 使用 Just
just dev
```

### 生产构建

```bash
# 构建优化的生产包
bun run tauri build  # 或：npm run tauri build

# 使用 Just。将进行更新器签名构建（需要设置私钥）
just build
```

构建的应用程序将位于 `src-tauri/target/release/bundle/` 中。

> **注意**：对于更新器签名的发布，请参阅[更新器签名指南](docs/UPDATER_SIGNING.md)以设置签名密钥。

### 可用命令（Just）

如果您安装了 [Just](https://github.com/casey/just)，可以使用这些便捷命令：

```bash
just setup          # 设置项目环境
just dev            # 启动开发服务器
just build          # 构建生产包
just build-debug    # 构建调试包

# 代码质量
just format         # 格式化所有代码（Rust + TypeScript）
just lint           # 运行所有 linter
just check          # 类型检查和静态分析
just test-all       # 运行所有测试

# 特定于平台
just format-front   # 仅格式化前端代码
just format-back    # 仅格式化后端代码
just lint-front     # 仅检查前端
just lint-back      # 仅检查后端

# 查看所有可用命令
just
```

---

## 🤝 贡献

欢迎贡献！无论是错误报告、功能请求、文档改进还是代码贡献 — 所有帮助都受到赞赏。

### 如何贡献

1. **报告问题**：发现错误？[提交问题](https://github.com/pilgrimlyieu/Focust/issues/new)
2. **建议功能**：有想法？[发起讨论](https://github.com/pilgrimlyieu/Focust/discussions)
3. **改进文档**：发现拼写错误或不清楚的说明？[提交 PR](https://github.com/pilgrimlyieu/Focust/pulls)
4. **编写代码**：查看 [CONTRIBUTING.md](CONTRIBUTING.md) 获取详细的开发指南

### 需要帮助的领域

- 🐧 **Linux 测试与支持**：帮助测试和修复 Linux 特定问题
- 🍎 **macOS 测试与支持**：验证 macOS 上的功能
- 🌍 **翻译**：添加新语言支持（日语、德语、法语、西班牙语等）
- 🎨 **UI/UX 改进**：设计建议和实现
- 🐛 **Bug 查找**：查找和报告问题
- 📝 **文档**：改进指南并添加教程

### 开发资源

- [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南和开发设置
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) - 项目架构和代码结构
- [docs/CONFIGURATION.md](docs/CONFIGURATION.md) - 配置文件参考

---

## 📖 文档

- **[架构指南](docs/ARCHITECTURE.md)** - 详细的项目架构、前端/后端设计和核心逻辑
- **[配置参考](docs/CONFIGURATION.zh-CN.md)** - 所有配置选项的完整指南
- **[贡献指南](CONTRIBUTING.md)** - 如何贡献、编码标准和开发工作流程
- **[快速开始](docs/QUICKSTART.zh-CN.md)** - 入门指南

---

## 🛠️ 技术栈

### 前端

- **Vue 3.5** - 使用 Composition API 的渐进式 JavaScript 框架
- **TypeScript 5.9** - 类型安全的 JavaScript
- **Pinia 3** - 直观的状态管理
- **Tailwind CSS 4** - 实用优先的 CSS 框架
- **DaisyUI 5** - Tailwind 的组件库
- **Vue I18n 11** - 国际化
- **Vite 7** - 快如闪电的构建工具
- **Vitest 4** - 单元测试框架

### 后端
- **Rust (2024 edition)** - 系统编程语言
- **Tauri 2** - 跨平台桌面框架
- **Tokio** - 异步运行时
- **Serde** - 序列化框架
- **ts-rs** - 从 Rust 生成 TypeScript 类型

### 工具
- **Just** - 命令运行器
- **Biome** - 代码格式化器和 linter
- **Cargo** - Rust 包管理器

---

## 🗺️ 路线图

- [ ] 项目图标
- [ ] 改进的错误处理和用户反馈
- [ ] 全面的平台测试（macOS、Linux）
- [ ] 白名单应用（v0.2.0 主要目标）

---

## 📄 许可证

本项目根据 MIT 许可证授权 - 有关详细信息，请参阅 [LICENSE](LICENSE) 文件。

---

## 🙏 致谢

- 灵感来自 [Stretchly](https://github.com/hovancik/stretchly) - 出色的休息提醒应用
- 使用 [Tauri](https://tauri.app/) 构建 - 用于构建桌面应用的绝佳框架
- 图标来自各种开源项目
- 音频文件来自免版税来源

<div align="center">

**如果您觉得 Focust 有帮助，请考虑在 GitHub 上给它一个 ⭐️！**

</div>
