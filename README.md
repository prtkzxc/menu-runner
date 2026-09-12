# Menu Runner

A low-overhead, cross-platform system-tray app for running your own commands. It stays dormant until you click a tray-menu item; it has no timer or background polling loop.

## Run while developing

```sh
cd /Users/prat/CODING/menu-runner
npm install
npm run tauri dev
```

Menu Runner starts in the macOS menu bar rather than opening a normal app window. Click its icon at the far right of the menu bar, then choose **Manage actions…** to open the settings page. Press `Ctrl+C` in the terminal to stop development mode.

## Install on macOS

Build a distributable app with:

```sh
npm run tauri build
```

The resulting `.dmg` is placed in `src-tauri/target/release/bundle/dmg/`. A user can open it, drag **Menu Runner** to Applications, and run it—no Xcode, Node, Rust, or Python is necessary to use the app itself.

## Configure actions

Choose **Manage actions…** from the menu bar to add, edit, and delete actions. The minimal settings page includes an action name, command, optional working folder, and platform selector. Saving changes updates the tray menu immediately.

The working folder is where the command starts. For example, use `npm run tauri dev` with `/Users/prat/CODING/menu-runner` as its working folder so npm finds this project's `package.json`. Leave it blank for commands that do not depend on a particular directory, such as `open ~`.

The actions are stored locally in this file, which Menu Runner creates on first launch:

```text
~/Library/Application Support/com.local.menurunner/actions.json
```

You can also edit the JSON directly. Every action supported by the current platform becomes a native tray-menu item.

```json
{
  "actions": [
    {
      "id": "backup",
      "label": "Run backup",
      "command": "/usr/bin/python3 daily.py",
      "working_directory": "/Users/me/scripts"
    },
    {
      "id": "open-project",
      "label": "Open project",
      "command": "open /Users/me/Code/project",
      "platforms": ["macos"]
    },
    {
      "id": "windows-task",
      "label": "Windows-only task",
      "command": "powershell -File C:\\Scripts\\task.ps1",
      "platforms": ["windows"]
    }
  ]
}
```

`platforms` is optional. Use `macos`, `windows`, or `linux` to hide an action where it does not apply. On macOS/Linux commands run through `sh -lc`; on Windows they run through `cmd /C`.

## Security note

`actions.json` intentionally has the same power as running commands in your terminal. Keep it writable only by people you trust, and review actions pulled from a repository before using them.
