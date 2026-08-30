# 功能说明

下列路径均需加语言前缀：`/ch` 或 `/en`。

## 首页与模式选择

| 路径 | 功能 |
| --- | --- |
| `/` | 项目介绍、主题与语言入口 |
| `/practice` | 选择词库练习 / 系列练习 / 打开编辑器；可加载内置词库或导入 `.nwpdict` |

## 词库模式

| 路径 | 功能 |
| --- | --- |
| `/lexicon` | 在已加载词库中按条件筛选并选题 |
| `/lexicon/practice` | 拼写练习：可配每页题量、提示字段（含词性）、回答字段预设与显隐答案 |
| `/lexicon/remember` | 记忆对照：同样可配字段，侧重浏览与对照 |
| `/lexicon/summary` | 本轮与累计正确率、错题列表；可返回上一练习页 |

**回答字段预设**：基础（仅原型）/ 常用（核心变体）/ 全面（全部可选题态）；可展开自定义勾选。字段若目标值为占位符 `NULL`（全大写）则跳过作答。

## 系列模式

入口：`/series`。统一加载 `series-word-bank.nwpdict`，再按 tag 过滤。

| 路径 | 标签（示意） | 题型要点 |
| --- | --- | --- |
| `/series/number` | `cardinal_number` / `ordinal_number` | 分块填写基数词与序数词 |
| `/series/month` | `month` | 中英提示 → 原型 |
| `/series/pronoun` | `pronoun` | 代词系列专用布局 |
| `/series/interrogative` | `interrogative` | 疑问词系列 |
| `/series/country` | `country` | 按国家分块；块标题随 UI 语言切换中/英国家名；页顶回答预设 |

## 词库编辑

| 路径 | 功能 |
| --- | --- |
| `/editor` | 浏览/搜索词条、列显示与拖拽排序、单条与批量新增、查词分流（Ordbok → 翻译 → 模型兜底）、CSV↔`.nwpdict` 加解密面板、导出加密词库 |

## Solana / NFT（可选）

与 OAOA 系列相关的 Mint、钱包连接、持仓门禁由 `solana-gate` 与 `solana-leptos-component` 提供；具体是否对某页启用取决于产品配置与门禁状态。链上资产与 Sugar 配置见仓库内 `candy_machine/`（通常不进入日常前端构建）。

## 应用内帮助

每页右下角可打开帮助浮层，内容对应 `crates/nwp-web/docs/help/` 下中英文 Markdown。
