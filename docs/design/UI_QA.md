# Relay UI QA Checklist

Use this checklist before finishing any UI work.

## Viewports

Check at least:

- 1920x1080
- 1440x900
- the app default window size, currently 1180x780

## Layout

- Top app bar is 40-44 px tall and visually calm.
- Left rail is stable and roughly 320-352 px.
- Right context pane is stable and roughly 360-440 px.
- Center pane owns the largest area.
- Terminal route is the default and visually dominant.
- Pane headers align horizontally.
- Dividers are 1 px and not visually heavy.
- Task rows do not change height when active/inactive.
- Tabs and segmented controls do not resize their parent.
- Long paths and task names do not overflow into adjacent columns.

## Style

- Visual style follows the Zed reference: native, compact, low chrome.
- Layout follows the Orca reference: left tasks, center terminal, right context.
- No dashboard card mosaic.
- No nested cards.
- No oversized hero text.
- No decorative gradients, blobs, or stock imagery.
- Accent color is sparse and meaningful.
- Focus and selection states are visible but restrained.
- Terminal surface reads as a real terminal, not a decorative panel.

## Content

- Task rows show enough metadata to identify worktree, agent, and state.
- Empty states are compact and operational.
- In-app text describes state or commands, not how the UI was designed.
- Status labels are short and scannable.
- File/diff/review labels fit in the right pane.

## Behavior

- Clicking a task activates it.
- Switching tasks returns focus to the terminal.
- Terminal/Preview route switching preserves context.
- Files/Diff/Review tabs switch without layout jumps.
- UI code dispatches commands and does not perform side effects directly.

## Verification

Record in the final note:

- formatting command run
- test command run
- screenshot path, or why screenshot capture was not possible
- any visible deviation intentionally left for a future task
