//! Screenshots of the app for the documentation, taken by the app itself: marco-sparko --profile demo --screenshots
//!
//! The shots are listed in a JSON file, docs/screenshots.json by default, see ShotFile. For each one the app goes to
//! the home page, follows the shot's steps, e.g. clicking on what has a given text, as someone using it would, then
//! saves a picture of the window, with its frame, to the shot's file. Then it exits.
//!
//! Use the demo profile, a redacted copy of a real one, see redact.rs, which is offline, so that the pictures do not
//! identify anyone, and are the same each time. The window is the size the file gives, which is not remembered as the
//! size of the app's window, and the log panel is hidden. See docs/Screenshots.md.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{anyhow, bail};
use dioxus::desktop::{DesktopService, LogicalSize};
use dioxus::prelude::*;
use serde::Deserialize;

/// The shot list used when --screenshots is not given a file.
pub const DEFAULT_FILE: &str = "docs/screenshots.json";
/// How long the app is given to settle after each step, e.g. for a page to draw.
const SETTLE: Duration = Duration::from_millis(800);
/// How long a wait for some text waits at most.
const WAIT_FOR_LIMIT: Duration = Duration::from_secs(60);

static RUNNING: AtomicBool = AtomicBool::new(false);

/// Whether screenshots are being taken, so that e.g. the log panel is hidden.
pub fn running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

/// Whether a module's list of pages is hidden, while a picture is taken, unless the shot is about it, see Shot::pages.
/// It is shown while the shot's steps are done, as they may click on it. A signal, so that the page is drawn again.
pub static PAGE_LIST_HIDDEN: GlobalSignal<bool> = Signal::global(|| false);

/// The shot list.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShotFile {
    /// The size of the app's window, its content, in logical pixels.
    pub window: WindowSize,
    pub shots: Vec<Shot>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Shot {
    /// Where the picture is saved, relative to the directory the app is run in, e.g. docs/readMe/Screenshot6.png
    file: String,
    /// Whether the help panel is shown, it is hidden unless the shot is of the help.
    #[serde(default)]
    help: bool,
    /// Whether a module's list of pages on the left is shown, it is hidden, as it takes up space, unless the shot is
    /// about it.
    #[serde(default)]
    pages: bool,
    /// Whose window is pictured, the main window, or the newest bill's.
    #[serde(default)]
    window: ShotWindow,
    /// What is done before the picture is taken, starting from the home page.
    #[serde(default)]
    steps: Vec<Step>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ShotWindow {
    #[default]
    Main,
    /// The newest of the windows opened from the main one, e.g. a bill's.
    Newest,
}

/// One step of a shot, as in the file, e.g. {"click": "Meters"}.
#[derive(Deserialize)]
#[serde(untagged, deny_unknown_fields)]
enum Step {
    /// Clicks on the innermost visible element whose text is this, e.g. a button, link or page in the list.
    Click { click: String },
    /// Clicks on the index'th element the CSS selector selects, the first if index is not given.
    Select {
        select: String,
        #[serde(default)]
        index: usize,
    },
    /// Clicks on the element the CSS selector selects in the first table row which has this text, ignoring case, e.g.
    /// the link of the import meter, {"row": "import", "select": ".link"}.
    Row { row: String, select: String },
    /// Types value into the index'th element the CSS selector selects, e.g. a date in the date picker.
    Type {
        input: String,
        #[serde(default)]
        index: usize,
        value: String,
    },
    /// Waits for this many milliseconds.
    Wait { wait: u64 },
    /// Waits until this text is on the page, e.g. once some data has been loaded.
    WaitFor {
        #[serde(rename = "waitFor")]
        wait_for: String,
    },
}

/// Reads the shot list, for its window size, before the window opens.
pub fn read_shot_file(path: &Path) -> anyhow::Result<ShotFile> {
    let text = std::fs::read_to_string(path).map_err(|error| anyhow!("Unable to read the shot list {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| anyhow!("The shot list {} is not valid: {error}", path.display()))
}

/// Takes the screenshots in the shot list at path, then exits the app. Called by App, in the main window, when
/// --screenshots is given.
pub fn use_screenshots(path: Option<PathBuf>) {
    use_future(move || {
        let path = path.clone();
        async move {
            let Some(path) = path else {
                return;
            };
            RUNNING.store(true, Ordering::SeqCst);
            let result = take_all(&path).await;
            let status = match result {
                Ok(count) => {
                    println!("Took {count} screenshots from {}", path.display());
                    0
                },
                Err(error) => {
                    eprintln!("Screenshots failed: {error:#}");
                    1
                },
            };
            std::process::exit(status);
        }
    });
}

async fn take_all(path: &Path) -> anyhow::Result<usize> {
    let shot_file = read_shot_file(path)?;
    check_permission()?;
    let main = dioxus::desktop::window();
    main.window.set_inner_size(LogicalSize::new(shot_file.window.width, shot_file.window.height));
    // The app is ready once it shows its navigation bar.
    wait_for_selector(".nav", WAIT_FOR_LIMIT).await?;
    let help_was_shown = crate::help::is_shown();
    let mut taken = 0;
    for (index, shot) in shot_file.shots.iter().enumerate() {
        println!("Shot {} of {}: {}", index + 1, shot_file.shots.len(), shot.file);
        if let Err(error) = take(shot, &main).await {
            eprintln!("  Not taken: {error:#}");
        }
        else {
            taken += 1;
        }
        crate::components::child_windows::close_all();
        tokio::time::sleep(SETTLE).await;
    }
    crate::help::show_for_now(help_was_shown);
    Ok(taken)
}

async fn take(shot: &Shot, main: &DesktopService) -> anyhow::Result<()> {
    crate::help::show_for_now(shot.help);
    click("Home").await?;
    tokio::time::sleep(SETTLE).await;
    for step in &shot.steps {
        match step {
            Step::Click { click: text } => click(text).await?,
            Step::Select { select, index } => run(&format!(
                "const element = document.querySelectorAll({})[{index}]; if (!element) {{ return false; }} element.click(); return true;",
                json(select),
            ), &format!("nothing is selected by {select} at {index}")).await?,
            Step::Row { row, select } => run(&format!(
                "const want = {}.toLowerCase();
                 const rows = [...document.querySelectorAll('tr')].filter((row) => row.innerText.toLowerCase().includes(want));
                 // The innermost, not e.g. a row of a table the rows are in.
                 const row = rows.find((row) => !rows.some((other) => other !== row && row.contains(other)));
                 const element = row && row.querySelector({});
                 if (!element) {{ return false; }} element.click(); return true;",
                json(row), json(select),
            ), &format!("there is no {select} in a row with the text {row:?}")).await?,
            Step::Type { input, index, value } => run(&format!(
                "const element = document.querySelectorAll({})[{index}]; if (!element) {{ return false; }}
                 element.value = {}; element.dispatchEvent(new Event('input', {{ bubbles: true }}));
                 element.dispatchEvent(new Event('change', {{ bubbles: true }})); return true;",
                json(input), json(value),
            ), &format!("nothing is selected by {input} at {index}")).await?,
            Step::Wait { wait } => tokio::time::sleep(Duration::from_millis(*wait)).await,
            Step::WaitFor { wait_for } => wait_for_text(wait_for, WAIT_FOR_LIMIT).await?,
        }
        tokio::time::sleep(SETTLE).await;
    }
    // Only while the picture is taken, as the steps may have clicked on it.
    if !shot.pages {
        *PAGE_LIST_HIDDEN.write() = true;
        tokio::time::sleep(SETTLE).await;
    }
    let result = picture(shot, main);
    *PAGE_LIST_HIDDEN.write() = false;
    result
}

/// Takes the picture of the shot's window.
fn picture(shot: &Shot, main: &DesktopService) -> anyhow::Result<()> {
    let window = match shot.window {
        ShotWindow::Main => main.window.clone(),
        ShotWindow::Newest => crate::components::child_windows::newest()
            .ok_or_else(|| anyhow!("there is no window opened from the main one"))?
            .window.clone(),
    };
    capture(&window, Path::new(&shot.file))
}

fn json(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// Runs a script in the main window which returns whether it did what it should, failing with problem if it has not
/// within CLICK_LIMIT, e.g. as what it clicks on has not appeared yet.
async fn run(script: &str, problem: &str) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    loop {
        match run_once(script, problem).await {
            Err(_) if started.elapsed() < CLICK_LIMIT => tokio::time::sleep(Duration::from_millis(250)).await,
            result => return result,
        }
    }
}

async fn run_once(script: &str, problem: &str) -> anyhow::Result<()> {
    match document::eval(script).join::<bool>().await {
        Ok(true) => Ok(()),
        Ok(false) => bail!("{problem}"),
        Err(error) => bail!("{problem}: {error:?}"),
    }
}

/// How long a click waits for what it clicks on to appear, e.g. while a page loads.
const CLICK_LIMIT: Duration = Duration::from_secs(15);

/// Clicks on the innermost visible element whose text is text, once there is one, see run.
async fn click(text: &str) -> anyhow::Result<()> {
    run(&format!(
        "const want = {};
         const matches = (element) => element.offsetParent !== null && (element.innerText || '').trim() === want;
         const element = [...document.querySelectorAll('body *')]
             .filter(matches)
             .find((element) => ![...element.children].some(matches));
         if (!element) {{ return false; }}
         element.click();
         return true;",
        json(text),
    ), &format!("there is nothing to click on with the text {text:?}")).await
}

async fn wait_for_text(text: &str, limit: Duration) -> anyhow::Result<()> {
    wait_until(&format!("return document.body.innerText.includes({});", json(text)), limit, &format!("the text {text:?} did not appear")).await
}

async fn wait_for_selector(selector: &str, limit: Duration) -> anyhow::Result<()> {
    wait_until(&format!("return document.querySelector({}) !== null;", json(selector)), limit, &format!("{selector} did not appear")).await
}

async fn wait_until(script: &str, limit: Duration, problem: &str) -> anyhow::Result<()> {
    let started = std::time::Instant::now();
    while started.elapsed() < limit {
        if let Ok(true) = document::eval(script).join::<bool>().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    bail!("{problem} within {} seconds", limit.as_secs())
}

/// Checks that the app may take pictures of its windows, asking macOS for the Screen Recording permission if not, which
/// shows its prompt, or adds the app to the list in System Settings, Privacy & Security, Screen & System Audio Recording.
#[cfg(target_os = "macos")]
fn check_permission() -> anyhow::Result<()> {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGPreflightScreenCaptureAccess() -> bool;
        fn CGRequestScreenCaptureAccess() -> bool;
    }
    // SAFETY: neither takes arguments, and both may be called at any time.
    if unsafe { CGPreflightScreenCaptureAccess() } || unsafe { CGRequestScreenCaptureAccess() } {
        return Ok(());
    }
    bail!("macOS has not given the Screen Recording permission, which is needed to take the pictures. Turn it on for \
        what started the app, e.g. Visual Studio Code or Terminal, in System Settings, Privacy & Security, Screen & System \
        Audio Recording, adding it with + if it is not listed, then quit and restart it and try again")
}

#[cfg(not(target_os = "macos"))]
fn check_permission() -> anyhow::Result<()> {
    Ok(())
}

/// Saves a picture of the window, with its frame and shadow, to path, with macOS's screencapture, which needs the
/// Screen Recording permission for whatever started the app, e.g. VS Code or Terminal.
#[cfg(target_os = "macos")]
fn capture(window: &dioxus::desktop::tao::window::Window, path: &Path) -> anyhow::Result<()> {
    use dioxus::desktop::tao::platform::macos::WindowExtMacOS;
    use objc2_app_kit::NSWindow;

    // SAFETY: tao's window is an NSWindow, which lives as long as the window.
    let number = unsafe { &*(window.ns_window() as *const NSWindow) }.windowNumber();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let status = std::process::Command::new("screencapture")
        .args(["-x", "-l", &number.to_string()])
        .arg(path)
        .status()?;
    if !status.success() || !path.exists() {
        bail!("screencapture failed, it needs the Screen Recording permission, in System Settings, Privacy & Security, for what started the app");
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn capture(_window: &dioxus::desktop::tao::window::Window, _path: &Path) -> anyhow::Result<()> {
    bail!("screenshots are only taken on macOS")
}
