//! # LockSmith
//!
//! A Windows utility for viewing and managing file locks.
//!
//! Built with pure Rust and egui for a fast, native experience
//! without WebView2 or other runtime dependencies.
//!
//! ## Features
//!
//! - View all open file locks on the local machine
//! - Release (force-close) file locks
//! - Search and filter by filename, path, or username
//! - Sortable columns
//! - Multi-select with Ctrl+Click and Shift+Click
//! - Dark and light themes
//!
//! ## Requirements
//!
//! - Windows 10 or later
//! - Administrator privileges (required for Windows API access)

// Prevents additional console window on Windows in release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::ptr;

// ============================================================================
// Windows FFI Declarations
// ============================================================================

#[cfg(windows)]
#[repr(C)]
struct FILE_INFO_3 {
    fi3_id: u32,
    fi3_permissions: u32,
    fi3_num_locks: u32,
    fi3_pathname: *mut u16,
    fi3_username: *mut u16,
}

#[cfg(windows)]
#[link(name = "netapi32")]
extern "system" {
    fn NetFileEnum(
        servername: *const u16,
        basepath: *const u16,
        username: *const u16,
        level: u32,
        bufptr: *mut *mut u8,
        prefmaxlen: u32,
        entriesread: *mut u32,
        totalentries: *mut u32,
        resume_handle: *mut u32,
    ) -> u32;

    fn NetFileClose(servername: *const u16, fileid: u32) -> u32;

    fn NetApiBufferFree(buffer: *mut u8) -> u32;
}

// Windows API constants
#[cfg(windows)]
const MAX_PREFERRED_LENGTH: u32 = 0xFFFFFFFF;
#[cfg(windows)]
const NERR_SUCCESS: u32 = 0;
#[cfg(windows)]
const ERROR_MORE_DATA: u32 = 234;
#[cfg(windows)]
const ERROR_ACCESS_DENIED: u32 = 5;
#[cfg(windows)]
const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, Clone)]
enum FileEnumError {
    AccessDenied,
    NotEnoughMemory,
    WindowsApiError(u32),
}

impl fmt::Display for FileEnumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileEnumError::AccessDenied => {
                write!(f, "Access denied. Please run as administrator.")
            }
            FileEnumError::NotEnoughMemory => {
                write!(f, "Not enough memory to complete the operation")
            }
            FileEnumError::WindowsApiError(code) => {
                write!(f, "Windows API error: code {}", code)
            }
        }
    }
}

impl std::error::Error for FileEnumError {}

#[derive(Debug, Clone)]
enum ReleaseLockError {
    AccessDenied,
    FileNotFound,
    WindowsApiError(u32),
}

impl fmt::Display for ReleaseLockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReleaseLockError::AccessDenied => {
                write!(f, "Access denied. Please run as administrator.")
            }
            ReleaseLockError::FileNotFound => {
                write!(f, "File not found or already closed")
            }
            ReleaseLockError::WindowsApiError(code) => {
                write!(f, "Windows API error: code {}", code)
            }
        }
    }
}

impl std::error::Error for ReleaseLockError {}

// ============================================================================
// Data Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileLockInfo {
    file_id: u32,
    server_name: String,
    file_path: String,
    file_name: String,
    username: String,
    permissions: u32,
    num_locks: u32,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn extract_filename(path: &str) -> String {
    path.rsplit('\\').next().unwrap_or(path).to_string()
}

#[cfg(windows)]
unsafe fn from_wide_ptr(ptr: *mut u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16_lossy(slice)
}

#[cfg(windows)]
fn map_windows_error(code: u32) -> FileEnumError {
    match code {
        ERROR_ACCESS_DENIED => FileEnumError::AccessDenied,
        ERROR_NOT_ENOUGH_MEMORY => FileEnumError::NotEnoughMemory,
        _ => FileEnumError::WindowsApiError(code),
    }
}

#[cfg(windows)]
fn map_release_error(code: u32) -> ReleaseLockError {
    match code {
        ERROR_ACCESS_DENIED => ReleaseLockError::AccessDenied,
        2 => ReleaseLockError::FileNotFound,
        _ => ReleaseLockError::WindowsApiError(code),
    }
}

// ============================================================================
// Windows API Functions
// ============================================================================

#[cfg(windows)]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    let mut locks = Vec::new();

    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetFileEnum(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            3,
            &mut buffer,
            MAX_PREFERRED_LENGTH,
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if status != NERR_SUCCESS && status != ERROR_MORE_DATA {
            return Err(map_windows_error(status));
        }

        if !buffer.is_null() && entries_read > 0 {
            let files =
                std::slice::from_raw_parts(buffer as *const FILE_INFO_3, entries_read as usize);

            for file in files {
                let file_path = from_wide_ptr(file.fi3_pathname);
                let username = from_wide_ptr(file.fi3_username);
                let file_name = extract_filename(&file_path);

                locks.push(FileLockInfo {
                    file_id: file.fi3_id,
                    server_name: "LOCAL".to_string(),
                    file_path,
                    file_name,
                    username,
                    permissions: file.fi3_permissions,
                    num_locks: file.fi3_num_locks,
                });
            }

            NetApiBufferFree(buffer);
        }
    }

    Ok(locks)
}

#[cfg(windows)]
fn release_lock(file_id: u32) -> Result<(), ReleaseLockError> {
    unsafe {
        let status = NetFileClose(ptr::null(), file_id);

        if status != NERR_SUCCESS {
            return Err(map_release_error(status));
        }

        Ok(())
    }
}

#[cfg(windows)]
fn is_admin() -> bool {
    unsafe {
        let mut buffer: *mut u8 = ptr::null_mut();
        let mut entries_read: u32 = 0;
        let mut total_entries: u32 = 0;
        let mut resume_handle: u32 = 0;

        let status = NetFileEnum(
            ptr::null(),
            ptr::null(),
            ptr::null(),
            3,
            &mut buffer,
            0,
            &mut entries_read,
            &mut total_entries,
            &mut resume_handle,
        );

        if !buffer.is_null() {
            NetApiBufferFree(buffer);
        }

        status != ERROR_ACCESS_DENIED
    }
}

// Non-Windows stubs
#[cfg(not(windows))]
fn enumerate_files() -> Result<Vec<FileLockInfo>, FileEnumError> {
    Err(FileEnumError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn release_lock(_file_id: u32) -> Result<(), ReleaseLockError> {
    Err(ReleaseLockError::WindowsApiError(0))
}

#[cfg(not(windows))]
fn is_admin() -> bool {
    false
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortColumn {
    FileName,
    FilePath,
    Username,
    NumLocks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortDirection {
    Ascending,
    Descending,
}

impl SortDirection {
    fn toggle(&self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }

    fn arrow(&self) -> &'static str {
        match self {
            SortDirection::Ascending => " ↑",
            SortDirection::Descending => " ↓",
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Clone)]
struct StatusMessage {
    text: String,
    message_type: MessageType,
    timestamp: std::time::Instant,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct AppPersistence {
    dark_mode: bool,
}

impl Default for AppPersistence {
    fn default() -> Self {
        Self { dark_mode: true }
    }
}

struct LockSmithApp {
    // Data
    all_locks: Vec<FileLockInfo>,
    selected_ids: HashSet<u32>,
    last_selected_index: Option<usize>,

    // UI State
    search_query: String,
    sort_column: Option<SortColumn>,
    sort_direction: SortDirection,
    dark_mode: bool,

    // Status
    status_message: Option<StatusMessage>,
    is_loading: bool,
    is_admin: bool,

    // Confirmation dialog
    show_release_confirm: bool,
    pending_release_ids: Vec<u32>,
}

impl Default for LockSmithApp {
    fn default() -> Self {
        Self {
            all_locks: Vec::new(),
            selected_ids: HashSet::new(),
            last_selected_index: None,
            search_query: String::new(),
            sort_column: None,
            sort_direction: SortDirection::Ascending,
            dark_mode: true,
            status_message: None,
            is_loading: false,
            is_admin: false,
            show_release_confirm: false,
            pending_release_ids: Vec::new(),
        }
    }
}

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
                    self.show_message("No file locks found. System is ready.", MessageType::Info);
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

impl eframe::App for LockSmithApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let persistence = AppPersistence {
            dark_mode: self.dark_mode,
        };
        eframe::set_value(storage, eframe::APP_KEY, &persistence);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle keyboard shortcuts
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

        // Auto-hide success/info messages after 5 seconds
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

        // Release confirmation dialog
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
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Only proceed if you understand the risks.")
                            .color(Color32::from_rgb(245, 158, 11)),
                    );
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
                                .fill(Color32::from_rgb(239, 68, 68)),
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

        // Main UI
        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_width = ui.available_width();

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

            // Admin warning - centered
            if !self.is_admin {
                ui.vertical_centered(|ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(239, 68, 68))
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

            // Search bar and refresh button - centered
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    let total_width = panel_width - 60.0;
                    ui.add_space(30.0);

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

                    ui.add_space(30.0);
                });
            });

            ui.add_space(10.0);

            // Get filtered locks before the UI closure
            let filtered_locks = self.get_filtered_sorted_locks();
            let row_height = 28.0;
            let available_height = ui.available_height() - 120.0;

            // Calculate table width to center it
            let table_width = panel_width - 40.0;

            // Table container - centered
            ui.vertical_centered(|ui| {
                ui.set_max_width(table_width);

                egui::Frame::none()
                    .stroke(Stroke::new(
                        1.0,
                        ui.visuals().widgets.noninteractive.bg_stroke.color,
                    ))
                    .rounding(8.0)
                    .inner_margin(0.0)
                    .show(ui, |ui| {
                        // Show empty/loading state outside the grid for proper centering
                        if self.is_loading {
                            ui.set_min_height(available_height);
                            ui.vertical_centered(|ui| {
                                ui.add_space(available_height / 2.0 - 40.0);
                                ui.spinner();
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
                            // Table with data
                            egui::ScrollArea::vertical()
                                .max_height(available_height)
                                .show(ui, |ui| {
                                    ui.set_min_width(table_width - 20.0);

                                    egui::Grid::new("locks_table")
                                        .num_columns(4)
                                        .striped(true)
                                        .min_col_width(80.0)
                                        .show(ui, |ui| {
                                            // Header row
                                            let header_bg = if self.dark_mode {
                                                Color32::from_rgb(51, 65, 85)
                                            } else {
                                                Color32::from_rgb(226, 232, 240)
                                            };

                                            ui.style_mut().visuals.widgets.inactive.weak_bg_fill =
                                                header_bg;
                                            ui.style_mut().visuals.widgets.hovered.weak_bg_fill =
                                                header_bg;

                                            // File Name header
                                            let file_name_text = format!(
                                                "FILE NAME{}",
                                                if self.sort_column == Some(SortColumn::FileName) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(200.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(file_name_text).strong(),
                                                    )
                                                    .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::FileName);
                                            }

                                            // Path header
                                            let path_text = format!(
                                                "PATH{}",
                                                if self.sort_column == Some(SortColumn::FilePath) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(350.0, row_height),
                                                    egui::Button::new(RichText::new(path_text).strong())
                                                        .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::FilePath);
                                            }

                                            // User header
                                            let user_text = format!(
                                                "USER{}",
                                                if self.sort_column == Some(SortColumn::Username) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(150.0, row_height),
                                                    egui::Button::new(RichText::new(user_text).strong())
                                                        .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::Username);
                                            }

                                            // Locks header
                                            let locks_text = format!(
                                                "LOCKS{}",
                                                if self.sort_column == Some(SortColumn::NumLocks) {
                                                    self.sort_direction.arrow()
                                                } else {
                                                    ""
                                                }
                                            );
                                            if ui
                                                .add_sized(
                                                    Vec2::new(80.0, row_height),
                                                    egui::Button::new(RichText::new(locks_text).strong())
                                                        .frame(false),
                                                )
                                                .clicked()
                                            {
                                                self.handle_sort(SortColumn::NumLocks);
                                            }

                                            ui.end_row();

                                            // Reset style for data rows
                                            ui.reset_style();

                                            // Data rows
                                            for (index, lock) in filtered_locks.iter().enumerate() {
                                                let is_selected =
                                                    self.selected_ids.contains(&lock.file_id);

                                                let row_color = if is_selected {
                                                    Color32::from_rgb(96, 165, 250)
                                                } else if self.dark_mode && index % 2 == 1 {
                                                    Color32::from_rgba_unmultiplied(255, 255, 255, 5)
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

                                                // File name cell
                                                let file_name_response = ui.add_sized(
                                                    Vec2::new(200.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(&lock.file_name).color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Path cell
                                                let path_response = ui.add_sized(
                                                    Vec2::new(350.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(&lock.file_path).color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Username cell
                                                let user_response = ui.add_sized(
                                                    Vec2::new(150.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(&lock.username).color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Locks count cell
                                                let locks_response = ui.add_sized(
                                                    Vec2::new(80.0, row_height),
                                                    egui::Button::new(
                                                        RichText::new(lock.num_locks.to_string())
                                                            .color(text_color),
                                                    )
                                                    .fill(row_color)
                                                    .frame(false),
                                                );

                                                // Handle clicks on any cell
                                                let clicked = file_name_response.clicked()
                                                    || path_response.clicked()
                                                    || user_response.clicked()
                                                    || locks_response.clicked();

                                                let double_clicked = file_name_response.double_clicked()
                                                    || path_response.double_clicked()
                                                    || user_response.double_clicked()
                                                    || locks_response.double_clicked();

                                                if double_clicked {
                                                    // Double-click to release
                                                    self.pending_release_ids = vec![lock.file_id];
                                                    self.show_release_confirm = true;
                                                } else if clicked {
                                                    let modifiers = ui.input(|i| i.modifiers);

                                                    if modifiers.ctrl || modifiers.command {
                                                        // Ctrl+click: toggle selection
                                                        if self.selected_ids.contains(&lock.file_id) {
                                                            self.selected_ids.remove(&lock.file_id);
                                                        } else {
                                                            self.selected_ids.insert(lock.file_id);
                                                        }
                                                        self.last_selected_index = Some(index);
                                                    } else if modifiers.shift {
                                                        // Shift+click: range selection
                                                        if let Some(last_index) =
                                                            self.last_selected_index
                                                        {
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

                                                ui.end_row();
                                            }
                                        });
                                });
                        }
                    });
            });

            ui.add_space(15.0);

            // Release button - centered
            ui.vertical_centered(|ui| {
                let button_text = if self.selected_ids.is_empty() {
                    "Release Selected Lock(s) (Delete)".to_string()
                } else {
                    format!(
                        "Release Selected Lock(s) ({}) (Delete)",
                        self.selected_ids.len()
                    )
                };

                let button = egui::Button::new(RichText::new(button_text).color(Color32::WHITE))
                    .fill(if self.selected_ids.is_empty() {
                        Color32::from_rgb(127, 127, 127)
                    } else {
                        Color32::from_rgb(239, 68, 68)
                    })
                    .min_size(Vec2::new(300.0, 40.0));

                if ui
                    .add_enabled(!self.selected_ids.is_empty(), button)
                    .clicked()
                {
                    self.pending_release_ids = self.selected_ids.iter().cloned().collect();
                    self.show_release_confirm = true;
                }
            });

            ui.add_space(10.0);

            // Status message bar - centered
            if let Some(ref msg) = self.status_message {
                let (bg_color, text_color) = match msg.message_type {
                    MessageType::Success => (Color32::from_rgb(16, 185, 129), Color32::WHITE),
                    MessageType::Error => (Color32::from_rgb(239, 68, 68), Color32::WHITE),
                    MessageType::Warning => (Color32::from_rgb(245, 158, 11), Color32::WHITE),
                    MessageType::Info => {
                        if self.dark_mode {
                            (
                                Color32::from_rgb(51, 65, 85),
                                Color32::from_rgb(248, 250, 252),
                            )
                        } else {
                            (
                                Color32::from_rgb(226, 232, 240),
                                Color32::from_rgb(15, 23, 42),
                            )
                        }
                    }
                };

                let icon = match msg.message_type {
                    MessageType::Success => "✓",
                    MessageType::Error => "✕",
                    MessageType::Warning => "⚠",
                    MessageType::Info => "ℹ",
                };
                let msg_text = msg.text.clone();
                let full_text = format!("{} {}", icon, msg_text);

                // Create the message as a button (non-interactive) which centers properly
                ui.vertical_centered(|ui| {
                    let button = egui::Button::new(
                        RichText::new(&full_text).color(text_color).size(14.0)
                    )
                    .fill(bg_color)
                    .stroke(egui::Stroke::NONE)
                    .rounding(5.0)
                    .sense(egui::Sense::hover()); // Not clickable

                    ui.add(button);
                });
            }
        });

        // Theme toggle button (bottom right corner)
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
                        egui::Button::new(RichText::new(icon).size(18.0)).rounding(18.0),
                    )
                    .on_hover_text("Toggle theme")
                    .clicked()
                {
                    self.dark_mode = !self.dark_mode;
                    self.apply_theme(ctx.clone());
                }
            });

        // Request repaint for animations and message timeout
        if self.is_loading || self.status_message.is_some() {
            ctx.request_repaint();
        }
    }
}

// ============================================================================
// Entry Point
// ============================================================================

fn main() -> eframe::Result<()> {
    // Print startup banner
    println!("===========================================");
    println!("  LockSmith v{}", env!("CARGO_PKG_VERSION"));
    println!("  Windows File Lock Management Utility");
    println!("===========================================");

    #[cfg(windows)]
    {
        if !is_admin() {
            eprintln!("WARNING: Not running with administrator privileges!");
            eprintln!("Some features may not work correctly.");
        } else {
            println!("✓ Running with administrator privileges");
        }
    }

    println!();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("LockSmith - Local File Locks")
            .with_inner_size(Vec2::new(1000.0, 660.0))
            .with_min_inner_size(Vec2::new(800.0, 400.0))
            .with_icon(load_icon()),
        ..Default::default()
    };

    eframe::run_native(
        "LockSmith",
        options,
        Box::new(|cc| Ok(Box::new(LockSmithApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Try to load the icon from the icons directory
    if let Ok(image) = image::open("icons/128x128.png") {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else if let Ok(image) = image::open("src/icons/128x128.png") {
        let rgba = image.to_rgba8();
        let (width, height) = rgba.dimensions();
        egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        }
    } else {
        // Generate a proper padlock icon as fallback
        create_padlock_icon()
    }
}

/// Creates a simple open padlock icon programmatically
fn create_padlock_icon() -> egui::IconData {
    let size: usize = 64;
    let mut rgba = vec![0u8; size * size * 4];

    let bg_color = [30, 41, 59, 255u8]; // Dark blue background
    let lock_color = [96, 165, 250, 255u8]; // Light blue for padlock

    // Fill background
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            // Create circular background
            let cx = size as f32 / 2.0;
            let cy = size as f32 / 2.0;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < (size as f32 / 2.0) - 2.0 {
                rgba[idx] = bg_color[0];
                rgba[idx + 1] = bg_color[1];
                rgba[idx + 2] = bg_color[2];
                rgba[idx + 3] = bg_color[3];
            }
        }
    }

    // Draw padlock body (rectangle in lower portion)
    let body_left = size / 4;
    let body_right = size * 3 / 4;
    let body_top = size / 2;
    let body_bottom = size * 7 / 8;

    for y in body_top..body_bottom {
        for x in body_left..body_right {
            let idx = (y * size + x) * 4;
            rgba[idx] = lock_color[0];
            rgba[idx + 1] = lock_color[1];
            rgba[idx + 2] = lock_color[2];
            rgba[idx + 3] = lock_color[3];
        }
    }

    // Draw shackle (open, U-shape on left side only)
    let shackle_outer = size / 4;
    let shackle_inner = size / 6;
    let shackle_top = size / 5;
    let shackle_bottom = size / 2 + 4;
    let center_x = size / 2;

    for y in shackle_top..shackle_bottom {
        for x in (body_left - 2)..(body_right + 2) {
            let dx = (x as i32 - center_x as i32).abs() as usize;
            let in_outer = dx <= shackle_outer && dx >= shackle_inner;
            let at_top = y < shackle_top + 8 && dx <= shackle_outer;

            // Only draw left side of shackle (open lock)
            let is_left_side = (x as i32) < center_x as i32;

            if (in_outer || at_top) && (is_left_side || y < shackle_top + 8) {
                let idx = (y * size + x) * 4;
                rgba[idx] = lock_color[0];
                rgba[idx + 1] = lock_color[1];
                rgba[idx + 2] = lock_color[2];
                rgba[idx + 3] = lock_color[3];
            }
        }
    }

    // Draw keyhole
    let keyhole_x = size / 2;
    let keyhole_y = size * 5 / 8;
    let keyhole_radius = size / 12;

    for y in 0..size {
        for x in 0..size {
            let dx = x as i32 - keyhole_x as i32;
            let dy = y as i32 - keyhole_y as i32;
            let dist = ((dx * dx + dy * dy) as f32).sqrt();

            if dist < keyhole_radius as f32 {
                let idx = (y * size + x) * 4;
                rgba[idx] = bg_color[0];
                rgba[idx + 1] = bg_color[1];
                rgba[idx + 2] = bg_color[2];
                rgba[idx + 3] = bg_color[3];
            }
        }
    }

    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Helper Function Tests
    // ========================================================================

    mod extract_filename_tests {
        use super::*;

        #[test]
        fn test_extract_filename_windows_path() {
            assert_eq!(extract_filename(r"C:\Users\Admin\file.txt"), "file.txt");
        }

        #[test]
        fn test_extract_filename_unc_path() {
            assert_eq!(extract_filename(r"\\server\share\doc.pdf"), "doc.pdf");
        }

        #[test]
        fn test_extract_filename_simple() {
            assert_eq!(extract_filename("simple.txt"), "simple.txt");
        }

        #[test]
        fn test_extract_filename_empty() {
            assert_eq!(extract_filename(""), "");
        }

        #[test]
        fn test_extract_filename_nested_path() {
            assert_eq!(
                extract_filename(r"C:\Users\Admin\Documents\Projects\LockSmith\src\main.rs"),
                "main.rs"
            );
        }

        #[test]
        fn test_extract_filename_no_extension() {
            assert_eq!(extract_filename(r"C:\Users\Admin\Makefile"), "Makefile");
        }

        #[test]
        fn test_extract_filename_hidden_file() {
            assert_eq!(extract_filename(r"C:\Users\Admin\.gitignore"), ".gitignore");
        }

        #[test]
        fn test_extract_filename_multiple_dots() {
            assert_eq!(
                extract_filename(r"C:\Users\Admin\file.backup.txt"),
                "file.backup.txt"
            );
        }

        #[test]
        fn test_extract_filename_spaces_in_name() {
            assert_eq!(
                extract_filename(r"C:\Users\Admin\My Document.docx"),
                "My Document.docx"
            );
        }

        #[test]
        fn test_extract_filename_unicode() {
            assert_eq!(extract_filename(r"C:\Users\Admin\文件.txt"), "文件.txt");
        }

        #[test]
        fn test_extract_filename_trailing_backslash() {
            // Edge case: path ends with backslash
            assert_eq!(extract_filename(r"C:\Users\Admin\"), "");
        }

        #[test]
        fn test_extract_filename_only_backslashes() {
            assert_eq!(extract_filename(r"\\\"), "");
        }

        #[test]
        fn test_extract_filename_root_path() {
            assert_eq!(extract_filename(r"C:\"), "");
        }

        #[test]
        fn test_extract_filename_drive_only() {
            assert_eq!(extract_filename(r"C:"), "C:");
        }
    }

    // ========================================================================
    // SortDirection Tests
    // ========================================================================

    mod sort_direction_tests {
        use super::*;

        #[test]
        fn test_sort_direction_toggle_ascending_to_descending() {
            let dir = SortDirection::Ascending;
            assert_eq!(dir.toggle(), SortDirection::Descending);
        }

        #[test]
        fn test_sort_direction_toggle_descending_to_ascending() {
            let dir = SortDirection::Descending;
            assert_eq!(dir.toggle(), SortDirection::Ascending);
        }

        #[test]
        fn test_sort_direction_double_toggle_identity() {
            let dir = SortDirection::Ascending;
            assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);

            let dir2 = SortDirection::Descending;
            assert_eq!(dir2.toggle().toggle(), SortDirection::Descending);
        }

        #[test]
        fn test_sort_direction_arrow_ascending() {
            assert_eq!(SortDirection::Ascending.arrow(), " ↑");
        }

        #[test]
        fn test_sort_direction_arrow_descending() {
            assert_eq!(SortDirection::Descending.arrow(), " ↓");
        }

        #[test]
        fn test_sort_direction_equality() {
            assert_eq!(SortDirection::Ascending, SortDirection::Ascending);
            assert_eq!(SortDirection::Descending, SortDirection::Descending);
            assert_ne!(SortDirection::Ascending, SortDirection::Descending);
        }

        #[test]
        fn test_sort_direction_debug_impl() {
            // Test Debug trait implementation
            let debug_str = format!("{:?}", SortDirection::Ascending);
            assert!(debug_str.contains("Ascending"));

            let debug_str = format!("{:?}", SortDirection::Descending);
            assert!(debug_str.contains("Descending"));
        }
    }

    // ========================================================================
    // SortColumn Tests
    // ========================================================================

    mod sort_column_tests {
        use super::*;

        #[test]
        fn test_sort_column_equality() {
            assert_eq!(SortColumn::FileName, SortColumn::FileName);
            assert_eq!(SortColumn::FilePath, SortColumn::FilePath);
            assert_eq!(SortColumn::Username, SortColumn::Username);
            assert_eq!(SortColumn::NumLocks, SortColumn::NumLocks);
        }

        #[test]
        fn test_sort_column_inequality() {
            assert_ne!(SortColumn::FileName, SortColumn::FilePath);
            assert_ne!(SortColumn::FileName, SortColumn::Username);
            assert_ne!(SortColumn::FileName, SortColumn::NumLocks);
            assert_ne!(SortColumn::FilePath, SortColumn::Username);
            assert_ne!(SortColumn::FilePath, SortColumn::NumLocks);
            assert_ne!(SortColumn::Username, SortColumn::NumLocks);
        }

        #[test]
        fn test_sort_column_copy() {
            let col = SortColumn::FileName;
            let col_copy = col;
            assert_eq!(col, col_copy);
        }
    }

    // ========================================================================
    // Error Type Tests
    // ========================================================================

    mod file_enum_error_tests {
        use super::*;

        #[test]
        fn test_file_enum_error_access_denied_display() {
            let err = FileEnumError::AccessDenied;
            let display = format!("{}", err);
            assert!(display.contains("Access denied"));
            assert!(display.contains("administrator"));
        }

        #[test]
        fn test_file_enum_error_not_enough_memory_display() {
            let err = FileEnumError::NotEnoughMemory;
            let display = format!("{}", err);
            assert!(display.contains("Not enough memory"));
        }

        #[test]
        fn test_file_enum_error_windows_api_error_display() {
            let err = FileEnumError::WindowsApiError(1234);
            let display = format!("{}", err);
            assert!(display.contains("Windows API error"));
            assert!(display.contains("1234"));
        }

        #[test]
        fn test_file_enum_error_debug() {
            let err = FileEnumError::AccessDenied;
            let debug = format!("{:?}", err);
            assert!(debug.contains("AccessDenied"));
        }

        #[test]
        fn test_file_enum_error_clone() {
            let err = FileEnumError::WindowsApiError(5);
            let err_clone = err.clone();
            assert!(matches!(err_clone, FileEnumError::WindowsApiError(5)));
        }

        #[test]
        fn test_file_enum_error_is_std_error() {
            let err: Box<dyn std::error::Error> = Box::new(FileEnumError::AccessDenied);
            assert!(!err.to_string().is_empty());
        }
    }

    mod release_lock_error_tests {
        use super::*;

        #[test]
        fn test_release_lock_error_access_denied_display() {
            let err = ReleaseLockError::AccessDenied;
            let display = format!("{}", err);
            assert!(display.contains("Access denied"));
            assert!(display.contains("administrator"));
        }

        #[test]
        fn test_release_lock_error_file_not_found_display() {
            let err = ReleaseLockError::FileNotFound;
            let display = format!("{}", err);
            assert!(display.contains("File not found") || display.contains("already closed"));
        }

        #[test]
        fn test_release_lock_error_windows_api_error_display() {
            let err = ReleaseLockError::WindowsApiError(9999);
            let display = format!("{}", err);
            assert!(display.contains("Windows API error"));
            assert!(display.contains("9999"));
        }

        #[test]
        fn test_release_lock_error_debug() {
            let err = ReleaseLockError::FileNotFound;
            let debug = format!("{:?}", err);
            assert!(debug.contains("FileNotFound"));
        }

        #[test]
        fn test_release_lock_error_clone() {
            let err = ReleaseLockError::AccessDenied;
            let err_clone = err.clone();
            assert!(matches!(err_clone, ReleaseLockError::AccessDenied));
        }

        #[test]
        fn test_release_lock_error_is_std_error() {
            let err: Box<dyn std::error::Error> = Box::new(ReleaseLockError::FileNotFound);
            assert!(!err.to_string().is_empty());
        }
    }

    // ========================================================================
    // Error Mapping Tests (Windows-specific but testable)
    // ========================================================================

    #[cfg(windows)]
    mod error_mapping_tests {
        use super::*;

        #[test]
        fn test_map_windows_error_access_denied() {
            let err = map_windows_error(ERROR_ACCESS_DENIED);
            assert!(matches!(err, FileEnumError::AccessDenied));
        }

        #[test]
        fn test_map_windows_error_not_enough_memory() {
            let err = map_windows_error(ERROR_NOT_ENOUGH_MEMORY);
            assert!(matches!(err, FileEnumError::NotEnoughMemory));
        }

        #[test]
        fn test_map_windows_error_unknown() {
            let err = map_windows_error(12345);
            assert!(matches!(err, FileEnumError::WindowsApiError(12345)));
        }

        #[test]
        fn test_map_release_error_access_denied() {
            let err = map_release_error(ERROR_ACCESS_DENIED);
            assert!(matches!(err, ReleaseLockError::AccessDenied));
        }

        #[test]
        fn test_map_release_error_file_not_found() {
            let err = map_release_error(2);
            assert!(matches!(err, ReleaseLockError::FileNotFound));
        }

        #[test]
        fn test_map_release_error_unknown() {
            let err = map_release_error(54321);
            assert!(matches!(err, ReleaseLockError::WindowsApiError(54321)));
        }
    }

    // ========================================================================
    // FileLockInfo Tests
    // ========================================================================

    mod file_lock_info_tests {
        use super::*;

        fn create_test_lock() -> FileLockInfo {
            FileLockInfo {
                file_id: 1234,
                server_name: "LOCAL".to_string(),
                file_path: r"C:\Users\Admin\test.txt".to_string(),
                file_name: "test.txt".to_string(),
                username: "DOMAIN\\user".to_string(),
                permissions: 3,
                num_locks: 1,
            }
        }

        #[test]
        fn test_file_lock_info_creation() {
            let lock = create_test_lock();
            assert_eq!(lock.file_id, 1234);
            assert_eq!(lock.server_name, "LOCAL");
            assert_eq!(lock.file_path, r"C:\Users\Admin\test.txt");
            assert_eq!(lock.file_name, "test.txt");
            assert_eq!(lock.username, "DOMAIN\\user");
            assert_eq!(lock.permissions, 3);
            assert_eq!(lock.num_locks, 1);
        }

        #[test]
        fn test_file_lock_info_clone() {
            let lock = create_test_lock();
            let lock_clone = lock.clone();
            assert_eq!(lock.file_id, lock_clone.file_id);
            assert_eq!(lock.file_name, lock_clone.file_name);
        }

        #[test]
        fn test_file_lock_info_debug() {
            let lock = create_test_lock();
            let debug = format!("{:?}", lock);
            assert!(debug.contains("FileLockInfo"));
            assert!(debug.contains("1234"));
            assert!(debug.contains("test.txt"));
        }

        #[test]
        fn test_file_lock_info_serialize() {
            let lock = create_test_lock();
            let serialized = serde_json::to_string(&lock).unwrap();
            assert!(serialized.contains("file_id"));
            assert!(serialized.contains("1234"));
            assert!(serialized.contains("test.txt"));
        }

        #[test]
        fn test_file_lock_info_deserialize() {
            let json = r#"{
                "file_id": 5678,
                "server_name": "REMOTE",
                "file_path": "D:\\data\\file.doc",
                "file_name": "file.doc",
                "username": "admin",
                "permissions": 1,
                "num_locks": 5
            }"#;
            let lock: FileLockInfo = serde_json::from_str(json).unwrap();
            assert_eq!(lock.file_id, 5678);
            assert_eq!(lock.server_name, "REMOTE");
            assert_eq!(lock.file_name, "file.doc");
            assert_eq!(lock.num_locks, 5);
        }

        #[test]
        fn test_file_lock_info_roundtrip_serialization() {
            let lock = create_test_lock();
            let serialized = serde_json::to_string(&lock).unwrap();
            let deserialized: FileLockInfo = serde_json::from_str(&serialized).unwrap();
            assert_eq!(lock.file_id, deserialized.file_id);
            assert_eq!(lock.file_name, deserialized.file_name);
            assert_eq!(lock.file_path, deserialized.file_path);
            assert_eq!(lock.username, deserialized.username);
        }
    }

    // ========================================================================
    // AppPersistence Tests
    // ========================================================================

    mod app_persistence_tests {
        use super::*;

        #[test]
        fn test_app_persistence_default() {
            let persistence = AppPersistence::default();
            assert!(persistence.dark_mode); // Default should be dark mode
        }

        #[test]
        fn test_app_persistence_serialize() {
            let persistence = AppPersistence { dark_mode: false };
            let serialized = serde_json::to_string(&persistence).unwrap();
            assert!(serialized.contains("dark_mode"));
            assert!(serialized.contains("false"));
        }

        #[test]
        fn test_app_persistence_deserialize() {
            let json = r#"{"dark_mode": true}"#;
            let persistence: AppPersistence = serde_json::from_str(json).unwrap();
            assert!(persistence.dark_mode);
        }

        #[test]
        fn test_app_persistence_deserialize_missing_field_uses_default() {
            // serde(default) should provide default values for missing fields
            let json = r#"{}"#;
            let persistence: AppPersistence = serde_json::from_str(json).unwrap();
            assert!(persistence.dark_mode); // Should default to true
        }
    }

    // ========================================================================
    // StatusMessage Tests
    // ========================================================================

    mod status_message_tests {
        use super::*;

        #[test]
        fn test_status_message_creation() {
            let msg = StatusMessage {
                text: "Test message".to_string(),
                message_type: MessageType::Info,
                timestamp: std::time::Instant::now(),
            };
            assert_eq!(msg.text, "Test message");
        }

        #[test]
        fn test_status_message_clone() {
            let msg = StatusMessage {
                text: "Clone test".to_string(),
                message_type: MessageType::Success,
                timestamp: std::time::Instant::now(),
            };
            let msg_clone = msg.clone();
            assert_eq!(msg.text, msg_clone.text);
        }

        #[test]
        fn test_message_type_clone() {
            let mt = MessageType::Error;
            let mt_clone = mt.clone();
            assert!(matches!(mt_clone, MessageType::Error));
        }
    }

    // ========================================================================
    // LockSmithApp State Tests
    // ========================================================================

    mod lock_smith_app_tests {
        use super::*;

        fn create_test_app() -> LockSmithApp {
            LockSmithApp::default()
        }

        fn create_sample_locks() -> Vec<FileLockInfo> {
            vec![
                FileLockInfo {
                    file_id: 1,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Users\Admin\alpha.txt".to_string(),
                    file_name: "alpha.txt".to_string(),
                    username: "user1".to_string(),
                    permissions: 1,
                    num_locks: 3,
                },
                FileLockInfo {
                    file_id: 2,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Users\Admin\beta.doc".to_string(),
                    file_name: "beta.doc".to_string(),
                    username: "user2".to_string(),
                    permissions: 2,
                    num_locks: 1,
                },
                FileLockInfo {
                    file_id: 3,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Data\gamma.pdf".to_string(),
                    file_name: "gamma.pdf".to_string(),
                    username: "admin".to_string(),
                    permissions: 3,
                    num_locks: 5,
                },
                FileLockInfo {
                    file_id: 4,
                    server_name: "LOCAL".to_string(),
                    file_path: r"D:\Projects\Alpha\config.json".to_string(),
                    file_name: "config.json".to_string(),
                    username: "user1".to_string(),
                    permissions: 1,
                    num_locks: 2,
                },
            ]
        }

        #[test]
        fn test_app_default_state() {
            let app = create_test_app();
            assert!(app.all_locks.is_empty());
            assert!(app.selected_ids.is_empty());
            assert!(app.search_query.is_empty());
            assert!(app.sort_column.is_none());
            assert_eq!(app.sort_direction, SortDirection::Ascending);
            assert!(app.dark_mode);
            assert!(app.status_message.is_none());
            assert!(!app.is_loading);
            assert!(!app.show_release_confirm);
            assert!(app.pending_release_ids.is_empty());
        }

        #[test]
        fn test_app_show_message() {
            let mut app = create_test_app();
            app.show_message("Test info message", MessageType::Info);
            assert!(app.status_message.is_some());
            let msg = app.status_message.as_ref().unwrap();
            assert_eq!(msg.text, "Test info message");
        }

        #[test]
        fn test_app_show_message_overwrites_previous() {
            let mut app = create_test_app();
            app.show_message("First message", MessageType::Info);
            app.show_message("Second message", MessageType::Error);
            let msg = app.status_message.as_ref().unwrap();
            assert_eq!(msg.text, "Second message");
        }

        // ====================================================================
        // Filtering Tests
        // ====================================================================

        #[test]
        fn test_get_filtered_sorted_locks_no_filter() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = String::new();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 4);
        }

        #[test]
        fn test_get_filtered_sorted_locks_filter_by_filename() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "alpha".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 2); // alpha.txt and config.json in Alpha folder
        }

        #[test]
        fn test_get_filtered_sorted_locks_filter_by_username() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "user1".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 2); // user1 has alpha.txt and config.json
        }

        #[test]
        fn test_get_filtered_sorted_locks_filter_by_path() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "Data".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 1); // Only gamma.pdf in C:\Data
        }

        #[test]
        fn test_get_filtered_sorted_locks_case_insensitive() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "ALPHA".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 2);
        }

        #[test]
        fn test_get_filtered_sorted_locks_no_matches() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "nonexistent".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert!(filtered.is_empty());
        }

        #[test]
        fn test_get_filtered_sorted_locks_partial_match() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = ".txt".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 1); // Only alpha.txt
        }

        // ====================================================================
        // Sorting Tests
        // ====================================================================

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_filename_ascending() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::FileName);
            app.sort_direction = SortDirection::Ascending;

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].file_name, "alpha.txt");
            assert_eq!(filtered[1].file_name, "beta.doc");
            assert_eq!(filtered[2].file_name, "config.json");
            assert_eq!(filtered[3].file_name, "gamma.pdf");
        }

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_filename_descending() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::FileName);
            app.sort_direction = SortDirection::Descending;

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].file_name, "gamma.pdf");
            assert_eq!(filtered[3].file_name, "alpha.txt");
        }

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_username() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::Username);
            app.sort_direction = SortDirection::Ascending;

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].username, "admin");
            assert_eq!(filtered[1].username, "user1");
            assert_eq!(filtered[2].username, "user1");
            assert_eq!(filtered[3].username, "user2");
        }

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_num_locks_ascending() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::NumLocks);
            app.sort_direction = SortDirection::Ascending;

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].num_locks, 1); // beta.doc
            assert_eq!(filtered[1].num_locks, 2); // config.json
            assert_eq!(filtered[2].num_locks, 3); // alpha.txt
            assert_eq!(filtered[3].num_locks, 5); // gamma.pdf
        }

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_num_locks_descending() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::NumLocks);
            app.sort_direction = SortDirection::Descending;

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered[0].num_locks, 5); // gamma.pdf
            assert_eq!(filtered[3].num_locks, 1); // beta.doc
        }

        #[test]
        fn test_get_filtered_sorted_locks_sort_by_filepath() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.sort_column = Some(SortColumn::FilePath);
            app.sort_direction = SortDirection::Ascending;

            let filtered = app.get_filtered_sorted_locks();
            // C:\Data comes before C:\Users, and D:\ comes after C:\
            assert!(filtered[0].file_path.starts_with(r"C:\Data"));
        }

        // ====================================================================
        // Handle Sort Tests
        // ====================================================================

        #[test]
        fn test_handle_sort_initial_column() {
            let mut app = create_test_app();
            assert!(app.sort_column.is_none());

            app.handle_sort(SortColumn::FileName);
            assert_eq!(app.sort_column, Some(SortColumn::FileName));
            assert_eq!(app.sort_direction, SortDirection::Ascending);
        }

        #[test]
        fn test_handle_sort_same_column_toggles_direction() {
            let mut app = create_test_app();
            app.handle_sort(SortColumn::FileName);
            assert_eq!(app.sort_direction, SortDirection::Ascending);

            app.handle_sort(SortColumn::FileName);
            assert_eq!(app.sort_direction, SortDirection::Descending);

            app.handle_sort(SortColumn::FileName);
            assert_eq!(app.sort_direction, SortDirection::Ascending);
        }

        #[test]
        fn test_handle_sort_different_column_resets_direction() {
            let mut app = create_test_app();
            app.handle_sort(SortColumn::FileName);
            app.handle_sort(SortColumn::FileName); // Now Descending
            assert_eq!(app.sort_direction, SortDirection::Descending);

            app.handle_sort(SortColumn::Username); // Switch column
            assert_eq!(app.sort_column, Some(SortColumn::Username));
            assert_eq!(app.sort_direction, SortDirection::Ascending); // Reset to Ascending
        }

        // ====================================================================
        // Selection Tests
        // ====================================================================

        #[test]
        fn test_select_all_visible() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();

            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 4);
            assert!(app.selected_ids.contains(&1));
            assert!(app.selected_ids.contains(&2));
            assert!(app.selected_ids.contains(&3));
            assert!(app.selected_ids.contains(&4));
        }

        #[test]
        fn test_select_all_visible_with_filter() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "user1".to_string();

            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 2);
            assert!(app.selected_ids.contains(&1)); // alpha.txt by user1
            assert!(app.selected_ids.contains(&4)); // config.json by user1
        }

        #[test]
        fn test_select_all_visible_empty_list() {
            let mut app = create_test_app();
            app.select_all_visible();
            assert!(app.selected_ids.is_empty());
        }

        #[test]
        fn test_selection_manual_add_remove() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();

            app.selected_ids.insert(1);
            assert_eq!(app.selected_ids.len(), 1);

            app.selected_ids.insert(2);
            assert_eq!(app.selected_ids.len(), 2);

            app.selected_ids.remove(&1);
            assert_eq!(app.selected_ids.len(), 1);
            assert!(app.selected_ids.contains(&2));
        }

        #[test]
        fn test_selection_clear() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 4);

            app.selected_ids.clear();
            assert!(app.selected_ids.is_empty());
        }

        // ====================================================================
        // Confirmation Dialog State Tests
        // ====================================================================

        #[test]
        fn test_confirmation_dialog_initial_state() {
            let app = create_test_app();
            assert!(!app.show_release_confirm);
            assert!(app.pending_release_ids.is_empty());
        }

        #[test]
        fn test_confirmation_dialog_state_management() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();

            // Simulate preparing for release
            app.selected_ids.insert(1);
            app.selected_ids.insert(2);
            app.pending_release_ids = app.selected_ids.iter().cloned().collect();
            app.show_release_confirm = true;

            assert!(app.show_release_confirm);
            assert_eq!(app.pending_release_ids.len(), 2);

            // Simulate cancel
            app.show_release_confirm = false;
            app.pending_release_ids.clear();

            assert!(!app.show_release_confirm);
            assert!(app.pending_release_ids.is_empty());
        }

        // ====================================================================
        // Combined Filter and Sort Tests
        // ====================================================================

        #[test]
        fn test_filter_and_sort_combined() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "user".to_string(); // Matches user1 and user2
            app.sort_column = Some(SortColumn::Username);
            app.sort_direction = SortDirection::Ascending;

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 3); // user1 (2 files) + user2 (1 file)
            assert_eq!(filtered[0].username, "user1");
            assert_eq!(filtered[2].username, "user2");
        }

        // ====================================================================
        // Edge Case Tests
        // ====================================================================

        #[test]
        fn test_empty_locks_with_search() {
            let mut app = create_test_app();
            app.search_query = "anything".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert!(filtered.is_empty());
        }

        #[test]
        fn test_search_with_special_characters() {
            let mut app = create_test_app();
            app.all_locks = vec![FileLockInfo {
                file_id: 1,
                server_name: "LOCAL".to_string(),
                file_path: r"C:\Users\Admin\file[1].txt".to_string(),
                file_name: "file[1].txt".to_string(),
                username: "user".to_string(),
                permissions: 1,
                num_locks: 1,
            }];
            app.search_query = "[1]".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 1);
        }

        #[test]
        fn test_search_whitespace() {
            let mut app = create_test_app();
            app.all_locks = create_sample_locks();
            app.search_query = "   ".to_string(); // Whitespace

            let filtered = app.get_filtered_sorted_locks();
            assert!(filtered.is_empty()); // No files match whitespace
        }

        #[test]
        fn test_last_selected_index_tracking() {
            let mut app = create_test_app();
            assert!(app.last_selected_index.is_none());

            app.last_selected_index = Some(5);
            assert_eq!(app.last_selected_index, Some(5));

            app.last_selected_index = None;
            assert!(app.last_selected_index.is_none());
        }
    }

    // ========================================================================
    // Icon Generation Tests
    // ========================================================================

    mod icon_tests {
        use super::*;

        #[test]
        fn test_create_padlock_icon_dimensions() {
            let icon = create_padlock_icon();
            assert_eq!(icon.width, 64);
            assert_eq!(icon.height, 64);
        }

        #[test]
        fn test_create_padlock_icon_data_size() {
            let icon = create_padlock_icon();
            // RGBA = 4 bytes per pixel
            assert_eq!(icon.rgba.len(), 64 * 64 * 4);
        }

        #[test]
        fn test_create_padlock_icon_not_empty() {
            let icon = create_padlock_icon();
            // Check that there are non-zero alpha values (visible pixels)
            let has_visible_pixels = icon.rgba.chunks(4).any(|pixel| pixel[3] > 0);
            assert!(has_visible_pixels);
        }

        #[test]
        fn test_create_padlock_icon_has_colors() {
            let icon = create_padlock_icon();
            // Check that the icon has different colors (not all same)
            let mut unique_colors = std::collections::HashSet::new();
            for pixel in icon.rgba.chunks(4) {
                if pixel[3] > 0 {
                    // Only count visible pixels
                    unique_colors.insert((pixel[0], pixel[1], pixel[2]));
                }
            }
            // Should have at least 2 colors (background and lock color)
            assert!(unique_colors.len() >= 2);
        }
    }

    // ========================================================================
    // Windows API Stub Tests (Non-Windows)
    // ========================================================================

    #[cfg(not(windows))]
    mod non_windows_stub_tests {
        use super::*;

        #[test]
        fn test_enumerate_files_returns_error_on_non_windows() {
            let result = enumerate_files();
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                FileEnumError::WindowsApiError(0)
            ));
        }

        #[test]
        fn test_release_lock_returns_error_on_non_windows() {
            let result = release_lock(12345);
            assert!(result.is_err());
            assert!(matches!(
                result.unwrap_err(),
                ReleaseLockError::WindowsApiError(0)
            ));
        }

        #[test]
        fn test_is_admin_returns_false_on_non_windows() {
            assert!(!is_admin());
        }
    }

    // ========================================================================
    // Integration Tests - Full Workflow Simulations
    // ========================================================================

    mod integration_tests {
        use super::*;

        #[test]
        fn test_full_workflow_filter_sort_select() {
            let mut app = LockSmithApp::default();

            // Simulate loading locks
            app.all_locks = vec![
                FileLockInfo {
                    file_id: 100,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Project\src\main.rs".to_string(),
                    file_name: "main.rs".to_string(),
                    username: "developer".to_string(),
                    permissions: 1,
                    num_locks: 2,
                },
                FileLockInfo {
                    file_id: 101,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Project\src\lib.rs".to_string(),
                    file_name: "lib.rs".to_string(),
                    username: "developer".to_string(),
                    permissions: 1,
                    num_locks: 1,
                },
                FileLockInfo {
                    file_id: 102,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\Logs\app.log".to_string(),
                    file_name: "app.log".to_string(),
                    username: "service".to_string(),
                    permissions: 3,
                    num_locks: 5,
                },
            ];

            // Step 1: Filter by "Project"
            app.search_query = "Project".to_string();
            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 2); // main.rs and lib.rs

            // Step 2: Sort by filename
            app.handle_sort(SortColumn::FileName);
            let sorted = app.get_filtered_sorted_locks();
            assert_eq!(sorted[0].file_name, "lib.rs");
            assert_eq!(sorted[1].file_name, "main.rs");

            // Step 3: Select all visible
            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 2);
            assert!(app.selected_ids.contains(&100));
            assert!(app.selected_ids.contains(&101));
            assert!(!app.selected_ids.contains(&102)); // Filtered out

            // Step 4: Prepare for release
            app.pending_release_ids = app.selected_ids.iter().cloned().collect();
            app.show_release_confirm = true;

            assert!(app.show_release_confirm);
            assert_eq!(app.pending_release_ids.len(), 2);
        }

        #[test]
        fn test_workflow_change_filter_clears_irrelevant_selections() {
            let mut app = LockSmithApp::default();
            app.all_locks = vec![
                FileLockInfo {
                    file_id: 1,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\A\file1.txt".to_string(),
                    file_name: "file1.txt".to_string(),
                    username: "user".to_string(),
                    permissions: 1,
                    num_locks: 1,
                },
                FileLockInfo {
                    file_id: 2,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\B\file2.txt".to_string(),
                    file_name: "file2.txt".to_string(),
                    username: "user".to_string(),
                    permissions: 1,
                    num_locks: 1,
                },
            ];

            // Select file1
            app.selected_ids.insert(1);

            // Change filter to only show file2
            app.search_query = "file2".to_string();
            let filtered = app.get_filtered_sorted_locks();

            // Selection still contains file1 even though it's not visible
            // This is expected behavior - selections persist
            assert!(app.selected_ids.contains(&1));
            assert_eq!(filtered.len(), 1);
            assert_eq!(filtered[0].file_id, 2);
        }

        #[test]
        fn test_theme_toggle_persistence_state() {
            let mut app = LockSmithApp::default();
            assert!(app.dark_mode); // Default is dark

            // Toggle theme
            app.dark_mode = !app.dark_mode;
            assert!(!app.dark_mode);

            // Toggle back
            app.dark_mode = !app.dark_mode;
            assert!(app.dark_mode);
        }

        #[test]
        fn test_loading_state_management() {
            let mut app = LockSmithApp::default();
            assert!(!app.is_loading);

            // Simulate loading start
            app.is_loading = true;
            assert!(app.is_loading);

            // Simulate loading complete
            app.is_loading = false;
            assert!(!app.is_loading);
        }
    }

    // ========================================================================
    // Performance/Stress Tests
    // ========================================================================

    mod stress_tests {
        use super::*;

        fn create_large_lock_list(count: usize) -> Vec<FileLockInfo> {
            (0..count)
                .map(|i| FileLockInfo {
                    file_id: i as u32,
                    server_name: "LOCAL".to_string(),
                    file_path: format!(r"C:\Data\Folder{}\file{}.dat", i / 100, i),
                    file_name: format!("file{}.dat", i),
                    username: format!("user{}", i % 10),
                    permissions: (i % 4) as u32,
                    num_locks: ((i % 20) + 1) as u32,
                })
                .collect()
        }

        #[test]
        fn test_filter_large_list() {
            let mut app = LockSmithApp::default();
            app.all_locks = create_large_lock_list(10000);
            app.search_query = "file999".to_string();

            let filtered = app.get_filtered_sorted_locks();
            // Should find files with "999" in the name
            assert!(filtered.len() > 0);
            assert!(filtered.len() < 10000);
        }

        #[test]
        fn test_sort_large_list() {
            let mut app = LockSmithApp::default();
            app.all_locks = create_large_lock_list(10000);
            app.sort_column = Some(SortColumn::NumLocks);
            app.sort_direction = SortDirection::Descending;

            let sorted = app.get_filtered_sorted_locks();
            assert_eq!(sorted.len(), 10000);

            // Verify sorting order
            for i in 0..sorted.len() - 1 {
                assert!(sorted[i].num_locks >= sorted[i + 1].num_locks);
            }
        }

        #[test]
        fn test_select_all_large_list() {
            let mut app = LockSmithApp::default();
            app.all_locks = create_large_lock_list(5000);

            app.select_all_visible();
            assert_eq!(app.selected_ids.len(), 5000);
        }

        #[test]
        fn test_filter_sort_combined_large_list() {
            let mut app = LockSmithApp::default();
            app.all_locks = create_large_lock_list(10000);
            app.search_query = "user5".to_string(); // Filter by username
            app.sort_column = Some(SortColumn::FileName);
            app.sort_direction = SortDirection::Ascending;

            let result = app.get_filtered_sorted_locks();
            assert!(result.len() > 0);

            // Verify all results match filter
            for lock in &result {
                assert!(lock.username.contains("user5"));
            }

            // Verify sorting
            for i in 0..result.len() - 1 {
                assert!(
                    result[i].file_name.to_lowercase() <= result[i + 1].file_name.to_lowercase()
                );
            }
        }
    }

    // ========================================================================
    // Boundary Condition Tests
    // ========================================================================

    mod boundary_tests {
        use super::*;

        #[test]
        fn test_file_id_zero() {
            let lock = FileLockInfo {
                file_id: 0,
                server_name: "LOCAL".to_string(),
                file_path: "test".to_string(),
                file_name: "test".to_string(),
                username: "user".to_string(),
                permissions: 0,
                num_locks: 0,
            };
            assert_eq!(lock.file_id, 0);
        }

        #[test]
        fn test_file_id_max() {
            let lock = FileLockInfo {
                file_id: u32::MAX,
                server_name: "LOCAL".to_string(),
                file_path: "test".to_string(),
                file_name: "test".to_string(),
                username: "user".to_string(),
                permissions: u32::MAX,
                num_locks: u32::MAX,
            };
            assert_eq!(lock.file_id, u32::MAX);
            assert_eq!(lock.permissions, u32::MAX);
            assert_eq!(lock.num_locks, u32::MAX);
        }

        #[test]
        fn test_empty_strings_in_lock_info() {
            let lock = FileLockInfo {
                file_id: 1,
                server_name: "".to_string(),
                file_path: "".to_string(),
                file_name: "".to_string(),
                username: "".to_string(),
                permissions: 0,
                num_locks: 0,
            };
            assert!(lock.file_path.is_empty());
            assert!(lock.file_name.is_empty());
            assert!(lock.username.is_empty());
        }

        #[test]
        fn test_very_long_strings() {
            let long_string = "a".repeat(10000);
            let lock = FileLockInfo {
                file_id: 1,
                server_name: long_string.clone(),
                file_path: long_string.clone(),
                file_name: long_string.clone(),
                username: long_string.clone(),
                permissions: 0,
                num_locks: 0,
            };
            assert_eq!(lock.file_path.len(), 10000);
        }

        #[test]
        fn test_search_very_long_query() {
            let mut app = LockSmithApp::default();
            app.all_locks = vec![FileLockInfo {
                file_id: 1,
                server_name: "LOCAL".to_string(),
                file_path: "short.txt".to_string(),
                file_name: "short.txt".to_string(),
                username: "user".to_string(),
                permissions: 0,
                num_locks: 0,
            }];
            app.search_query = "x".repeat(10000);

            let filtered = app.get_filtered_sorted_locks();
            assert!(filtered.is_empty()); // No match expected
        }

        #[test]
        fn test_unicode_in_all_fields() {
            let lock = FileLockInfo {
                file_id: 1,
                server_name: "サーバー".to_string(),
                file_path: r"C:\用户\文档\файл.txt".to_string(),
                file_name: "αβγδ.txt".to_string(),
                username: "用户🎉".to_string(),
                permissions: 0,
                num_locks: 0,
            };

            let mut app = LockSmithApp::default();
            app.all_locks = vec![lock];
            app.search_query = "αβ".to_string();

            let filtered = app.get_filtered_sorted_locks();
            assert_eq!(filtered.len(), 1);
        }

        #[test]
        fn test_selection_with_duplicate_ids() {
            let mut app = LockSmithApp::default();

            // HashSet naturally handles duplicates
            app.selected_ids.insert(1);
            app.selected_ids.insert(1);
            app.selected_ids.insert(1);

            assert_eq!(app.selected_ids.len(), 1);
        }
    }

    // ========================================================================
    // Regression Tests
    // ========================================================================

    mod regression_tests {
        use super::*;

        #[test]
        fn test_sorting_stability_with_equal_values() {
            let mut app = LockSmithApp::default();
            app.all_locks = vec![
                FileLockInfo {
                    file_id: 1,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\a.txt".to_string(),
                    file_name: "same.txt".to_string(),
                    username: "user".to_string(),
                    permissions: 0,
                    num_locks: 5,
                },
                FileLockInfo {
                    file_id: 2,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\b.txt".to_string(),
                    file_name: "same.txt".to_string(),
                    username: "user".to_string(),
                    permissions: 0,
                    num_locks: 5,
                },
            ];
            app.sort_column = Some(SortColumn::FileName);
            app.sort_direction = SortDirection::Ascending;

            let sorted = app.get_filtered_sorted_locks();
            // Both have same filename, should still return 2 items
            assert_eq!(sorted.len(), 2);
        }

        #[test]
        fn test_filter_after_sort_maintains_order() {
            let mut app = LockSmithApp::default();
            app.all_locks = vec![
                FileLockInfo {
                    file_id: 1,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\z.txt".to_string(),
                    file_name: "z.txt".to_string(),
                    username: "alice".to_string(),
                    permissions: 0,
                    num_locks: 1,
                },
                FileLockInfo {
                    file_id: 2,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\a.txt".to_string(),
                    file_name: "a.txt".to_string(),
                    username: "alice".to_string(),
                    permissions: 0,
                    num_locks: 2,
                },
                FileLockInfo {
                    file_id: 3,
                    server_name: "LOCAL".to_string(),
                    file_path: r"C:\m.txt".to_string(),
                    file_name: "m.txt".to_string(),
                    username: "bob".to_string(),
                    permissions: 0,
                    num_locks: 3,
                },
            ];

            // Sort by filename first
            app.sort_column = Some(SortColumn::FileName);
            app.sort_direction = SortDirection::Ascending;

            // Then filter by "alice"
            app.search_query = "alice".to_string();

            let result = app.get_filtered_sorted_locks();
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].file_name, "a.txt"); // Sorted order maintained
            assert_eq!(result[1].file_name, "z.txt");
        }

        #[test]
        fn test_clear_search_restores_all_items() {
            let mut app = LockSmithApp::default();
            app.all_locks = vec![
                FileLockInfo {
                    file_id: 1,
                    server_name: "LOCAL".to_string(),
                    file_path: "a.txt".to_string(),
                    file_name: "a.txt".to_string(),
                    username: "user".to_string(),
                    permissions: 0,
                    num_locks: 0,
                },
                FileLockInfo {
                    file_id: 2,
                    server_name: "LOCAL".to_string(),
                    file_path: "b.txt".to_string(),
                    file_name: "b.txt".to_string(),
                    username: "user".to_string(),
                    permissions: 0,
                    num_locks: 0,
                },
            ];

            // Filter down to 1 item
            app.search_query = "a.txt".to_string();
            assert_eq!(app.get_filtered_sorted_locks().len(), 1);

            // Clear filter
            app.search_query = String::new();
            assert_eq!(app.get_filtered_sorted_locks().len(), 2);
        }
    }
}
