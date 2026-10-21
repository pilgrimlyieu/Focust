# 配置参考

<div align="center">

**[简体中文](CONFIGURATION.zh-CN.md)** | **[English](CONFIGURATION.md)**

</div>

> [!WARNING]
> 由于实现限制，配置选项不能使用 TOML 标准的 snake_case，全部使用 camelCase。

本文档提供了 Focust 中所有可用配置选项的全面指南。配置以 TOML 格式存储，可以通过设置界面或手动编辑配置文件来编辑。

## 目录

- [配置文件位置](#配置文件位置)
- [配置结构](#配置结构)
- [通用设置](#通用设置)
- [休息计划设置](#休息计划设置)
- [注意设置](#注意设置)
- [主题设置](#主题设置)
- [音频设置](#音频设置)
- [建议设置](#建议设置)
- [应用程序排除设置](#应用程序排除设置)
- [高级设置](#高级设置)
- [示例](#示例)
- [最佳实践](#最佳实践)

---

## 配置文件位置

Focust 将其配置存储在特定于平台的位置：

**正式版（Release 构建）：**
- **Windows**：`%APPDATA%\com.fesmoph.focust\config.toml`
- **macOS**：`~/Library/Application Support/com.fesmoph.focust/config.toml`
- **Linux**: `~/.config/com.fesmoph.focust/config.toml`

**开发版（Debug 构建）：**
- **Windows**：`%APPDATA%\com.fesmoph.focust.dev\config.toml`
- **macOS**：`~/Library/Application Support/com.fesmoph.focust.dev/config.toml`
- **Linux**: `~/.config/com.fesmoph.focust.dev/config.toml`

> **注意**：开发版本使用 `.dev` 后缀来区分开发和生产配置。`.dev` 后缀会追加到平台返回路径的最后一段 —— 在 Windows 与 Linux 上，这会导致日志目录变为 `.../com.fesmoph.focust/logs.dev`，而在 macOS 上日志目录为 `~/Library/Logs/com.fesmoph.focust.dev/`。这允许开发者测试更改而不会影响他们的生产设置。

您可以从设置界面快速打开配置目录：
1. 转到**高级选项**选项卡
2. 点击**打开配置目录**

> **注意**：Focust 只在启动时读取一次配置文件。缺少必填字段、非法的枚举取值，或没有写全的表（`theme`、`audio`、`suggestions`、`timeRange` 一旦出现就必须写全所有必填键）都会导致整份文件加载失败：此时 Focust 以默认设置启动，并在你下次从设置界面保存时用这些默认值覆盖该文件。Focust 运行期间手动修改文件也会因同样的原因丢失。颜色字符串与字体名称会原样保存，不做校验。

## 配置结构

配置文件分为几个主要部分：

```toml
# 通用应用程序设置
autostart = false
monitorDnd = true
# ...

# 休息计划（表数组）
[[schedules]]
name = "工作时间"
enabled = true
# ...

# 定时提醒（表数组）
[[attentions]]
name = "喝水提醒"
enabled = true
# ...

# 应用程序排除（表数组）
[[appExclusions]]
active = true
rule = "pause"
processes = ["chrome.exe", "firefox"]
# ...
```

---

## 通用设置

### `autostart`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：系统启动时自动启动应用程序

### `monitorDnd`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：检测到系统勿扰模式时暂停所有休息

### `inactiveS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）
- **说明**：调度器自动暂停前的不活动持续时间。检测到活动时调度器恢复。

### `allScreens`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：在多显示器设置中在所有显示器上显示休息窗口。禁用时，休息仅出现在主显示器上。

### `showTrayIcon`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：在系统托盘显示应用图标。需要重启应用生效。
- **注意**：隐藏后，可通过再次启动应用来打开设置（使用单实例模式），或通过任务管理器（Windows）/活动监视器（macOS）/系统监视器（Linux）关闭应用。

### `language`
- **类型**：字符串
- **默认值**：从系统区域设置自动检测
- **选项**：`"de-DE"`, `"en-US"`, `"es-ES"`, `"fr-FR"`, `"it-IT"`, `"ja-JP"`, `"ko-KR"`, `"pt-BR"`, `"ru-RU"`, `"zh-CN"`
- **说明**：界面语言。首次运行时自动检测，但可以手动更改。

### `themeMode`
- **类型**：字符串
- **默认值**：`"system"`
- **选项**：`"light"`, `"dark"`, `"system"`
- **说明**：设置界面的颜色主题。`"system"` 遵循您操作系统的主题首选项。

### `postponeShortcut`
- **类型**：字符串
- **默认值**：`""`（空）
- **格式**：键组合，如 `"Ctrl+Shift+P"` 或 `"Alt+P"`
- **说明**：推迟下一次休息的全局快捷键。留空以禁用。示例：
  - `"Ctrl+Shift+P"`
  - `"Alt+B"`

### `windowSize`
- **类型**：浮点数
- **默认值**：`0.8`（屏幕大小的 80%）
- **范围**：`0.5` 到 `1.0`
- **说明**：休息窗口的大小，作为屏幕的一部分。
  - `0.5` = 50%（半屏）
  - `0.8` = 80%（默认）
  - `1.0` = 100%（全屏）

---

## 休息计划设置

休息计划定义何时以及多久发生一次休息。您可以拥有多个具有不同时间范围和活动日期的休息计划。每种短休息、长休息都有一个由应用生成的内部 `id`，通常可以省略，也不应手动修改。

### 基本休息计划字段

#### `name`
- **类型**：字符串
- **默认值**：`"Default Schedule"`
- **说明**：休息计划的人类可读名称

#### `enabled`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：此休息计划是否活动

#### `notificationBeforeS`
- **类型**：整数（秒）
- **默认值**：`10`
- **说明**：在休息开始前 X 秒发送系统通知。设置为 `0` 可禁用通知。

### 时间范围

#### `timeRange`
- **类型**：表
- **默认值**：`{ start = "00:00:00", end = "00:00:00" }`
- **说明**：定义此休息计划在一天中何时活动。将开始和结束时间都设置为「00:00」以表示全天休息计划。该表一旦出现，`start` 与 `end` 均为必填。

```toml
[schedules.timeRange]
start = "09:00"  # 24 小时格式
end = "17:00"    # 24 小时格式

# 对于全天休息计划
[schedules.timeRange]
start = "00:00"
end = "00:00"

# 应用程序可能在内部保存为 hh:mm:ss
[schedules.timeRange]
start = "00:00:00"
end = "00:00:00"
```

#### `daysOfWeek`
- **类型**：字符串数组
- **默认值**：`["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]`
- **选项**：`"Mon"`, `"Tue"`, `"Wed"`, `"Thu"`, `"Fri"`, `"Sat"`, `"Sun"`
- **说明**：此休息计划活动的日期

### 短休息

短休息是频繁发生的短提醒（通常持续时间为 20 秒）。

#### `miniBreaks.id`
- **类型**：整数
- **必需**：否
- **说明**：此短休息的内部标识符，应用程序用它把休息事件对应到本条目。必须在所有 `miniBreaks.id` 中唯一 —— 若出现重复，只有第一个匹配的条目会生效。该字段可以省略，此时由应用程序生成一个值，但生成时不会与文件中已有的 id 比对，因此显式填写与省略混用时可能产生重复。

#### `miniBreaks.enabled`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：此短休息是否活动

#### `miniBreaks.durationS`
- **类型**：整数（秒）
- **默认值**：`20`
- **说明**：短休息持续多长时间

#### `miniBreaks.postponedS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）
- **说明**：使用推迟功能时推迟休息多长时间

#### `miniBreaks.maxPostponeCount`
- **类型**：整数
- **默认值**：`2`
- **说明**：一次休息允许推迟的最大次数

#### `miniBreaks.strictMode`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：启用时，休息不能被跳过或推迟

#### `miniBreaks.intervalS`
- **类型**：整数（秒）
- **默认值**：`1200`（20 分钟）
- **说明**：短休息之间的时间

#### 短休息主题

有关主题配置详细信息，请参阅[主题设置](#主题设置)部分。

```toml
[schedules.miniBreaks.theme]
background = { current = "solid", solid = "#1f2937" }
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"
```

#### 短休息音频

有关音频配置详细信息，请参阅[音频设置](#音频设置)部分。

```toml
[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7
```

#### 短休息建议

有关建议配置详细信息，请参阅[建议设置](#建议设置)部分。

```toml
[schedules.miniBreaks.suggestions]
show = true
```

### 长休息

长休息是较少发生的延长休息时段（通常为 5 分钟）。

#### `longBreaks.id`
- **类型**：整数
- **必需**：否
- **说明**：此长休息的内部标识符，应用程序用它把休息事件对应到本条目。必须在所有 `longBreaks.id` 中唯一 —— 若出现重复，只有第一个匹配的条目会生效。该字段可以省略，此时由应用程序生成一个值，但生成时不会与文件中已有的 id 比对，因此显式填写与省略混用时可能产生重复。

#### `longBreaks.enabled`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：此长休息是否活动

#### `longBreaks.durationS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）
- **说明**：长休息持续多长时间

#### `longBreaks.postponedS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）
- **说明**：使用推迟功能时推迟休息多长时间

#### `longBreaks.maxPostponeCount`
- **类型**：整数
- **默认值**：`2`
- **说明**：一次长休息允许推迟的最大次数

#### `longBreaks.strictMode`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：启用时，休息不能被跳过或推迟

#### `longBreaks.afterMiniBreaks`
- **类型**：整数
- **默认值**：`4`
- **说明**：在这么多短休息后触发长休息。例如，使用默认设置：
  - 20 分钟、40 分钟、60 分钟、80 分钟时短休息
  - 100 分钟时长休息（4 次短休息后）
  - 循环重复

长休息也支持主题、音频和建议设置（格式与短休息相同）。

**休息计划示例：**
```toml
[[schedules]]
name = "工作时间"
enabled = true
notificationBeforeS = 5
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]

[schedules.timeRange]
start = "09:00"
end = "17:00"

[schedules.miniBreaks]
enabled = true
durationS = 20
postponedS = 300
maxPostponeCount = 2
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
background = { current = "solid", solid = "#1e293b" }
textColor = "#f1f5f9"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 300
postponedS = 600
maxPostponeCount = 2
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
background = { current = "imagePath", imagePath = "/path/to/background.jpg" }
textColor = "#ffffff"
blurRadius = 10
opacity = 0.85
fontSize = 28
fontFamily = "Helvetica"

[schedules.longBreaks.audio]
current = "filePath"
filePath = "/path/to/sound.mp3"
volume = 0.8

[schedules.longBreaks.suggestions]
show = true
```

---

## 注意设置

注意提醒是基于时间的通知，像闹钟一样工作。与休息不同，它们不会中断您的工作 — 只是在特定时间显示简短消息，并可以直接关闭。

### 基本注意字段

注意项的每个字段在文件中都是必填的；下面列出的取值是设置界面新建提醒时使用的值。缺少字段会导致整份配置加载失败。

#### `id`
- **类型**：整数
- **必需**：是
- **说明**：此提醒的内部标识符。必须在所有注意项的 id 中唯一 —— 若出现重复，只有第一个匹配的条目会生效。不应手动更改！

#### `name`
- **类型**：字符串
- **必需**：是
- **说明**：提醒的名称（例如，「喝水提醒」「站立提醒」）。新建提醒使用 `"New attention"`。

#### `enabled`
- **类型**：布尔值
- **必需**：是
- **说明**：此提醒是否活动。新建提醒使用 `true`。

#### `times`
- **类型**：字符串数组（24 小时时间格式）
- **必需**：是
- **说明**：提醒应该触发的时间列表

#### `daysOfWeek`
- **类型**：字符串数组
- **必需**：是
- **选项**：`"Mon"`, `"Tue"`, `"Wed"`, `"Thu"`, `"Fri"`, `"Sat"`, `"Sun"`
- **说明**：此提醒活动的日期。新建提醒使用全部七天。

#### `title`
- **类型**：字符串
- **必需**：是
- **说明**：弹出窗口中显示的标题。新建提醒使用 `"Attention Reminder"`。

#### `message`
- **类型**：字符串
- **必需**：是
- **说明**：要显示的消息内容。新建提醒使用 `"This is an attention reminder."`。

#### `durationS`
- **类型**：整数（秒）
- **必需**：是
- **说明**：显示提醒弹出窗口多长时间。新建提醒使用 `5`。

#### `theme`
- **类型**：表
- **必需**：是
- **说明**：提醒窗口的外观，参见[主题设置](#主题设置)。

**注意示例：**
```toml
# 喝水提醒
[[attentions]]
id = 0
name = "补水"
enabled = true
times = ["10:00", "14:00", "16:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "该喝水了"
message = "保持水分充足！喝一杯水。"
durationS = 10

[attentions.theme]
background = { current = "solid", solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

# 站立提醒
[[attentions]]
id = 1
name = "站起来"
enabled = true
times = ["11:00", "15:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "站起来并活动"
message = "花点时间站起来并伸展您的腿！"
durationS = 15

[attentions.theme]
background = { current = "solid", solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"
```

---

## 主题设置

主题设置控制休息窗口的外观。

### `background`
- **类型**：对象
- **说明**：休息窗口的背景源

**结构：**

```toml
[schedules.miniBreaks.theme.background]
current = "solid"                # 当前活跃的背景类型
solid = "#1f2937"                # 即使切换到图片也会保留
imagePath = "/path/to/image.jpg" # 即使切换到文件夹也会保留
imageFolder = "/path/to/images/" # 即使切换到纯色也会保留
```

**背景类型选项：**

1. **纯色**（`current = "solid"`）：
   ```toml
   [schedules.miniBreaks.theme.background]
   current = "solid"
   solid = "#1f2937"
   ```

2. **单张图片**（`current = "imagePath"`）：
   ```toml
   [schedules.miniBreaks.theme.background]
   current = "imagePath"
   imagePath = "/path/to/image.jpg"
   ```
   - 支持格式：JPG、PNG、WebP 等

3. **文件夹中的随机图片**（`current = "imageFolder"`）：
   ```toml
   [schedules.miniBreaks.theme.background]
   current = "imageFolder"
   imageFolder = "/path/to/images/"
   ```

### `textColor`
- **类型**：字符串（十六进制颜色）
- **默认值**：`"#f8fafc"`
- **格式**：`#RRGGBB`
- **说明**：休息窗口中所有文本的颜色

### `blurRadius`
- **类型**：整数（0-255）
- **默认值**：`8`
- **说明**：应用于背景图像的高斯模糊半径（以像素为单位）

### `opacity`
- **类型**：浮点数（0.0-1.0）
- **默认值**：`0.9`
- **说明**：休息窗口覆盖层的不透明度
  - `0.0` = 完全透明（内容仍然可见）
  - `1.0` = 完全不透明

### `fontSize`
- **类型**：整数（像素）
- **默认值**：`24`
- **说明**：休息窗口中文本的基本字体大小

### `fontFamily`
- **类型**：字符串
- **默认值**：`"Arial"`
- **说明**：字体系列名称。使用系统上安装的字体。

**主题示例：**

以下示例任选其一，不能合并到同一文件中。

```toml
# 简约深色主题
[schedules.miniBreaks.theme]
textColor = "#e2e8f0"
blurRadius = 0
opacity = 1.0
fontSize = 20
fontFamily = "Segoe UI"

[schedules.miniBreaks.theme.background]
current = "solid"
solid = "#0f172a"
```

```toml
# 带图片的自然主题
[schedules.miniBreaks.theme]
textColor = "#ffffff"
blurRadius = 12
opacity = 0.75
fontSize = 28
fontFamily = "Georgia"

[schedules.miniBreaks.theme.background]
current = "imagePath"
imagePath = "/path/to/forest.jpg"
```

```toml
# 充满活力的主题并保留设置
[schedules.miniBreaks.theme]
textColor = "#fef3c7"
blurRadius = 0
opacity = 0.95
fontSize = 26
fontFamily = "Tahoma"

[schedules.miniBreaks.theme.background]
current = "solid"
solid = "#7c3aed"
imagePath = "/path/to/sunset.jpg" # 保留
imageFolder = "C:\\Wallpapers"    # 保留
```

---

## 音频设置

音频设置控制休息通知声音。

> **说明**：相同的音频会在休息**开始**和**自然结束**时播放，帮助您感知两个转换时刻。

**结构：**

```toml
[schedules.miniBreaks.audio]
current = "builtin"              # 当前活跃的音频类型
builtinName = "gentle-bell"      # 即使切换到 filePath 也会保留
filePath = "/path/to/custom.mp3" # 即使切换到 builtin 也会保留
volume = 0.6
```

### `current`
- **类型**：字符串枚举
- **默认值**：`"none"`
- **选项**：`"none"`、`"builtin"`、`"filePath"`
- **说明**：当前活跃的音频类型

### `builtinName`
- **类型**：字符串（可选）
- **说明**：内置音效的名称
- **可用音效**：
  - `"gentle-bell"` - 温和的铃声
  - `"soft-gong"` - 柔和的锣声
  - `"notification"` - 简单的通知
  - `"bright-notification"` - 明亮的通知

### `filePath`
- **类型**：字符串（可选）
- **说明**：自定义音频文件的绝对路径
- **支持格式**：MP3、WAV、OGG、FLAC

### `volume`
- **类型**：浮点数（0.0-1.0）
- **默认值**：`0.6`
- **说明**：播放音量
  - `0.0` = 静音
  - `1.0` = 最大音量

**音频示例：**

以下示例任选其一，不能合并到同一文件中。

```toml
# 无音频
[schedules.miniBreaks.audio]
current = "none"
volume = 0.6
```

```toml
# 内置声音
[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7
```

```toml
# 自定义声音文件
[schedules.miniBreaks.audio]
current = "filePath"
filePath = "C:\\Users\\YourName\\Music\\zen-bell.mp3"
volume = 0.8
```

---

## 建议设置

建议设置控制休息期间是否显示激励性消息。

### `show`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：在每次休息期间随机显示一条建议

**建议配置：**

建议存储在单独的 TOML 文件中，与主配置文件位于同一目录：
- `suggestions.toml`

**建议文件格式：**
```toml
[byLanguage.en-US]
suggestions = [] # 旧版兼容占位；不会作为公共池使用。
shortSuggestions = [
    "Look away from your screen and focus on a distant object.",
    "Roll your shoulders back and take a deep breath.",
    "Blink slowly and relax your eyes.",
    ...
]
longSuggestions = [
    "Drink a glass of water.",
    "Stand up and stretch your legs.",
    ...
]

[byLanguage.zh-CN]
suggestions = [] # 旧版兼容占位；不会作为公共池使用。
shortSuggestions = [
    "将目光从屏幕移开，专注于远处的物体。",
    "向后转动肩膀，深呼吸。",
    "慢慢眨眼，放松眼睛。",
    ...
]
longSuggestions = [
    "喝一杯水。",
    "站起来伸展腿部。",
    ...
]
```

您可以为不同语言定义建议列表。短休息使用 `shortSuggestions`，长休息使用 `longSuggestions`。

`suggestions` 仅用于旧版 Focust 解析文件，新版本保存时会保持为空。这只是解析层面的兼容：如果旧版只读取 `suggestions`，在新版保存过文件后可能不会再显示建议。加载只包含 `suggestions` 的旧文件时，当前版本的 Focust 会将该旧列表作为缺失的短休息或长休息池的回退来源。如果文件中显式写了 `shortSuggestions = []` 或 `longSuggestions = []`，表示用户有意清空该池，不会再回退到 `suggestions`。

---

## 应用程序排除设置

应用程序排除功能允许您根据正在运行的应用程序控制提醒的暂停。这对于在演示、视频通话或使用特定应用程序时防止休息中断非常有用。

规则每 10 秒检查一次，并且只有**第一条启用的规则**会被求值 —— 其后的规则会被忽略。规则修改需要重启应用后生效。休息或提醒窗口显示期间不会检查，且排除导致的暂停不会被界面上的“继续”操作清除。

### `appExclusions`
- **类型**：对象数组
- **默认值**：`[]`（空数组）
- **说明**：排除规则列表

### 排除规则结构

每个排除规则具有以下结构：

```toml
[[appExclusions]]
active = true
rule = "pause"  # 或 "resume"
processes = ["chrome.exe", "firefox"]
```

#### `active`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：此排除规则是否活动

#### `rule`
- **类型**：字符串枚举
- **默认值**：`"pause"`
- **选项**：`"pause"`、`"resume"`（仅小写）
- **说明**：列出的进程运行时会发生什么
  - `"pause"`：当 `processes` 中任一进程正在运行时暂停提醒
  - `"resume"`：当 `processes` 中没有进程正在运行时暂停提醒，即只有至少一个在运行时才提醒

#### `processes`
- **类型**：字符串数组
- **默认值**：`[]`（空数组）
- **说明**：要匹配的进程名称或路径列表。匹配不区分大小写，对进程名及其完整可执行文件路径使用子串匹配。
  - 可以是进程名称（例如 `"chrome.exe"`、`"firefox"`）
  - 可以是部分名称（例如 `"chrome"` 匹配 `"chrome.exe"`）
  - 可以是完整路径（例如 `"C:\\Program Files\\App\\app.exe"`）
  - 列表为空的规则永不匹配：`rule = "pause"` 时不起作用，但 `rule = "resume"` 时提醒会一直保持暂停

### 配置示例

**演示期间暂停：**
```toml
[[appExclusions]]
active = true
rule = "pause"
processes = ["POWERPNT.EXE", "Keynote", "impress"]  # PowerPoint、Keynote、LibreOffice Impress
```

**视频通话期间暂停：**
```toml
[[appExclusions]]
active = true
rule = "pause"
processes = ["zoom.exe", "Teams.exe", "Slack.exe", "Discord.exe"]
```

**仅在编辑器运行时提醒：**
```toml
[[appExclusions]]
active = true
rule = "resume"
processes = ["code.exe", "idea64.exe", "pycharm64.exe"]
```

**跨平台示例：**
```toml
[[appExclusions]]
active = true
rule = "pause"
processes = [
    # Windows
    "POWERPNT.EXE", "chrome.exe", "firefox.exe",
    # macOS
    "Keynote", "Google Chrome", "Firefox",
    # Linux
    "chrome", "firefox", "libreoffice"
]
```

### 平台特定说明

- **Windows**：进程名称通常包含 `.exe` 扩展名（例如 `"chrome.exe"`）
- **macOS**：应用程序名称匹配 `.app` 名称，不含扩展名（例如 `"Google Chrome"`）
- **Linux**：进程名称通常为小写，不含扩展名（例如 `"chrome"`、`"firefox"`）

**示例：**
```toml
autostart = false
monitorDnd = true
inactiveS = 300
allScreens = false
language = "zh-CN"
themeMode = "system"
postponeShortcut = "Ctrl+Shift+P"
windowSize = 0.8

[[appExclusions]]
active = true
rule = "pause"
processes = ["POWERPNT.EXE", "zoom.exe", "Teams.exe"]

[[appExclusions]]
active = true
rule = "resume"
processes = ["code.exe", "idea64.exe"]
```

---

## 高级设置

高级设置允许用户自定义应用程序的行为，这部分配置无法在设置界面中修改，必须手动编辑配置文件。所有相关设置都在 `[advanced]` 表下。

高级设置的向后兼容性保证弱于普通设置。未来如果需要进行不兼容变更，会先在设置界面向用户提示更新；但这些选项仍主要面向手动配置，而不是常规界面工作流。

### `logLevel`
- **类型**：字符串枚举
- **默认值**：发布版为 `"info"`，调试版为 `"trace"`
- **选项**：`"trace"`、`"debug"`、`"info"`、`"warn"`、`"error"`
- **说明**：日志记录的详细程度。级别越高，日志越详细，便于排查问题。

示例：

```toml
[advanced]
logLevel = "debug" # 启用详细调试日志记录
```

### `resetMiniBreakCounterOnPauseReasons`
- **类型**：暂停原因字符串数组
- **默认值**：`[]`
- **选项**：`"userIdle"`、`"dnd"`、`"manual"`、`"timedManual"`、`"appExclusion"`
- **说明**：当调度器因为列出的原因暂停时，重置用于判断长休息节奏的短休息计数器。例如加入 `"userIdle"` 后，用户离开前累计的短休息次数不会在回来后继续推动长休息。

示例：

```toml
[advanced]
resetMiniBreakCounterOnPauseReasons = ["userIdle"]
```

### `resetOnWake`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：控制系统从睡眠或休眠恢复时，是否关闭旧休息窗口，重新开始休息间隔，并清零用于安排长休息的短休息次数。

```toml
[advanced]
resetOnWake = true
```

### `trayPauseDurationsMinutes`
- **类型**：正整数数组
- **默认值**：`[15, 30, 60]`
- **说明**：控制系统托盘菜单中显示的定时暂停时长，单位为分钟。选择某个时长后，休息提醒和定时提醒都会暂停，直到该时长结束。列表会自动升序排序并去重，`0` 会被忽略；负数无效，会导致整份配置加载失败。设置为空数组（`[]`）可以隐藏定时暂停子菜单。编辑后需要重启应用才会更新托盘菜单。

示例：

```toml
[advanced]
trayPauseDurationsMinutes = [15, 30, 60]
```

---

## 示例

### 示例一：最小配置

```toml
# 通用设置
autostart = false
monitorDnd = true
inactiveS = 300
allScreens = false
language = "zh-CN"
themeMode = "system"
postponeShortcut = ""
windowSize = 0.8
attentions = []

[[schedules]]
name = "Default Schedule"
enabled = true
daysOfWeek = [
    "Mon",
    "Tue",
    "Wed",
    "Thu",
    "Fri",
    "Sat",
    "Sun",
]
notificationBeforeS = 10

[schedules.timeRange]
start = "00:00:00"
end = "00:00:00"

[schedules.miniBreaks]
id = 0
enabled = true
durationS = 20
postponedS = 300
maxPostponeCount = 2
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.theme.background]
current = "solid"
solid = "#1f2937"

[schedules.miniBreaks.audio]
current = "none"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
id = 1
enabled = true
durationS = 300
postponedS = 300
maxPostponeCount = 2
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.longBreaks.theme.background]
current = "solid"
solid = "#1f2937"

[schedules.longBreaks.audio]
current = "none"
volume = 0.6

[schedules.longBreaks.suggestions]
show = true
```

### 示例二：综合工作设置

```toml
monitorDnd = true
inactiveS = 300
allScreens = true
language = "en-US"
themeMode = "system"
postponeShortcut = "Ctrl+Shift+B"
windowSize = 0.85

# 工作时间休息计划（较长的休息时间，更严格）
[[schedules]]
name = "Work Hours"
enabled = true
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
notificationBeforeS = 10

[schedules.timeRange]
start = "09:00"
end = "17:30"

[schedules.miniBreaks]
enabled = true
durationS = 20
postponedS = 300
maxPostponeCount = 2
strictMode = false
intervalS = 1200 # 20 分钟

[schedules.miniBreaks.theme]
background = { current = "imagePath", imagePath = "/home/user/wallpapers/calm-lake.jpg" }
textColor = "#e0f2fe"
blurRadius = 10
opacity = 0.85
fontSize = 24
fontFamily = "Helvetica"

[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 600  # 10 分钟
postponedS = 600
maxPostponeCount = 2
strictMode = true
afterMiniBreaks = 3

[schedules.longBreaks.theme]
background = { current = "imagePath", imagePath = "/home/user/wallpapers/mountain.jpg" }
textColor = "#ffffff"
blurRadius = 15
opacity = 0.8
fontSize = 28
fontFamily = "Georgia"

[schedules.longBreaks.audio]
current = "filePath"
filePath = "/home/user/sounds/meditation-bell.mp3"
volume = 0.7

[schedules.longBreaks.suggestions]
show = true

# 晚上休息计划（较短的休息时间，较不严格）
[[schedules]]
name = "Evening"
enabled = true
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
notificationBeforeS = 5

[schedules.timeRange]
start = "18:00"
end = "22:00"

[schedules.miniBreaks]
enabled = true
durationS = 15
postponedS = 600
maxPostponeCount = 2
strictMode = false
intervalS = 1800  # 30 分钟

[schedules.miniBreaks.theme]
background = { current = "solid", solid = "#1e3a8a" }
textColor = "#dbeafe"
blurRadius = 5
opacity = 0.9
fontSize = 22
fontFamily = "Arial"

[schedules.miniBreaks.audio]
current = "builtin"
builtinName = "soft-gong"
volume = 0.5

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = false

# 喝水提醒
[[attentions]]
id = 0
name = "Drink Water"
enabled = true
times = ["10:00", "14:00", "16:00", "20:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "Hydration Time"
message = "Remember to drink water! Stay hydrated throughout the day."
durationS = 10

[attentions.theme]
background = { current = "solid", solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

# 眼保健操提醒
[[attentions]]
id = 1
name = "Eye Exercise"
enabled = true
times = ["11:00", "15:00", "19:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "Eye Care"
message = "Look at something 20 feet away for 20 seconds (20-20-20 rule)"
durationS = 25

[attentions.theme]
background = { current = "solid", solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 22
fontFamily = "Arial"
```

---

## 最佳实践

1. **从简单开始**：从默认配置开始，逐步自定义
2. **测试设置**：在保存之前使用设置 UI 中的预览/测试按钮
3. **备份配置**：保持配置文件的备份，特别是在重大更改之前
4. **明智地使用严格模式**：仅对绝对需要的休息时间启用严格模式
5. **图像路径**：使用绝对路径来避免图像加载问题
6. **音频音量**：从较低的音量（0.5-0.7）开始，根据需要进行调整
7. **多个休息计划**：根据一天中的不同时间或一周中的不同天使用不同的休息计划
8. **注意事项时间安排**：合理安排注意事项提醒，避免通知疲劳

---

有关更多信息，请参阅：
- [README](../README.zh-CN.md) - 项目概述
- [架构文档](ARCHITECTURE.md) - 技术信息
- [贡献指南](../CONTRIBUTING.md) - 开发指南
