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

#[derive(Clone, Copy, PartialEq, Eq)]
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
    } else if let Ok(image) = image::open("src-tauri/icons/128x128.png") {
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

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename(r"C:\Users\Admin\file.txt"), "file.txt");
        assert_eq!(extract_filename(r"\\server\share\doc.pdf"), "doc.pdf");
        assert_eq!(extract_filename("simple.txt"), "simple.txt");
        assert_eq!(extract_filename(""), "");
    }

    #[test]
    fn test_sort_direction_toggle() {
        let dir = SortDirection::Ascending;
        assert_eq!(dir.toggle(), SortDirection::Descending);
        assert_eq!(dir.toggle().toggle(), SortDirection::Ascending);
    }

    #[test]
    fn test_sort_direction_arrow() {
        assert_eq!(SortDirection::Ascending.arrow(), " ↑");
        assert_eq!(SortDirection::Descending.arrow(), " ↓");
    }
}
