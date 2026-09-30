---
"tray-icon": minor
---

Add `TrayIconBuilder::with_autosave_name` and `TrayIconAttributes::autosave_name`, which set `NSStatusItem.autosaveName` on macOS. macOS already restores where the user ⌘+dragged a tray icon on the next launch, keyed by the order in which the process created its status items (`Item-0`, `Item-1`, and so on); an autosave name replaces that generated key with a stable one, so an application that creates several tray icons, or that creates one conditionally, does not have icons come back holding each other's positions. Ignored on Linux and Windows.
