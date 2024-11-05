# <img src="./assets/mondrian.ico" width="20" height="20"> Mondrian

_Mondrian_ is a tiling window manager built with Rust for Windows 11.

### 🌟 Key Features

- Automatic window placement with different (and customizable) tiling layouts;
- Customizable keybindings;
- Multi-monitor support;
- Mouse movements support (moving/resizing windows);
- System tray application;
- Fullscreen support;
- Multiple animations.

## Getting Started

### Usage

To start _Mondrian_, just download the `mondrian.exe` executable from the latest [release](https://github.com/policastro/Mondrian/releases) and run it.

By default, the application doesn't log any messages to a file. To enable logging, you can use the `--log` command-line argument: this will create a file called `log/log.log` in the application directory.

#### Moving windows

You can swap two windows in the same monitor just by dragging one of them into the other. By holding `ALT` while dragging, the windows will be swapped and the direction of the tiles will be inverted.

When the window is dragged to another monitor, by default it will be inserted. In this case, you can:

- hold `SHIFT` while dragging the window to swap the windows;
- hold `ALT` while dragging the window to insert the windows and to invert the direction of the tiles.

By changing the `insert_in_monitor` configuration option to `false`, the window will be swapped in the other monitor by default. In this case, you can:

- hold `SHIFT` while dragging the window to insert the windows;
- hold `ALT` while dragging the window to insert the window and to invert the direction of the tiles.

#### Resizing windows

Windows can be resized as usual just by dragging their borders.

### Configuration

_Mondrian_ can be configured by editing the `mondrian.toml` file located in the `~/.config/mondrian` directory. If the configuration file does not exist, it will be created automatically when the program starts. The default configuration is shown below:

```toml
[layout]
tiling_strategy = "golden_ratio" # can be "golden_ratio", "horizontal", "vertical", "twostep" or "squared"
tiles_padding = 12               # padding between tiles, it must be between 0 and 60
border_padding = 18              # padding between border and tiles, it must be between 0 and 60
focalized_padding = 8            # padding for the focalized window, it must be between 0 and 60
insert_in_monitor = true         # if true, when the window is moved to a new monitor, it will be inserted instead of being swapped
animations_enabled = true        # enables or disables the animations
animation_type = "ease_out_back" # can be "linear" or any of the easings functions from https://easings.net/ (in snake_case)
animations_duration = 350        # duration of the animation in milliseconds
animations_framerate = 60        # animations framerate

[layout.golden_ratio] # if tiling_strategy = "golden_ratio"
clockwise = true # true = clockwise, false = counterclockwise
vertical = false # if true, the first split will be vertical
ratio = 65       # ratio of the first split

[layout.horizontal] # if tiling_strategy = "horizontal"
grow_right = true # if true, new windows will be placed on the right side of the screen

[layout.vertical] # if tiling_strategy = "vertical"
grow_down = true # if true, new windows will be placed on the bottom side of the screen

[layout.twostep] # if tiling_strategy = "twostep"
first_step = "right" # first insertion direction, can be <"right"|"left"|"up"|"down">
second_step = "down" # second insertion direction, can be <"right"|"left"|"up"|"down">
ratio = 65           # ratio of the first split

[modules.keybindings]
enabled = true # enables or disables the keybindings module
default_modifier = "ALT" # default modifier for keybindings, it can be "CTRL", "ALT", "SHIFT" or any combination of them (e.g. "CTRL+ALT+SHIFT")
bindings = [
    # { key = "X", action = "quit" },                                   # e.g. when pressing <default_modifier>+X, the action will be "quit"
    # { modifier = "CTRL+ALT", key = "R", action = "refresh-config" },  # e.g. when pressing CTRL+ALT+R, the action will be "refresh-config"
    # Navigation
    { modifier = "CTRL+ALT", key = "left", action = "focus left" },
    { modifier = "CTRL+ALT", key = "right", action = "focus right" },
    { modifier = "CTRL+ALT", key = "up", action = "focus up" },
    { modifier = "CTRL+ALT", key = "down", action = "focus down" },
    # Movement
    { modifier = "CTRL+ALT+SHIFT", key = "left", action = "move left" },
    { modifier = "CTRL+ALT+SHIFT", key = "right", action = "move right" },
    { modifier = "CTRL+ALT+SHIFT", key = "up", action = "move up" },
    { modifier = "CTRL+ALT+SHIFT", key = "down", action = "move down" },
    # Resize
    { modifier = "CTRL+SHIFT", key = "left", action = "resize left 150" },
    { modifier = "CTRL+SHIFT", key = "right", action = "resize right 150" },
    { modifier = "CTRL+SHIFT", key = "up", action = "resize up 150" },
    { modifier = "CTRL+SHIFT", key = "down", action = "resize down 150" },
    # Others
    { key = "r", action = "release" },
    { key = "f", action = "focalize" },
    { key = "m", action = "minimize" },
    { key = "v", action = "invert" },
    { modifier = "ALT+SHIFT", key = "p", action = "pause" },
    { modifier = "ALT+SHIFT", key = "r", action = "refresh-config" },
]

[modules.overlays]
enabled = true               # enables or disables the overlays module
update_while_resizing = true # the overlays will be updated while resizing
active.enabled = true        # shows the active overlay will be shown
active.thickness = 4         # thickness of the border
active.color = "#FE4A49"     # color of the border, as [r, g, b] or as hex string ("#rrggbb")
active.padding = 0           # padding between the overlay and the window
inactive.enabled = true      # shows the inactive overlays
inactive.thickness = 4       # thickness of the border
inactive.color = "#FED766"   # color of the border, as [r, g, b] or as hex string ("#rrggbb")
inactive.padding = 0         # padding between the overlay and the window

# Other settings
[advanced]
detect_maximized_windows = true # if true, maximized windows will be detected and they will not be managed by the tiling engine

[core]
rules = [
    # { title = "Title", exename = "app.exe", classname = "ApplicationWindow" },                       # match any window with a title="Title" and exename="app.exe" and classname="ApplicationWindow"
    # { title = "Title" },                                                                             # match any window with a title="Title" (title, exename, classname are optional, but at least one of them must be specified)
    # { title = "/Title[0-9]/" },                                                                      # match any window with a title that matches the regex "/Title[0-9]/" (you can use regex in title, exename and classname by enclosing them in slashes)
    { exename = "OpenWith.exe" },                                                                    # "Open with" dialog
    { classname = "OperationStatusWindow" },                                                         # Explorer operation status
    { title = "/[Pp]icture.in.[Pp]icture/", classname = "/Chrome_WidgetWin_1|MozillaDialogClass/" }, # PIP Firefox/Chrome
]
```

The available actions that can be binded to keys are:

- `refresh-config`: reloads the configuration and restarts the application;
- `open-config`: opens the configuration file in the default editor;
- `retile`: re-tiles the windows;
- `minimize`: minimizes the focused window;
- `focus <left|right|up|down>`: focuses the window in the specified direction;
- `move <left|right|up|down>`: moves the focused window in the specified direction;
- `resize <left|right|up|down> <40-250>`: resizes the focused window in the specified direction by the specified amount;
- `invert`: inverts the orientation of the focused window and the neighboring windows;
- `release`: removes the focused window from the tiling manager, or adds it back;
- `focalize`: focalizes the focused window (i.e. hides the neighboring windows) or unfocalizes it (i.e. restores the neighboring windows);
- `pause`: pauses/unpauses the application;
- `module <keybindings|overlays>`: pauses/unpauses the specified module;
- `quit`: closes the application;

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

If you want to ignore it temporarily, you can bind the `release` action to a key. Otherwise, you can create a rule in the configuration file (`core.rules`).

## License

This project is licensed under the GPLv3 license. See the `LICENSE.md` for more information.

## Acknowledgments

- [Andrey Sitnik](https://github.com/ai) for the website [easings.net](https://easings.net), which I used as reference for implementing the animations.
