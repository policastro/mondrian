# <img src="./assets/mondrian.ico" width="20" height="20"> Mondrian

_Mondrian_ is a tiling window manager built with Rust for Windows 11.

## 🌟 Key Features

- Automatic/manual window placement with different tiling layouts;
- Keybindings;
- Multi-monitor support;
- Mouse movements support (moving/resizing windows);
- Compatible with Virtual Desktops;
- System tray application;
- Multiple animations;
- Highly customizable.

## Getting Started

### Usage

To start _Mondrian_, just download the `mondrian.exe` executable from the latest [release](https://github.com/policastro/Mondrian/releases) and run it.

The application takes the following arguments (all of them are optional):

    ./mondrian.exe --log <LOG_TYPE> --loglevel <LOGLEVEL>

Where:

- `<LOG_TYPE>` can be 0 (no log file is created), 1 (error log files is created) or 2 (all log files are created). By default, it is set to 1.
- `<LOG_LEVEL>` can be 0 (off), 1 (error), 2 (warn), 3 (info), 4 (debug) or 5 (trace). By default, it is set to 3.

All the log files will be stored in the application directory under the `logs` subfolder. When a log file reaches 10MB, it will be archived in a `.gz` file (up to three previous versions).

#### Moving windows

You can swap two windows in the same monitor just by dragging one of them into the other. While dragging, you can:

- hold `ALT`, to swap the windows and to invert the direction of the tiles;

When the window is dragged to another monitor, by default it will be inserted. In this case, you can:

- hold `SHIFT` while dragging the window to swap the windows;
- hold `ALT` while dragging the window to insert the windows and to invert the direction of the tiles.

By changing the `insert_in_monitor` configuration option to `false`, the window will be swapped in the other monitor by default. In this case, you can:

- hold `SHIFT` while dragging the window to insert the windows;
- hold `ALT` while dragging the window to insert the window and to invert the direction of the tiles.

If you drag a window while holding `CTRL`, you can place the window freely based on the cursor position relative to an other window.
In particular:

- if the cursor is at the top of an other window (i.e. <=20% of its height), the moving window will be placed above it;
- if the cursor is at the bottom of an other window (i.e. >=80% of its height), the moving window will be placed below it;
- if the cursor is to the left of an other window (i.e. <=50% of its width), the moving window will be placed to the left of it;
- if the cursor is to the right of an other window (i.e. >50% of its width), the moving window will be placed to the right of it.

Holding `CTRL` has the same effect when dragging the window to another monitor (by default).

You can set the `free_move_in_monitor` configuration option to `true` if you want to place the window freely in another monitor without holding `CTRL` (in this case, holding `CTRL` will position the window automatically).

Below a table that shows the keybindings for moving/swapping windows in different monitors, depending on the values of the `insert_in_monitor` and `free_move_in_monitor` configuration options:

| `insert_in_monitor` | `free_move_in_monitor` |     No key     |     `CTRL`     |   `SHIFT`    |            `ALT`             |
| :-----------------: | :--------------------: | :------------: | :------------: | :----------: | :--------------------------: |
|       `false`       |     `false`/`true`     |     swaps      | inserts freely | inserts auto | inserts auto + inverts tiles |
|       `true`        |        `false`         |  inserts auto  | inserts freely |    swaps     | inserts auto + inverts tiles |
|       `true`        |         `true`         | inserts freely |  inserts auto  |    swaps     | inserts auto + inverts tiles |

If more than one modifier is held, the precedence order is as follows: `ALT > CTRL > SHIFT`.

#### Resizing windows

Windows can be resized as usual just by dragging their borders.

### Configuration

> [!WARNING]
> The application is still evolving and changes between versions may introduce breaking changes. Be sure to check the release notes before updating.

_Mondrian_ can be configured by editing the `mondrian.toml` file located in the `~/.config/mondrian` directory.
If the configuration file does not exist, it will be created automatically when the application starts. The configuration generated by the application can be found [here](https://github.com/policastro/mondrian/tree/main/assets/configs/mondrian.toml).

#### Configuration options

| **Option**                               | **Description**                                                                                       | **Values**                                                                                                               | **Default**                        |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | ---------------------------------- |
| `layout.tiling_strategy`                 | Tiling strategy                                                                                       | `"golden_ratio"`, `"horizontal"`, `"vertical"`, `"twostep"`, `"squared"`                                                 | `"golden_ratio"`                   |
| `layout.paddings.tiles`                  | Padding between tiles (in px)                                                                         | 0 - 100                                                                                                                  | 12                                 |
| `layout.paddings.borders`                | Padding between border and tiles (in px)                                                              | 0 - 100                                                                                                                  | 18                                 |
| `layout.paddings.focalized`              | Padding between border and focalized window (in px)                                                   | 0 - 120                                                                                                                  | 8                                  |
| `layout.strategy.golden_ratio.ratio`     | The ratio of the first split                                                                          | 10 - 90                                                                                                                  | 50                                 |
| `layout.strategy.golden_ratio.clockwise` | Places the windows clockwise or counterclockwise                                                      | `true`, `false`                                                                                                          | `true`                             |
| `layout.strategy.golden_ratio.vertical`  | If true, the layout will be vertical                                                                  | `true`, `false`                                                                                                          | `false`                            |
| `layout.strategy.twostep.first_step`     | First insertion direction                                                                             | `"right"`, `"left"`, `"up"`, `"down"`                                                                                    | `"right"`                          |
| `layout.strategy.twostep.second_step`    | Second insertion direction                                                                            | `"right"`, `"left"`, `"up"`, `"down"`                                                                                    | `"down"`                           |
| `layout.strategy.twostep.ratio`          | Ratio of the first split                                                                              | 10 - 90                                                                                                                  | 50                                 |
| `layout.strategy.horizontal.grow_right`  | If true, the layout will grow on the right side                                                       | `true`, `false`                                                                                                          | `true`                             |
| `layout.strategy.vertical.grow_down`     | If true, the layout will grow on the bottom side                                                      | `true`, `false`                                                                                                          | `true`                             |
| `general.history_based_navigation`       | If true, navigation will prioritize the most recently focused window in the given direction           | `true`, `false`                                                                                                          | `false`                            |
| `general.insert_in_monitor`              | If true, moving the window to a new monitor inserts it rather than swapping                           | `true`, `false`                                                                                                          | `true`                             |
| `general.free_move_in_monitor`           | If true, free moving the window to a new monitor is enabled by default                                | `true`, `false`                                                                                                          | `false`                            |
| `general.detect_maximized_windows`       | Prevents maximized windows from being managed                                                         | `true`, `false`                                                                                                          | `true`                             |
| `general.move_cursor_on_focus`           | Moves the mouse cursor to the center of the focused window                                            | `true`, `false`                                                                                                          | `false`                            |
| `general.auto_reload_configs`            | Reloads the configuration on changes                                                                  | `true`, `false`                                                                                                          | `true`                             |
| `general.animations.type`                | Animation type                                                                                        | `"linear"`/any of the easings functions from https://easings.net/ (in snake_case)                                        | `"linear"`                         |
| `general.animations.enabled`             | Enables/disables the animations                                                                       | `true`, `false`                                                                                                          | `true`                             |
| `general.animations.duration`            | Duration of the animations in ms                                                                      | 100 - 10000                                                                                                              | 300                                |
| `general.animations.framerate`           | Framerate of the animations                                                                           | 10 - 240                                                                                                                 | 60                                 |
| `general.floating_wins.topmost`          | If true, floating windows will always be on top of other windows                                      | `true`, `false`                                                                                                          | `true`                             |
| `general.floating_wins.size`             | How floating windows should be resized                                                                | `"preserve"` (keep previous size)<br>`"relative"` (resize based on monitor resolution)<br>`"fixed"` (fixed pixel values) | `"relative"`                       |
| `general.floating_wins.size_ratio`       | The ratio of the floating window's size relative to the monitor (used only if `size` is `"relative"`) | [0.1 - 1.0, 0.1 - 1.0]                                                                                                   | [0.5, 0.5]                         |
| `general.floating_wins.size_fixed`       | The fixed pixel values of the floating window's size (used only if `size` is `"fixed"`)               | [100 - 10000, 100 - 10000]                                                                                               | [700, 400]                         |
| `modules.keybindings.enabled`            | Enables/disables the keybindings module                                                               | `true`, `false`                                                                                                          | `false`                            |
| `modules.keybindings.bindings`           | Custom keybindings                                                                                    | check the relative [section](#keybindings-guide) for more info.                                                          | -                                  |
| `modules.overlays.enabled`               | Enables/disables the overlays module                                                                  | `true`, `false`                                                                                                          | `true`                             |
| `modules.overlays.update_while_resizing` | Updates the overlays while resizing                                                                   | `true`, `false`                                                                                                          | `true`                             |
| `modules.overlays.thickness`             | Thickness of the border (in px)                                                                       | 0 - 100                                                                                                                  | 4                                  |
| `modules.overlays.padding`               | Padding between the overlay and the window (in px)                                                    | 0 - 30                                                                                                                   | 0                                  |
| `modules.overlays.border_radius`         | Border radius of the overlay                                                                          | 0 - 100                                                                                                                  | 15                                 |
| `modules.overlays.active.enabled`        | Enables/disables the overlay for the window in focus                                                  | `true`, `false`                                                                                                          | `true`                             |
| `modules.overlays.active.color`          | Color of the overlay                                                                                  | `[r, g, b]` or as hex string (`"#rrggbb"`)                                                                               | `[155, 209, 229]` (or `"#9BD1E5"`) |
| `modules.overlays.inactive.enabled`      | Enables/disables the overlays for the windows not in focus                                            | `true`, `false`                                                                                                          | `true`                             |
| `modules.overlays.inactive.color`        | Color of the overlay                                                                                  | `[r, g, b]` or as hex string (`"#rrggbb"`)                                                                               | `[156, 156, 156]` (or `"#9C9C9C`)  |
| `modules.overlays.focalized.enabled`     | Enables/disables the overlay for the focalized windows in focused                                     | `true`,`false`                                                                                                           | `true`                             |
| `modules.overlays.focalized.color`       | Color of the overlay                                                                                  | `[r, g, b]` or as hex string (`"#rrggbb"`)                                                                               | `[234, 153, 153]` (or `"#EA9999"`) |
| `modules.overlays.floating.enabled`      | Enables/disables the overlay for the floating windows in focused                                      | `true`,`false`                                                                                                           | `true`                             |
| `modules.overlays.floating.color`        | Color of the overlay                                                                                  | `[r, g, b]` or as hex string (`"#rrggbb"`)                                                                               | `[220, 198, 224]` (or `"#DCC6E0"`) |
| `core.ignore_rules`                      | Custom rules to exclude windows from being managed                                                    | check the relative [section](#core-ignore-rules-guide) for more info.                                                    | -                                  |

All the options are optional and if not specified, the default values will be used.

#### Custom keybindings with `modules.keybindings.bindings` <a name="keybindings-guide"></a>

You can specify custom keybindings with the `modules.keybindings.bindings` option.
Each binding has the following format:

```toml
bindings = [
    { modifiers = "MODIFIERS", key = "KEY", action = "ACTION" } # "modifiers" can be also spelled as "modifier" or "mod"
]
```

The **available modifiers** are `ALT`, `CTRL`, `SHIFT`, `WIN` or any combination of them joined by `+` (e.g. `ALT+SHIFT`).
This parameter is required, except when the `key` is a function key, in which case it can be omitted.

The **available keys** are:

- alphanumeric keys (`A` to `Z`, `a` to `z`, `0` to `9`);
- arrow keys (`up`, `down`, `left`, `right`);
- `SPACE` key;
- symbols `` ` ``, `'`, `.`, `,`, `;`, `[`, `]`, `-`, `=`, `/`, `\`;
- function keys (`F1` to `F24`).

The keys and modifiers are case-insensitive.

The **available actions** are:

- `refresh-config`: reloads the configuration and restarts the application;
- `open-config`: opens the configuration file in the default editor;
- `retile`: re-tiles the windows;
- `minimize`: minimizes the focused window;
- `focus <left|right|up|down>`: focuses the window in the specified direction;
- `move <left|right|up|down>`: swaps the focused window with the window in the specified direction;
- `insert <left|right|up|down>`: adds the focused window in the monitor in the specified direction;
- `moveinsert <left|right|up|down>`: first tries the `move` and then the `insert` action if no window is found in the specified direction;
- `resize <left|right|up|down> <40-250>`: resizes the focused window in the specified direction by the specified amount (in pixels);
- `invert`: inverts the orientation of the focused window and the neighboring windows;
- `release`: removes the focused window from the tiling manager, or adds it back;
- `focalize`: focalizes the focused window (i.e. hides the neighboring windows) or unfocalizes it (i.e. restores the neighboring windows);
- `amplify`: swaps the focused window with the biggest one in the same monitor;
- `pause [keybindings|overlays]`: if no parameter is specified, pauses/unpauses the application. Otherwise, pauses/unpauses the specified module;
- `quit`: closes the application.

The syntax of the actions is as follows:

- `action <v1|v2>` means "action v1" or "action v2" (i.e. required parameter);
- `action [v1|v2]` means "action", "action v1" or "action v2" (i.e. optional parameter);

Some examples:

```toml
[modules.keybindings]
enabled = true
bindings = [
    { key = "F4", action = "quit" },                                  # F4 to "quit"
    { modifiers = "WIN+ALT",  key = "F4", action = "release" },       # WIN+ALT+F4 to "release"
    { modifiers = "CTRL+ALT", key = "left", action = "focus left" }   # CTRL+ALT+Left to "focus left"
]
```

#### Ignore windows with `core.ignore_rules` <a name="core-ignore-rules-guide"></a>

You can ignore windows with the `core.ignore_rules` option.
Each rule has the following format:

```toml
[core]
ignore_rules = [
    { title = "TITLE", exename = "EXENAME", classname = "CLASSNAME" }
]
```

You can specify at least one or more parameters, and the rule will be matched if all the parameters match the corresponding window property.
Each parameter can be either a string or a regex (enclosed in slashes).
Some example:

```toml
[core]
ignore_rules = [
    { title = "Title", exename = "app.exe", classname = "ApplicationWindow" },    # match any window with a title="Title" and exename="app.exe" and classname="ApplicationWindow"
    { title = "Title" },                                                          # match any window with a title="Title"
    { title = "/Title[0-9]/" }                                                    # match any window with a title that matches the regex "Title[0-9]"
]
```

## FAQ

### 1. Why another tiling window manager?

It sounded like a fun project to build, and I used it to learn Rust and the Win32 API.

In the beginning, I just wanted to build a simple tiling window manager for Windows, which allowed me to:

- automatically place windows in the correct positions, in multiple monitors;
- use the mouse to move and resize windows.

Then, I started working on it and new features were added. In any case, the main idea is to have an application that "just works" out-of-box, without any special configuration.

### 2. Are there any alternatives?

Yes, there are others tiling window managers for Windows out there. In particular, I used [komorebi](https://github.com/LGUG2Z/komorebi) and [GlazeWM](https://github.com/glzr-io/glazewm) before building this project. Both of them are really good and with great features, and they are in active development. If you need a more mature and established TWM, I recommend trying them.

### 3. Are there any options to improve the performance?

There are different configurations options that can improve the performances. Here the most important ones:

- `general.animations.enabled = false`: disables the animations;
- `general.animations.framerate`: you can set this option to reduce the framerate of the animations;
- `modules.overlays.enabled = false`: disables the overlays;
- `modules.overlays.update_while_resizing = false`: the overlays will be updated only when the window resize operation is done;
- `general.detect_maximized_windows = false`: disables the detection of maximized windows. Disabling this feature may cause issues when overlays are enabled.

### 4. How can I ignore a window?

If you want to ignore it temporarily, you can bind the `release` action to a key. Otherwise, you can create a rule in the configuration file (see the `core.ignore_rules` [section](#core-ignore-rules-guide)).

### 5. How do the `move`/`focus` actions determine the target window?

The target window is selected from those touching the currently focused window in the specified direction.
If there are multiple candidates, the selection depends on the value of the `general.history_based_navigation` config option:

- `false`: the window at the top is chosen for `left`/`right` directions, or the leftmost window for `up`/`down` directions;
- `true`: the most recently focused window among the candidates will be selected.

#### Example

Imagine a layout like this:

```
+-----------------+
|         |   C   |
|    A    |-------+
|         |       |
|---------|   D   |
|         |       |
|    B    |-------+
|         |   E   |
+----+------------+
```

If the currently focused window is _B_, and you use the `focus right` command, windows _D_ and _E_ will be considered as adjacent in the right direction:

- If `general.history_based_navigation = false`, _D_ will be selected, as it is the window at the top;
- If `general.history_based_navigation = true`, _E_ will be selected if it was the most recently focused window.

## License

This project is licensed under the GPLv3 license. See the `LICENSE.md` for more information.

## Acknowledgments

- [Andrey Sitnik](https://github.com/ai) for the website [easings.net](https://easings.net), which I used as reference for implementing the animations.
