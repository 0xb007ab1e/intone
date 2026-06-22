//! The Windows **UI Automation** adapter.
//!
//! Reads the focused element via UIA and hands it to [`oxeye_core`] for announcement
//! composition — the same policy that drives the Linux back-end. v1 **polls** the focused
//! element and prints announcements (`[say] …`); event-based handling (an
//! `IUIAutomationFocusChangedEventHandler` COM sink), SAPI speech, richer states, and
//! structured navigation are follow-ups.
//!
//! COM/UIA is an `unsafe` FFI boundary; it is confined to this module.

use std::time::Duration;

use anyhow::{Context as _, Result};
use oxeye_core::announcement::{self, Element, States};
use oxeye_core::exclusions::{Context as UiaContext, ExclusionEngine};
use oxeye_core::{Settings, Verbosity};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, UIA_ButtonControlTypeId,
    UIA_CheckBoxControlTypeId, UIA_ComboBoxControlTypeId, UIA_EditControlTypeId,
    UIA_HyperlinkControlTypeId, UIA_ListItemControlTypeId, UIA_MenuItemControlTypeId,
    UIA_RadioButtonControlTypeId, UIA_TabItemControlTypeId, UIA_TextControlTypeId,
    UIA_CONTROLTYPE_ID,
};

/// How often the focused element is polled (event-based handling is a follow-up).
const POLL_INTERVAL: Duration = Duration::from_millis(150);

/// Initialize UI Automation and read focus in a loop, announcing changes as text.
pub(crate) fn run() -> Result<()> {
    let settings = Settings::load().unwrap_or_default();
    let exclusions = ExclusionEngine::compile(&settings.exclusions).unwrap_or_default();

    // SAFETY: standard per-thread COM apartment initialization; released implicitly at exit.
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }
        .ok()
        .context("CoInitializeEx")?;
    // SAFETY: create the UI Automation root object via COM.
    let automation: IUIAutomation =
        unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
            .context("creating the UI Automation client")?;

    eprintln!("oxeye-windows: reading focus (text output). Ctrl-C to quit.");
    let mut last = String::new();
    loop {
        match read_focused(&automation, &exclusions, settings.verbosity) {
            Ok(Some(text)) if text != last => {
                println!("[say] {text}");
                last = text;
            }
            Ok(_) => {}
            Err(err) => tracing::debug!(%err, "could not read focused element"),
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Read the focused element's name and control type and compose its announcement via the shared
/// core policy. Returns `None` when an exclusion suppresses it.
fn read_focused(
    automation: &IUIAutomation,
    exclusions: &ExclusionEngine,
    verbosity: Verbosity,
) -> Result<Option<String>> {
    // SAFETY: UIA COM calls reading the focused element's properties.
    let element: IUIAutomationElement =
        unsafe { automation.GetFocusedElement() }.context("GetFocusedElement")?;
    let name = unsafe { element.CurrentName() }
        .map(|bstr| bstr.to_string())
        .unwrap_or_default();
    let control_type = unsafe { element.CurrentControlType() }.unwrap_or(UIA_CONTROLTYPE_ID(0));
    let role = control_type_role(control_type);

    let ident = UiaContext {
        app: "",
        role,
        name: &name,
    };
    let action = exclusions.evaluate(&ident);
    let element = Element {
        ident,
        description: "",
        // Value (UIA Value/RangeValue patterns) and states are follow-ups.
        value: None,
        states: States::default(),
    };
    Ok(announcement::compose(&element, verbosity, action).map(|announcement| announcement.text))
}

/// Map a UIA control type to a human-readable role label for announcements.
fn control_type_role(control_type: UIA_CONTROLTYPE_ID) -> &'static str {
    if control_type == UIA_ButtonControlTypeId {
        "button"
    } else if control_type == UIA_EditControlTypeId {
        "edit"
    } else if control_type == UIA_TextControlTypeId {
        "text"
    } else if control_type == UIA_HyperlinkControlTypeId {
        "link"
    } else if control_type == UIA_CheckBoxControlTypeId {
        "check box"
    } else if control_type == UIA_RadioButtonControlTypeId {
        "radio button"
    } else if control_type == UIA_ComboBoxControlTypeId {
        "combo box"
    } else if control_type == UIA_MenuItemControlTypeId {
        "menu item"
    } else if control_type == UIA_ListItemControlTypeId {
        "list item"
    } else if control_type == UIA_TabItemControlTypeId {
        "tab"
    } else {
        "element"
    }
}
