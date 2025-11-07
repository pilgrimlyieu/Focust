# Quick Start Guide

<div align="center">

**[简体中文](QUICKSTART.zh-CN.md)** | **[English](QUICKSTART.md)**

</div>

Welcome to Focust! This guide will help you get started with your first break reminders in just a few minutes.

## First Launch

When you first launch Focust, you'll find it in your system tray icon. The application **is already running in the background** and will start reminding you to take breaks based on the default schedule.

### Understanding the Default Setup

Out of the box, Focust is configured with:
- **Mini breaks**: Every 20 minutes (20 seconds duration)
- **Long breaks**: After every 4 mini breaks (5 minutes duration)
- **Active time**: Fullday through the week
- **Language**: Automatically detected from your system

## Basic Configuration

### 1. Adjust Break Intervals

If you want to change how often breaks occur:

1. Go to the **Break Schedules** tab
2. Find the "Default Schedule" section
3. Under **Mini Break**:
   - Change "Interval" to your preferred time (e.g., 30 minutes)
   - Adjust "Duration" if you want longer/shorter breaks
4. Under **Long Break**:
   - Change "After mini breaks" (e.g., 3 instead of 4)
   - Adjust "Duration" (e.g., 10 minutes instead of 5)
5. Click **Save changes** at the top

### 2. Customize Break Appearance

To make breaks more visually appealing:

1. Stay in the **Break Schedules** tab
2. Scroll down to **Theme** section (under Mini Breaks or Long Breaks)
3. Choose a background:
   - Click **Solid Color** and pick a color
   - Or click **Single Image** and browse for a background image
   - Or click **Image Folder** to select a folder of images for random backgrounds
4. Adjust text color, blur, and opacity sliders
5. Click **Save changes**

### 3. Add Break Sounds

To get audio notifications:

1. In the **Break Schedules** tab, find the **Audio** section
2. Select "Built-in Sound" and choose from:
   - Gentle Bell
   - Soft Gong
   - Notification
   - Bright Notification
3. Adjust volume slider
4. Click "Preview" to test the sound
5. Click **Save changes**

### 4. Set Up Timed Reminders (Optional)

For specific time-based reminders (like drinking water):

1. Go to the **Timed Reminders** tab
2. Click **Add Reminder**
3. Fill in:
   - Name: "Water Reminder"
   - Title: "Hydration Time"
   - Message: "Drink a glass of water!"
   - Reminder times: Add times like "10:00", "14:00", "16:00"
4. Choose which days it should appear
5. Click **Save changes**

## Using Focust

### During a Break

When a break appears:
- **Timer** shows time remaining
- **Suggestion** gives you ideas (stretches, eye exercises, etc.)
- **Resume button**: Finish the break early
- **Postpone button**: Delay the break by 5 minutes (configurable)
- **Keyboard shortcuts**: Press Enter to finish, or your configured key to postpone

### System Tray

Focust lives in your system tray (notification area):
- **Left click**: Show settings window
- **Right click**: Menu with options:
  - Show Settings
  - Pause/Resume
  - Quit

### Pausing Breaks

Sometimes you need to pause all breaks (important meeting, presentation, etc.):

1. Click **Pause** button at the top of settings
2. Or right-click tray icon → Pause
3. Click **Resume** when you're ready to continue

### Postponing a Break

If a break interrupts at a bad time:
1. Click **Postpone** button in the break window
2. Or press your configured postpone shortcut (set in General settings)
3. The break will reappear in 5 minutes (or your configured postpone duration)

## Tips for Beginners

### 1. Start with Longer Intervals
If 20-minute intervals feel too frequent, increase to 30 or 45 minutes while you adjust.

### 2. Enable Notifications
Turn on "Notify before" (5-10 seconds warning) so breaks don't surprise you.

### 3. Use Strict Mode Sparingly
Strict mode prevents skipping breaks. Only enable this if you really need the discipline!

### 4. Customize for Your Schedule
Create different break schedules for:
- Work hours (frequent breaks)
- Evening (less frequent)
- Weekends (optional, or different timing)

### 5. Experiment with Themes
Try different background images that relax you. Nature scenes, minimalist patterns, or solid calming colors work well.

## Common Questions

### "How do I test break reminders without waiting?"
- You can use the **Preview** feature in the **Break Schedules** tab to see how your breaks will look and feel without waiting for the actual reminders
- If you want to see the real break/reminder window, or if you have selected "Image Folder" for backgrounds and want to see the random effect:
- Go to the **Advanced Settings** tab and click the icon to show the Debug area, where you can manually enable short breaks, long breaks, reminder windows, or skip the current break (v0.2.3+)

### "The breaks are interrupting important work!"
- Use the **Postpone** feature liberally at first
- Set a **global postpone shortcut** for quick delays
- Consider increasing your break intervals
- Use **system idle detection** (under General) to auto-pause when you're away

### "I want breaks to appear on all my monitors"
- Go to **General Settings** tab
- Enable "Show breaks on all monitors"
- Click **Save changes**

### "How do I make breaks fullscreen?"
- Go to **General Settings** tab
- Set "Break window size" to 100%
- Breaks will now fill the entire screen

### "Can I disable suggestions?"
- Go to **Break Schedules** tab
- Under your schedule's **Suggestions** section
- Uncheck "Show suggestions during breaks"
- Click **Save changes**

### "I want to use my own audio files"
- Go to **Break Schedules** → **Audio**
- Select "Custom File"
- Click **Browse** and select your MP3/WAV/OGG file
- Adjust volume and save

### "Scheduler status frequently pauses on Windows"
- This may be due to the "Do Not Disturb Mode Detection" option being enabled, as Windows automatically enables "Do Not Disturb" in certain scenarios by default
- You can open settings with `Win` + `I`, navigate to "System" → "Notifications", and configure the automatic rules for "Do Not Disturb" yourself
- Or disable this detection option in Focust

## Next Steps

Once you're comfortable with the basics:

1. **Read [CONFIGURATION.md](CONFIGURATION.md)** for detailed options
2. **Create multiple break schedules** for different times of day
3. **Add timed reminders** for specific tasks
4. **Customize themes** for different break types
5. **Experiment with suggestions** by editing the TOML files

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/pilgrimlyieu/Focust/issues)
- **Discussions**: [GitHub Discussions](https://github.com/pilgrimlyieu/Focust/discussions)
- **Documentation**: Check the `docs/` folder in the repository

## Important Notes

- **Config location**: Open via Advanced tab → "Open configuration directory"
- **Logs location**: Open via Advanced tab → "Open log directory" (for debugging)
- **Save regularly**: Always click "Save changes" after modifying settings

---

**Enjoy healthier work habits with Focust! Remember: Regular breaks improve focus, productivity, and well-being.** 🎯💚
