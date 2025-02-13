# <img src="./assets/mondrian.ico" width="20" height="20"> Mondrian

_Mondrian_ is a tiling window manager built with Rust for Windows 11.

### 🌟 Key Features

- Automatic/manual window placement with different tiling layouts;
- Keybindings;
- Multi-monitor support;
- Mouse movements support (moving/resizing windows);
- System tray application;
- Multiple animations;
- Highly customizable.

## Getting Started

### Usage

To start _Mondrian_, just download the `mondrian.exe` executable from the latest [release](https://github.com/policastro/Mondrian/releases) and run it.

By default, the application doesn't log any messages to a file. To enable logging, you can use the `--log` command-line argument: this will create a file called `log/log.log` in the application directory.

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

_Mondrian_ can be configured by editing the `mondrian.toml` file located in the `~/.config/mondrian` directory.
If the configuration file does not exist, it will be created automatically when the application starts. The configuration generated by the application can be found [here](https://github.com/policastro/mondrian/tree/main/assets/configs/mondrian.toml).

#### Configuration options

| **Option**                               | **Description**                                                             | **Values**                                                                         | **Default**                        |
| ---------------------------------------- | --------------------------------------------------------------------------- | ---------------------------------------------------------------------------------- | ---------------------------------- |
| `layout.tiling_strategy`                 | Tiling strategy                                                             | `"golden_ratio"`, `"horizontal"`, `"vertical"`, `"twostep"`, `"squared"`           | `"golden_ratio"`                   |
| `layout.animations_enabled`              | Enables/disables the animations                                             | `true`, `false`                                                                    | `true`                             |
| `layout.animations_duration`             | Duration of the animations in ms                                            | 100 - 10000                                                                        | 300                                |
| `layout.animations_framerate`            | Framerate of the animations                                                 | 10 - 240                                                                           | 60                                 |
| `layout.animation_type`                  | Animation type                                                              | `"linear"`/any of the easings functions from https://easings.net/ (in snake_case)  | `"linear"`                         |
| `layout.tiles_padding`                   | Padding between tiles                                                       | 0 - 100                                                                            | 12                                 |
| `layout.border_padding`                  | Padding between border and tiles                                            | 0 - 100                                                                            | 18                                 |
| `layout.focalized_padding`               | Padding between border and focalized window                                 | 0 - 120                                                                            | 8                                  |
| `layout.insert_in_monitor`               | If true, moving the window to a new monitor inserts it rather than swapping | `true`, `false`                                                                    | `true`                             |
| `layout.free_move_in_monitor`            | If true, free moving the window to a new monitor is enabled by default      | `true`, `false`                                                                    | `false`                            |
| `layout.golden_ratio.ratio`              | The ratio of the first split                                                | 10 - 90                                                                            | 50                                 |
| `layout.golden_ratio.clockwise`          | Places the windows clockwise or counterclockwise                            | `true`, `false`                                                                    | `true`                             |
| `layout.golden_ratio.vertical`           | If true, the layout will be vertical                                        | `true`, `false`                                                                    | `false`                            |
| `layout.twostep.first_step`              | First insertion direction                                                   | `"right"`, `"left"`, `"up"`, `"down"`                                              | `"right"`                          |
| `layout.twostep.second_step`             | Second insertion direction                                                  | `"right"`, `"left"`, `"up"`, `"down"`                                              | `"down"`                           |
| `layout.twostep.ratio`                   | Ratio of the first split                                                    | 10 - 90                                                                            | 50                                 |
| `layout.horizontal.grow_right`           | If true, the layout will grow on the right side                             | `true`, `false`                                                                    | `true`                             |
| `layout.vertical.grow_down`              | If true, the layout will grow on the bottom side                            | `true`, `false`                                                                    | `true`                             |
| `modules.keybindings.enabled`            | Enables/disables the keybindings module                                     | `true`, `false`                                                                    | `false`                            |
| `modules.keybindings.default_modifier`   | Default modifier for keybindings                                            | any combination of `"ALT"`, `"CTRL"`, `"SHIFT"` joined by `+` (e.g. `"ALT+SHIFT"`) | `ALT`                              |
| `modules.keybindings.bindings`           | Custom keybindings                                                          | check the relative [section](#keybindings-guide) for more info.                    | -                                  |
| `modules.overlays.enabled`               | Enables/disables the overlays module                                        | `true`, `false`                                                                    | `true`                             |
| `modules.overlays.update_while_resizing` | Updates the overlays while resizing                                         | `true`, `false`                                                                    | `true`                             |
| `modules.overlays.active.enabled`        | Enables/disables the overlay for the active window                          | `true`, `false`                                                                    | `true`                             |
| `modules.overlays.active.thickness`      | Thickness of the border                                                     | 0 - 100                                                                            | 4                                  |
| `modules.overlays.active.padding`        | Padding between the overlay and the window                                  | 0 - 30                                                                             | 0                                  |
| `modules.overlays.active.color`          | Color of the overlay                                                        | `[r, g, b]` or as hex string (`"#rrggbb"`)                                         | `[254, 74, 73]` (or `"#FE4A49"`)   |
| `modules.overlays.inactive.enabled`      | Enables/disables the overlays for the inactive windows                      | `true`,`false`                                                                     | `true`                             |
| `modules.overlays.inactive.thickness`    | Thickness of the border                                                     | 0 - 100                                                                            | 4                                  |
| `modules.overlays.inactive.padding`      | Padding between the overlay and the window                                  | 0 - 30                                                                             | 0                                  |
| `modules.overlays.inactive.color`        | Color of the overlay                                                        | `[r, g, b]` or as hex string (`"#rrggbb"`)                                         | `[254, 215, 102]` (or `"#FED766"`) |
| `advanced.detect_maximized_windows`      | Prevents maximized windows from being managed                               | `true`, `false`                                                                    | `true`                             |
| `core.rules`                             | Custom rules to exclude windows from being managed                          | check the relative [section](#core-rules-guide) for more info.                     | -                                  |

All the options are optional and if not specified, the default values will be used.

The only exception is for the `modules.overlays.active.*` and `modules.overlays.inactive.*` options. When at least one of them is specified, the other options will have the following default values:

- `modules.overlays.*.enabled = false`;
- `modules.overlays.*.thickness = 0`;
- `modules.overlays.*.padding = 0`;
- `modules.overlays.*.color = [0, 0, 0]`.

#### Custom keybindings with `modules.keybindings.bindings` <a name="keybindings-guide"></a>

You can specify custom keybindings with the `modules.keybindings.bindings` option.
Each binding has the following format:

```toml
bindings = [
    { modifier = "MODIFIER", key = "KEY", action = "ACTION" }
]
```

The **available modifiers** are `ALT`, `CTRL`, `SHIFT`, `WIN`[^1] or any combination of them joined by `+` (e.g. `ALT+SHIFT`). This parameter is optional and if not specified, the default modifier defined in the `modules.keybindings.default_modifier` option will be used.

The **available keys** are:

- All the alphanumeric keys (`A` to `Z`, `a` to `z`, `0` to `9`);
- The arrow keys (`up`, `down`, `left`, `right`);
- The `SPACE` key;
- The symbols `` ` ``, `'`, `.`, `,`, `;`, `[`, `]`, `-`, `=`, `/`, `\`, .

The **available actions** are:

- `refresh-config`: reloads the configuration and restarts the application;
- `open-config`: opens the configuration file in the default editor;
- `retile`: re-tiles the windows;
- `minimize`: minimizes the focused window;
- `focus <left|right|up|down>`: focuses the window in the specified direction;
- `move <left|right|up|down>`: swaps the focused window with the window in the specified direction;
- `insert <left|right|up|down>`: adds the focused window in the monitor in the specified direction;
- `moveinsert <left|right|up|down>`: first tries the `move` and then the `insert` action if no window is found in the specified direction;
- `resize <left|right|up|down> <40-250>`: resizes the focused window in the specified direction by the specified amount;
- `invert`: inverts the orientation of the focused window and the neighboring windows;
- `release`: removes the focused window from the tiling manager, or adds it back;
- `focalize`: focalizes the focused window (i.e. hides the neighboring windows) or unfocalizes it (i.e. restores the neighboring windows);
- `pause [keybindings|overlays]`: if no parameter is specified, pauses/unpauses the application. Otherwise, pauses/unpauses the specified module;
- `quit`: closes the application.

The syntax of the actions is as follows:

- `action <v1|v2>` means "action v1" or "action v2" (i.e. required parameter);
- `action [v1|v2]` means "action", "action v1" or "action v2" (i.e. optional parameter);

Some examples:

```toml
[modules.keybindings]
enabled = true
default_modifier = "ALT"
bindings = [
    { key = "X", action = "quit" },                                  # when pressing ALT+X, the action will be "quit"
    { modifier = "CTRL+ALT", key = "left", action = "focus left" }   # when pressing CTRL+ALT+Left, the action will be "focus left"
]
```

[^1]: when a keybinding uses `WIN` as its only modifier, the start menu opens. This is a known issue and will be fixed in the future. As a workaround, you can combine `WIN` with other modifiers (e.g. `WIN+CTRL`, `WIN+CTRL+ALT`, ...).

#### Ignore windows with `core.rules` <a name="core-rules-guide"></a>

You can ignore windows with the `core.rules` option.
Each rule has the following format:

```toml
[core]
rules = [
    { title = "TITLE", exename = "EXENAME", classname = "CLASSNAME" }
]
```

You can specify at least one or more parameters, and the rule will be matched if all the parameters match the corresponding window property.
Each parameter can be either a string or a regex (enclosed in slashes).
Some example:

```toml
[core]
rules = [
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

- `layout.animations_enabled = false`: disables the animations;
- `layout.animations_framerate`: you can set this option to reduce the framerate of the animations;
- `modules.overlays.enabled = false`: disables the overlays (both the "active" and "inactives" ones);
- `modules.overlays.update_while_resizing = false`: the overlays will be updated only when the window resize operation is done;
- `modules.overlays.active.enabled = false`: disables the "active" overlay;
- `modules.overlays.inactive.enabled = false`: disables all the "inactives" overlays;
- `advanced.detect_maximized_windows = false`: disables the detection of maximized windows. Disabling this option doesn't work very well when the overlays are enabled.

### 4. How can I ignore a window?

If you want to ignore it temporarily, you can bind the `release` action to a key. Otherwise, you can create a rule in the configuration file (see the `core.rules` [section](#core-rules-guide)).

## License

This project is licensed under the GPLv3 license. See the `LICENSE.md` for more information.

## Acknowledgments

- [Andrey Sitnik](https://github.com/ai) for the website [easings.net](https://easings.net), which I used as reference for implementing the animations.
