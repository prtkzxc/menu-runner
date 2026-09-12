# Menu Runner

A low-overhead, cross-platform system-tray app for running your own commands. It stays dormant until you click a tray-menu item; it has no timer or background polling loop.

## Run while developing

```sh
npm install
npm run tauri dev
```

## Install on macOS

Build a distributable app with:

```sh
npm run tauri build
```

The resulting `.dmg` is placed in `src-tauri/target/release/bundle/dmg/`. A user can open it, drag **Menu Runner** to Applications, and run it—no Xcode, Node, Rust, or Python is necessary to use the app itself.

## Configure actions

On first launch, Menu Runner creates this file:

```text
~/Library/Application Support/com.local.menurunner/actions.json
```

Restart the app after editing it. Every supported action becomes a native tray-menu item.

```json
{
  "actions": [
    {
      "id": "backup",
      "label": "Run backup",
      "command": "/usr/bin/python3 /Users/me/scripts/backup.py"
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
