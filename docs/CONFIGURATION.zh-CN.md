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
- [示例](#示例)
- [最佳实践](#最佳实践)

---

## 配置文件位置

Focust 将其配置存储在特定于平台的位置：
- **Windows**：`%APPDATA%\com.fesmoph.focust\config.toml`
- **macOS**：`~/Library/Application Support/com.fesmoph.focust/config.toml`
- **Linux**: `~/.config/com.fesmoph.focust/config.toml`

您可以从设置界面快速打开配置目录：
1. 转到**高级**选项卡
2. 点击**打开配置目录**

## 配置结构

配置文件分为几个主要部分：

```toml
# 通用应用程序设置
checkForUpdates = true
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
```

---

## 通用设置

### `checkForUpdates`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：启动时自动检查应用程序更新
- **注意**：尚未实现

### `autostart`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：系统启动时自动启动应用程序

### `monitorDnd`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：检测到系统勿扰模式时暂停所有休息
- **注意**：尚未实现

### `inactiveS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）
- **说明**：调度器自动暂停前的不活动持续时间。检测到活动时调度器恢复。

### `allScreens`
- **类型**：布尔值
- **默认值**：`false`
- **说明**：在多显示器设置中在所有显示器上显示休息窗口。禁用时，休息仅出现在主显示器上。

### `language`
- **类型**：字符串
- **默认值**：从系统区域设置自动检测
- **选项**：`"en-US"`, `"zh-CN"`
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

**示例：**
```toml
checkForUpdates = true
autostart = false
monitorDnd = true
inactiveS = 300
allScreens = false
language = "zh-CN"
themeMode = "system"
postponeShortcut = "Ctrl+Shift+P"
windowSize = 0.8
```

---

## 休息计划设置

休息计划定义何时以及多久发生一次休息。您可以拥有多个具有不同时间范围和活动日期的休息计划。

### 基本休息计划字段

#### `id`
- **类型**：整数
- **说明**：休息计划的唯一标识符，由应用程序内部使用，不应手动更改！
- **注意**：本篇示例中省略此字段。

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
- **说明**：定义此休息计划在一天中何时活动。将开始和结束时间都设置为「00:00」以表示全天休息计划。

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

#### `miniBreaks.enabled`
- **类型**：布尔值
- **默认值**：`true`

#### `miniBreaks.durationS`
- **类型**：整数（秒）
- **默认值**：`20`
- **说明**：短休息持续多长时间

#### `miniBreaks.postponedS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）
- **说明**：使用推迟功能时推迟休息多长时间

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
background = { solid = "#1f2937" }
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
source = "builtin"
name = "gentle-bell"
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

#### `longBreaks.enabled`
- **类型**：布尔值
- **默认值**：`true`

#### `longBreaks.durationS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）

#### `longBreaks.postponedS`
- **类型**：整数（秒）
- **默认值**：`300`（5 分钟）

#### `longBreaks.strictMode`
- **类型**：布尔值
- **默认值**：`false`

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
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
background = { solid = "#1e293b" }
textColor = "#f1f5f9"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.audio]
source = "builtin"
name = "gentle-bell"
volume = 0.7

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 300
postponedS = 600
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
background = { imagePath = "/path/to/background.jpg" }
textColor = "#ffffff"
blurRadius = 10
opacity = 0.85
fontSize = 28
fontFamily = "Helvetica"

[schedules.longBreaks.audio]
source = "filePath"
path = "/path/to/sound.mp3"
volume = 0.8

[schedules.longBreaks.suggestions]
show = true
```

---

## 注意设置

注意提醒是基于时间的通知，像闹钟一样工作。与休息不同，它们不会中断您的工作 — 只是在特定时间显示简短消息，并可以直接关闭。

### 基本注意字段

#### `name`
- **类型**：字符串
- **必需**：是
- **说明**：提醒的名称（例如，「喝水提醒」「站立提醒」）

#### `enabled`
- **类型**：布尔值
- **默认值**：`true`
- **说明**：此提醒是否活动

#### `times`
- **类型**：字符串数组（24 小时时间格式）
- **必需**：是
- **说明**：提醒应该触发的时间列表

#### `daysOfWeek`
- **类型**：字符串数组
- **默认值**：`["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]`
- **选项**：`"Mon"`, `"Tue"`, `"Wed"`, `"Thu"`, `"Fri"`, `"Sat"`, `"Sun"`

#### `title`
- **类型**：字符串
- **默认值**：`"Attention"`
- **说明**：弹出窗口中显示的标题

#### `message`
- **类型**：字符串
- **必需**：是
- **说明**：要显示的消息内容

#### `durationS`
- **类型**：整数（秒）
- **默认值**：`10`
- **说明**：显示提醒弹出窗口多长时间

注意提醒也支持主题、音频和建议设置。

**注意示例：**
```toml
# 喝水提醒
[[attentions]]
name = "补水"
enabled = true
times = ["10:00", "14:00", "16:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "该喝水了"
message = "保持水分充足！喝一杯水。"
durationS = 10

[attentions.theme]
background = { solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

[attentions.audio]
source = "builtin"
name = "notification"
volume = 0.6

# 站立提醒
[[attentions]]
name = "站起来"
enabled = true
times = ["11:00", "15:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "站起来并活动"
message = "花点时间站起来并伸展您的腿！"
durationS = 15

[attentions.theme]
background = { solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

[attentions.audio]
source = "none"
```

---

## 主题设置

主题设置控制休息窗口的外观。

### `background`
- **类型**：对象
- **说明**：休息窗口的背景源

**结构：**

```toml
[theme.background]
current = "solid"                # 当前活跃的背景类型
solid = "#1f2937"                # 即使切换到图片也会保留
imagePath = "/path/to/image.jpg" # 即使切换到文件夹也会保留
imageFolder = "/path/to/images/" # 即使切换到纯色也会保留
```

**背景类型选项：**

1. **纯色**（`current = "solid"`）：
   ```toml
   [theme.background]
   current = "solid"
   solid = "#1f2937"
   ```

2. **单张图片**（`current = "imagePath"`）：
   ```toml
   [theme.background]
   current = "imagePath"
   imagePath = "/path/to/image.jpg"
   ```
   - 支持格式：JPG、PNG、WebP 等

3. **文件夹中的随机图片**（`current = "imageFolder"`）：
   ```toml
   [theme.background]
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

```toml
# 简约深色主题
[theme.background]
current = "solid"
solid = "#0f172a"

[theme]
textColor = "#e2e8f0"
blurRadius = 0
opacity = 1.0
fontSize = 20
fontFamily = "Segoe UI"

# 带图片的自然主题
[theme.background]
current = "imagePath"
imagePath = "/path/to/forest.jpg"

[theme]
textColor = "#ffffff"
blurRadius = 12
opacity = 0.75
fontSize = 28
fontFamily = "Georgia"

# 充满活力的主题并保留设置
[theme.background]
current = "solid"
solid = "#7c3aed"
imagePath = "/path/to/sunset.jpg" # 保留
imageFolder = "C:\\Wallpapers"    # 保留

[theme]
textColor = "#fef3c7"
blurRadius = 0
opacity = 0.95
fontSize = 26
fontFamily = "Tahoma"
```

---

## 音频设置

音频设置控制休息通知声音。

**结构：**

```toml
[audio]
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

```toml
# 无音频
[audio]
current = "none"
volume = 0.6

# 内置声音
[audio]
current = "builtin"
builtinName = "gentle-bell"
volume = 0.7

# 自定义声音文件
[audio]
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
- **说明**：在休息期间是否显示建议

**建议示例：**

建议存储在单独的 TOML 文件中：`suggestions.toml`

**结构：**
```toml
[byLanguage.en-US]
suggestions = [
    "Look away from your screen and focus on a distant object.",
    "Roll your shoulders back and take a deep breath.",
    "Drink a glass of water.",
    ...
]

[byLanguage.zh-CN]
suggestions = [
    "将目光从屏幕移开，专注于远处的物体。",
    "向后转动肩膀，深呼吸。",
    "喝一杯水。",
    ...
]
```

您可以为不同语言定义建议列表。应用程序将根据当前语言选择适当的建议。

---

## 示例

### 示例一：最小配置

```toml
# 通用设置
checkForUpdates = true
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
strictMode = false
intervalS = 1200

[schedules.miniBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.miniBreaks.theme.background]
solid = "#1f2937"

[schedules.miniBreaks.audio]
source = "none"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
id = 1
enabled = true
durationS = 300
postponedS = 300
strictMode = false
afterMiniBreaks = 4

[schedules.longBreaks.theme]
textColor = "#f8fafc"
blurRadius = 8
opacity = 0.9
fontSize = 24
fontFamily = "Arial"

[schedules.longBreaks.theme.background]
solid = "#1f2937"

[schedules.longBreaks.audio]
source = "none"
volume = 0.6

[schedules.longBreaks.suggestions]
show = true
```

### 示例二：综合工作设置

```toml
checkForUpdates = true
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
strictMode = false
intervalS = 1200 # 20 分钟

[schedules.miniBreaks.theme]
background = { imagePath = "/home/user/wallpapers/calm-lake.jpg" }
textColor = "#e0f2fe"
blurRadius = 10
opacity = 0.85
fontSize = 24
fontFamily = "Helvetica"

[schedules.miniBreaks.audio]
source = "builtin"
name = "gentle-bell"
volume = 0.6

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = true
durationS = 600  # 10 分钟
postponedS = 600
strictMode = true
afterMiniBreaks = 3

[schedules.longBreaks.theme]
background = { imagePath = "/home/user/wallpapers/mountain.jpg" }
textColor = "#ffffff"
blurRadius = 15
opacity = 0.8
fontSize = 28
fontFamily = "Georgia"

[schedules.longBreaks.audio]
source = "filePath"
path = "/home/user/sounds/meditation-bell.mp3"
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
strictMode = false
intervalS = 1800  # 30 分钟

[schedules.miniBreaks.theme]
background = { Solid = "#1e3a8a" }
textColor = "#dbeafe"
blurRadius = 5
opacity = 0.9
fontSize = 22
fontFamily = "Arial"

[schedules.miniBreaks.audio]
source = "builtin"
name = "soft-gong"
volume = 0.5

[schedules.miniBreaks.suggestions]
show = true

[schedules.longBreaks]
enabled = false

# 喝水提醒
[[attentions]]
name = "Drink Water"
enabled = true
times = ["10:00", "14:00", "16:00", "20:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
title = "Hydration Time"
message = "Remember to drink water! Stay hydrated throughout the day."
durationS = 10

[attentions.theme]
background = { solid = "#0ea5e9" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 20
fontFamily = "Arial"

[attentions.audio]
source = "builtin"
name = "notification"
volume = 0.5

# 眼保健操提醒
[[attentions]]
name = "Eye Exercise"
enabled = true
times = ["11:00", "15:00", "19:00"]
daysOfWeek = ["Mon", "Tue", "Wed", "Thu", "Fri"]
title = "Eye Care"
message = "Look at something 20 feet away for 20 seconds (20-20-20 rule)"
durationS = 25

[attentions.theme]
background = { solid = "#10b981" }
textColor = "#ffffff"
blurRadius = 5
opacity = 0.95
fontSize = 22
fontFamily = "Arial"

[attentions.audio]
source = "builtin"
name = "bright-notification"
volume = 0.6
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
