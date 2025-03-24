# Changelog

All notable changes to this project will be documented in this file.

## [1.0.0] - 2025-03-22

### 🚀 Features

- *(overlays)* Add options to configure floating window overlays
- Add options to resize floating windows
- Add "peek" action to restrict tiling to a portion of the screen area
- Handle changes in monitor workareas
- Add per-monitor config options
- Position new windows on the monitor at cursor location
- Respect "move_cursor_on_focus" config for "move"/"insert"/"moveinsert" actions
- *(actions)* Add "dumpstateinfo" action to dump application state to file
- *(overlays)* Add support for colors with alpha value
- Respect "move_cursor_on_focus" config for "amplify" action
- Add option to exclude windows based on their style
- *(actions)* Add action to swap focalized window with others on the same monitor
- *(actions)* [**breaking**] Add action to create a two-window split layout

### 🐛 Bug Fixes

- *(overlays)* Resolve bug preventing immediate color update
- *(overlays)* Correct bug preventing overlays update on virtual desktop switch
- Properly detect close event of windows opened before app startup
- Remove focalized state on restore
- Resolve overlay rendering performance issue on window resize
- Correct wrong log levels enumeration
- *(tm)* Remove floating windows on close
- *(tm)* Resolve issue preventing some closed windows from being removed
- Resolve bug resetting tiles manager state on virtual desktop switch
- Prevent multiple instances from running

### 🚜 Refactor

- Revise internal management of focalized state
- Reorganize module structure
- Make monitor ID generation more consistent

### 📚 Documentation

- Correct typos in animation configuration docs

### ⚡ Performance

- *(tm)* Restart only when configuration changes require it
- Send windows updated event to modules only when necessary
- Pause application when system enters standby, hibernation or session lock
- *(overlays)* Simplify overlays management and reduce unnecessary redraws
- *(overlays)* Selectively suspend overlays during layout updates
- Limit focus history to avoid uncontrolled memory growth
- *(overlays)* [**breaking**] Improve overlay performance and rendering
- Improve internal virtual desktops management

### ⚙️ Miscellaneous Tasks

- Update windows crate to version 0.58.0

## [0.8.0] - 2025-02-28

### 🚀 Features

- *(overlays)* Add configurable border radius
- *(overlays)* [**breaking**] Add specific options for focused windows
- Add config option to prioritize recent focused windows for "focus" and "move" actions

### 🐛 Bug Fixes

- *(configs)* Prevent incorrect overlays defaults when partially defined
- *(overlays)* Correct inconsistent border thickness
- Resolve issue with animation not playing on focalized windows resize
- Prevent "amplify" action on focalized/maximized/floating windows
- Resolve layout issue when maximizing window on a different monitor

### 🚜 Refactor

- *(configs)* [**breaking**] Modify config file schema

### 📚 Documentation

- Add FAQ explaining how "focus"/"move" actions find the target window
- Add JSON Schema definition for config
- Fix links in JSON Schema
- Add reference to JSON Schema in default config file
- Fix "modules.keybindings.bindings" definition in JSON Schema

## [0.7.0] - 2025-02-22

### 🚀 Features

- Add action to swap the current window with the biggest one in the same monitor
- Add option to auto reload configs on change
- Add gray tray icon when app is paused
- *(logs)* Add log file for application errors
- *(tray)* Add tray menu item to open logs folder

### 🐛 Bug Fixes

- Prevent windows flickering on app restart
- Correct typo in default config file
- Prevent Windows Start menu from opening when setting window focus

### 💼 Other

- Use feature flag to build the no-console version of the app

### 🚜 Refactor

- *(configs)* [**breaking**] Require explicit modifiers in keybindings
- *(logs)* Add log rotation with .gz compression

### ⚙️ Miscellaneous Tasks

- Reorganize project structure

## [0.6.1] - 2025-02-17

### 🐛 Bug Fixes

- *(keybindings)* Fix issue preventing some function keys from working

## [0.6.0] - 2025-02-16

### 🚀 Features

- *(bindings)* Add support for function keys in bindings
- Add a config option that moves the cursor to the center of the focused window

### 💼 Other

- Update Cargo.lock

## [0.5.0] - 2025-02-15

### 🚀 Features

- Add Windows virtual desktops support

### 🐛 Bug Fixes

- Address incorrect windows resizing
- Handle move/resize events properly for maximized windows
- Address bugs in some animations
- Prevent tiles manager state reset when new virtual desktop is created
- *(overlays)* Address incorrect overlays sizing
- Ensure pinned windows are placed immediately when creating new virtual desktops

### 💼 Other

- Update deps

### 📚 Documentation

- Fix README
- Update README

### ⚡ Performance

- *(animations)* Prevent moving the window when it's already in position

## [0.4.0] - 2025-02-10

### 🚀 Features

- *(actions)* Add action for inserting windows in a monitor ("insert"/"moveinsert")

### 🚜 Refactor

- Replace HWND type with custom type (wip)

### 📚 Documentation

- Update version in cargo.toml
- Update README

## [0.3.0] - 2025-02-06

### 🚀 Features

- Keep previous window position when window is restored from maximized state
- Set released windows to top most

### 🐛 Bug Fixes

- Prevent wrong detection of window close events
- Allow out-of-bounds windows insertion
- Restore maximized windows when new windows are opened

### 🚜 Refactor

- *(overlays)* Generalize overlay creation
- *(logs)* Centralize Windows events logs
- Change logic to detect window move/resize events

### 🎨 Styling

- Remove unused code

### ⚙️ Miscellaneous Tasks

- Add a new rule to the default config file

## [0.2.0] - 2024-11-16

### 🚀 Features

- Add option to place windows freely based on the cursor position
- *(tray)* Add icons to the system tray application

### 🐛 Bug Fixes

- Define priority of the different move operations

### 💼 Other

- *(deps)* Update deps

### 🚜 Refactor

- Externalize Windows events detection in a new module

## [0.1.2] - 2024-11-06

### 🚜 Refactor

- *(configs)* Change some configs default values/limits
- *(configs)* [**breaking**] Replace action "module" with "pause"
- *(configs)* Add serde serialization for the configs

### 📚 Documentation

- Update README.md

## [0.1.1] - 2024-10-29

### 🐛 Bug Fixes

- Take into account invisible borders to keep windows sizes consistent
- Use cursor to differentiate between move and resize events
- *(animations)* Prevent window automatic resize to minimun dimensions during animation

### 📚 Documentation

- Fix typo

## [0.1.0] - 2024-10-27

### 🚀 Features

- Add tray icon
- Add windows overlay
- Add configurable keybindings
- Add focus change command
- Add command to invert orientation of neighboring windows
- *(overlays)* Reimplement the previous overlays module
- Add command to move the focused window
- Add command to make a window floating
- Add command to focalize a window
- *(animations)* Add animations when moving/resizing windows
- *(monitor-events)* Refresh the configs on monitor layout events
- *(overlays)* Add config option to prevent overlay update while resizing
- *(animations)* Add new window animations

### 🐛 Bug Fixes

- Take into account the borders when moving/resizing windows
- *(animations)* Disable overlay while animating, allowing for smoother animations
- Window resized incorrectly when restored
- Focus shift when removing a window from focalized state

### 🚜 Refactor

- *(win-events)* Win events manager from thread_local to global
- Reorganize core module
- Change "release", "pause", "module ..." syntax

### 📚 Documentation

- Add README and LICENSE

### ⚡ Performance

- *(animator)* Animate windows in batches
- *(animations)* Reduce the number of resize animations

### 🎨 Styling

- Flatten imports in overlay utils

<!-- generated by git-cliff -->
