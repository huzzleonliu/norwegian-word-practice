# Norwegian Word Practice

**Norwegian Word Practice（NWP）** 是一款在浏览器中使用的挪威语词汇练习工具，面向希望巩固拼写与词形变化的学习者。

它是作者 Huzz 个人艺术创作系列 **Once and Once Again（OAOA）** 下的子项目：核心是可配置的挪威语练习体验；与 OAOA 相关的链上 / NFT 能力可作为可选扩展存在。

> English: A browser-based Norwegian vocabulary practice app for spelling and inflection drills. It is part of the art project *Once and Once Again* (OAOA), with optional Solana/NFT-related capabilities.

仓库：<https://github.com/huzzleonliu/norwegian-word-practice>

---

## 软件能做什么

### 词库练习与记忆

- 从内置词库或自行导入的加密词库（`.nwpdict`）中选题。
- 可按中文、英文、词性、标签等条件筛选，针对性地练习。
- **拼写练习**：配置提示字段与回答字段；支持「基础 / 常用 / 全面」回答预设，也可自定义勾选变位项；可临时显示参考答案。
- **记忆对照**：同一套选题与字段配置，侧重对照记忆而非判题节奏。
- 目标值为占位符 `NULL`（全大写）的字段不会作为作答项。

### 系列练习

固定主题、按标签组织，适合集中攻克一类词：

| 系列 | 大致内容 |
| --- | --- |
| 数词 | 基数词与序数词分块练习 |
| 月份 | 月份相关词汇 |
| 代词 | 代词相关词形 |
| 疑问词 | 疑问词相关词汇 |
| 国家 | 按国家分块：国名、语言、形容词、国人等；块标题随界面语言切换中/英国家名 |

### 跟读

- 打开内置音频书（如 Byen Er Bergen），或从本地导入音频与歌词。
- 按文件名匹配 `.lrc` / `.srt`；同一时间戳的两行会显示为原文与译文。
- 点击歌词跳转、调速、循环当前句。

### 本地词库编辑

- 浏览、搜索词条；配置显示列（含拖拽排序）。
- 单条或批量新增；查词辅助（词典 / 翻译 / 模型等分流）。
- 导入、导出加密词库；支持明文 CSV 与 `.nwpdict` 之间的加解密转换（编辑器内面板）。

### 练习总结与多语言界面

- 查看本轮与累计正确率，回顾错题，便于查漏补缺。
- 界面提供中文与英文；地址使用语言前缀，例如 `/ch/...`、`/en/...`。
- 各页可打开应用内帮助（中英说明）。

---

## 使用方式概要

1. 打开站点后选择界面语言（中文 / 英文）。
2. 进入练习模式：加载内置词库或导入自己的 `.nwpdict`。
3. 选择 **词库练习**（筛选后练习 / 记忆）或 **系列练习**（固定主题）。
4. 需要私人词库时，打开 **本地词库编辑** 维护词条并导出。
5. 练习结束后可在 **总结页** 查看正确率与错题。

内置词库包括通用词库（如 Byen er Bergen 等）以及统一的系列词库；系列按标签过滤，详见 [documents/data-format.md](documents/data-format.md)。

问题反馈：可通过应用内入口前往 Discord，或在 GitHub 提交 Issue。

---

## 文档索引

本 README 只说明「软件是什么、能做什么」。实现细节、开发与部署请看下列文档。

| 文档 | 内容 |
| --- | --- |
| [documents/README.md](documents/README.md) | 文档目录总览 |
| [documents/overview.md](documents/overview.md) | 项目定位与仓库地图 |
| [documents/features.md](documents/features.md) | 功能与页面路径 |
| [documents/architecture.md](documents/architecture.md) | 技术架构与数据流 |
| [documents/development.md](documents/development.md) | 本地运行、测试、Docker、约定 |
| [documents/data-format.md](documents/data-format.md) | 词库格式、标签、加密导出 |
| [documents/license.md](documents/license.md) | 开源许可说明 |
| [CONTRIBUTING.md](CONTRIBUTING.md) | 如何贡献 |
| `crates/nwp-web/docs/help/` | 应用内逐页帮助（中/英，编译期嵌入） |

想自己构建或二次开发时，请从 [documents/development.md](documents/development.md) 与 [CONTRIBUTING.md](CONTRIBUTING.md) 开始。

---

## 贡献与许可

欢迎 Issue 与 Pull Request。提交前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

本项目以 [Apache License 2.0](LICENSE) 发布。
