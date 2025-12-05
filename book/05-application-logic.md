# Chapter 5: Application Logic

Now that we have our data types and Windows API functions, it's time to build the application logic—the methods that make LockSmith actually work. This chapter covers state management, data processing, and the core functionality that powers our UI.

## The Application Constructor

Every egui application needs a way to initialize. Here's our constructor:

```rust
impl LockSmithApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load persisted state
        let persistence: AppPersistence = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();

        let mut app = Self {
            dark_mode: persistence.dark_mode,
            ..Default::default()
        };

        // Apply theme
        app.apply_theme(cc.egui_ctx.clone());

        // Check admin status
        app.is_admin = is_admin();

        // Initial load
        app.refresh_locks();

        app
    }
}
```

### Breaking Down the Constructor

#### The CreationContext

```rust
fn new(cc: &eframe::CreationContext<'_>) -> Self {
```

The `CreationContext` provides access to:
- `cc.egui_ctx` — The egui context for styling/configuration
- `cc.storage` — Persistent storage for saving settings

The `'_` is an anonymous lifetime—Rust infers it for us.

#### Loading Persisted Data

```rust
let persistence: AppPersistence = cc
    .storage
    .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
    .unwrap_or_default();
```

Let's trace this chain:

1. `cc.storage` — Returns `Option<&dyn Storage>` (might be None if storage is unavailable)

2. `.and_then(|s| ...)` — If Some, apply the closure; if None, stay None. This is like "optional chaining" in other languages.

3. `eframe::get_value(s, eframe::APP_KEY)` — Deserialize saved data with the key "APP_KEY". Returns `Option<AppPersistence>`.

4. `.unwrap_or_default()` — If Some, use it; if None, use `Default::default()`.

#### The Struct Update Syntax

```rust
let mut app = Self {
    dark_mode: persistence.dark_mode,
    ..Default::default()
};
```

The `..Default::default()` syntax means "for all other fields, use the defaults." This is cleaner than listing every field.

#### Method Chaining

```rust
app.apply_theme(cc.egui_ctx.clone());
app.is_admin = is_admin();
app.refresh_locks();
app
```

We call initialization methods, then return `app`. The last expression in a function (without semicolon) is the return value.

## Theme Management

```rust
fn apply_theme(&self, ctx: egui::Context) {
    if self.dark_mode {
        ctx.set_visuals(egui::Visuals::dark());
    } else {
        ctx.set_visuals(egui::Visuals::light());
    }
}
```

### Understanding the Context

egui uses a `Context` to manage global state like:
- Visual style (colors, spacing)
- Input state (mouse, keyboard)
- Fonts and text rendering

`Visuals::dark()` and `Visuals::light()` are predefined color schemes.

### Why Clone the Context?

```rust
cc.egui_ctx.clone()
```

egui's `Context` is internally reference-counted (`Arc<Mutex<...>>`). Cloning is cheap—it just increments the reference count. We clone because:
1. The context lives beyond this function
2. We need ownership, not a borrow

## Status Messages

User feedback is essential. Here's how we manage status messages:

```rust
fn show_message(&mut self, text: impl Into<String>, message_type: MessageType) {
    self.status_message = Some(StatusMessage {
        text: text.into(),
        message_type,
        timestamp: std::time::Instant::now(),
    });
}
```

### The impl Into<String> Pattern

```rust
text: impl Into<String>
```

This accepts anything that can be converted into a `String`:
- `String` itself
- `&str` (string slices)
- `Cow<str>` (copy-on-write strings)
- etc.

Usage examples:
```rust
app.show_message("Hello", MessageType::Info);           // &str
app.show_message(format!("Count: {}", n), MessageType::Info);  // String
```

This is more flexible than taking `String` or `&str` alone.

### Field Init Shorthand

```rust
StatusMessage {
    text: text.into(),
    message_type,      // Same as message_type: message_type
    timestamp: std::time::Instant::now(),
}
```

When a field name matches the variable name, you can just write the name once.

## Refreshing Lock Data

The core function that loads file locks:

```rust
fn refresh_locks(&mut self) {
    self.is_loading = true;
    self.selected_ids.clear();
    self.last_selected_index = None;

    match enumerate_files() {
        Ok(locks) => {
            let count = locks.len();
            self.all_locks = locks;
            self.is_loading = false;

            if count > 0 {
                self.show_message(
                    format!("Loaded {} file lock(s)", count),
                    MessageType::Success,
                );
            } else {
                self.show_message(
                    "No file locks found. System is ready.",
                    MessageType::Info,
                );
            }
        }
        Err(e) => {
            self.all_locks.clear();
            self.is_loading = false;
            self.show_message(e.to_string(), MessageType::Error);
        }
    }
}
```

### State Management

Before loading:
```rust
self.is_loading = true;      // Show loading indicator
self.selected_ids.clear();   // Clear selection
self.last_selected_index = None;  // Reset range-select anchor
```

This ensures the UI reflects the loading state and old selections don't persist.

### Match on Result

```rust
match enumerate_files() {
    Ok(locks) => { /* success */ }
    Err(e) => { /* failure */ }
}
```

This is pattern matching on a `Result`. Every possible case is handled—Rust won't compile if you miss one.

### The to_string() Method

```rust
self.show_message(e.to_string(), MessageType::Error);
```

Because we implemented `Display` for our error types, `.to_string()` gives us the user-friendly message.

## Filtering and Sorting

The most complex logic is filtering and sorting the lock list:

```rust
fn get_filtered_sorted_locks(&self) -> Vec<FileLockInfo> {
    let query = self.search_query.to_lowercase();

    let mut filtered: Vec<FileLockInfo> = self
        .all_locks
        .iter()
        .filter(|lock| {
            if query.is_empty() {
                true
            } else {
                lock.file_name.to_lowercase().contains(&query)
                    || lock.file_path.to_lowercase().contains(&query)
                    || lock.username.to_lowercase().contains(&query)
            }
        })
        .cloned()
        .collect();

    // Sort
    if let Some(column) = &self.sort_column {
        filtered.sort_by(|a, b| {
            let cmp = match column {
                SortColumn::FileName => {
                    a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
                }
                SortColumn::FilePath => {
                    a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase())
                }
                SortColumn::Username => {
                    a.username.to_lowercase().cmp(&b.username.to_lowercase())
                }
                SortColumn::NumLocks => a.num_locks.cmp(&b.num_locks),
            };

            match self.sort_direction {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }

    filtered
}
```

### Iterator Adapters

Rust's iterators are powerful and efficient. Let's trace this chain:

```rust
self.all_locks
    .iter()           // Create iterator over references
    .filter(...)      // Keep only matching items
    .cloned()         // Clone each item (turn &T into T)
    .collect()        // Collect into Vec
```

#### The iter() Method

```rust
self.all_locks.iter()
```

Creates an iterator that yields `&FileLockInfo` (references, not copies). This is efficient—we're not copying data yet.

#### The filter() Adapter

```rust
.filter(|lock| {
    if query.is_empty() {
        true
    } else {
        lock.file_name.to_lowercase().contains(&query)
            || lock.file_path.to_lowercase().contains(&query)
            || lock.username.to_lowercase().contains(&query)
    }
})
```

`filter` takes a **closure** (anonymous function) that returns `bool`. Items where the closure returns `true` are kept; others are discarded.

The `|lock|` is the closure's parameter list. The type is inferred as `&&FileLockInfo` (reference to reference, because `iter()` yields references and `filter` takes references to those).

#### Closures in Rust

Closures are like functions but can capture variables from their environment:

```rust
let query = self.search_query.to_lowercase();  // Captured by closure

.filter(|lock| {
    // 'query' is used here - captured from outer scope
    lock.file_name.to_lowercase().contains(&query)
})
```

Closures can capture by:
- Reference (borrowing) — default
- Mutable reference
- Value (moving)

Here, `query` is borrowed immutably.

#### The cloned() Adapter

```rust
.cloned()
```

Transforms `Iterator<Item = &T>` into `Iterator<Item = T>` by cloning each item. We need owned `FileLockInfo` values, not references, because we're building a new `Vec`.

#### The collect() Method

```rust
.collect()
```

Consumes the iterator and collects items into a collection. The type annotation tells Rust what collection to create:

```rust
let mut filtered: Vec<FileLockInfo> = /* ... */.collect();
```

### Sorting

```rust
filtered.sort_by(|a, b| {
    let cmp = match column {
        SortColumn::FileName => {
            a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
        }
        // ... other cases
    };

    match self.sort_direction {
        SortDirection::Ascending => cmp,
        SortDirection::Descending => cmp.reverse(),
    }
});
```

#### The sort_by Method

`sort_by` takes a comparison function that returns `Ordering`:
- `Ordering::Less` — a comes before b
- `Ordering::Equal` — a and b are equivalent
- `Ordering::Greater` — a comes after b

#### The cmp Method

```rust
a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
```

The `cmp` method compares two values and returns an `Ordering`. It's defined by the `Ord` trait.

#### Reversing Order

```rust
cmp.reverse()
```

`Ordering::reverse()` flips the comparison:
- `Less` → `Greater`
- `Greater` → `Less`
- `Equal` → `Equal`

#### Why to_lowercase()?

```rust
a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
```

Case-insensitive sorting. Without this, "Zebra" would come before "apple" (uppercase letters have lower ASCII values).

## Handling Column Clicks

When a user clicks a column header:

```rust
fn handle_sort(&mut self, column: SortColumn) {
    if self.sort_column == Some(column) {
        self.sort_direction = self.sort_direction.toggle();
    } else {
        self.sort_column = Some(column);
        self.sort_direction = SortDirection::Ascending;
    }
}
```

The logic:
1. If clicking the same column: toggle direction (asc ↔ desc)
2. If clicking a different column: set new column, reset to ascending

## Releasing Locks

The most dangerous operation—force-closing file handles:

```rust
fn release_selected_locks(&mut self) {
    let ids: Vec<u32> = self.selected_ids.iter().cloned().collect();
    if ids.is_empty() {
        return;
    }

    let mut success_count = 0;
    let mut errors: Vec<String> = Vec::new();

    for id in &ids {
        match release_lock(*id) {
            Ok(()) => success_count += 1,
            Err(e) => {
                if let Some(lock) = self.all_locks.iter().find(|l| l.file_id == *id) {
                    errors.push(format!("{}: {}", lock.file_name, e));
                } else {
                    errors.push(format!("ID {}: {}", id, e));
                }
            }
        }
    }

    if success_count > 0 {
        self.show_message(
            format!("Successfully released {} lock(s)", success_count),
            MessageType::Success,
        );
    }

    if !errors.is_empty() {
        let error_text = if errors.len() > 3 {
            format!("{} failed: {}, ...", errors.len(), errors[..3].join(", "))
        } else {
            format!("{} failed: {}", errors.len(), errors.join(", "))
        };
        self.show_message(error_text, MessageType::Error);
    }

    // Refresh the list
    self.refresh_locks();
}
```

### Collecting IDs

```rust
let ids: Vec<u32> = self.selected_ids.iter().cloned().collect();
```

We collect into a `Vec` because we need to iterate while potentially modifying state. Can't borrow `selected_ids` immutably while mutating `self`.

### Finding Locks by ID

```rust
if let Some(lock) = self.all_locks.iter().find(|l| l.file_id == *id) {
    errors.push(format!("{}: {}", lock.file_name, e));
}
```

The `find` method returns `Option<&FileLockInfo>`—the first item matching the predicate, or `None`.

`if let Some(lock) = ...` is pattern matching that only executes if the value is `Some`.

### Truncating Error Lists

```rust
if errors.len() > 3 {
    format!("{} failed: {}, ...", errors.len(), errors[..3].join(", "))
}
```

The `errors[..3]` syntax creates a **slice** of the first 3 elements. `.join(", ")` concatenates them with commas.

This prevents overwhelming the user with a wall of error text.

## Select All Visible

```rust
fn select_all_visible(&mut self) {
    let ids: Vec<u32> = self
        .get_filtered_sorted_locks()
        .iter()
        .map(|lock| lock.file_id)
        .collect();
    for id in ids {
        self.selected_ids.insert(id);
    }
}
```

### The map() Adapter

```rust
.map(|lock| lock.file_id)
```

Transforms each item. Here, we extract `file_id` from each lock. This transforms `Iterator<Item = &FileLockInfo>` into `Iterator<Item = u32>`.

### Why Not Just filtered_locks.iter()?

We could write:
```rust
for lock in self.get_filtered_sorted_locks() {
    self.selected_ids.insert(lock.file_id);
}
```

But this would create the vector and immediately iterate it. The current approach collects IDs first, avoiding borrowing issues (we call `get_filtered_sorted_locks` which borrows `self`, then we mutate `self.selected_ids`).

## Persistence

The egui framework calls `save` when the app closes:

```rust
impl eframe::App for LockSmithApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let persistence = AppPersistence {
            dark_mode: self.dark_mode,
        };
        eframe::set_value(storage, eframe::APP_KEY, &persistence);
    }
    
    // ... update method
}
```

### The dyn Keyword

```rust
storage: &mut dyn eframe::Storage
```

`dyn Storage` is a **trait object**—a reference to some type that implements `Storage`, where the specific type isn't known at compile time.

This is Rust's dynamic dispatch (like virtual methods in C++/C#).

### Serialization

```rust
eframe::set_value(storage, eframe::APP_KEY, &persistence);
```

This serializes `persistence` (using serde) and stores it with the given key. On next launch, `get_value` retrieves it.

## Putting It All Together

Here's the complete application logic implementation:

```rust
impl LockSmithApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load persisted state
        let persistence: AppPersistence = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();

        let mut app = Self {
            dark_mode: persistence.dark_mode,
            ..Default::default()
        };

        // Apply theme
        app.apply_theme(cc.egui_ctx.clone());

        // Check admin status
        app.is_admin = is_admin();

        // Initial load
        app.refresh_locks();

        app
    }

    fn apply_theme(&self, ctx: egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }

    fn show_message(&mut self, text: impl Into<String>, message_type: MessageType) {
        self.status_message = Some(StatusMessage {
            text: text.into(),
            message_type,
            timestamp: std::time::Instant::now(),
        });
    }

    fn refresh_locks(&mut self) {
        self.is_loading = true;
        self.selected_ids.clear();
        self.last_selected_index = None;

        match enumerate_files() {
            Ok(locks) => {
                let count = locks.len();
                self.all_locks = locks;
                self.is_loading = false;

                if count > 0 {
                    self.show_message(
                        format!("Loaded {} file lock(s)", count),
                        MessageType::Success,
                    );
                } else {
                    self.show_message(
                        "No file locks found. System is ready.",
                        MessageType::Info,
                    );
                }
            }
            Err(e) => {
                self.all_locks.clear();
                self.is_loading = false;
                self.show_message(e.to_string(), MessageType::Error);
            }
        }
    }

    fn get_filtered_sorted_locks(&self) -> Vec<FileLockInfo> {
        let query = self.search_query.to_lowercase();

        let mut filtered: Vec<FileLockInfo> = self
            .all_locks
            .iter()
            .filter(|lock| {
                if query.is_empty() {
                    true
                } else {
                    lock.file_name.to_lowercase().contains(&query)
                        || lock.file_path.to_lowercase().contains(&query)
                        || lock.username.to_lowercase().contains(&query)
                }
            })
            .cloned()
            .collect();

        // Sort
        if let Some(column) = &self.sort_column {
            filtered.sort_by(|a, b| {
                let cmp = match column {
                    SortColumn::FileName => {
                        a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase())
                    }
                    SortColumn::FilePath => {
                        a.file_path.to_lowercase().cmp(&b.file_path.to_lowercase())
                    }
                    SortColumn::Username => {
                        a.username.to_lowercase().cmp(&b.username.to_lowercase())
                    }
                    SortColumn::NumLocks => a.num_locks.cmp(&b.num_locks),
                };

                match self.sort_direction {
                    SortDirection::Ascending => cmp,
                    SortDirection::Descending => cmp.reverse(),
                }
            });
        }

        filtered
    }

    fn handle_sort(&mut self, column: SortColumn) {
        if self.sort_column == Some(column) {
            self.sort_direction = self.sort_direction.toggle();
        } else {
            self.sort_column = Some(column);
            self.sort_direction = SortDirection::Ascending;
        }
    }

    fn release_selected_locks(&mut self) {
        let ids: Vec<u32> = self.selected_ids.iter().cloned().collect();
        if ids.is_empty() {
            return;
        }

        let mut success_count = 0;
        let mut errors: Vec<String> = Vec::new();

        for id in &ids {
            match release_lock(*id) {
                Ok(()) => success_count += 1,
                Err(e) => {
                    if let Some(lock) = self.all_locks.iter().find(|l| l.file_id == *id) {
                        errors.push(format!("{}: {}", lock.file_name, e));
                    } else {
                        errors.push(format!("ID {}: {}", id, e));
                    }
                }
            }
        }

        if success_count > 0 {
            self.show_message(
                format!("Successfully released {} lock(s)", success_count),
                MessageType::Success,
            );
        }

        if !errors.is_empty() {
            let error_text = if errors.len() > 3 {
                format!("{} failed: {}, ...", errors.len(), errors[..3].join(", "))
            } else {
                format!("{} failed: {}", errors.len(), errors.join(", "))
            };
            self.show_message(error_text, MessageType::Error);
        }

        // Refresh the list
        self.refresh_locks();
    }

    fn select_all_visible(&mut self) {
        let ids: Vec<u32> = self
            .get_filtered_sorted_locks()
            .iter()
            .map(|lock| lock.file_id)
            .collect();
        for id in ids {
            self.selected_ids.insert(id);
        }
    }
}
```

## What We've Accomplished

You now understand:

✅ Application initialization — Loading persisted state, setting up defaults  
✅ State management — Tracking loading states, selections, messages  
✅ Iterator patterns — filter, map, cloned, collect  
✅ Closures — Capturing environment variables  
✅ Error handling — Match on Result, collecting errors  
✅ Persistence — Saving/loading with serde  

## Key Takeaways

1. **Iterators are powerful** — Filter, map, collect form expressive data pipelines
2. **Closures capture context** — They can use variables from the surrounding scope
3. **State management requires care** — Clear old state before loading new
4. **User feedback matters** — Show success/error messages for all operations
5. **Defensive programming** — Handle empty inputs, truncate long outputs

## Looking Ahead

In the next chapter, we'll build the user interface using egui. We'll learn about the immediate-mode paradigm and how to create responsive layouts.

---

**Previous Chapter:** [← Windows API Integration](04-windows-api.md)  
**Next Chapter:** [Building the User Interface →](06-user-interface.md)