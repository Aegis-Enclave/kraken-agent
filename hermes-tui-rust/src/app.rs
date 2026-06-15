//! Application module - Main App struct and event loop
//!
//! This module contains the main application state and the event loop
//! that drives the TUI.

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle},
};
use log::{debug, error, info, warn};
use ratatui::{layout::{Constraint, Direction, Layout, Rect}, Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout, Write};
use std::time::Duration;

use crate::ui::cards::{CardManager, ToolCardData, ToolStatus};
use crate::ui::hashline::HashlineViewer;
use crate::ui::subagent::SubagentList;

use crate::protocol::client::GatewayClient;
use crate::protocol::types::*;
use crate::state::{config::TuiConfig, messages::{Message, MessageHistory}, session::{Session, SessionManager}};
use crate::state::config::InputMode;
use crate::ui::{chat::ChatComponent, composer::InputComposer, toolbar::Toolbar};
use crate::ui::prompts::PromptManager;
use crate::ui::completions::CompletionPopup;
use crate::ui::session_picker::SessionPicker;
use crate::handlers::mouse::MouseContext;

/// Main application struct
///
/// This struct holds all the state needed for the Hermes Rust TUI application.
/// It manages the terminal, configuration, sessions, messages, and gateway communication.
#[derive(Debug)]
pub struct App {
    /// Ratatui terminal instance with Crossterm backend
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// TUI configuration (theme, display, editor settings)
    config: TuiConfig,
    /// Session manager for handling multiple chat sessions
    session_manager: SessionManager,
    /// Message history for the current session
    message_history: MessageHistory,
    /// Client for communicating with the Hermes gateway
    gateway_client: GatewayClient,
    /// Whether the gateway is currently thinking/responding
    thinking: bool,
    /// Whether the application is currently running
    running: bool,
    /// Current input mode (Normal, Insert, Command)
    input_mode: InputMode,
    /// Current user input text (legacy, kept for backward compatibility)
    current_input: String,
    /// Toolbar for displaying status information
    toolbar: Toolbar,
    /// Banner for branding logo
    banner: crate::ui::banner::Banner,
    /// Card manager for tool call result cards
    card_manager: CardManager,
    /// Subagent list for tracking spawned subagents
    subagent_list: SubagentList,
    /// Hashline viewer for rendering file edits
    hashline_viewer: HashlineViewer,
    /// Gateway child process (killed before transport to avoid blocking)
    gateway_process: Option<std::process::Child>,
    /// Cursor position within the current input (legacy, kept for backward compatibility)
    cursor_position: usize,
    /// Chat component for displaying conversation messages
    chat_component: ChatComponent,
    /// Input composer for user text input
    input_composer: InputComposer,
    /// Prompt manager for approval/clarify/secret overlays
    prompt_manager: PromptManager,
    /// Pending approval request ID
    pending_approval_id: Option<String>,
    /// Autocomplete completions popup widget
    completion_popup: CompletionPopup,
    /// Session picker popup widget
    session_picker: SessionPicker,
    /// Mouse interaction context
    mouse_context: MouseContext,
    /// Whether the gateway is in a reconnection cycle
    reconnecting: bool,
    /// Number of reconnect attempts (for exponential backoff)
    reconnect_attempts: u32,
    /// Animated activity panel height (smooth transition)
    actual_activity_height: f32,
}

impl App {
    /// Create a new application instance with terminal initialized and defaults set.
    ///
    /// # Returns
    /// - `Result<Self>`: New App instance or error during initialization.
    ///
    /// # Errors
    /// - Fails if terminal cannot be initialized
    /// - Fails if raw mode cannot be enabled
    pub fn new() -> Result<Self> {
        // Enable raw mode for terminal input
        enable_raw_mode()
            .context("Failed to enable raw mode")?;

        // Enter alternate screen mode
        execute!(io::stdout(), EnterAlternateScreen)
            .context("Failed to enter alternate screen mode")?;

        // Set terminal title
        execute!(io::stdout(), SetTitle("Kraken Agent (Rust)"))
            .context("Failed to set terminal title")?;

        // Initialize terminal with Crossterm backend
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)
            .context("Failed to create terminal")?;
        // Initialize config with Kraken theme by default
        let mut config = TuiConfig::default();
        config.theme = crate::state::config::BuiltinTheme::Kraken.to_config();
        
        let theme_colors_rgb = config.theme.colors.to_rgb_colors();
        let chat_colors_rgb = config.theme.chat.to_rgb_colors();
        
        let mut input_composer = InputComposer::new(chat_colors_rgb);
        input_composer.set_input_mode(InputMode::Insert);

        Ok(Self {
            terminal,
            config,
            session_manager: SessionManager::new(),
            message_history: MessageHistory::new(),
            gateway_client: GatewayClient::new(),
            thinking: false,
            reconnecting: false,
            reconnect_attempts: 0,
            actual_activity_height: 0.0,
            card_manager: CardManager::new(chat_colors_rgb),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            gateway_process: None,
            running: true,
            input_mode: InputMode::Insert,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb, true),
            input_composer,
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb),
            banner: crate::ui::banner::Banner::default(),
            prompt_manager: PromptManager::new(chat_colors_rgb),
            pending_approval_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb),
            session_picker: SessionPicker::new(chat_colors_rgb),
            mouse_context: MouseContext::new(),
        })
    }

    // ============================================================================
    // Getter Methods
    // ============================================================================

    /// Get a reference to the terminal
    pub fn terminal(&self) -> &Terminal<CrosstermBackend<Stdout>> {
        &self.terminal
    }

    /// Get a mutable reference to the terminal
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &TuiConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut TuiConfig {
        &mut self.config
    }

    /// Get a reference to the session manager
    pub fn sessions(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Get a mutable reference to the session manager
    pub fn sessions_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    /// Get a reference to the message history
    pub fn messages(&self) -> &MessageHistory {
        &self.message_history
    }

    /// Get a mutable reference to the message history
    pub fn messages_mut(&mut self) -> &mut MessageHistory {
        &mut self.message_history
    }

    /// Get a reference to the gateway client
    pub fn gateway_client(&self) -> &GatewayClient {
        &self.gateway_client
    }

    /// Get a mutable reference to the gateway client
    pub fn gateway_client_mut(&mut self) -> &mut GatewayClient {
        &mut self.gateway_client
    }
    
    /// Get the current input mode
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Get the current input text
    pub fn current_input(&self) -> &str {
        &self.current_input
    }

    /// Get the current cursor position
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    // ============================================================================
    // UI Component Getters
    // ============================================================================

    /// Get a reference to the chat component
    pub fn chat_component(&self) -> &ChatComponent {
        &self.chat_component
    }

    /// Get a mutable reference to the chat component
    pub fn chat_component_mut(&mut self) -> &mut ChatComponent {
        &mut self.chat_component
    }

    /// Get a reference to the input composer
    pub fn input_composer(&self) -> &InputComposer {
        &self.input_composer
    }

    /// Get a mutable reference to the input composer
    pub fn input_composer_mut(&mut self) -> &mut InputComposer {
        &mut self.input_composer
    }

    /// Get a reference to the prompt manager
    pub fn prompt_manager(&self) -> &PromptManager {
        &self.prompt_manager
    }

    /// Get a mutable reference to the prompt manager
    pub fn prompt_manager_mut(&mut self) -> &mut PromptManager {
        &mut self.prompt_manager
    }

    /// Get a reference to the completion popup
    pub fn completion_popup(&self) -> &CompletionPopup {
        &self.completion_popup
    }

    /// Get a mutable reference to the completion popup
    pub fn completion_popup_mut(&mut self) -> &mut CompletionPopup {
        &mut self.completion_popup
    }

    /// Get a reference to the session picker
    pub fn session_picker(&self) -> &SessionPicker {
        &self.session_picker
    }

    /// Get a mutable reference to the session picker
    pub fn session_picker_mut(&mut self) -> &mut SessionPicker {
        &mut self.session_picker
    }

    /// Get a reference to the toolbar
    pub fn toolbar(&self) -> &Toolbar {
        &self.toolbar
    }

    /// Get a mutable reference to the toolbar
    pub fn toolbar_mut(&mut self) -> &mut Toolbar {
        &mut self.toolbar
    }

    /// Get a reference to the card manager
    pub fn card_manager(&self) -> &CardManager {
        &self.card_manager
    }

    /// Get a mutable reference to the card manager
    pub fn card_manager_mut(&mut self) -> &mut CardManager {
        &mut self.card_manager
    }

    /// Get a reference to the subagent list
    pub fn subagent_list(&self) -> &SubagentList {
        &self.subagent_list
    }

    /// Get a mutable reference to the subagent list
    pub fn subagent_list_mut(&mut self) -> &mut SubagentList {
        &mut self.subagent_list
    }

    /// Get a reference to the hashline viewer
    pub fn hashline_viewer(&self) -> &HashlineViewer {
        &self.hashline_viewer
    }

    // ============================================================================
    // Setter Methods
    // ============================================================================

    /// Set the running state of the application
    /// Check if the application is running
    pub fn is_running(&self) -> bool {
        self.running
    }
    
    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    /// Set the current input mode
    pub fn set_input_mode(&mut self, mode: InputMode) {
        self.input_mode = mode;
        // Update UI components with the new mode
        self.input_composer.set_input_mode(mode);
        self.toolbar.set_input_mode(mode);
    }

    /// Set the current input text
    pub fn set_current_input(&mut self, input: String) {
        self.current_input = input;
    }

    /// Set the cursor position
    pub fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position;
    }

    /// Clear the current input
    pub fn clear_input(&mut self) {
        self.current_input.clear();
        self.cursor_position = 0;
    }

    // ============================================================================
    // Gateway Methods
    // ============================================================================

    /// Connect to the gateway by spawning the Python child process
    pub fn connect_gateway(&mut self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let project_root = if current_dir.join("tui_gateway").exists() {
            current_dir
        } else if current_dir.parent().map(|p| p.join("tui_gateway").exists()).unwrap_or(false) {
            current_dir.parent().unwrap().to_path_buf()
        } else {
            current_dir
        };

        // Try to find the virtual environment python
        let venv_python = project_root.join("venv").join("bin").join("python3");
        let python_cmd = if venv_python.exists() {
            venv_python.to_string_lossy().to_string()
        } else {
            std::env::var("HERMES_PYTHON").unwrap_or_else(|_| "python3".into())
        };

        // Open log file for gateway stderr redirection
        let stderr_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("hermes-tui.log")
            .unwrap();

        let current_pythonpath = std::env::var("PYTHONPATH").unwrap_or_default();
        let project_root_str = project_root.to_string_lossy();
        let new_pythonpath = if current_pythonpath.is_empty() {
            project_root_str.to_string()
        } else {
            format!("{}:{}", project_root_str, current_pythonpath)
        };

        let mut child = std::process::Command::new(&python_cmd)
            .args(["-m", "tui_gateway.entry"])
            .env("PYTHONPATH", new_pythonpath) // Ensure tui_gateway and hermes packages can be found
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::from(stderr_file)) // Redirect to log file
            .spawn()
            .context(format!("Failed to spawn gateway process using '{}'.", python_cmd))?;

        let child_stdin = child.stdin.take()
            .context("Failed to take child stdin")?;
        let child_stdout = child.stdout.take()
            .context("Failed to take child stdout")?;

        info!("Gateway process spawned: {} (PID: {})", python_cmd, child.id());

        self.gateway_client.connect(child_stdout, child_stdin)?;
        self.gateway_process = Some(child);

        Ok(())
    }

    /// Disconnect from the gateway
    pub fn disconnect_gateway(&mut self) -> Result<()> {
        if self.gateway_client.is_connected() {
            self.gateway_client.disconnect()
        } else {
            Ok(())
        }
    }
    /// Send a request to the gateway
    pub fn send_gateway_request(&mut self, request: crate::protocol::types::TuiRequest) -> Result<()> {
        self.gateway_client.send_request(request)
    }

    /// Receive a message from the gateway
    pub fn receive_gateway_message(&self) -> Option<crate::protocol::types::GatewayMessage> {
        self.gateway_client.receive_message()
    }

    /// Check if gateway is connected
    pub fn is_gateway_connected(&self) -> bool {
        self.gateway_client.is_connected()
    }

    /// Disconnect from the gateway
    /// Run the main event loop
    pub fn run(&mut self) -> Result<()> {
        info!("Starting event loop");
        
        while self.running {
            // Poll for events at ~60fps (16ms per frame) — non-blocking
            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    Event::Key(key) => {
                        self.handle_key_event(key)?;
                    }
                    Event::Mouse(mouse) => {
                        self.handle_mouse_event(mouse)?;
                    }
                    Event::Resize(width, height) => {
                        self.handle_resize(width, height)?;
                    }
                    Event::Paste(text) => {
                        self.handle_paste(&text)?;
                    }
                    Event::FocusGained => {
                        debug!("Focus gained");
                        self.draw()?;
                    }
                    Event::FocusLost => {
                        debug!("Focus lost");
                        self.draw()?;
                    }
                }
            }
            
            // Check for gateway messages
            if let Some(message) = self.receive_gateway_message() {
                self.handle_gateway_message(message)?;
            }
            
            // Monitor child process health
            self.check_gateway_health()?;
            // Advance spinner animations for tool cards
            self.card_manager.tick_spinners();

            // Update toolbar animation
            self.toolbar.tick(self.thinking);

            // Draw UI
            self.draw()?;
        }
        
        info!("Event loop ended");
        Ok(())
    }

    /// Check gateway child process health and reconnect if needed
    fn check_gateway_health(&mut self) -> Result<()> {
        // If reconnecting, skip health checks
        if self.reconnecting {
            return Ok(());
        }

        // Check if the child process has exited unexpectedly
        if let Some(ref mut child) = self.gateway_process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    warn!("Gateway process exited unexpectedly with status: {}", status);
                    self.trigger_reconnect()?;
                }
                Ok(None) => {
                    // Process is still running — nothing to do
                }
                Err(e) => {
                    error!("Error checking gateway process: {}", e);
                    self.trigger_reconnect()?;
                }
            }
        } else {
            // No child but we're supposed to be connected — reconnect
            if self.gateway_client.is_connected() {
                warn!("No gateway process but client reports connected, triggering reconnect");
                self.trigger_reconnect()?;
            }
        }
        Ok(())
    }

    /// Trigger gateway reconnection
    fn trigger_reconnect(&mut self) -> Result<()> {
        if self.reconnecting {
            return Ok(()); // Already reconnecting
        }
        self.reconnecting = true;
        self.reconnect_attempts += 1;
        info!("Triggering gateway reconnect (attempt {})", self.reconnect_attempts);
        
        // Reset state
        self.thinking = false;
        
        // Disconnect existing client
        if self.gateway_client.is_connected() {
            let _ = self.gateway_client.disconnect();
        }
        
        // Kill old process if it's still around
        if let Some(mut child) = self.gateway_process.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        
        // Add a system message about the reconnection
        let msg = Message::system(format!("Connection lost. Reconnecting (attempt {})...", self.reconnect_attempts));
        self.messages_mut().add_message(msg.clone());
        self.chat_component.add_message(msg, &self.card_manager);
        
        // Attempt to reconnect
        if let Err(e) = self.connect_gateway() {
            error!("Reconnect attempt {} failed: {}", self.reconnect_attempts, e);
            
            // Reset state after too many failures
            if self.reconnect_attempts >= 5 {
                let fail_msg = Message::system("Failed to reconnect after multiple attempts. Restart the TUI or press Ctrl+C to exit.");
                self.messages_mut().add_message(fail_msg.clone());
                self.chat_component.add_message(fail_msg, &self.card_manager);
                self.reconnecting = false;
            }
            // Will try again on next loop iteration
        } else {
            info!("Reconnect successful after {} attempt(s)", self.reconnect_attempts);
            // Reset reconnecting state — the gateway.ready handler will fire
            self.reconnecting = false;
        }
        
        Ok(())
    }

    /// Handle a key event
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        use crossterm::event::KeyCode;
        
        // Check for quit keys first
        if self.check_quit_key(key) {
            return Ok(());
        }

        // If a prompt is active, let the prompt manager handle keys
        if self.prompt_manager.has_active_prompt() {
            if self.prompt_manager.is_approval_active() {
                match key.code {
                    KeyCode::Char('y' | 'Y') => {
                        self.prompt_manager.approve();
                        if let Some(req_id) = self.pending_approval_id.take() {
                            let response = TuiRequest::ApprovalRespond(ApprovalResponse {
                                request_id: req_id,
                                approved: true,
                                choice: "once".to_string(),
                            });
                            let _ = self.send_gateway_request(response);
                        }
                        self.input_composer_mut().set_active(true);
                        self.set_input_mode(InputMode::Insert);
                    }
                    KeyCode::Char('n' | 'N') => {
                        self.prompt_manager.deny();
                        if let Some(req_id) = self.pending_approval_id.take() {
                            let response = TuiRequest::ApprovalRespond(ApprovalResponse {
                                request_id: req_id,
                                approved: false,
                                choice: "deny".to_string(),
                            });
                            let _ = self.send_gateway_request(response);
                        }
                        self.input_composer_mut().set_active(true);
                        self.set_input_mode(InputMode::Insert);
                    }
                    KeyCode::Esc => {
                        self.prompt_manager.cancel_all();
                        self.input_composer_mut().set_active(true);
                        self.set_input_mode(InputMode::Insert);
                    }
                    _ => {}
                }
            } else if self.prompt_manager.is_clarify_active() {
                if let Some(prompt) = &mut self.prompt_manager.clarify_prompt {
                    match key.code {
                        KeyCode::Char(c) => {
                            prompt.append_response(&c.to_string());
                        }
                        KeyCode::Backspace => {
                            let mut resp = prompt.response().to_string();
                            resp.pop();
                            prompt.set_response(resp);
                        }
                        KeyCode::Enter => {
                            let _val = prompt.submit();
                            self.input_composer_mut().set_active(true);
                            self.set_input_mode(InputMode::Insert);
                        }
                        KeyCode::Esc => {
                            prompt.cancel();
                            self.input_composer_mut().set_active(true);
                            self.set_input_mode(InputMode::Insert);
                        }
                        _ => {}
                    }
                }
            } else if self.prompt_manager.is_secret_active() {
                if let Some(prompt) = &mut self.prompt_manager.secret_prompt {
                    match key.code {
                        KeyCode::Char(c) => {
                            prompt.append_secret(c);
                        }
                        KeyCode::Backspace => {
                            prompt.pop_secret();
                        }
                        KeyCode::Enter => {
                            let _val = prompt.submit();
                            self.input_composer_mut().set_active(true);
                            self.set_input_mode(InputMode::Insert);
                        }
                        KeyCode::Esc => {
                            prompt.cancel();
                            self.input_composer_mut().set_active(true);
                            self.set_input_mode(InputMode::Insert);
                        }
                        _ => {}
                    }
                }
            }
            return Ok(());
        }

        // If session picker is visible, route keys to it
        if self.session_picker.is_visible() {
            match key.code {
                KeyCode::Down => {
                    self.session_picker.select_next();
                }
                KeyCode::Up => {
                    self.session_picker.select_prev();
                }
                KeyCode::Enter => {
                    if let Some(session) = self.session_picker.selected_session().cloned() {
                        let request = SessionResumeRequest {
                            session_id: session.id.clone(),
                        };
                        let tui_request = TuiRequest::SessionResume(request);
                        let _ = self.send_gateway_request(tui_request);

                        // Show feedback to user
                        let title = if session.title.is_empty() { "Unnamed".to_string() } else { session.title.clone() };
                        let system_message = Message::system(format!("Resuming session: {}", title));
                        self.messages_mut().add_message(system_message.clone());
                        self.chat_component.add_message(system_message, &self.card_manager);
                        self.chat_component.scroll_to_bottom(&self.card_manager);
                        self.session_picker.hide();
                    }
                }
                KeyCode::Esc => {
                    self.session_picker.hide();
                }
                _ => {}
            }
            return Ok(());
        }

        // If completion popup is visible, route keys to it
        if self.completion_popup.is_visible() {
            match key.code {
                KeyCode::Down => {
                    self.completion_popup.select_next();
                    return Ok(());
                }
                KeyCode::Up => {
                    self.completion_popup.select_prev();
                    return Ok(());
                }
                KeyCode::Enter => {
                    if let Some(item) = self.completion_popup.selected_item().cloned() {
                        // Set the input composer input to item text
                        self.input_composer.set_input(&item.text);
                        self.current_input = item.text.clone();
                        self.completion_popup.hide();
                    }
                    return Ok(());
                }
                KeyCode::Esc => {
                    self.completion_popup.hide();
                    return Ok(());
                }
                _ => {}
            }
        }

        let KeyEvent { code, modifiers, .. } = key;
        // Normal mode navigation (only in Normal mode)
        if self.input_mode == InputMode::Normal {
            match code {
                KeyCode::Char('i') => {
                    self.set_input_mode(InputMode::Insert);
                    self.input_composer.set_active(true);
                    return Ok(());
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.chat_component.select_next(&self.card_manager);
                    self.chat_component.ensure_selected_in_view(&self.card_manager);
                    return Ok(());
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.chat_component.select_prev(&self.card_manager);
                    self.chat_component.ensure_selected_in_view(&self.card_manager);
                    return Ok(());
                }
                KeyCode::Enter => {
                    // Toggle expansion for selected message
                    if let Some(idx) = self.chat_component.get_selected_index() {
                        let msg_id_opt = self.messages().get(idx).and_then(|m| m.message_id.clone());
                        if let Some(msg_id) = &msg_id_opt {
                            if !msg_id.starts_with("subagent:") {
                                if let Some(msg) = self.messages().get(idx) {
                                    if msg.role == MessageRole::System {
                                        self.chat_component.toggle_system_expanded(msg_id);
                                    } else if msg.role == MessageRole::Tool {
                                        self.card_manager.toggle_expanded(msg_id);
                                    }
                                }
                            }
                        }
                    }
                    return Ok(());
                }
                KeyCode::PageUp => {
                    self.chat_component.scroll_up(10);
                    return Ok(());
                }
                KeyCode::PageDown => {
                    self.chat_component.scroll_down(10, &self.card_manager);
                    return Ok(());
                }
                _ => {}
            }
        }

        // Check for Esc key in Insert mode -> go to Normal
        if code == KeyCode::Esc && self.input_mode == InputMode::Insert {
            self.set_input_mode(InputMode::Normal);
            self.input_composer.set_active(false);
            // Select the last message
            let len = self.chat_component.messages().len();
            if len > 0 {
                self.chat_component.select_prev(&self.card_manager);
                self.chat_component.ensure_selected_in_view(&self.card_manager);
            }
            return Ok(());
        }

        // Check for Enter key in Insert mode - submit prompt
        if self.input_mode == InputMode::Insert && code == KeyCode::Enter && !modifiers.contains(KeyModifiers::SHIFT) {
            if !self.input_composer.get_input().is_empty() {
                self.submit_prompt()?;
                self.current_input = self.input_composer.get_input().to_string();
                return Ok(());
            }
        }

        // Check for '/' key - switch to command mode
        if code == KeyCode::Char('/') {
            if self.input_mode == InputMode::Normal {
                self.set_input_mode(InputMode::Command);
                self.input_composer.set_input_mode(InputMode::Command);
                self.input_composer.set_active(true);
                self.current_input = self.input_composer.get_input().to_string();
                self.input_mode = InputMode::Command;
                return Ok(());
            }
        }

        // Check for Esc key - exit command mode
        if code == KeyCode::Esc {
            if self.input_mode == InputMode::Command {
                self.set_input_mode(InputMode::Normal);
                self.input_composer.set_input_mode(InputMode::Normal);
                self.input_composer.clear();
                self.current_input = self.input_composer.get_input().to_string();
                self.input_mode = InputMode::Normal;
                return Ok(());
            }
        }

        // Check for Ctrl+N - new session
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('n') {
            self.create_new_session()?;
            return Ok(());
        }

        // Check for Ctrl+R - resume session (for now, resume most recent)
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('r') {
            self.resume_session()?;
            return Ok(());
        }

        // Check for Ctrl+L - list sessions
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('l') {
            self.list_sessions()?;
            return Ok(());
        }

        // Pass key to input composer for handling (only in Insert/Command mode)
        if self.input_mode == InputMode::Insert || self.input_mode == InputMode::Command {
            let handled = self.input_composer.handle_key_event(key);
            if handled {
                self.current_input = self.input_composer.get_input().to_string();
                self.input_mode = self.input_composer.input_mode();
            }
        }

        Ok(())
    }
    
    /// Submit the current input as a prompt to the gateway
    fn submit_prompt(&mut self) -> Result<()> {
        let input = self.input_composer.get_input().to_string();
        if input.is_empty() {
            return Ok(());
        }
        
        // Clear the input
        self.input_composer.clear();
        
        // Check if this is a slash command
        if self.input_mode == InputMode::Command {
            self.execute_slash_command(input)?;
            self.set_input_mode(InputMode::Normal);
            return Ok(());
        }
        
        // Send the prompt to the gateway
        let message_content = input.clone();
        let session_id = self.sessions().current_session().map(|s| s.id.clone()).unwrap_or_default();
        
        let request = PromptSubmitRequest {
            session_id,
            text: input,
            images: None,
            truncate_before_user_ordinal: None,
        };
        
        let tui_request = TuiRequest::PromptSubmit(request);
        
        info!("Submitting prompt");

        self.thinking = true;
        self.send_gateway_request(tui_request)?;
        
        // Add user message to chat immediately (optimistic update)
        let user_message = Message::user(message_content);
        self.messages_mut().add_message(user_message.clone());
        self.chat_component.add_message(user_message, &self.card_manager);
        // Auto-scroll to bottom
        self.chat_component.scroll_to_bottom(&self.card_manager);
        
        Ok(())
    }
    
    /// Execute a slash command
    fn execute_slash_command(&mut self, command: String) -> Result<()> {
        info!("Executing slash command: {}", command);
        
        let command_clone = command.clone();
        let session_id = self.sessions().current_session().map(|s| s.id.clone()).unwrap_or_default();
        
        // Send slash.exec request to gateway
        let request = SlashExecRequest {
            session_id,
            command,
        };
        
        let tui_request = TuiRequest::SlashExec(request);
        self.send_gateway_request(tui_request)?;
        
        // Add the command to chat for visibility
        let user_message = Message::user(format!("/{}", command_clone));
        self.messages_mut().add_message(user_message.clone());
        self.chat_component.add_message(user_message, &self.card_manager);
        self.chat_component.scroll_to_bottom(&self.card_manager);
        
        Ok(())
    }
    
    /// Create a new session
    fn create_new_session(&mut self) -> Result<()> {
        info!("Creating new session");
        
        // Send session.create request
        let request = SessionCreateRequest {
            model: None,      // Use default model
            provider: None,   // Use default provider
            toolsets: None,   // Use default toolsets
            skills: None,     // Use default skills
            worktree: None,   // Don't create worktree
        };
        
        let tui_request = TuiRequest::SessionCreate(request);
        self.send_gateway_request(tui_request)?;
        
        // Show feedback to user
        let system_message = Message::system("Creating new session...");
        self.messages_mut().add_message(system_message.clone());
        self.chat_component.add_message(system_message, &self.card_manager);
        self.chat_component.scroll_to_bottom(&self.card_manager);
        
        Ok(())
    }
    
    /// Resume the most recent session
    fn resume_session(&mut self) -> Result<()> {
        info!("Resuming session");
        
        // Get the most recent session ID first
        let session_id = self.sessions().most_recent_session().map(|s| s.id.clone());
        let session_name = self.sessions().most_recent_session().and_then(|s| s.name.clone());
        
        if let Some(session_id) = session_id {
            let request = SessionResumeRequest {
                session_id,
            };
            
            let tui_request = TuiRequest::SessionResume(request);
            self.send_gateway_request(tui_request)?;
            
            // Show feedback to user
            let session_name_display = session_name.as_deref().unwrap_or("Unnamed");
            let system_message = Message::system(format!("Resuming session: {}", session_name_display));
            self.messages_mut().add_message(system_message.clone());
            self.chat_component.add_message(system_message, &self.card_manager);
            self.chat_component.scroll_to_bottom(&self.card_manager);
        } else {
            let system_message = Message::system("No previous sessions to resume");
            self.messages_mut().add_message(system_message.clone());
            self.chat_component.add_message(system_message, &self.card_manager);
            self.chat_component.scroll_to_bottom(&self.card_manager);
        }
        
        Ok(())
    }
    
    /// List available sessions
    fn list_sessions(&mut self) -> Result<()> {
        info!("Listing sessions");
        
        // Send session.list request
        let tui_request = TuiRequest::SessionList;
        self.send_gateway_request(tui_request)?;
        
        // Show feedback to user
        let system_message = Message::system("Fetching session list...");
        self.messages_mut().add_message(system_message.clone());
        self.chat_component.add_message(system_message, &self.card_manager);
        self.chat_component.scroll_to_bottom(&self.card_manager);
        
        Ok(())
    }

    /// Check if a key event should quit the application
    fn check_quit_key(&mut self, key: KeyEvent) -> bool {
        let KeyEvent { code, modifiers, .. } = key;
        
        // Ctrl+C - Exit
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
            info!("Ctrl+C pressed, exiting");
            self.running = false;
            return true;
        }
        
        // Ctrl+Q - Exit
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('q') {
            info!("Ctrl+Q pressed, exiting");
            self.running = false;
            return true;
        }
        
        
        false
    }

    /// Handle a mouse event
    fn handle_mouse_event(&mut self, mouse: MouseEvent) -> Result<()> {
        use crossterm::event::MouseEventKind;
        self.mouse_context.update(&mouse);

        // Route to chat component for scroll
        if mouse.kind == MouseEventKind::ScrollUp {
            self.chat_component.scroll_up(3);
        } else if mouse.kind == MouseEventKind::ScrollDown {
            self.chat_component.scroll_down(3, &self.card_manager);
        }
        Ok(())
    }

    /// Handle a terminal resize event
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        execute!(io::stdout(), crossterm::terminal::SetSize(width, height))?;
        let _ = self.send_gateway_request(TuiRequest::TerminalResize { cols: width, rows: height });
        Ok(())
    }

    /// Handle a paste event
    fn handle_paste(&mut self, text: &str) -> Result<()> {
        self.input_composer.insert_text(text);
        self.current_input = self.input_composer.get_input().to_string();
        Ok(())
    }

    /// Handle a message from the gateway
    fn handle_gateway_message(&mut self, message: GatewayMessage) -> Result<()> {
        debug!("Received gateway message: {:?}", message);
        
        match message {
            // Gateway lifecycle
            GatewayMessage::Ready(ready) => {
                self.handle_gateway_ready(ready)?;
            }
            GatewayMessage::Stderr(stderr) => {
                self.handle_gateway_stderr(stderr)?;
            }
            GatewayMessage::Activity(activity) => {
                self.handle_gateway_activity(activity)?;
            }
            
            // Session lifecycle
            GatewayMessage::SessionCreate(response) => {
                self.handle_session_create(response)?;
            }
            GatewayMessage::SessionResume(response) => {
                self.handle_session_resume(response)?;
            }
            GatewayMessage::SessionList(response) => {
                self.handle_session_list(response)?;
            }
            GatewayMessage::SessionActivate(response) => {
                self.handle_session_activate(response)?;
            }
            GatewayMessage::SessionInflight(response) => {
                self.handle_session_inflight(response)?;
            }
            GatewayMessage::SessionInfo(info) => {
                self.handle_session_info(info)?;
            }

            GatewayMessage::StatusUpdate(status) => {
                debug!("Status update: {:?}", status);
            }
            GatewayMessage::ReasoningAvailable(reasoning) => {
                debug!("Reasoning available: {:?}", reasoning);
            }
            GatewayMessage::ReasoningDelta(delta) => {
                debug!("Reasoning delta: {:?}", delta);
            }
            GatewayMessage::MessageStart(start) => {
                debug!("Message start: {:?}", start);
            }
            GatewayMessage::ThinkingDelta(delta) => {
                debug!("Thinking delta: {:?}", delta);
            }
            GatewayMessage::NoticeUpsert(notice) => {
                debug!("Notice upsert: {:?}", notice);
            }
            GatewayMessage::NoticeClear(notice) => {
                debug!("Notice clear: {:?}", notice);
            }
            GatewayMessage::NotificationShow(notice) => {
                debug!("Notification show: {:?}", notice);
            }
            GatewayMessage::NotificationClear(notice) => {
                debug!("Notification clear: {:?}", notice);
            }
            GatewayMessage::PreviewRestartProgress(progress) => {
                debug!("Preview restart progress: {:?}", progress);
            }
            GatewayMessage::PreviewRestartComplete(complete) => {
                debug!("Preview restart complete: {:?}", complete);
            }
            GatewayMessage::VoiceTranscript(transcript) => {
                debug!("Voice transcript: {:?}", transcript);
            }
            GatewayMessage::VoiceStatus(status) => {
                debug!("Voice status: {:?}", status);
            }
            GatewayMessage::BrowserProgress(progress) => {
                debug!("Browser progress: {:?}", progress);
            }
            GatewayMessage::SkinChanged(skin) => {
                debug!("Skin changed: {:?}", skin);
            }
            GatewayMessage::BackgroundComplete(complete) => {
                debug!("Background complete: {:?}", complete);
            }
            GatewayMessage::ReviewSummary(summary) => {
                debug!("Review summary: {:?}", summary);
            }

            // Messages
            GatewayMessage::MessageDelta(delta) => {
                self.handle_message_delta(delta)?;
            }
            GatewayMessage::MessageComplete(complete) => {
                self.handle_message_complete(complete)?;
            }
            
            // Subagents
            GatewayMessage::SubagentStart(event) => {
                self.handle_subagent_start(event)?;
            }
            GatewayMessage::SubagentThinking(event) => {
                self.handle_subagent_thinking(event)?;
            }
            GatewayMessage::SubagentProgress(event) => {
                self.handle_subagent_progress(event)?;
            }
            GatewayMessage::SubagentTool(event) => {
                self.handle_subagent_tool(event)?;
            }
            GatewayMessage::SubagentComplete(event) => {
                self.handle_subagent_complete(event)?;
            }
            // Tools
            GatewayMessage::ToolStart(tool_start) => {
                self.handle_tool_start(tool_start)?;
            }
            GatewayMessage::ToolProgress(tool_progress) => {
                self.handle_tool_progress(tool_progress)?;
            }
            GatewayMessage::ToolComplete(tool_complete) => {
                self.handle_tool_complete(tool_complete)?;
            }
            GatewayMessage::ToolGenerating(generating) => {
                debug!("Tool generating: {:?}", generating);
            }

            // Approvals
            GatewayMessage::ApprovalRequest(request) => {
                self.handle_approval_request(request)?;
            }
            
            // Completions
            GatewayMessage::SlashCompletion(completion) => {
                self.handle_slash_completion(completion)?;
            }
            GatewayMessage::PathCompletion(completion) => {
                self.handle_path_completion(completion)?;
            }
            
            // Slash exec
            GatewayMessage::SlashExec(response) => {
                self.handle_slash_exec(response)?;
            }
            
            // Config
            GatewayMessage::ConfigGet(response) => {
                self.handle_config_get(response)?;
            }
            
            // Prompt
            GatewayMessage::PromptSubmit(response) => {
                self.handle_prompt_submit(response)?;
            }
            
            // Error
            GatewayMessage::Error(error) => {
                self.handle_gateway_error(error)?;
            }
        }
        
        Ok(())
    }
    
    /// Handle gateway ready message (first message after connection)
    fn handle_gateway_ready(&mut self, ready: GatewayReadyResponse) -> Result<()> {
        info!("Gateway ready: {} sessions, {} models", 
            ready.sessions.as_ref().map_or(0, |v| v.len()),
            ready.models.as_ref().map_or(0, |v| v.len())
        );
        
        // Populate sessions in manager
        if let Some(sessions) = &ready.sessions {
            self.sessions_mut().set_sessions(sessions.clone());
        }

        // Auto-resume latest session or create a new one if none exist
        if let Some(sessions) = &ready.sessions {
            if let Some(latest) = sessions.first() {
                info!("Auto-resuming latest session: {}", latest.id);
                let request = SessionResumeRequest {
                    session_id: latest.id.clone(),
                };
                let tui_request = TuiRequest::SessionResume(request);
                let _ = self.send_gateway_request(tui_request);
            } else {
                self.create_new_session()?;
            }
        } else {
            self.create_new_session()?;
        }
        
        Ok(())
    }
    /// Handle gateway stderr output
    fn handle_gateway_stderr(&mut self, stderr: GatewayStderr) -> Result<()> {
        let level = stderr.level.as_deref().unwrap_or("INFO");
        match level {
            "ERROR" | "WARNING" | "WARN" => log::warn!("Gateway stderr: {}", stderr.line),
            "DEBUG" => log::debug!("Gateway stderr: {}", stderr.line),
            _ => log::info!("Gateway stderr: {}", stderr.line),
        }
        Ok(())
    }
    
    /// Handle gateway activity notification
    fn handle_gateway_activity(&mut self, activity: String) -> Result<()> {
        debug!("Gateway activity: {}", activity);
        // TODO: Update toolbar or status bar with activity
        Ok(())
    }
    
    /// Handle session create response
    fn handle_session_create(&mut self, response: SessionCreateResponse) -> Result<()> {
        info!("Session created: {}", response.session_id);
        
        // Add session to local store if it doesn't exist yet
        let session_id = response.session_id.clone();
        if self.sessions().get_session(&session_id).is_none() {
            let mut session = Session::new(session_id.clone());
            if let Some(ref info) = response.info {
                if let Some(ref title) = info.title {
                    session.name = Some(title.clone());
                }
            }
            self.sessions_mut().add_session(session);
        }
        
        // Set the new session as current
        self.sessions_mut().set_current_session(session_id);
        
        // Clear message history for new session
        self.messages_mut().clear();
        self.chat_component_mut().clear_messages();
        
        Ok(())
    }
    
    /// Handle session resume response
    fn handle_session_resume(&mut self, response: SessionResumeResponse) -> Result<()> {
        info!("Session resumed: {}", response.session_id);
        
        // Ensure session exists in local store
        let session_id = response.session_id.clone();
        if self.sessions().get_session(&session_id).is_none() {
            self.sessions_mut().add_session(Session::new(session_id.clone()));
        }
        
        // Set the resumed session as current
        self.sessions_mut().set_current_session(session_id);
        
        // Clear existing messages
        self.messages_mut().clear();
        self.chat_component_mut().clear_messages();
        
        // Add resumed messages to chat
        if let Some(messages) = response.messages {
            for msg in messages {
                let message = Message::new(
                    msg.role,
                    msg.text.unwrap_or_default(),
                );
                self.messages_mut().add_message(message);
            }
            // Sync chat component with message history
            self.sync_chat_with_messages();
        }
        
        // TODO: Handle inflight turn if present
        if let Some(inflight) = response.inflight {
            debug!("Inflight turn detected: streaming={:?}", inflight.streaming);
            // TODO: Show inflight indicator
        }
        
        Ok(())
    }
    fn handle_tool_start(&mut self, tool_start: ToolStart) -> Result<()> {
        debug!("Tool started: {} ({})", tool_start.tool_name, tool_start.call_id);

        self.thinking = true;

        // Create a tool card for this tool call with proper call_id
        let data = ToolCardData::running(&tool_start.tool_name)
            .with_call_id(&tool_start.call_id)
            .with_arguments(
                tool_start.arguments.clone().unwrap_or_default()
            );
        self.card_manager_mut().add_tool_card(data);

        // Show in chat with call_id as message_id for in-place updates later
        let mut message = Message::tool(
            format!("Tool '{}' started...", tool_start.tool_name),
            tool_start.tool_name,
        );
        message.message_id = Some(tool_start.call_id.clone());
        self.messages_mut().add_message(message.clone());
        self.chat_component.add_message(message, &self.card_manager);

        Ok(())
    }

    /// Handle tool progress
    fn handle_tool_progress(&mut self, tool_progress: ToolProgress) -> Result<()> {
        debug!("Tool progress: {} -> {}", tool_progress.call_id, tool_progress.output);
        
        // Update card_manager with progress output via call_id
        if let Some(card) = self.card_manager_mut().find_by_call_id_mut(&tool_progress.call_id) {
            let data = ToolCardData {
                tool_name: card.title().to_string(),
                call_id: tool_progress.call_id.clone(),
                status: ToolStatus::Running,
                duration_ms: None,
                arguments: None,
                result: Some(tool_progress.output.clone()),
                error: None,
            };
            card.update_from_data(&data);
        }
        
        Ok(())
    }

    /// Handle tool complete
    fn handle_tool_complete(&mut self, tool_complete: ToolComplete) -> Result<()> {
        debug!("Tool completed: {} ({}ms)", tool_complete.call_id, tool_complete.duration_ms.unwrap_or(0));
        
        self.thinking = false;
        
        // Update card manager with result via call_id
        let status = if tool_complete.error.is_some() {
            ToolStatus::Failed
        } else {
            ToolStatus::Completed
        };
        
        if let Some(card) = self.card_manager_mut().find_by_call_id_mut(&tool_complete.call_id) {
            let data = ToolCardData {
                tool_name: card.title().to_string(),
                call_id: tool_complete.call_id.clone(),
                status,
                duration_ms: tool_complete.duration_ms,
                arguments: None,
                result: Some(tool_complete.result.to_string()),
                error: tool_complete.error.clone(),
            };
            card.update_from_data(&data);
        }
        // Update tool message in-place using message_id (set in handle_tool_start)
        let result_text = if let Some(ref error) = tool_complete.error {
            format!("Error: {}", error)
        } else {
            tool_complete.result.to_string()
        };
        let duration_str = tool_complete.duration_ms.map_or(String::new(), |ms| format!(" ({}ms)", ms));
        let content = format!("Tool completed: {}{}", result_text, duration_str);

        if let Some(updated_msg) = self.messages_mut().update_message_by_id(&tool_complete.call_id, content.clone()) {
            self.chat_component.update_message(updated_msg);
        } else {
            // Fallback: starting message was missed, add a new one
            let message = Message::tool(content, format!("{} ({:?}ms)", tool_complete.call_id, tool_complete.duration_ms));
            self.messages_mut().add_message(message.clone());
            self.chat_component.add_message(message, &self.card_manager);
        }
        Ok(())
    }
    
    fn handle_session_activate(&mut self, response: SessionActivateResponse) -> Result<()> {
        info!("Session activated: {}", response.session_id);
        
        // Ensure session exists in local store
        let session_id = response.session_id.clone();
        if self.sessions().get_session(&session_id).is_none() {
            self.sessions_mut().add_session(Session::new(session_id.clone()));
        }
        
        self.sessions_mut().set_current_session(session_id);
        
        // Add activated messages to chat
        if let Some(messages) = response.messages {
            for msg in messages {
                let message = Message::new(
                    msg.role,
                    msg.text.unwrap_or_default(),
                );
                self.messages_mut().add_message(message);
            }
            self.sync_chat_with_messages();
        }
        
        Ok(())
    }
    
    /// Handle session inflight response
    fn handle_session_inflight(&mut self, response: SessionInflightResponse) -> Result<()> {
        debug!("Session inflight: {:?}", response.inflight);
        // TODO: Show inflight state in UI
        Ok(())
    }

    /// Handle session list response
    fn handle_session_list(&mut self, response: SessionListResponse) -> Result<()> {
        info!("Session list received with {} sessions", response.sessions.as_ref().map_or(0, |v| v.len()));
        if let Some(sessions) = response.sessions {
            self.sessions_mut().set_sessions(sessions.clone());
            self.session_picker.show(sessions);
        }
        Ok(())
    }
    /// Handle session info
    fn handle_session_info(&mut self, info_val: serde_json::Value) -> Result<()> {
        if let Some(model) = info_val.get("model").and_then(|v| v.as_str()) {
            debug!("Session info: model={}", model);
        }
        if let Some(provider) = info_val.get("provider").and_then(|v| v.as_str()) {
            debug!("Session info: provider={}", provider);
        }
        Ok(())
    }
    
    fn handle_message_delta(&mut self, delta: MessageDelta) -> Result<()> {
        debug!("Message delta: text='{}'", delta.text);
        let session_id = delta.session_id.clone().unwrap_or_default();
        let message_id = format!("{}:streaming", session_id);
        if let Some(last_msg) = self.messages().last() {
            if last_msg.message_id == Some(message_id.clone()) {
                if let Some(last_msg_mut) = self.messages_mut().last_mut() {
                    last_msg_mut.content.push_str(&delta.text);
                    let updated_message = last_msg_mut.clone();
                    self.chat_component_mut().update_message(updated_message);
                }
                return Ok(());
            }
        }
        let mut message = Message::streaming_delta(MessageRole::Assistant, delta.text, Some(message_id));
        message.complete = false;
        let message_clone = message.clone();
        self.messages_mut().add_message(message);
        self.chat_component.add_message(message_clone, &self.card_manager);
        Ok(())
    }
    
    fn handle_message_complete(&mut self, complete: MessageComplete) -> Result<()> {
        debug!("Message complete: text length={}", complete.text.len());
        self.thinking = false;
        let message = Message::new(MessageRole::Assistant, complete.text);
        self.messages_mut().add_message(message.clone());
        self.chat_component.add_message(message, &self.card_manager);
        self.chat_component.scroll_to_bottom(&self.card_manager);
        Ok(())
    }
    fn handle_approval_request(&mut self, request: ApprovalRequest) -> Result<()> {
        info!("Approval request: {} - {}", request.tool_name, request.message);
        
        self.pending_approval_id = Some(request.request_id.clone());

        // Show approval prompt to user
        self.prompt_manager.show_approval(
            format!("{} - {}", request.tool_name, request.message),
            Some(request.tool_name),
        );
        self.input_composer_mut().set_active(false);
        self.set_input_mode(InputMode::Normal);
        
        Ok(())
    }
    
    /// Handle slash completion
    fn handle_slash_completion(&mut self, completion: CompletionResponse) -> Result<()> {
        debug!("Slash completion: {:?}", completion.items);
        if let Some(items) = completion.items {
            self.completion_popup.show(items);
        }
        Ok(())
    }
    
    /// Handle path completion
    fn handle_path_completion(&mut self, completion: CompletionResponse) -> Result<()> {
        debug!("Path completion: {:?}", completion.items);
        if let Some(items) = completion.items {
            self.completion_popup.show(items);
        }
        Ok(())
    }
    
    /// Handle slash exec response
    fn handle_slash_exec(&mut self, response: SlashExecResponse) -> Result<()> {
        if let Some(output) = response.output {
            info!("Slash command output: {}", output);
            
            let message = Message::system(output);
            self.messages_mut().add_message(message.clone());
            self.chat_component.add_message(message, &self.card_manager);
            self.chat_component.scroll_to_bottom(&self.card_manager);
        }
        if let Some(warning) = response.warning {
            log::warn!("Slash command warning: {}", warning);
            
            let message = Message::system(format!("Warning: {}", warning));
            self.messages_mut().add_message(message.clone());
            self.chat_component.add_message(message, &self.card_manager);
            self.chat_component.scroll_to_bottom(&self.card_manager);
        }
        Ok(())
    }
    
    /// Handle config get response
    fn handle_config_get(&mut self, _response: ConfigGetResponse) -> Result<()> {
        debug!("Config get response");
        // TODO: Update config if needed
        Ok(())
    }
    
    /// Handle prompt submit response
    fn handle_prompt_submit(&mut self, response: PromptSubmitResponse) -> Result<()> {
        if response.ok.unwrap_or(true) {
            debug!("Prompt submitted successfully");
            // Clear input on successful submission
            self.input_composer_mut().clear();
        } else {
            log::warn!("Prompt submission failed");
        }
        Ok(())
    }
    
    /// Handle gateway error
    fn handle_gateway_error(&mut self, error: GatewayError) -> Result<()> {
        log::error!("Gateway error: {}", error.message);
        self.thinking = false; // Reset thinking on error
        if let Some(details) = error.details {
            log::error!("Details: {}", details);
        }
        Ok(())
    }
    
    /// Sync chat component with message history
    fn sync_chat_with_messages(&mut self) {
        let messages = self.messages().all_messages().clone();
        self.chat_component.set_messages(messages, &self.card_manager);
    }
    /// Draw the UI
    pub fn draw(&mut self) -> Result<()> {
        // Update toolbar state before drawing
        let connected = self.is_gateway_connected();
        let model = self.config.theme.name.clone();
        let session_id: Option<String> = self.sessions().current_session().map(|s| s.id.clone());
        let session: Option<&str> = session_id.as_deref();

        self.toolbar.update_status(connected, Some(&model), session);

        // Pre-compute layout values to avoid borrow conflicts inside the closure
        let hashline_block = self.message_history.last()
            .and_then(|m| if m.is_edit_tool_message() {
                crate::state::hashline::HashlineParser::parse(&m.content)
            } else {
                None
            });
        let has_hashline = hashline_block.is_some();
        let target_height = if has_hashline {
            8
        } else {
            0
        };
        // Smooth interpolation: ease-out to target height
        self.actual_activity_height += (target_height as f32 - self.actual_activity_height) * 0.3;
        let activity_height = self.actual_activity_height.round() as u16;
        let n_sessions = self.sessions().len() as u16;
        let session_manager_ptr: *const crate::state::session::SessionManager = self.sessions();
        
        // Use raw pointers for all component references to avoid Rust's borrow checking conflicts
        // This is safe because terminal.draw is FnOnce (synchronous) and each component is
        // only accessed from one code path within the closure.
        let banner_ptr: *const crate::ui::banner::Banner = &self.banner;
        let card_manager_ptr: *const CardManager = &self.card_manager;
        let hashline_viewer_ptr: *const HashlineViewer = &self.hashline_viewer;
        let input_composer_ptr: *const InputComposer = &self.input_composer;
        let toolbar_ptr: *const Toolbar = &self.toolbar;
        let session_picker_ptr: *const SessionPicker = &self.session_picker;
        let prompt_manager_ptr: *const PromptManager = &self.prompt_manager;
        let completion_popup_ptr: *const CompletionPopup = &self.completion_popup;
        let config_ptr: *const TuiConfig = &self.config;
        let subagent_list_ptr: *const SubagentList = &self.subagent_list;
        let chat_component_ptr: *mut ChatComponent = &mut self.chat_component;
        
        self.terminal.draw(move |frame| {

            let area = frame.area();
            if area.width < 20 || area.height < 10 {
                return;
            }

            let show_sidebar = area.width >= 80;
            let (main_area, sidebar_area) = if show_sidebar {
                let horiz = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(24),
                    ])
                    .split(area);
                (horiz[0], horiz[1])
            } else {
                (area, Rect::default())
            };

            let banner_height = if area.height > 40 { 7 } else { 1 };

            let constraints = [
                Constraint::Length(banner_height),
                Constraint::Min(1),
                Constraint::Length(activity_height),
                Constraint::Length(3),
                Constraint::Length(1),
            ];
            
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(constraints)
                .split(main_area);
            
            // Banner
            let banner = unsafe { &*banner_ptr };
            if banner_height > 1 {
                banner.render(frame, main_layout[0]);
            } else {
                banner.render_mini(frame, main_layout[0]);
            }
            
            // Chat
            // Chat
            let chat_component = unsafe { &mut *chat_component_ptr };
            let chat_area = main_layout[1];
            chat_component.set_visible_height(chat_area.height.saturating_sub(2));
            chat_component.render(frame, chat_area, unsafe { &*card_manager_ptr }, unsafe { &*subagent_list_ptr });
            
            // Activity (hashline only - cards are drawn inline in chat)
            if activity_height > 0 {
                let hashline_viewer = unsafe { &*hashline_viewer_ptr };
                let activity_area = main_layout[2];
                if let Some(ref block) = hashline_block {
                    hashline_viewer.render(block, activity_area, frame);
                }
            }
            
            // Composer
            let input_composer = unsafe { &*input_composer_ptr };
            input_composer.render_clean(frame, main_layout[3]);
            
            // Toolbar
            let toolbar = unsafe { &*toolbar_ptr };
            toolbar.render(frame, main_layout[4]);

            // Sidebar
            if show_sidebar && sidebar_area.width > 0 {
                let sidebar_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Max(n_sessions * 3 + 3),
                    ])
                    .split(sidebar_area);

                // Subagents
                let subagent_list = unsafe { &*subagent_list_ptr };
                let sub_area = sidebar_chunks[0];
                if !subagent_list.is_empty() {
                    subagent_list.render(sub_area, frame);
                }

                // Session sidebar
                Self::draw_session_sidebar_inner(frame, sidebar_chunks[1], unsafe { &*config_ptr }, session_manager_ptr);
            }

            // Overlays
            let overlay_area = frame.area();
            let session_picker = unsafe { &*session_picker_ptr };
            if session_picker.is_visible() {
                session_picker.render(frame, overlay_area);
            }
            let prompt_manager = unsafe { &*prompt_manager_ptr };
            if prompt_manager.has_active_prompt() {
                prompt_manager.render(frame, overlay_area);
            }
            let completion_popup = unsafe { &*completion_popup_ptr };
            if completion_popup.is_visible() {
                let footer_area = main_layout[3];
                let popup_area = Rect {
                    x: footer_area.x + 5,
                    y: footer_area.y.saturating_sub(10),
                    width: 40.min(footer_area.width),
                    height: 10.min(footer_area.y),
                };
                completion_popup.render(frame, popup_area);
            }
        })?;

        Ok(())
    }

    /// Inline helper for rendering the session sidebar (uses raw pointers for borrow-free access)
    fn draw_session_sidebar_inner(
        frame: &mut ratatui::Frame,
        area: Rect,
        config: &crate::state::config::TuiConfig,
        sessions_ptr: *const crate::state::session::SessionManager,
    ) {
        use ratatui::{
            style::{Color, Modifier, Style, Stylize},
            text::{Line, Span, Text},
            widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
        };
        
        // SAFETY: We're only reading the session manager, not writing to it.
        // This is safe because we have a shared reference and the draw closure
        // doesn't mutate the session manager.
        let sessions = unsafe { &*sessions_ptr };
        
        let block = Block::default()
            .title(" Sessions ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(config.theme.chat.to_rgb_colors().border));
        
        let inner = block.inner(area);
        frame.render_widget(block, area);
        
        let mut lines: Vec<Line> = Vec::new();
        let session_list = sessions.session_list();
        let current_id = sessions.current_session_id();
        
        for session in session_list.iter().take(10) {
            let name = session.name.as_deref().unwrap_or("Unnamed");
            let is_current = current_id.map(|id| id == &session.id).unwrap_or(false);
            
            let prefix = if is_current { "▸ " } else { "  " };
            let style = if is_current {
                Style::new().fg(Color::Rgb(166, 226, 46)).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::Rgb(117, 113, 94))
            };
            
            let msg_count = session.message_count();
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, name), style)));
            lines.push(Line::from(Span::styled(
                format!("  {} msgs", msg_count),
                Style::new().fg(Color::Rgb(102, 217, 239)),
            )));
        }
        
        if session_list.is_empty() {
            lines.push(Line::from(Span::styled(
                "No sessions yet",
                Style::new().fg(Color::Rgb(117, 113, 94)).italic(),
            )));
        }
        
        let paragraph = Paragraph::new(Text::from(lines))
            .wrap(Wrap { trim: false })
            .block(Block::default().padding(Padding::new(1, 0, 0, 0)));
        frame.render_widget(paragraph, inner);
    }

    // ============================================================================
    // Cleanup
    // ============================================================================

    /// Clean up terminal state on exit.
    ///
    /// 1. Kill the gateway child process first.
    /// 2. Disconnect the gateway client.
    /// 3. Restore terminal to normal mode.
    pub fn cleanup(&mut self) -> Result<()> {
        // Leave alternate screen immediately to restore terminal state
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
        let _ = io::stdout().flush();

        // Kill gateway process - don't block too long
        if let Some(mut child) = self.gateway_process.take() {
            let _ = child.kill();
        }
        
        // Disconnect gateway client
        if self.gateway_client.is_connected() {
            let _ = self.gateway_client.disconnect();
        }
        
        Ok(())
    }
    fn handle_subagent_start(&mut self, event: SubagentEvent) -> Result<()> {
        debug!("Subagent start: {}", event.subagent_id.as_deref().unwrap_or("?"));
        let agent_id = event.subagent_id.clone().unwrap_or_else(|| format!("agent-{}", event.task_index));
        let info = crate::ui::subagent::SubagentInfo::new(
            &agent_id,
            &event.goal,
            event.parent_id.clone(),
        );
        self.subagent_list_mut().upsert(info);
        // Add system message to chat with special ID for inline rendering
        let mut msg = Message::system(format!("Subagent '{}' started: {}", agent_id, event.goal));
        msg.message_id = Some(format!("subagent:{}", agent_id));
        self.messages_mut().add_message(msg.clone());
        self.chat_component.add_message(msg, &self.card_manager);
        Ok(())
    }

    fn handle_subagent_thinking(&mut self, event: SubagentEvent) -> Result<()> {
        debug!("Subagent thinking: {:?}", event.text);
        Ok(())
    }

    fn handle_subagent_progress(&mut self, event: SubagentEvent) -> Result<()> {
        debug!("Subagent progress: {:?}", event.text);
        Ok(())
    }

    fn handle_subagent_tool(&mut self, event: SubagentEvent) -> Result<()> {
        debug!("Subagent tool: {} - {}", event.subagent_id.as_deref().unwrap_or("?"), event.tool_name.as_deref().unwrap_or("?"));
        // Update subagent goal to show current tool
        if let Some(agent_id) = &event.subagent_id {
            if let Some(tool_name) = &event.tool_name {
                let info = crate::ui::subagent::SubagentInfo::new(
                    agent_id,
                    format!("Running tool '{}'...", tool_name),
                    event.parent_id.clone(),
                );
                self.subagent_list_mut().upsert(info);
            }
        }
        Ok(())
    }

    fn handle_subagent_complete(&mut self, event: SubagentEvent) -> Result<()> {
        debug!("Subagent complete: {}", event.subagent_id.as_deref().unwrap_or("?"));
        if let Some(agent_id) = &event.subagent_id {
            let summary = event.summary.clone().unwrap_or_default();
            let mut info = crate::ui::subagent::SubagentInfo::new(
                agent_id,
                &event.goal,
                event.parent_id.clone(),
            );
            if event.status.as_deref() == Some("failed") || event.status.as_deref() == Some("error") {
                info.mark_failed();
            } else {
                info.mark_completed(summary.clone());
            }
            self.subagent_list_mut().upsert(info);
            // Update the chat message inline
            let status_str = if event.status.as_deref() == Some("failed") || event.status.as_deref() == Some("error") {
                "Failed"
            } else {
                "Completed"
            };
            let content = format!("Subagent '{}' {}: {}", agent_id, status_str, summary);
            let msg_id = format!("subagent:{}", agent_id);
            if let Some(updated_msg) = self.messages_mut().update_message_by_id(&msg_id, content) {
                self.chat_component.update_message(updated_msg);
            }
        }
        Ok(())
    }

}

impl Drop for App {
    /// Automatically clean up terminal state when App is dropped.
    fn drop(&mut self) {
        let _ = self.cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_app_struct_size() {
        let config = TuiConfig::default();
        let theme_colors_rgb = config.theme.colors.to_rgb_colors();
        let chat_colors_rgb = config.theme.chat.to_rgb_colors();
        
        let _app: App = App {
            terminal: Terminal::new(CrosstermBackend::new(std::io::stdout())).unwrap(),
            config,
            session_manager: SessionManager::new(),
            message_history: MessageHistory::new(),
            gateway_client: GatewayClient::new(),
            thinking: false,
            reconnecting: false,
            reconnect_attempts: 0,
            card_manager: CardManager::new(chat_colors_rgb),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            gateway_process: None,
            running: true,
            input_mode: InputMode::Insert,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb, true),
            input_composer: InputComposer::new(chat_colors_rgb),
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb),
            banner: crate::ui::banner::Banner::default(),
            prompt_manager: PromptManager::new(chat_colors_rgb),
            pending_approval_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb),
            session_picker: SessionPicker::new(chat_colors_rgb),
            actual_activity_height: 0.0,
            mouse_context: MouseContext::new(),
        };
    }
    
    #[test]
    fn test_getters_setters() {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend).unwrap();
        
        let config = TuiConfig::default();
        let theme_colors_rgb = config.theme.colors.to_rgb_colors();
        let chat_colors_rgb = config.theme.chat.to_rgb_colors();
        
        let mut app = App {
            terminal,
            config,
            session_manager: SessionManager::new(),
            message_history: MessageHistory::new(),
            gateway_client: GatewayClient::new(),
            thinking: false,
            reconnecting: false,
            reconnect_attempts: 0,
            card_manager: CardManager::new(chat_colors_rgb),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            gateway_process: None,
            running: true,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb, true),
            input_composer: InputComposer::new(chat_colors_rgb),
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb),
            banner: crate::ui::banner::Banner::default(),
            prompt_manager: PromptManager::new(chat_colors_rgb),
            pending_approval_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb),
            session_picker: SessionPicker::new(chat_colors_rgb),
            actual_activity_height: 0.0,
            mouse_context: MouseContext::new(),
        };
        assert!(app.input_composer().get_input().is_empty());
        assert!(app.toolbar().input_mode() == InputMode::Normal);
        
        app.set_running(false);
        assert!(!app.is_running());
        
        app.set_running(true);
        app.set_input_mode(InputMode::Insert);
        assert_eq!(app.input_mode(), InputMode::Insert);
        
        app.set_current_input("test".to_string());
        assert_eq!(app.current_input(), "test");
        
        app.set_cursor_position(5);
        assert_eq!(app.cursor_position(), 5);
        
        app.clear_input();
        assert_eq!(app.current_input(), "");
        assert_eq!(app.cursor_position(), 0);
    }
}
