use anyhow::Result;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use serde::Deserialize;
use std::{
    io::{BufRead, BufReader, Write},
    os::windows::process::CommandExt,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Mutex, OnceLock},
};
use tracing::{debug, info, warn};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

use crate::terminal_focus::{ForegroundTabInfo, SessionTabCache};

#[derive(Debug)]
struct TabAutomationHelper {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

#[derive(Debug, Deserialize)]
struct HelperResponse {
    status: String,
    pid: Option<u32>,
    hwnd: Option<i64>,
    rid: Option<String>,
    title: Option<String>,
    match_mode: Option<String>,
    candidates: Option<Vec<String>>,
    error: Option<String>,
}

static TAB_HELPER: OnceLock<Mutex<Option<TabAutomationHelper>>> = OnceLock::new();

pub(super) fn select_windows_terminal_tab(
    target_window_hwnd: i64,
    target_window_pid: u32,
    cached_tab: Option<&SessionTabCache>,
    tokens: &[String],
) -> Result<Option<SessionTabCache>> {
    if target_window_hwnd <= 0 || (cached_tab.is_none() && tokens.is_empty()) {
        return Ok(None);
    }
    let response = with_tab_helper(|helper| {
        helper.request(serde_json::json!({
            "cmd": "select",
            "target_window_hwnd": target_window_hwnd,
            "target_window_pid": target_window_pid,
            "cached_pid": cached_tab.map(|value| value.terminal_pid).unwrap_or(0),
            "cached_window_hwnd": cached_tab.map(|value| value.window_hwnd).unwrap_or(0),
            "cached_runtime_id": cached_tab.map(|value| value.runtime_id.as_str()).unwrap_or_default(),
            "cached_title": cached_tab.map(|value| value.title.as_str()).unwrap_or_default(),
            "tokens": tokens,
        }))
    }, true)?
    .ok_or_else(|| anyhow::anyhow!("tab helper unavailable"))?;
    info!(
        has_cached_tab = cached_tab.is_some(),
        token_count = tokens.len(),
        status = %response.status,
        pid = ?response.pid,
        hwnd = ?response.hwnd,
        rid = ?response.rid,
        title = ?response.title,
        match_mode = ?response.match_mode,
        "windows terminal tab selection output"
    );
    if let Some(candidates) = &response.candidates {
        debug!(candidates = ?candidates, "windows terminal tab candidates");
    }
    if response.status == "error" {
        return Err(anyhow::anyhow!(
            "powershell tab selection failed: {}",
            response
                .error
                .unwrap_or_else(|| "unknown error".to_string())
        ));
    }
    Ok(match (response.pid, response.hwnd, response.rid) {
        (Some(terminal_pid), Some(window_hwnd), Some(runtime_id)) => Some(SessionTabCache {
            terminal_pid,
            window_hwnd,
            runtime_id,
            title: response
                .title
                .or_else(|| cached_tab.map(|value| value.title.clone()))
                .unwrap_or_default(),
        }),
        _ => None,
    })
}

pub(crate) fn foreground_windows_terminal_tab() -> Result<Option<ForegroundTabInfo>> {
    foreground_windows_terminal_tab_with_helper(true)
}

pub(crate) fn foreground_windows_terminal_tab_if_helper_running()
-> Result<Option<ForegroundTabInfo>> {
    foreground_windows_terminal_tab_with_helper(false)
}

fn foreground_windows_terminal_tab_with_helper(
    start_helper: bool,
) -> Result<Option<ForegroundTabInfo>> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_null() {
        warn!("foreground window handle is null");
        return Ok(None);
    }

    let mut pid = 0u32;
    unsafe { GetWindowThreadProcessId(hwnd, &mut pid) };
    if pid == 0 {
        warn!("foreground window pid is 0");
        return Ok(None);
    }

    let response = match with_tab_helper(
        |helper| {
            helper.request(serde_json::json!({
                "cmd": "inspect_hwnd",
                "pid": pid,
                "hwnd": hwnd as isize as i64,
            }))
        },
        start_helper,
    ) {
        Ok(Some(response)) => response,
        Ok(None) => return Ok(None),
        Err(error) => {
            warn!(error = %error, "failed to inspect foreground Windows Terminal tab");
            return Ok(None);
        }
    };

    Ok(
        match (response.rid, response.title, response.status.as_str()) {
            (Some(runtime_id), Some(title), "ok") => Some(ForegroundTabInfo {
                cache: SessionTabCache {
                    terminal_pid: pid,
                    window_hwnd: hwnd as isize as i64,
                    runtime_id,
                    title,
                },
            }),
            _ => None,
        },
    )
}

fn encode_powershell_command(script: &str) -> String {
    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    STANDARD.encode(bytes)
}

fn with_tab_helper<T>(
    action: impl Fn(&mut TabAutomationHelper) -> Result<T>,
    start_helper: bool,
) -> Result<Option<T>> {
    let mutex = TAB_HELPER.get_or_init(|| Mutex::new(None));
    let mut guard = mutex.lock().expect("tab helper mutex poisoned");

    for attempt in 0..2 {
        if guard.is_none() {
            if !start_helper {
                return Ok(None);
            }
            *guard = Some(TabAutomationHelper::spawn()?);
        }

        if let Some(helper) = guard.as_mut() {
            match action(helper) {
                Ok(value) => return Ok(Some(value)),
                Err(error) if attempt == 0 => {
                    warn!(error = %error, "tab helper request failed; restarting helper");
                    *guard = None;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
    }

    Err(anyhow::anyhow!("tab helper unavailable"))
}

impl TabAutomationHelper {
    fn spawn() -> Result<Self> {
        const CREATE_NO_WINDOW: u32 = 0x08000000;

        let script = tab_helper_script();
        let encoded = encode_powershell_command(&script);
        let mut command = Command::new("powershell");
        command
            .creation_flags(CREATE_NO_WINDOW)
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-WindowStyle",
                "Hidden",
                "-ExecutionPolicy",
                "Bypass",
                "-EncodedCommand",
                &encoded,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command.spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open tab helper stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("failed to open tab helper stdout"))?;

        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn request(&mut self, payload: serde_json::Value) -> Result<HelperResponse> {
        if self.child.try_wait()?.is_some() {
            return Err(anyhow::anyhow!("tab helper exited"));
        }

        let line = serde_json::to_string(&payload)?;
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()?;

        let mut response_bytes = Vec::new();
        let bytes = self.stdout.read_until(b'\n', &mut response_bytes)?;
        if bytes == 0 {
            return Err(anyhow::anyhow!("tab helper closed stdout"));
        }
        let response_line = String::from_utf8_lossy(&response_bytes);
        let response = serde_json::from_str::<HelperResponse>(response_line.trim())?;
        Ok(response)
    }
}

fn tab_helper_script() -> String {
    r#"
$ErrorActionPreference = 'Stop'
[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false)
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

function Write-JsonLine($value) {
  [Console]::Out.WriteLine(($value | ConvertTo-Json -Compress))
}

function Get-WindowTerminalTabsByHandle([int64]$hwndValue) {
  if ($hwndValue -le 0) { return @() }
  $root = [System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]::new($hwndValue))
  if (-not $root) { return @() }
  $cond = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty,[System.Windows.Automation.ControlType]::TabItem)
  $items = $root.FindAll([System.Windows.Automation.TreeScope]::Subtree,$cond)
  $result = @()
  for ($i = 0; $i -lt $items.Count; $i++) {
    $item = $items.Item($i)
    $runtimeId = (($item.GetRuntimeId() | ForEach-Object { $_.ToString() }) -join ',')
    $result += [pscustomobject]@{
      Item = $item
      Name = [string]$item.Current.Name
      RuntimeId = $runtimeId
    }
  }
  return $result
}

while (($line = [Console]::In.ReadLine()) -ne $null) {
  try {
    $request = $line | ConvertFrom-Json
    switch ($request.cmd) {
      'inspect_hwnd' {
        if ([int64]$request.hwnd -le 0) {
          Write-JsonLine @{ status = 'none' }
          continue
        }
        $tabs = Get-WindowTerminalTabsByHandle ([int64]$request.hwnd)
        $selected = $null
        foreach ($tab in $tabs) {
          $selection = $null
          if (-not $tab.Item.TryGetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern, [ref]$selection)) { continue }
          if (-not ([System.Windows.Automation.SelectionItemPattern]$selection).Current.IsSelected) { continue }
          if ([string]::IsNullOrWhiteSpace($tab.RuntimeId)) { continue }
          $selected = $tab
          break
        }
        if ($null -ne $selected) {
          Write-JsonLine @{ status = 'ok'; pid = [int]$request.pid; hwnd = [int64]$request.hwnd; rid = $selected.RuntimeId; title = $selected.Name }
        } else {
          Write-JsonLine @{ status = 'none' }
        }
      }
      'select' {
        $best = $null
        $selected = $null
        $matchMode = $null
        $debugCandidates = @()
        $cachedTitle = [string]$request.cached_title
        $targetHwnd = [int64]$request.target_window_hwnd
        $targetPid = [int]$request.target_window_pid
        if ($targetHwnd -le 0) {
          Write-JsonLine @{ status = 'none' }
          continue
        }
        $tabs = Get-WindowTerminalTabsByHandle $targetHwnd
        foreach ($tab in $tabs) {
            $procId = if ($targetPid -gt 0) { $targetPid } elseif ($request.cached_pid -gt 0) { [int]$request.cached_pid } else { $null }
            $lower = ''
            if (-not [string]::IsNullOrWhiteSpace($tab.Name)) {
              $lower = $tab.Name.ToLowerInvariant()
            }
            if ([int64]$request.cached_window_hwnd -gt 0 -and $targetHwnd -eq [int64]$request.cached_window_hwnd -and -not [string]::IsNullOrWhiteSpace($request.cached_runtime_id) -and $tab.RuntimeId -eq [string]$request.cached_runtime_id) {
              $acceptCached = $false
              $normalizedLower = ($lower -replace '[^\p{L}\p{Nd}]+', ' ').Trim()
              if (-not [string]::IsNullOrWhiteSpace($cachedTitle)) {
                $cachedLower = $cachedTitle.ToLowerInvariant()
                $normalizedCachedLower = ($cachedLower -replace '[^\p{L}\p{Nd}]+', ' ').Trim()
                if ($lower -eq $cachedLower -or $normalizedLower -eq $normalizedCachedLower -or $lower.Contains($cachedLower) -or $cachedLower.Contains($lower) -or $normalizedLower.Contains($normalizedCachedLower) -or $normalizedCachedLower.Contains($normalizedLower)) {
                  $acceptCached = $true
                }
              }
              if (-not $acceptCached) {
                foreach ($token in @($request.tokens)) {
                  if ([string]::IsNullOrWhiteSpace($token)) { continue }
                  $t = ([string]$token).ToLowerInvariant()
                  $normalizedToken = ($t -replace '[^\p{L}\p{Nd}]+', ' ').Trim()
                  if ($lower -eq $t -or $normalizedLower -eq $normalizedToken -or $lower.Contains($t) -or $normalizedLower.Contains($normalizedToken)) {
                    $acceptCached = $true
                    break
                  }
                }
              }
              if ($acceptCached) {
                $pattern = $tab.Item.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
                ([System.Windows.Automation.SelectionItemPattern]$pattern).Select()
                $selected = [pscustomobject]@{ ProcId = $procId; Hwnd = $targetHwnd; RuntimeId = $tab.RuntimeId; Name = $tab.Name }
                break
              }
            }
            if ([string]::IsNullOrWhiteSpace($tab.Name)) { continue }
            $score = 0
            $normalizedLower = ($lower -replace '[^\p{L}\p{Nd}]+', ' ').Trim()
            if ([int64]$request.cached_window_hwnd -gt 0 -and $targetHwnd -eq [int64]$request.cached_window_hwnd -and -not [string]::IsNullOrWhiteSpace($cachedTitle)) {
              $cachedLower = $cachedTitle.ToLowerInvariant()
              $normalizedCachedLower = ($cachedLower -replace '[^\p{L}\p{Nd}]+', ' ').Trim()
              if ($lower -eq $cachedLower -or $normalizedLower -eq $normalizedCachedLower) { $score += 260 }
              elseif ($lower.Contains($cachedLower) -or $cachedLower.Contains($lower) -or $normalizedLower.Contains($normalizedCachedLower) -or $normalizedCachedLower.Contains($normalizedLower)) { $score += 140 }
            }
            foreach ($token in @($request.tokens)) {
              if ([string]::IsNullOrWhiteSpace($token)) { continue }
              $t = ([string]$token).ToLowerInvariant()
              $normalizedToken = ($t -replace '[^\p{L}\p{Nd}]+', ' ').Trim()
              if ($lower -eq $t -or $normalizedLower -eq $normalizedToken) { $score += 300 }
              elseif ($lower.Contains($t) -or $normalizedLower.Contains($normalizedToken)) { $score += 120 }
              elseif ($normalizedToken.Length -ge 8) {
                $parts = @($normalizedToken.Split(' ') | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
                if ($parts.Count -gt 0) {
                  $allPartsMatch = $true
                  foreach ($part in $parts) {
                    if (-not $normalizedLower.Contains($part)) {
                      $allPartsMatch = $false
                      break
                    }
                  }
                  if ($allPartsMatch) { $score += 90 }
                }
              }
            }
            $debugCandidates += ("score={0} title={1}" -f $score, $tab.Name)
            if ($score -gt 0 -and ($null -eq $best -or $score -gt $best.Score)) {
              $best = [pscustomobject]@{ Score = $score; ProcId = $procId; Hwnd = $targetHwnd; Tab = $tab }
            }
        }
        if ($null -ne $selected) {
          Write-JsonLine @{ status = 'ok'; pid = $selected.ProcId; hwnd = $selected.Hwnd; rid = $selected.RuntimeId; title = $selected.Name; match_mode = 'cache'; candidates = @($debugCandidates | Select-Object -First 8) }
          continue
        }
        if ([int64]$request.cached_window_hwnd -gt 0 -and $null -eq $best) {
          Write-JsonLine @{ status = 'none'; candidates = @($debugCandidates | Select-Object -First 8) }
          continue
        }
        if ($null -eq $best) {
          Write-JsonLine @{ status = 'none'; candidates = @($debugCandidates | Select-Object -First 8) }
          continue
        }
        $pattern = $best.Tab.Item.GetCurrentPattern([System.Windows.Automation.SelectionItemPattern]::Pattern)
        ([System.Windows.Automation.SelectionItemPattern]$pattern).Select()
        Write-JsonLine @{ status = 'ok'; pid = $best.ProcId; hwnd = $best.Hwnd; rid = $best.Tab.RuntimeId; title = $best.Tab.Name; match_mode = 'score'; candidates = @($debugCandidates | Select-Object -First 8) }
      }
      default {
        Write-JsonLine @{ status = 'error'; error = 'unsupported command' }
      }
    }
  } catch {
    Write-JsonLine @{ status = 'error'; error = $_.Exception.Message }
  }
}
"#
    .to_string()
}
