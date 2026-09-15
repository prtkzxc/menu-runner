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

### From a GitHub release

Open the repository's [Releases page](https://github.com/prtkzxc/menu-runner/releases), download the DMG that matches your Mac, then open it and drag **Menu Runner** to **Applications**. No developer tools are required.

The initial release uses ad-hoc signing. macOS may ask you to approve it in **System Settings → Privacy & Security** before first launch. A future signed and notarized release will remove that extra step.

### Build locally

Build a distributable app with:

```sh
npm run tauri build
```

The resulting `.dmg` is placed in `src-tauri/target/release/bundle/dmg/`. A user can open it, drag **Menu Runner** to Applications, and run it—no Xcode, Node, Rust, or Python is necessary to use the app itself.

## Publishing a release

Pushing a tag such as `v0.1.0` starts the GitHub Actions release workflow. It builds Apple Silicon and Intel DMGs, then attaches them to a public GitHub Release. The workflow uses the version in `src-tauri/tauri.conf.json`, so update that version before tagging a new release.

To produce signed and notarized releases, add these repository secrets after enrolling in the Apple Developer Program: `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `KEYCHAIN_PASSWORD`, `APPLE_ID`, `APPLE_PASSWORD` (an app-specific password), and `APPLE_TEAM_ID`. Until then, the workflow creates ad-hoc signed builds; they work, but macOS may require the user to approve the first launch.

## In-app updates

Choose **Check for Updates…** from the menu bar whenever you want to check GitHub Releases. The app is otherwise idle; it does not poll in the background. Available releases are cryptographically verified with Tauri's update-signing key before installation, then Menu Runner restarts.

The private update key is stored only as the `TAURI_SIGNING_PRIVATE_KEY` GitHub Actions secret. Keep a secure backup of the matching local key at `~/.tauri/menu-runner.key`; losing it prevents future releases from updating existing installations.

## Configure actions

Choose **Manage actions…** from the menu bar to add, edit, duplicate, and delete actions. The minimal settings page includes an action name, command, optional working folder, and platform selector. Duplicating copies every setting into a new action named `Copy of …`. Saving changes updates the tray menu immediately.

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
