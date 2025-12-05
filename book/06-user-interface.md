# Chapter 6: Building the User Interface

This chapter covers building LockSmith's graphical interface using egui. We'll explore the immediate-mode paradigm, layout systems, and how to create a professional-looking application.

## Understanding Immediate Mode GUI

Traditional GUI frameworks (like Windows Forms, Qt, WPF) use **retained mode**:
1. You create widget objects that persist in memory
2. You set their properties
3. The framework tracks state and redraws as needed

egui uses **immediate mode**:
1. Every frame, you describe what the UI should look like
2. The framework figures out what changed
3. No persistent widget objects—just function calls

### The Update Function

The heart of an egui app is the `update` function, called every frame:

```rust
impl eframe::App for LockSmithApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Build UI here—called 60+ times per second
    }
}
```

Each call rebuilds the entire UI description. This sounds expensive, but it's actually very efficient—egui only redraws changed regions and caches heavily.

### Advantages of Immediate Mode

1. **No widget synchronization** — UI always matches data (no stale states)
2. **Simple mental model** — Just describe what you want
3. **Easy dynamic UIs** — Conditional widgets are trivial
4. **Less boilerplate** — No event binding, no callbacks

## Basic Building Blocks

### Panels

Panels are the top-level containers:

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    // Main content goes here
});
```

Available panel types:
- `CentralPanel` — Fills remaining space
- `TopBottomPanel` — Fixed at top or bottom
- `SidePanel` — Fixed at left or right

Panels are processed in order—side/top/bottom panels reserve space, then `CentralPanel` fills the rest.

### The UI Object

Inside panels, you get a `ui` parameter for adding widgets:

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    ui.heading("Title");
    ui.label("Some text");
    if ui.button("Click me").clicked() {
        // Handle click
    }
});
```

`ui` provides methods for:
- Adding widgets (`label`, `button`, `text_edit`, etc.)
- Layout control (`horizontal`, `vertical`, `add_space`)
- Styling (`visuals`, `spacing`)

### Common Widgets

```rust
// Labels
ui.label("Plain text");
ui.heading("Large heading");

// Buttons
if ui.button("Click").clicked() {
    println!("Button clicked!");
}

// Text input
ui.text_edit_singleline(&mut self.search_query);

// Checkboxes
ui.checkbox(&mut self.dark_mode, "Dark mode");

// Sliders
ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0));
```

### Rich Text

For styled text, use `RichText`:

```rust
use eframe::egui::RichText;

ui.label(RichText::new("Bold text").strong());
ui.label(RichText::new("Colored").color(Color32::RED));
ui.label(RichText::new("Large").size(24.0));
ui.label(
    RichText::new("Combined")
        .strong()
        .size(18.0)
        .color(Color32::from_rgb(100, 200, 100))
);
```

## Layout Systems

### Vertical Layout (Default)

By default, widgets stack vertically:

```rust
ui.heading("Title");
ui.label("Line 1");
ui.label("Line 2");
// Results in:
// Title
// Line 1
// Line 2
```

### Horizontal Layout

Group widgets horizontally:

```rust
ui.horizontal(|ui| {
    ui.label("Name:");
    ui.text_edit_singleline(&mut self.name);
    if ui.button("Submit").clicked() {
        // ...
    }
});
```

### Centering Content

```rust
ui.vertical_centered(|ui| {
    ui.heading("Centered Title");
    ui.label("This is also centered");
});
```

### Adding Space

```rust
ui.add_space(20.0);  // Add 20 pixels of vertical space
```

### Sized Widgets

Control widget dimensions:

```rust
ui.add_sized(
    egui::Vec2::new(200.0, 30.0),  // width x height
    egui::TextEdit::singleline(&mut self.query)
        .hint_text("Search..."),
);
```

## Building LockSmith's Interface

Let's walk through building LockSmith's actual UI.

### The Main Structure

```rust
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    // 1. Handle keyboard shortcuts
    // 2. Auto-hide old messages
    // 3. Show confirmation dialog (if active)
    // 4. Main UI panel
    // 5. Theme toggle button
    // 6. Request repaint if needed
}
```

### Keyboard Shortcuts

```rust
ctx.input(|i| {
    if i.key_pressed(egui::Key::F5) {
        self.refresh_locks();
    }
    if i.key_pressed(egui::Key::Delete) && !self.selected_ids.is_empty() {
        self.pending_release_ids = self.selected_ids.iter().cloned().collect();
        self.show_release_confirm = true;
    }
    if i.modifiers.ctrl && i.key_pressed(egui::Key::A) {
        self.select_all_visible();
    }
});
```

The `ctx.input()` method takes a closure that receives input state. We check for:
- F5 alone → refresh
- Delete with selection → show confirmation
- Ctrl+A → select all

### Auto-Hiding Messages

```rust
if let Some(ref msg) = self.status_message {
    match msg.message_type {
        MessageType::Success | MessageType::Info => {
            if msg.timestamp.elapsed().as_secs() >= 5 {
                self.status_message = None;
            }
        }
        _ => {}
    }
}
```

Success and info messages disappear after 5 seconds. Warnings and errors persist.

### The Title Section

```rust
egui::CentralPanel::default().show(ctx, |ui| {
    ui.add_space(20.0);

    // Title - centered
    ui.vertical_centered(|ui| {
        ui.horizontal(|ui| {
            ui.add_space((ui.available_width() - 180.0) / 2.0);
            ui.heading(RichText::new("🔓").size(28.0));
            ui.heading(RichText::new("LockSmith").size(28.0).strong());
        });
    });

    ui.add_space(15.0);
});
```

We manually center by calculating padding based on `available_width()`.

### Admin Warning Banner

```rust
if !self.is_admin {
    ui.vertical_centered(|ui| {
        egui::Frame::none()
            .fill(Color32::from_rgb(239, 68, 68))  // Red background
            .rounding(5.0)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(
                    RichText::new(
                        "⚠️ Not running as Administrator. File lock operations may fail.",
                    )
                    .color(Color32::WHITE),
                );
            });
    });
    ui.add_space(10.0);
}
```

### Frames for Styling

`egui::Frame` wraps content with background, border, and padding:

```rust
egui::Frame::none()
    .fill(Color32::from_rgb(100, 100, 100))  // Background color
    .stroke(Stroke::new(1.0, Color32::WHITE)) // Border
    .rounding(8.0)                            // Corner radius
    .inner_margin(10.0)                       // Padding
    .show(ui, |ui| {
        ui.label("Styled content");
    });
```

### Search Bar

```rust
ui.horizontal(|ui| {
    ui.label("Search:");

    let search_width = total_width - 200.0;
    let _search_response = ui.add_sized(
        Vec2::new(search_width, 30.0),
        egui::TextEdit::singleline(&mut self.search_query)
            .hint_text("Filter by filename, path, or username..."),
    );

    ui.add_space(10.0);

    if ui
        .add_sized(Vec2::new(130.0, 30.0), egui::Button::new("🔄 Refresh (F5)"))
        .clicked()
    {
        self.refresh_locks();
    }
});
```

### Building a Data Grid

egui doesn't have a built-in data grid, so we build one with `Grid`:

```rust
egui::ScrollArea::vertical()
    .max_height(available_height)
    .show(ui, |ui| {
        egui::Grid::new("locks_table")
            .num_columns(4)
            .striped(true)
            .min_col_width(80.0)
            .show(ui, |ui| {
                // Header row
                // ... header buttons ...
                ui.end_row();

                // Data rows
                for (index, lock) in filtered_locks.iter().enumerate() {
                    // ... row content ...
                    ui.end_row();
                }
            });
    });
```

### Grid Basics

```rust
egui::Grid::new("unique_id")
    .num_columns(4)      // Expected columns (helps with layout)
    .striped(true)       // Alternating row backgrounds
    .min_col_width(80.0) // Minimum column width
    .show(ui, |ui| {
        ui.label("Col 1");
        ui.label("Col 2");
        ui.label("Col 3");
        ui.label("Col 4");
        ui.end_row();  // Move to next row
        
        ui.label("Row 2 Col 1");
        // ...
    });
```

### Sortable Column Headers

```rust
let file_name_text = format!(
    "FILE NAME{}",
    if self.sort_column == Some(SortColumn::FileName) {
        self.sort_direction.arrow()  // " ↑" or " ↓"
    } else {
        ""
    }
);

if ui
    .add_sized(
        Vec2::new(200.0, row_height),
        egui::Button::new(RichText::new(file_name_text).strong())
            .frame(false),  // No button border
    )
    .clicked()
{
    self.handle_sort(SortColumn::FileName);
}
```

Headers are buttons without frames. Clicking toggles sort.

### Selectable Rows

```rust
for (index, lock) in filtered_locks.iter().enumerate() {
    let is_selected = self.selected_ids.contains(&lock.file_id);

    // Determine row colors
    let row_color = if is_selected {
        Color32::from_rgb(96, 165, 250)  // Blue for selected
    } else if self.dark_mode && index % 2 == 1 {
        Color32::from_rgba_unmultiplied(255, 255, 255, 5)  // Subtle stripe
    } else if !self.dark_mode && index % 2 == 1 {
        Color32::from_rgba_unmultiplied(0, 0, 0, 10)
    } else {
        Color32::TRANSPARENT
    };

    let text_color = if is_selected {
        Color32::WHITE
    } else if self.dark_mode {
        Color32::from_rgb(248, 250, 252)
    } else {
        Color32::from_rgb(15, 23, 42)
    };

    // Create cells as buttons for click detection
    let file_name_response = ui.add_sized(
        Vec2::new(200.0, row_height),
        egui::Button::new(RichText::new(&lock.file_name).color(text_color))
            .fill(row_color)
            .frame(false),
    );

    // ... more cells ...

    // Handle clicks
    let clicked = file_name_response.clicked()
        || path_response.clicked()
        || user_response.clicked()
        || locks_response.clicked();

    if clicked {
        // Handle selection logic
    }

    ui.end_row();
}
```

### Multi-Select Logic

```rust
if clicked {
    let modifiers = ui.input(|i| i.modifiers);

    if modifiers.ctrl || modifiers.command {
        // Ctrl+Click: toggle individual selection
        if self.selected_ids.contains(&lock.file_id) {
            self.selected_ids.remove(&lock.file_id);
        } else {
            self.selected_ids.insert(lock.file_id);
        }
        self.last_selected_index = Some(index);
    } else if modifiers.shift {
        // Shift+Click: range selection
        if let Some(last_index) = self.last_selected_index {
            let start = last_index.min(index);
            let end = last_index.max(index);
            for i in start..=end {
                if let Some(l) = filtered_locks.get(i) {
                    self.selected_ids.insert(l.file_id);
                }
            }
        } else {
            self.selected_ids.clear();
            self.selected_ids.insert(lock.file_id);
            self.last_selected_index = Some(index);
        }
    } else {
        // Normal click: single selection
        self.selected_ids.clear();
        self.selected_ids.insert(lock.file_id);
        self.last_selected_index = Some(index);
    }
}
```

### Double-Click Handling

```rust
let double_clicked = file_name_response.double_clicked()
    || path_response.double_clicked()
    || user_response.double_clicked()
    || locks_response.double_clicked();

if double_clicked {
    self.pending_release_ids = vec![lock.file_id];
    self.show_release_confirm = true;
}
```

### Scroll Areas

For long lists, wrap in a `ScrollArea`:

```rust
egui::ScrollArea::vertical()
    .max_height(500.0)  // Limit height, enable scrolling
    .show(ui, |ui| {
        // Content that may exceed height
    });
```

Options:
- `.vertical()` — Scroll vertically
- `.horizontal()` — Scroll horizontally
- `.both()` — Both directions
- `.max_height()` / `.max_width()` — Constrain size

## Confirmation Dialogs

egui supports popup windows:

```rust
if self.show_release_confirm {
    egui::Window::new("⚠️ Confirm Release")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new(format!(
                    "Release {} file lock(s)?",
                    self.pending_release_ids.len()
                ))
                .size(16.0)
                .strong(),
            );
            ui.add_space(10.0);
            ui.label("This will force-close the file handle(s) and may cause:");
            ui.label("• Data loss if unsaved changes exist");
            ui.label("• Application crashes");
            ui.label("• File corruption");
            ui.add_space(15.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    self.show_release_confirm = false;
                    self.pending_release_ids.clear();
                }
                ui.add_space(10.0);
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Release Locks").color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(239, 68, 68)),  // Red
                    )
                    .clicked()
                {
                    self.show_release_confirm = false;
                    self.selected_ids = self.pending_release_ids.drain(..).collect();
                    self.release_selected_locks();
                }
            });
        });
}
```

### Window Options

```rust
egui::Window::new("Title")
    .collapsible(false)   // No collapse button
    .resizable(false)     // Fixed size
    .anchor(              // Position
        egui::Align2::CENTER_CENTER,
        Vec2::ZERO,       // Offset from anchor
    )
    .show(ctx, |ui| { });
```

## Status Messages

```rust
if let Some(ref msg) = self.status_message {
    let (bg_color, text_color) = match msg.message_type {
        MessageType::Success => (Color32::from_rgb(16, 185, 129), Color32::WHITE),
        MessageType::Error => (Color32::from_rgb(239, 68, 68), Color32::WHITE),
        MessageType::Warning => (Color32::from_rgb(245, 158, 11), Color32::WHITE),
        MessageType::Info => {
            if self.dark_mode {
                (Color32::from_rgb(51, 65, 85), Color32::from_rgb(248, 250, 252))
            } else {
                (Color32::from_rgb(226, 232, 240), Color32::from_rgb(15, 23, 42))
            }
        }
    };

    let icon = match msg.message_type {
        MessageType::Success => "✓",
        MessageType::Error => "✕",
        MessageType::Warning => "⚠",
        MessageType::Info => "ℹ",
    };

    let full_text = format!("{} {}", icon, msg.text);

    ui.vertical_centered(|ui| {
        let button = egui::Button::new(
            RichText::new(&full_text).color(text_color).size(14.0)
        )
        .fill(bg_color)
        .stroke(egui::Stroke::NONE)
        .rounding(5.0)
        .sense(egui::Sense::hover());  // Not clickable

        ui.add(button);
    });
}
```

## Floating UI Elements

For the theme toggle button in the corner, we use `Area`:

```rust
egui::Area::new(egui::Id::new("theme_toggle"))
    .fixed_pos(egui::pos2(
        ctx.screen_rect().max.x - 50.0,
        ctx.screen_rect().max.y - 50.0,
    ))
    .show(ctx, |ui| {
        let icon = if self.dark_mode { "☀" } else { "🌙" };
        if ui
            .add_sized(
                Vec2::new(36.0, 36.0),
                egui::Button::new(RichText::new(icon).size(18.0))
                    .rounding(18.0),  // Circular
            )
            .on_hover_text("Toggle theme")
            .clicked()
        {
            self.dark_mode = !self.dark_mode;
            self.apply_theme(ctx.clone());
        }
    });
```

`Area` creates a floating container at a fixed position, outside the normal layout flow.

### Hover Text (Tooltips)

```rust
ui.button("?").on_hover_text("This is helpful information");
```

## Empty and Loading States

Good UX includes handling empty/loading states:

```rust
if self.is_loading {
    ui.set_min_height(available_height);
    ui.vertical_centered(|ui| {
        ui.add_space(available_height / 2.0 - 40.0);
        ui.spinner();  // Built-in loading spinner
        ui.add_space(10.0);
        ui.label("Loading file locks...");
    });
} else if filtered_locks.is_empty() {
    ui.set_min_height(available_height);
    ui.vertical_centered(|ui| {
        ui.add_space(available_height / 2.0 - 60.0);
        ui.label(RichText::new("🔓").size(48.0));
        ui.add_space(10.0);
        if self.all_locks.is_empty() {
            ui.label(RichText::new("No Locks").size(20.0).strong());
            ui.add_space(5.0);
            ui.label("No files are currently locked on this system.");
        } else {
            ui.label(RichText::new("No matching locks").size(20.0).strong());
            ui.add_space(5.0);
            ui.label("No locks match your search criteria.");
        }
    });
} else {
    // Normal data display
}
```

## Requesting Repaints

egui only repaints when needed. For animations or timed events, request repaints:

```rust
if self.is_loading || self.status_message.is_some() {
    ctx.request_repaint();  // Keep updating for spinner/message timeout
}
```

Without this, the loading spinner wouldn't animate and messages wouldn't auto-hide.

## Color Scheme

LockSmith uses a consistent color palette:

| Purpose | Light Mode | Dark Mode |
|---------|------------|-----------|
| Background | White | Dark gray |
| Text | Near black | Near white |
| Selected | Blue | Blue |
| Success | Green | Green |
| Error | Red | Red |
| Warning | Orange | Orange |

Colors are defined using `Color32::from_rgb(r, g, b)` for full opacity or `Color32::from_rgba_unmultiplied(r, g, b, a)` for transparency.

## What We've Accomplished

You now understand:

✅ Immediate mode GUI — Rebuilding UI every frame  
✅ Panels and layout — Organizing content with horizontal/vertical containers  
✅ Widgets — Labels, buttons, text inputs, grids  
✅ Styling — RichText, Frame, colors  
✅ User interaction — Clicks, double-clicks, keyboard shortcuts  
✅ State-dependent UI — Loading, empty, error states  
✅ Modal dialogs — Popup windows for confirmations  

## Key Takeaways

1. **Describe, don't manage** — Just describe what the UI should look like each frame
2. **Layout is hierarchical** — Panels contain layouts contain widgets
3. **Styling is inline** — Apply styles directly via method chaining
4. **Interaction is immediate** — `if button.clicked()` right where you create the button
5. **Empty states matter** — Always handle loading, empty, and error cases

## Looking Ahead

In the next chapter, we'll add polish: theme persistence, the programmatic icon, and final touches that make LockSmith feel professional.

---

**Previous Chapter:** [← Application Logic](05-application-logic.md)  
**Next Chapter:** [Polish and Finishing Touches →](07-polish.md)