---
"tray-icon": minor
---

**Breaking change** Changed `serde` derive implementation for `TrayIconEvent` to use `serde(tag = "type")` and `rename_all = "camelCase"` on variants so the expected JSON serialization would look like this

```json
{
  "type": "Click",
  "button": "Left",
  "buttonState": "Down",
  "id": "some id",
  "position": {
    "x": 0,
    "y": 0
  },
  "rect": {
    "size": {
      "width": 0,
      "height": 0
    },
    "position": {
      "x": 0,
      "y": 0
    }
  }
}
```
