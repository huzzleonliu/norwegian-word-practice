# Norwegian Word Practice 维护文档（readme-gen）

本文档用于帮助新维护者快速理解本项目的结构、调用流程与关键约束。

---

## 1. 项目定位

这是一个基于 **Rust + Leptos (CSR/WASM)** 的挪威语词汇练习工具，支持：

- 词库练习（可选词条、可配置题型）
- 系列练习（数词、月份、代词、疑问词）
- 本地词库编辑（单条新增、批量新增、表格编辑、CSV 导入导出）
- 练习结果导入导出（`.pracresult`）

---

## 2. 技术栈与运行方式

- 前端框架：`leptos`
- 目标平台：`wasm32-unknown-unknown`
- 打包/开发：`trunk`
- 数据文件：CSV（`data/lexicon-word-bank`、`data/series-word-bank`）

常用命令：

```bash
NO_COLOR=true trunk build
trunk serve --open
cargo test
```

> 说明：项目历史上出现过 `NO_COLOR=1` 与 `trunk` 参数解析冲突，建议保持 `NO_COLOR=true` 或不设置。

---

## 3. 目录结构（重点）

```text
src/
  main.rs                         # 应用入口、Context 注入、AppPage 路由分发
  app_state.rs                    # WordBankState（全局状态）
  pages/                          # 页面级组件
  components/                     # 复用组件
  structures/                     # 核心数据结构（WordBankEntry, PracticeResult）
  utils/                          # 领域工具（校验、哈希、i18n、加密、shuffle）
build.rs                          # 编译期扫描词库 CSV，生成内置词库列表常量
```

---

## 4. 全局状态模型（最关键）

`WordBankState` 位于 `src/app_state.rs`，是全局共享状态核心。

### 4.1 词库相关

- `entries`：**已提交词库**（全局真相源）
- `set_entries`：写入已提交词库
- `data_version`：词库外部重载版本号
  - 例如：导入 CSV、切换内置词库、单条/批量新增后会 `+1`
  - 词库浏览器监听该值来重置内部草稿
- `source_name`：当前词库来源描述

### 4.2 练习相关

- `selected_word_entry_ids`：当前待练习 id 列表
- `practice_result`：历史累计结果
- `temp_practice_result`：当前轮临时结果
- `last_completed_practice_result`：最近完成一轮的快照
- `summary_return_page`：总结页“继续/返回”目标页

---

## 5. 路由机制与页面关系

项目不使用 URL 路由，而是通过 `AppPage` 枚举切换页面（见 `src/pages/mod.rs` + `src/main.rs`）。

主流程：

1. `Home`（导入历史或直接开始）
2. `PracticeModeSelect`（词库模式 / 系列模式 / 本地词库修改器）
3. 分流：
   - `LexiconMode` -> `LexiconPractice` -> `LexiconSummary`
   - `SeriseSelect` -> 各 `Serise*Practice` -> `LexiconSummary`
   - `LocalLexiconEditor`（词库维护）

---

## 6. 词库编辑调用链（LocalLexiconEditor）

入口：`src/pages/local_lexicon_editor.rs`

页面由以下模块组成：

- `LexiconEditorAddSingle`：单条新增
- `AiResearcher`：词典/API 查询并分流为单条或多条
- `LexiconEditorAddMulti`：多条批量新增（可编辑表格）
- `LexiconBrowser(mode=Edit)`：词库表格浏览 + 搜索 + 批量编辑 + 确认修改
- `ImportCsvButton` / 导出按钮

### 6.1 草稿与提交的区别（必须理解）

`LexiconBrowser` 内部维护草稿（`draft_entries`），并将传入的 `set_entries` 重定向为草稿写接口。  
只有点击“确认修改”后，才会写回全局 `entries`（已提交词库）。

这意味着：

- 你在表格里改了值但没点确认 -> 全局 `entries` 仍是旧数据
- 导出 CSV 读取的是全局 `entries`，因此导出前必须先确认修改

### 6.2 确认修改（confirm_changes）关键流程

位于：`src/components/lexicon_browser/components.rs`

流程简化：

1. 处理删除标记（编辑模式）
2. 仅对 `hash_dirty_entry_ids` 中的行重算 id
3. 先在变更集内去重，再与未变更行做冲突检查
4. 冲突则报错并终止提交
5. 通过后写回全局 `entries`，并重置草稿/撤销/删除标记

---

## 7. 词库练习调用链（Lexicon 模式）

### 7.1 选词页

文件：`src/pages/lexicon_select.rs`

- 使用 `LexiconBrowser(mode=Query)` 做筛选与勾选
- 点击“开始练习”时读取的是 **全局 entries 的 selected 字段**
- 因此在 Query 表格中勾选后也需要“确认修改”才能生效

### 7.2 练习页

文件：`src/pages/lexicon_practice.rs`

核心状态：

- `active_question_ids`：当前页正在答的题
- `solved_question_ids`：本轮已完整通过的题
- `answer_inputs`：输入缓存（键格式 `entry_id::field`）

`check_click` 流程：

1. 根据 `active_question_ids` 取出题目
2. 对每个字段执行规范化比较（`normalize_for_compare`）
3. 通过 `record_field_check_result` 写入临时结果
4. 整题全对则移入 solved，并自动补充新题

---

## 8. 系列练习调用链（Series 模式）

### 8.1 选择页

文件：`src/pages/serise_select.rs`

- 固定加载 `series-word-bank.csv`
- 按 tag 筛选当前系列的 id（如 `cardinal_number`、`month` 等）
- 写入 `selected_word_entry_ids` 并跳转对应系列练习页

### 8.2 各系列页面

- `number.rs`：按数字分组、双输入（基数词/序数词）
- `month.rs`：通用练习模板（固定字段）
- `pronoun.rs`：按“人称 x 形式”配置驱动出题
- `interrogative.rs`：通用练习模板（固定字段）

公共初始化：`serise_practice/mod.rs::initialize_temp_practice_result`

---

## 9. 数据结构与字段约束

### 9.1 WordBankEntry

文件：`src/structures/word_bank_entry.rs`

- 包含词性、原型、中英、tags、各类词形变体
- `PartOfSpeech` 扩展为 10 类
- `PART_OF_SPEECH_OPTIONS` 是 UI 与校验共同依赖的常量

### 9.2 PracticeResult

文件：`src/structures/pracresult.rs`

- `practice_result`：历史累计
- `temp_practice_result`：本轮临时
- `last_completed_practice_result`：总结页“本次结果”

---

## 10. CSV 结构与兼容策略

相关文件：

- `src/components/lexicon_browser/structures.rs`（CSV<->Entry 转换）
- `src/components/lexicon_browser/utils.rs`（解析/导出）

策略要点：

- 解析会自动检测是否有表头
- 若失败会切换“有/无表头”模式重试
- 兼容 UTF-8 BOM
- 解析后用规范化 id 检测重复词条并报冲突行号

---

## 11. 哈希与判重规则

文件：`src/utils/dictionary.rs`

- `compute_word_entry_id` 使用 `v2` payload + SHA256
- 哈希只包含词性和词形字段，不包含 `selected/tags/english/chinese`
- 目的：把“词条形态唯一性”与“翻译展示信息”解耦

可选低频字段由 `OPTIONAL_VARIANT_FIELD_KEYS` 统一管理，默认在部分 UI 中不勾选。

---

## 12. 组件分层建议（维护视角）

### 12.1 LexiconBrowser 已拆分

- `search_panel.rs`：搜索 UI
- `table_panel.rs`：表格 UI + 操作按钮
- `components.rs`：状态与流程编排（排序、筛选、确认修改）
- `utils.rs`：排序/筛选/CSV 工具
- `structures.rs`：模式与 CSV 结构

### 12.2 Practice 三件套

- `practice_settings.rs`
- `practice_entry.rs`
- `practice_buttons.rs`

建议新增练习模式时优先复用这三层，减少重复逻辑。

---

## 13. 常见改动操作清单

### 13.1 新增词形字段（高风险改动）

至少同步以下位置：

1. `WordBankEntry` 与 `PracticedWordEntryResult`
2. `compute_word_entry_id`
3. `validate_forms_by_part_of_speech`
4. `DATA_COLUMN_KEYS` + `column_value_text`
5. `CsvWordEntry` + CSV 读写表头
6. `entry_field_value` + `answer_stats_mut`
7. 相关 UI（单条新增、多条编辑、练习设置）

### 13.2 修改练习判题规则

优先看：

- `normalize_for_compare`
- `record_field_check_result`
- `merge_practice_result`

### 13.3 修改词库确认逻辑

优先看：

- `lexicon_browser/components.rs::confirm_changes`
- `dictionary.rs::compute_word_entry_id`

---

## 14. 性能与稳定性关注点

- `table_panel.rs` 行列较多，DOM 开销大，改动需关注重渲染
- 表格列定义分散，字段同步容易漏项
- 词库编辑草稿与全局提交语义不同，易造成“看起来改了但未生效”

建议每次改动后至少执行：

1. `NO_COLOR=true trunk build`
2. 从“导入词库 -> 编辑 -> 确认 -> 导出 -> 再导入”跑一遍回归

---

## 15. 故障排查速查

### 15.1 开始练习后题目不对/为空

- 检查 `selected_word_entry_ids` 是否已更新
- 检查选词页是否点过“确认修改”

### 15.2 导出内容不是最新编辑

- 检查本地词库修改器里是否先点了“确认修改”

### 15.3 CSV 导入报重复

- 查看报错中的首行与冲突行，通常是词性+词形组合重复

### 15.4 Docker/Trunk 构建异常

- 确认 `build.rs` 在构建上下文中被复制
- 优先使用 `trunk build` 复现前端构建问题

---

## 16. 后续可迭代方向（可选）

- 抽离字段元数据（单一真相源），减少多处手工同步
- 对大表格引入更细粒度渲染优化（按需编辑、虚拟滚动）
- 为关键流程补自动化回归（CSV 解析、确认修改、练习统计合并）

---

如需继续扩展本文件，建议保持“功能视角 + 调用链 + 改动清单”三段式结构，便于交接与排障。
