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
use log::{debug, info};
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
    pub fn disconnect_gateway(&mut self) -> Result<()> {
        self.gateway_client.disconnect()
    }

    // ============================================================================
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

            // Update toolbar animation
            self.toolbar.tick(self.thinking);

            // Draw UI if no event was processed
            self.draw()?;
            }
        
        info!("Event loop ended");
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
                        self.chat_component_mut().add_message(system_message);
                        self.chat_component_mut().scroll_to_bottom();
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


        
        // Check for Enter key in Insert mode - submit prompt
        let KeyEvent { code, modifiers, .. } = key;
        if code == KeyCode::Enter && !modifiers.contains(KeyModifiers::SHIFT) {
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
        
        // Check for Esc key - exit command mode or clear input
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
        
        // Pass key to input composer for handling
        let handled = self.input_composer.handle_key_event(key);
        
        if handled {
            // If input composer handled the key, sync the app's legacy input state
            self.current_input = self.input_composer.get_input().to_string();
            // cursor_pos is private, so we can't access it directly
            // For now, just update from the composer's internal state
            self.input_mode = self.input_composer.input_mode();
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
        self.chat_component_mut().add_message(user_message);
        
        // Auto-scroll to bottom
        self.chat_component_mut().scroll_to_bottom();
        
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
        self.chat_component_mut().add_message(user_message);
        self.chat_component_mut().scroll_to_bottom();
        
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
        self.chat_component_mut().add_message(system_message);
        self.chat_component_mut().scroll_to_bottom();
        
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
            self.chat_component_mut().add_message(system_message);
            self.chat_component_mut().scroll_to_bottom();
        } else {
            let system_message = Message::system("No previous sessions to resume");
            self.messages_mut().add_message(system_message.clone());
            self.chat_component_mut().add_message(system_message);
            self.chat_component_mut().scroll_to_bottom();
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
        self.chat_component_mut().add_message(system_message);
        self.chat_component_mut().scroll_to_bottom();
        
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
            self.chat_component.scroll_down(3);
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

            // Messages
            GatewayMessage::MessageDelta(delta) => {
                self.handle_message_delta(delta)?;
            }
            GatewayMessage::MessageComplete(complete) => {
                self.handle_message_complete(complete)?;
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
        
        // Set the new session as current
        self.sessions_mut().set_current_session(response.session_id.clone());
        
        // Clear message history for new session
        self.messages_mut().clear();
        self.chat_component_mut().clear_messages();
        
        // TODO: Show notification to user
        
        Ok(())
    }
    
    /// Handle session resume response
    fn handle_session_resume(&mut self, response: SessionResumeResponse) -> Result<()> {
        info!("Session resumed: {}", response.session_id);
        
        // Set the resumed session as current
        self.sessions_mut().set_current_session(response.session_id.clone());
        
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
    
    /// Handle session list response
    fn handle_session_list(&mut self, response: SessionListResponse) -> Result<()> {
        if let Some(sessions) = response.sessions {
            self.sessions_mut().set_sessions(sessions.clone());
            self.session_picker.show(sessions);
        }
        Ok(())
    }
    
    /// Handle session activate response
    fn handle_session_activate(&mut self, response: SessionActivateResponse) -> Result<()> {
        info!("Session activated: {}", response.session_id);
        
        self.sessions_mut().set_current_session(response.session_id.clone());
        
        // Clear existing messages
        self.messages_mut().clear();
        self.chat_component_mut().clear_messages();
        
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
    
    /// Handle message delta (streaming)
    fn handle_message_delta(&mut self, delta: MessageDelta) -> Result<()> {
        debug!("Message delta: role={:?}, delta='{}', done={:?}",
            delta.role, delta.delta, delta.done);

        // Get or create a streaming message
        let session_id = delta.session_id.clone().unwrap_or_default();
        let message_id = format!("{}:streaming", session_id);
        
        // Check if we have an existing streaming message for this session
        if let Some(last_msg) = self.messages().last() {
            if last_msg.message_id == Some(message_id.clone()) {
                // Need to update the message - get mutable access and clone first
                if let Some(last_msg_mut) = self.messages_mut().last_mut() {
                    let updated_message = {
                        last_msg_mut.content.push_str(&delta.delta);
                        last_msg_mut.streaming = true;
                        last_msg_mut.complete = delta.done.unwrap_or(false);
                        last_msg_mut.clone()
                    };
                    
                    // Update chat component with cloned message
                    self.chat_component_mut().update_message(updated_message);
                }
                return Ok(());
            }
        }
        
        // Create new streaming message
        let role = delta.role.unwrap_or(MessageRole::Assistant);
        let mut message = Message::streaming_delta(role, delta.delta, Some(message_id));
        message.complete = delta.done.unwrap_or(false);
        
        // Add to both message history and chat component
        let message_clone = message.clone();
        self.messages_mut().add_message(message);
        self.chat_component_mut().add_message(message_clone);
        
        Ok(())
    }
    
    /// Handle message complete
    fn handle_message_complete(&mut self, complete: MessageComplete) -> Result<()> {
        debug!("Message complete: role={:?}, content length={}",
            complete.role, complete.content.len());

        self.thinking = false;
        
        let message = Message::new(complete.role, complete.content);
        
        self.messages_mut().add_message(message.clone());
        self.chat_component_mut().add_message(message);
        
        // Auto-scroll to bottom on new message
        self.chat_component_mut().scroll_to_bottom();
        
        Ok(())
    }

    /// Handle session info message
    fn handle_session_info(&mut self, info_val: serde_json::Value) -> Result<()> {
        if let Ok(info) = serde_json::from_value::<SessionInfo>(info_val) {
            if let Some(id) = &info.id {
                info!("Session info received: {}", id);
                self.sessions_mut().set_current_session(id.clone());
                
                // If it's a new session we didn't know about, add it
                if self.sessions().get_session(id).is_none() {
                    let mut session = Session::new(id.clone());
                    if let Some(title) = &info.title {
                        session.name = Some(title.clone());
                    }
                    self.sessions_mut().add_session(session);
                }
            }
        }
        Ok(())
    }
    
    /// Handle tool start
    fn handle_tool_start(&mut self, tool_start: ToolStart) -> Result<()> {
        debug!("Tool started: {} ({})", tool_start.tool_name, tool_start.call_id);

        self.thinking = true;

        // Create a tool card for this tool call
        let data = ToolCardData::running(&tool_start.tool_name)
            .with_arguments(
                serde_json::to_string(&tool_start.arguments.clone().unwrap_or_default())
                    .unwrap_or_default()
            );
        self.card_manager_mut().add_tool_card(data);

        // Also show in chat for compatibility
        let message = Message::tool(
            format!("Tool '{}' started...", tool_start.tool_name),
            tool_start.tool_name,
        );
        self.messages_mut().add_message(message.clone());
        self.chat_component_mut().add_message(message);

        Ok(())
    }
    
    /// Handle tool progress
    fn handle_tool_progress(&mut self, tool_progress: ToolProgress) -> Result<()> {
        debug!("Tool progress: {} -> {}", tool_progress.call_id, tool_progress.output);
        
        // Update card_manager with progress output
        self.card_manager_mut().update_tool_status(
            &tool_progress.call_id,
            ToolStatus::Running,
            Some(tool_progress.output.clone()),
            None,
        );
        
        Ok(())
    }
    
    /// Handle tool complete
    fn handle_tool_complete(&mut self, tool_complete: ToolComplete) -> Result<()> {
        debug!("Tool completed: {} ({}ms)", tool_complete.call_id, tool_complete.duration_ms.unwrap_or(0));
        
        self.thinking = false;
        
        // Update card manager with result
        if let Some(ref error) = tool_complete.error {
            self.card_manager_mut().update_tool_status(
                &tool_complete.call_id,
                ToolStatus::Failed,
                None,
                Some(error.clone()),
            );
        } else {
            self.card_manager_mut().update_tool_status(
                &tool_complete.call_id,
                ToolStatus::Completed,
                Some(tool_complete.result.clone()),
                None,
            );
        }
        
        // Also show in chat for compatibility
        let result_text = if let Some(ref error) = tool_complete.error {
            format!("Error: {}", error)
        } else {
            tool_complete.result.clone()
        };
        
        let message = Message::tool(
            format!("Tool completed: {}", result_text),
            format!("{} ({}ms)", tool_complete.call_id, tool_complete.duration_ms.unwrap_or(0)),
        );
        self.messages_mut().add_message(message.clone());
        self.chat_component_mut().add_message(message);
        
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
            self.chat_component_mut().add_message(message);
            self.chat_component_mut().scroll_to_bottom();
        }
        if let Some(warning) = response.warning {
            log::warn!("Slash command warning: {}", warning);
            
            let message = Message::system(format!("Warning: {}", warning));
            self.messages_mut().add_message(message.clone());
            self.chat_component_mut().add_message(message);
            self.chat_component_mut().scroll_to_bottom();
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
        self.chat_component_mut().set_messages(messages);
    }

    /// Draw the UI
    pub fn draw(&mut self) -> Result<()> {
        // Update toolbar state before drawing
        let connected = self.is_gateway_connected();
        let model = self.config.theme.name.clone();
        let session_id: Option<String> = self.sessions().current_session().map(|s| s.id.clone());
        let session: Option<&str> = session_id.as_deref();

        self.toolbar.update_status(connected, Some(&model), session);

        self.terminal.draw(|frame| {
            let size = frame.area();

            // Skip rendering if dimensions are too small
            if size.width < 20 || size.height < 10 {
                return;
            }

            // Determine layout: with or without subagent sidebar
            let has_subagents = !self.subagent_list.is_empty();

            let (main_area, sidebar_area) = if has_subagents && size.width >= 100 {
                let horiz = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Min(1),
                        Constraint::Length(30),
                    ])
                    .split(size);
                (horiz[0], horiz[1])
            } else {
                (size, Rect::default())
            };

            // Check for tool cards and hashline edits to render
            let has_cards = !self.card_manager.is_empty();
            let hashline_block = self.message_history.last()
                .and_then(|m| if m.is_edit_tool_message() {
                    crate::state::hashline::HashlineParser::parse(&m.content)
                } else {
                    None
                });
            let has_hashline = hashline_block.is_some();
            
            // Build constraints dynamically
            let activity_height = if has_cards {
                (self.card_manager.len() as u16 * 4).min(12).max(4)
            } else if has_hashline {
                8
            } else {
                0
            };

            // Banner height: 7 for full logo + spacing, 1 for mini
            let banner_height = if size.height > 40 { 7 } else { 1 };
            let footer_height = if self.input_composer.get_input().is_empty() { 3 } else { 1 };

            let constraints = [
                Constraint::Length(banner_height), // Top Banner
                Constraint::Min(1),                // Main chat/landing area (Boxed)
                Constraint::Length(activity_height), // activity pane (if any)
                Constraint::Length(footer_height),   // footer area (Unboxed)
                Constraint::Length(1),             // toolbar
            ];
            
            let main_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints(constraints)
                .split(main_area);
            
            // 0. Render banner
            if banner_height > 1 {
                self.banner.render(frame, main_layout[0]);
            } else {
                self.banner.render_mini(frame, main_layout[0]);
            }
            
            // 1. Render chat component (Main Area - Boxed)
            let chat_area = main_layout[1];
            self.chat_component.set_visible_height(chat_area.height.saturating_sub(2)); // Subtract 2 for borders
            self.chat_component.render(frame, chat_area);
            
            // 2. Render activity area (cards or hashline) if present
            if activity_height > 0 {
                let activity_area = main_layout[2];
                if has_cards {
                    self.card_manager.render_stack(frame, activity_area);
                } else if let Some(block) = hashline_block {
                    self.hashline_viewer.render(&block, activity_area, frame);
                }
            }
            
            // 3. Render clean footer (Unboxed)
            self.input_composer.render_clean(frame, main_layout[3]);
            
            // 4. Render toolbar
            self.toolbar.render(frame, main_layout[4]);

            // Render subagent sidebar if there are active subagents
            if has_subagents && sidebar_area.width > 0 {
                self.subagent_list.set_visible_height(sidebar_area.height.saturating_sub(2));
                self.subagent_list.render(sidebar_area, frame);
            }

            // --- Overlays ---
            if self.session_picker.is_visible() {
                self.session_picker.render(frame, size);
            } else if self.prompt_manager.has_active_prompt() {
                self.prompt_manager.render(frame, size);
            } else if self.completion_popup.is_visible() {
                let footer_area = main_layout[3];
                let popup_area = Rect {
                    x: footer_area.x + 5,
                    y: footer_area.y.saturating_sub(10),
                    width: 40.min(footer_area.width),
                    height: 10.min(footer_area.y),
                };
                self.completion_popup.render(frame, popup_area);
            }
        })?;

        Ok(())
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
            gateway_process: None,
            running: true,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb, true),
            input_composer: InputComposer::new(chat_colors_rgb),
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb),
            banner: crate::ui::banner::Banner::default(),
            card_manager: CardManager::new(chat_colors_rgb),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            prompt_manager: PromptManager::new(chat_colors_rgb),
            pending_approval_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb),
            session_picker: SessionPicker::new(chat_colors_rgb),
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
            gateway_process: None,
            running: true,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb, true),
            input_composer: InputComposer::new(chat_colors_rgb),
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb),
            banner: crate::ui::banner::Banner::default(),
            card_manager: CardManager::new(chat_colors_rgb),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            prompt_manager: PromptManager::new(chat_colors_rgb),
            pending_approval_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb),
            session_picker: SessionPicker::new(chat_colors_rgb),
            mouse_context: MouseContext::new(),
        };
        
        assert!(app.is_running());
        assert_eq!(app.input_mode(), InputMode::Normal);
        assert_eq!(app.current_input(), "");
        assert_eq!(app.cursor_position(), 0);
        
        assert!(app.chat_component().messages().is_empty());
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
