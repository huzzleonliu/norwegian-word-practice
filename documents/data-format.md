# 词库与数据格式

## 词条模型

运行时核心类型为 `WordBankEntry`（`structures/word_bank_entry.rs`），主要包括：

- **身份**：`id`（由词性 + 词形字段哈希生成，翻译字段不参与）
- **选题**：`selected`
- **词性**：固定枚举（动词、名词、形容词等）
- **标签**：`tags: Vec<String>`（系列练习靠 tag 过滤）
- **翻译**：`chinese` / `english`（均为列表，CSV 中用 `|` 连接）
- **原型**：`base_form`
- **添加时间**：`added_at`（ISO-8601；旧数据可空）
- **各词性变体字段**：现在时、复数、形容词中性等（`Option<String>`）

CSV 有 schema 版本行（如 `#schema_version=1`）；解析与序列化见 `utils/csv_schema.rs`。

## 文件形态

| 扩展名 | 用途 |
| --- | --- |
| `.nwpdict` | 加密词库（导入/导出/内置加载的正式格式） |
| `.csv` | 明文编辑与迁移中间态；应用内导入**不**直接接受明文 CSV（可经编辑器加解密面板转换） |

加密与解密入口：`utils/dictionary_crypto.rs`、`utils/lexicon_file.rs`。

## 内置数据位置

```text
crates/nwp-web/data/
├─ lexicon-word-bank/          # 通用词库练习
│  └─ *.nwpdict
├─ series-word-bank/
│  └─ series-word-bank.nwpdict # 系列统一词库（按 tag 选题）
└─ audio/                      # 跟读音频书
   └─ byen-er-bergen/          # mp3 + lrc，按文件名配对
```

系列选择页加载统一文件后，用 tag 过滤，例如：

| 系列 | 典型 tag |
| --- | --- |
| 数词 | `cardinal_number`、`ordinal_number` |
| 月份 | `month` |
| 代词 | `pronoun` |
| 疑问词 | `interrogative` |
| 国家 | `country`，并常带挪威语国家名如 `country\|Norge` |

国家系列会把共享语言词条（多国家写在同一 tags）复制归入多个国家块。

## 练习相关约定

- 回答字段目标值为空或恰好为 **`NULL`（全大写）** 时视为不可练习。
- 判题比较会做规范化（含挪威语特殊字母与常见别名），见练习引擎工具函数。
- 练习结果结构见 `structures/pracresult.rs`；可与词库类似地做导入导出（以产品页为准）。

## 安全提示

- 不要把 API Key、钱包私钥、助记词写入仓库或词库文件。
- 公开仓库中的 `.nwpdict` 仍属于「混淆/封装」，不是对内容保密的强保证；分享词库前请确认没有私人备注或密钥。
