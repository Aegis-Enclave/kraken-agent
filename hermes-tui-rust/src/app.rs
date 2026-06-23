//! Application module - Main App struct and event loop
//!
//! This module contains the main application state and the event loop
//! that drives the TUI.

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
    },
};
use log::{debug, error, info, warn};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Block,
    Terminal,
};
use std::io::{self, Stdout, Write};

use crate::ui::cards::{CardManager, ToolCardData, ToolStatus};
use crate::ui::hashline::HashlineViewer;
use crate::ui::subagent::SubagentList;

use crate::handlers::mouse::MouseContext;
use crate::protocol::client::GatewayClient;
use crate::protocol::types::{
    ApprovalRequest, ApprovalResponse, ClarifyRequest, ClarifyResponse, CompletionResponse,
    ConfigGetResponse, GatewayError, GatewayMessage, GatewayReadyResponse, GatewayStderr,
    MessageComplete, MessageDelta, MessageRole, ModelOptionsResponse, PromptSubmitRequest,
    PromptSubmitResponse, SecretRequest, SecretResponse, SessionActivateResponse,
    SessionCreateRequest, SessionCreateResponse, SessionInflightResponse, SessionListResponse,
    SessionResumeRequest, SessionResumeResponse, SlashExecRequest, SlashExecResponse,
    SubagentEvent, SudoRequest, SudoResponse, ToolComplete, ToolProgress, ToolStart, TuiRequest,
};
use crate::state::config::FocusPane;
use crate::state::config::InputMode;
use crate::state::{
    capabilities::Capabilities,
    config::TuiConfig,
    messages::{Message, MessageHistory},
    session::{Session, SessionManager},
};
use crate::ui::completions::CompletionPopup;
use crate::ui::model_picker::ModelPicker;
use crate::ui::prompts::PromptManager;
use crate::ui::session_picker::SessionPicker;
use crate::ui::{
    chat::ChatAction, chat::ChatCommand, chat::ChatComponent, composer::InputComposer,
    toolbar::Toolbar,
};
use crate::utils::clipboard::Clipboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewState {
    Dashboard,
    Ide,
    Kanban,
    Chat,
}

/// Main application struct
///
/// This struct holds all the state needed for the Hermes Rust TUI application.
/// It manages the terminal, configuration, sessions, messages, and gateway communication.
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
    /// Chat state for scroll position and selection
    chat_state: crate::ui::chat::ChatState,
    /// IDE state for file tree and editor
    ide_state: crate::ui::ide::IdeState,
    /// Input composer for user text input
    input_composer: InputComposer,
    /// Prompt manager for approval/clarify/secret overlays
    prompt_manager: PromptManager,
    /// Pending approval request ID
    pending_approval_id: Option<String>,
    /// Pending clarify request ID
    pending_clarify_id: Option<String>,
    /// Pending sudo request ID
    pending_sudo_id: Option<String>,
    /// Pending secret request ID
    pending_secret_id: Option<String>,
    /// Autocomplete completions popup widget
    completion_popup: CompletionPopup,
    /// Model picker popup widget
    model_picker: ModelPicker,
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
    /// Current model name from the gateway session info
    current_model: Option<String>,
    /// Current provider name from the gateway session info
    current_provider: Option<String>,
    /// Optional animated GIF for the dashboard
    /// Current view (Dashboard, IDE, Kanban, Chat)
    current_view: ViewState,
    /// Previous view (for transition animations)
    previous_view: Option<ViewState>,
    /// Transition progress (0.0 to 1.0)
    transition_progress: f32,
    /// System telemetry
    sys: sysinfo::System,
    /// Cached CPU usage percentage
    cpu_usage: f32,
    /// Cached memory usage percentage
    memory_usage: f32,
    /// History of CPU usage for sparkline
    cpu_history: Vec<u64>,
    /// History of memory usage for sparkline
    memory_history: Vec<u64>,
    /// History of token generation speeds (for sparkline)
    token_speed_history: Vec<u64>,
    /// Currently focused pane for keyboard navigation
    focus_pane: FocusPane,
    /// Global animation frame counter for animated borders
    animation_frame: u64,
    /// Sine-wave loading footer ticker (Aetheric Shader, Phase 4)
    wave_ticker: crate::ui::wave::WaveTicker,
    /// Clipboard backend (arboard or OSC 52 fallback).
    clipboard: Box<dyn Clipboard>,
    /// Gateway-reported capabilities for the empty-state landing page.
    capabilities: Capabilities,
    /// Last completion query sent to gateway (to deduplicate)
    last_completion_query: Option<String>,
    /// Whether the keybinding help overlay is visible
    show_help: bool,
    /// Whether we are in tmux-style prefix mode (Alt+A then a command key)
    prefix_mode: bool,
    /// Accumulator for streaming reasoning text received from the gateway
    /// via ReasoningAvailable / ReasoningDelta / ThinkingDelta events.
    /// Resets when a new assistant turn begins.
    pub streaming_reasoning: String,
    /// message_id of the currently-streaming assistant message. Used as a
    /// marker so reasoning updates only target the active turn.
    pub streaming_message_id: Option<String>,
}

impl App {
    /// # Returns
    /// - `Result<Self>`: New App instance or error during initialization.
    ///
    /// # Errors
    /// - Fails if terminal cannot be initialized
    /// - Fails if raw mode cannot be enabled
    pub fn new() -> Result<Self> {
        // Enable raw mode for terminal input
        enable_raw_mode().context("Failed to enable raw mode")?;
        // Enter alternate screen mode, enable mouse capture, and enable bracketed paste
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        )
        .context("Failed to enter alternate screen mode and enable mouse/paste")?;

        // Set terminal title
        execute!(io::stdout(), SetTitle("Hermes TUI (Rust)"))
            .context("Failed to set terminal title")?;

        // Initialize terminal with Crossterm backend
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend).context("Failed to create terminal")?;
        // Initialize config with Gruvbox theme by default
        let mut config = TuiConfig::default();
        config.theme = crate::state::config::BuiltinTheme::Gruvbox.to_config();

        let theme_colors_rgb = config.theme.colors.to_rgb_colors();
        let chat_colors_rgb = config.theme.chat.to_rgb_colors();

        let mut input_composer = InputComposer::new(chat_colors_rgb.clone());
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
            card_manager: CardManager::new(chat_colors_rgb.clone()),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            gateway_process: None,
            running: true,
            input_mode: InputMode::Insert,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb.clone(), true),
            chat_state: crate::ui::chat::ChatState::default(),
            ide_state: crate::ui::ide::IdeState::default(),
            input_composer,
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb.clone()),
            banner: crate::ui::banner::Banner,
            prompt_manager: PromptManager::new(chat_colors_rgb.clone()),
            pending_approval_id: None,
            pending_clarify_id: None,
            pending_sudo_id: None,
            pending_secret_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb.clone()),
            model_picker: ModelPicker::new(chat_colors_rgb.clone()),
            session_picker: SessionPicker::new(chat_colors_rgb.clone()),
            mouse_context: MouseContext::new(),
            current_provider: None,
            current_model: None,
            current_view: ViewState::Dashboard,
            previous_view: None,
            transition_progress: 1.0,
            sys: sysinfo::System::new_all(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            cpu_history: Vec::with_capacity(500),
            memory_history: Vec::with_capacity(500),
            token_speed_history: Vec::with_capacity(500),
            focus_pane: FocusPane::default(),
            animation_frame: 0,
            wave_ticker: crate::ui::wave::WaveTicker::new(),
            clipboard: Self::make_clipboard(),
            capabilities: Capabilities::default(),
            last_completion_query: None,
            show_help: false,
            prefix_mode: false,
            streaming_reasoning: String::new(),
            streaming_message_id: None,
        })
    }

    // ============================================================================
    // Getter Methods
    // ============================================================================

    /// Get a reference to the terminal
    #[must_use]
    pub fn terminal(&self) -> &Terminal<CrosstermBackend<Stdout>> {
        &self.terminal
    }

    /// Get a mutable reference to the terminal
    pub fn terminal_mut(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    /// Construct a clipboard backend: `arboard` if available, else OSC 52
    /// to a side-channel sink that is overwritten on every call.
    fn make_clipboard() -> Box<dyn Clipboard> {
        #[cfg(feature = "clipboard-support")]
        {
            match crate::utils::clipboard::ArboardClipboard::new() {
                Ok(cb) => return Box::new(cb),
                Err(e) => log::warn!("arboard clipboard unavailable ({e}); falling back to OSC 52"),
            }
        }
        // OSC 52 writes go to /dev/null: the side-effect is invisible but the
        // API is consistent. Terminal-side support is the user's responsibility.
        Box::new(crate::utils::clipboard::Osc52Clipboard::new(std::io::sink()))
    }

    /// Get a reference to the configuration
    #[must_use]
    pub fn config(&self) -> &TuiConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut TuiConfig {
        &mut self.config
    }

    /// Get a reference to the session manager
    #[must_use]
    pub fn sessions(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Get a mutable reference to the session manager
    pub fn sessions_mut(&mut self) -> &mut SessionManager {
        &mut self.session_manager
    }

    /// Get a reference to the message history
    #[must_use]
    pub fn messages(&self) -> &MessageHistory {
        &self.message_history
    }

    /// Get a mutable reference to the message history
    pub fn messages_mut(&mut self) -> &mut MessageHistory {
        &mut self.message_history
    }

    /// Get a reference to the gateway client
    #[must_use]
    pub fn gateway_client(&self) -> &GatewayClient {
        &self.gateway_client
    }

    /// Get a mutable reference to the gateway client
    pub fn gateway_client_mut(&mut self) -> &mut GatewayClient {
        &mut self.gateway_client
    }

    /// Get the current input mode
    #[must_use]
    pub fn input_mode(&self) -> InputMode {
        self.input_mode
    }

    /// Get the current input text
    #[must_use]
    pub fn current_input(&self) -> &str {
        &self.current_input
    }

    /// Get the current cursor position
    #[must_use]
    pub fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    // ============================================================================
    // UI Component Getters
    // ============================================================================

    /// Get a reference to the chat component
    #[must_use]
    pub fn chat_component(&self) -> &ChatComponent {
        &self.chat_component
    }

    /// Get a mutable reference to the chat component
    pub fn chat_component_mut(&mut self) -> &mut ChatComponent {
        &mut self.chat_component
    }

    /// Get a reference to the chat state
    #[must_use]
    pub fn chat_state(&self) -> &crate::ui::chat::ChatState {
        &self.chat_state
    }

    /// Get a mutable reference to the chat state
    pub fn chat_state_mut(&mut self) -> &mut crate::ui::chat::ChatState {
        &mut self.chat_state
    }

    /// Get a reference to the input composer
    #[must_use]
    pub fn input_composer(&self) -> &InputComposer {
        &self.input_composer
    }

    /// Get a mutable reference to the input composer
    pub fn input_composer_mut(&mut self) -> &mut InputComposer {
        &mut self.input_composer
    }

    /// Get a reference to the prompt manager
    #[must_use]
    pub fn prompt_manager(&self) -> &PromptManager {
        &self.prompt_manager
    }

    /// Get a mutable reference to the prompt manager
    pub fn prompt_manager_mut(&mut self) -> &mut PromptManager {
        &mut self.prompt_manager
    }

    /// Get a reference to the completion popup
    #[must_use]
    pub fn completion_popup(&self) -> &CompletionPopup {
        &self.completion_popup
    }

    /// Get a mutable reference to the completion popup
    pub fn completion_popup_mut(&mut self) -> &mut CompletionPopup {
        &mut self.completion_popup
    }

    /// Get a reference to the session picker
    #[must_use]
    pub fn session_picker(&self) -> &SessionPicker {
        &self.session_picker
    }

    /// Get a mutable reference to the session picker
    pub fn session_picker_mut(&mut self) -> &mut SessionPicker {
        &mut self.session_picker
    }

    /// Get a reference to the toolbar
    #[must_use]
    pub fn toolbar(&self) -> &Toolbar {
        &self.toolbar
    }

    /// Get a mutable reference to the toolbar
    pub fn toolbar_mut(&mut self) -> &mut Toolbar {
        &mut self.toolbar
    }

    /// Get a reference to the card manager
    #[must_use]
    pub fn card_manager(&self) -> &CardManager {
        &self.card_manager
    }

    /// Get a mutable reference to the card manager
    pub fn card_manager_mut(&mut self) -> &mut CardManager {
        &mut self.card_manager
    }

    /// Get a reference to the subagent list
    #[must_use]
    pub fn subagent_list(&self) -> &SubagentList {
        &self.subagent_list
    }

    /// Get a mutable reference to the subagent list
    pub fn subagent_list_mut(&mut self) -> &mut SubagentList {
        &mut self.subagent_list
    }

    /// Get a reference to the hashline viewer
    #[must_use]
    pub fn hashline_viewer(&self) -> &HashlineViewer {
        &self.hashline_viewer
    }

    // ============================================================================
    // Setter Methods
    // ============================================================================

    /// Set the running state of the application
    /// Check if the application is running
    #[must_use]
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

    /// Set the focused pane and adjust input mode accordingly
    pub fn set_focus_pane(&mut self, pane: FocusPane) {
        self.focus_pane = pane;
        match pane {
            FocusPane::Chat => {
                self.set_input_mode(InputMode::Normal);
                self.input_composer.set_active(false);
            }
            FocusPane::Sidebar => {
                self.set_input_mode(InputMode::Normal);
                self.input_composer.set_active(false);
            }
            FocusPane::Composer => {
                self.set_input_mode(InputMode::Insert);
                self.input_composer.set_active(true);
            }
            FocusPane::Toolbar => {
                self.set_input_mode(InputMode::Normal);
                self.input_composer.set_active(false);
            }
        }
        // Update toolbar with new focus pane
        self.toolbar.set_focus_pane(pane);
    }

    /// Get the current focus pane
    pub fn focus_pane(&self) -> FocusPane {
        self.focus_pane
    }

    /// Cycle to the next pane in the order: Chat → Composer → Toolbar → Sidebar → Chat
    pub fn focus_next_pane(&mut self) {
        let next = match self.focus_pane {
            FocusPane::Chat => FocusPane::Composer,
            FocusPane::Composer => FocusPane::Toolbar,
            FocusPane::Toolbar => FocusPane::Sidebar,
            FocusPane::Sidebar => FocusPane::Chat,
        };
        self.set_focus_pane(next);
    }

    /// Cycle to the previous pane in the order: Chat → Sidebar → Toolbar → Composer → Chat
    pub fn focus_prev_pane(&mut self) {
        let prev = match self.focus_pane {
            FocusPane::Chat => FocusPane::Sidebar,
            FocusPane::Composer => FocusPane::Chat,
            FocusPane::Toolbar => FocusPane::Composer,
            FocusPane::Sidebar => FocusPane::Toolbar,
        };
        self.set_focus_pane(prev);
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
        } else if current_dir
            .parent()
            .is_some_and(|p| p.join("tui_gateway").exists())
        {
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
            format!("{project_root_str}:{current_pythonpath}")
        };

        let mut child = std::process::Command::new(&python_cmd)
            .args(["-m", "tui_gateway.entry"])
            .env("PYTHONPATH", new_pythonpath) // Ensure tui_gateway and hermes packages can be found
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::from(stderr_file)) // Redirect to log file
            .spawn()
            .context(format!(
                "Failed to spawn gateway process using '{python_cmd}'."
            ))?;

        let child_stdin = child.stdin.take().context("Failed to take child stdin")?;
        let child_stdout = child.stdout.take().context("Failed to take child stdout")?;

        info!(
            "Gateway process spawned: {} (PID: {})",
            python_cmd,
            child.id()
        );

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
    pub fn send_gateway_request(
        &mut self,
        request: crate::protocol::types::TuiRequest,
    ) -> Result<()> {
        self.gateway_client.send_request(request)
    }

    /// Receive a message from the gateway
    #[must_use]
    pub fn receive_gateway_message(&self) -> Option<crate::protocol::types::GatewayMessage> {
        self.gateway_client.receive_message()
    }

    /// Check if gateway is connected
    #[must_use]
    pub fn is_gateway_connected(&self) -> bool {
        self.gateway_client.is_connected()
    }

    /// Disconnect from the gateway
    /// Run the main event loop
    pub fn run(&mut self) -> Result<()> {
        info!("Starting event loop (demand-driven render engine)");

        while self.running {
            // Demand-driven poll timeout: respects the active-animation counter
            // so the event loop sleeps deeply when nothing is animating.
            let timeout = crate::engine::poll_timeout();
            if event::poll(timeout)? {
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

            // Periodically refresh telemetry (approx. once per second at 60 FPS)
            if self.animation_frame % 60 == 0 {
                self.refresh_system_stats();
            }

            // Update toolbar animation
            self.toolbar.tick(self.thinking);
            // Sync prefix mode state to the toolbar indicator
            self.toolbar.set_prefix_mode(self.prefix_mode);

            // Advance or stop the sine-wave loading footer (Phase 4)
            if self.thinking {
                self.wave_ticker.advance();
            }

            // Increment animation frame for animated borders
            self.animation_frame = self.animation_frame.wrapping_add(1);

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
                    warn!("Gateway process exited unexpectedly with status: {status}");
                    self.trigger_reconnect()?;
                }
                Ok(None) => {
                    // Process is still running — nothing to do
                }
                Err(e) => {
                    error!("Error checking gateway process: {e}");
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
        info!(
            "Triggering gateway reconnect (attempt {})",
            self.reconnect_attempts
        );

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
        let msg = Message::system(format!(
            "Connection lost. Reconnecting (attempt {})...",
            self.reconnect_attempts
        ));
        self.messages_mut().add_message(msg.clone());
        self.chat_component
            .add_message(msg, &mut self.chat_state, &self.card_manager);

        // Attempt to reconnect
        if let Err(e) = self.connect_gateway() {
            error!(
                "Reconnect attempt {} failed: {}",
                self.reconnect_attempts, e
            );

            // Reset state after too many failures
            if self.reconnect_attempts >= 5 {
                let fail_msg = Message::system("Failed to reconnect after multiple attempts. Restart the TUI or press Ctrl+C to exit.");
                self.messages_mut().add_message(fail_msg.clone());
                self.chat_component
                    .add_message(fail_msg, &mut self.chat_state, &self.card_manager);
                self.reconnecting = false;
            }
            // Will try again on next loop iteration
        } else {
            info!(
                "Reconnect successful after {} attempt(s)",
                self.reconnect_attempts
            );
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

        // Tmux-style prefix key: Alt+A enters prefix mode, then a command key follows
        if key.modifiers.contains(KeyModifiers::ALT) && key.code == KeyCode::Char('a') {
            self.prefix_mode = true;
            return Ok(());
        }

        // Prefix mode: interpret the next key as a tmux command (one-shot, then exits)
        if self.prefix_mode {
            self.prefix_mode = false;
            match key.code {
                KeyCode::Esc => {} // cancel prefix, do nothing
                KeyCode::Char('?') => {
                    self.show_help = !self.show_help;
                }
                KeyCode::Char('1') => {
                    self.switch_view(ViewState::Dashboard);
                }
                KeyCode::Char('2') => {
                    self.switch_view(ViewState::Ide);
                }
                KeyCode::Char('3') => {
                    self.switch_view(ViewState::Kanban);
                }
                KeyCode::Char('4') => {
                    self.switch_view(ViewState::Chat);
                }
                // Tmux-style pane navigation: h/j/k/l
                KeyCode::Char('h') => {
                    self.focus_prev_pane();
                }
                KeyCode::Char('j') => {
                    self.focus_next_pane();
                }
                KeyCode::Char('k') => {
                    self.focus_prev_pane();
                }
                KeyCode::Char('l') => {
                    self.focus_next_pane();
                }
                // Tmux-style: r = reload config
                KeyCode::Char('r') => {
                    info!("Config reload requested");
                    let msg = Message::system("Config reload requested (r)");
                    self.messages_mut().add_message(msg.clone());
                    self.chat_component
                        .add_message(msg, &mut self.chat_state, &self.card_manager);
                    self.chat_component
                        .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                }
                // Tmux-style: R = rename current session
                KeyCode::Char('R') => {
                    if let Some(session) = self.sessions().current_session() {
                        info!("Rename session requested: {}", session.id);
                        let msg = Message::system("Rename: rename is not yet interactive in TUI");
                        self.messages_mut().add_message(msg.clone());
                        self.chat_component.add_message(
                            msg,
                            &mut self.chat_state,
                            &self.card_manager,
                        );
                        self.chat_component
                            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                    }
                }
                // Tmux-style: x = kill-pane / close pane
                KeyCode::Char('x') => {
                    info!("Kill pane requested (x)");
                }
                // Tmux-style: c = new-window / new view
                KeyCode::Char('c') => {
                    if let Err(e) = self.create_new_session() {
                        error!("Failed to create new session: {e}");
                    }
                }
                // Tmux-style: , = rename-window / rename view
                KeyCode::Char(',') => {
                    info!("Rename window requested (,)");
                    let msg = Message::system("Rename window: not yet interactive in TUI");
                    self.messages_mut().add_message(msg.clone());
                    self.chat_component
                        .add_message(msg, &mut self.chat_state, &self.card_manager);
                    self.chat_component
                        .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                }
                // Tmux-style: & = kill-window / kill view
                KeyCode::Char('&') => {
                    if let Some(session) = self.sessions().current_session() {
                        info!("Kill window (session): {}", session.id);
                        let msg = Message::system(format!(
                            "Killed view: {}",
                            session.name.as_deref().unwrap_or("Unnamed")
                        ));
                        self.messages_mut().add_message(msg.clone());
                        self.chat_component.add_message(
                            msg,
                            &mut self.chat_state,
                            &self.card_manager,
                        );
                        self.chat_component
                            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                        self.sessions_mut().clear_current_session();
                        self.current_input.clear();
                        self.input_composer.clear();
                    }
                }
                // Tmux-style: " = split-window / new session
                KeyCode::Char('\"') => {
                    if let Err(e) = self.create_new_session() {
                        error!("Failed to create new session: {e}");
                    }
                }
                _ => {} // unknown prefix command
            }
            return Ok(());
        }

        // If help is shown, Esc closes it (close help does not require prefix)
        if self.show_help {
            if key.code == KeyCode::Esc {
                self.show_help = false;
            }
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
                    let has_choices = prompt.choices().is_some();
                    match key.code {
                        KeyCode::Down => {
                            if has_choices {
                                prompt.next_choice();
                            }
                        }
                        KeyCode::Up => {
                            if has_choices {
                                prompt.prev_choice();
                            }
                        }
                        KeyCode::Char(c) => {
                            if !has_choices {
                                prompt.append_response(&c.to_string());
                            }
                        }
                        KeyCode::Backspace => {
                            if !has_choices {
                                let mut resp = prompt.response().to_string();
                                resp.pop();
                                prompt.set_response(resp);
                            }
                        }
                        KeyCode::Enter => {
                            let answer = self.prompt_manager.submit_clarify().unwrap_or_default();
                            if let Some(req_id) = self.pending_clarify_id.take() {
                                let response = TuiRequest::ClarifyRespond(ClarifyResponse {
                                    request_id: req_id,
                                    answer,
                                });
                                let _ = self.send_gateway_request(response);
                            }
                            self.input_composer_mut().set_active(true);
                            self.set_input_mode(InputMode::Insert);
                        }
                        KeyCode::Esc => {
                            self.prompt_manager.cancel_clarify();
                            if let Some(req_id) = self.pending_clarify_id.take() {
                                let response = TuiRequest::ClarifyRespond(ClarifyResponse {
                                    request_id: req_id,
                                    answer: String::new(),
                                });
                                let _ = self.send_gateway_request(response);
                            }
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
                            let value = prompt.submit();
                            if let Some(req_id) = self.pending_secret_id.take() {
                                let response = TuiRequest::SecretRespond(SecretResponse {
                                    request_id: req_id,
                                    value,
                                });
                                let _ = self.send_gateway_request(response);
                            } else if let Some(req_id) = self.pending_sudo_id.take() {
                                let response = TuiRequest::SudoRespond(SudoResponse {
                                    request_id: req_id,
                                    password: value,
                                });
                                let _ = self.send_gateway_request(response);
                            }
                            self.input_composer_mut().set_active(true);
                            self.set_input_mode(InputMode::Insert);
                        }
                        KeyCode::Esc => {
                            self.prompt_manager.cancel_secret();
                            if let Some(req_id) = self.pending_secret_id.take() {
                                let response = TuiRequest::SecretRespond(SecretResponse {
                                    request_id: req_id,
                                    value: String::new(),
                                });
                                let _ = self.send_gateway_request(response);
                            } else if let Some(req_id) = self.pending_sudo_id.take() {
                                let response = TuiRequest::SudoRespond(SudoResponse {
                                    request_id: req_id,
                                    password: String::new(),
                                });
                                let _ = self.send_gateway_request(response);
                            }
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
                        let title = if session.title.is_empty() {
                            "Unnamed".to_string()
                        } else {
                            session.title.clone()
                        };
                        let system_message = Message::system(format!("Resuming session: {title}"));
                        self.messages_mut().add_message(system_message.clone());
                        self.chat_component.add_message(
                            system_message,
                            &mut self.chat_state,
                            &self.card_manager,
                        );
                        self.chat_component
                            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                        self.session_picker.hide();
                    }
                }
                KeyCode::Esc => {
                    self.session_picker.hide();
                }
                KeyCode::Backspace => {
                    self.session_picker.pop_filter();
                }
                KeyCode::Char(c) => {
                    self.session_picker.append_filter(c);
                }
                _ => {}
            }
            return Ok(());
        }

        // If model picker is visible, route keys to it
        if self.model_picker.is_visible() {
            match key.code {
                KeyCode::Down => {
                    self.model_picker.select_next();
                }
                KeyCode::Up => {
                    self.model_picker.select_prev();
                }
                KeyCode::Enter => {
                    if self.model_picker.stage() == crate::ui::model_picker::ModelPickerStage::Model
                    {
                        let _ = self.apply_selected_model();
                    } else {
                        self.model_picker.enter_provider();
                    }
                }
                KeyCode::Esc => {
                    if self.model_picker.stage() == crate::ui::model_picker::ModelPickerStage::Model
                    {
                        self.model_picker.back_to_providers();
                    } else {
                        self.model_picker.hide();
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        // If completion popup is visible, route keys to it
        if self.completion_popup.is_visible() {
            match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    self.completion_popup.select_next();
                    return Ok(());
                }
                KeyCode::BackTab | KeyCode::Up => {
                    self.completion_popup.select_prev();
                    return Ok(());
                }
                KeyCode::Enter => {
                    if let Some(item) = self.completion_popup.selected_item().cloned() {
                        let replace_from = self.completion_popup.replace_from().unwrap_or(0);
                        let input = self.input_composer.get_input().to_string();

                        // In command mode, we want to strip the leading slash if the item has one
                        let mut text = item.text.clone();
                        if self.input_mode == InputMode::Command && text.starts_with('/') {
                            text.remove(0);
                        }

                        let new_input =
                            format!("{}{}", &input[..replace_from.min(input.len())], text);
                        self.input_composer.set_input(&new_input);
                        self.current_input = new_input;
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

        let KeyEvent {
            code, modifiers, ..
        } = key;

        // IDE view key handling
        if self.current_view == ViewState::Ide && self.input_mode == InputMode::Normal {
            match code {
                KeyCode::Tab => {
                    self.ide_state.focus_tree = !self.ide_state.focus_tree;
                    return Ok(());
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.ide_state.focus_tree {
                        self.ide_state.file_tree.next();
                    } else {
                        self.ide_state.editor.textarea.input(key);
                    }
                    return Ok(());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.ide_state.focus_tree {
                        self.ide_state.file_tree.previous();
                    } else {
                        self.ide_state.editor.textarea.input(key);
                    }
                    return Ok(());
                }
                KeyCode::Enter => {
                    if self.ide_state.focus_tree {
                        if let Some(path) = self.ide_state.file_tree.selected_path() {
                            if path.is_file() {
                                self.ide_state.editor.load_file(path);
                            }
                        }
                    }
                    return Ok(());
                }
                _ => {
                    if !self.ide_state.focus_tree {
                        self.ide_state.editor.textarea.input(key);
                        return Ok(());
                    }
                }
            }
        }

        // Normal mode navigation (only in Normal mode)
        if self.input_mode == InputMode::Normal {
            match code {
                KeyCode::Char('i') => {
                    self.set_input_mode(InputMode::Insert);
                    self.input_composer.set_active(true);
                    return Ok(());
                }
                // Alternative scroll keys (non-vim)
                KeyCode::Down => {
                    self.chat_component
                        .select_next(&mut self.chat_state, &self.card_manager);
                    self.chat_component
                        .ensure_selected_in_view(&mut self.chat_state, &self.card_manager);
                    return Ok(());
                }
                KeyCode::Up => {
                    self.chat_component
                        .select_prev(&mut self.chat_state, &self.card_manager);
                    self.chat_component
                        .ensure_selected_in_view(&mut self.chat_state, &self.card_manager);
                    return Ok(());
                }
                KeyCode::Enter => {
                    // Toggle expansion for selected message
                    if let Some(idx) = self.chat_component.get_selected_index(&self.chat_state) {
                        let msg_id_opt =
                            self.messages().get(idx).and_then(|m| m.message_id.clone());
                        if let Some(msg_id) = &msg_id_opt {
                            if !msg_id.starts_with("subagent:") {
                                if let Some(msg) = self.messages().get(idx) {
                                    if msg.role == MessageRole::System {
                                        self.chat_component
                                            .toggle_system_expanded(&mut self.chat_state, msg_id);
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
                    self.chat_component.scroll_up(&mut self.chat_state, 10);
                    return Ok(());
                }
                KeyCode::PageDown => {
                    self.chat_component
                        .scroll_down(&mut self.chat_state, 10, &self.card_manager);
                    return Ok(());
                }
                // Chat-mode message actions
                KeyCode::Char('y') => {
                    self.perform_chat_action(crate::ui::chat::ChatAction::YankToComposer)?;
                    return Ok(());
                }
                KeyCode::Char('c') => {
                    self.perform_chat_action(crate::ui::chat::ChatAction::Copy)?;
                    return Ok(());
                }
                KeyCode::Char('r') => {
                    self.perform_chat_action(crate::ui::chat::ChatAction::Regenerate)?;
                    return Ok(());
                }
                KeyCode::Char('G') => {
                    // Jump to bottom of chat (and clear the "↓ N new" pill)
                    self.chat_component
                        .jump_to_bottom(&mut self.chat_state, &self.card_manager);
                    return Ok(());
                }
                KeyCode::Char('?') => {
                    self.show_help = !self.show_help;
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
                self.chat_component
                    .select_prev(&mut self.chat_state, &self.card_manager);
                self.chat_component
                    .ensure_selected_in_view(&mut self.chat_state, &self.card_manager);
            }
            return Ok(());
        }

        // Check for Enter key in Insert or Command mode - submit prompt
        if (self.input_mode == InputMode::Insert || self.input_mode == InputMode::Command)
            && code == KeyCode::Enter
            && !modifiers.contains(KeyModifiers::SHIFT)
        {
            if !self.input_composer.get_input().is_empty() {
                self.submit_prompt()?;
                return Ok(());
            }
        }

        // Check for '/' key - switch to command mode
        if code == KeyCode::Char('/') && self.input_mode == InputMode::Normal {
            self.set_input_mode(InputMode::Command);
            self.input_composer.set_input_mode(InputMode::Command);
            self.input_composer.set_active(true);
            self.current_input = self.input_composer.get_input().to_string();
            self.input_mode = InputMode::Command;
            return Ok(());
        }

        // Check for Esc key - exit command mode
        if code == KeyCode::Esc && self.input_mode == InputMode::Command {
            self.set_input_mode(InputMode::Normal);
            self.input_composer.set_input_mode(InputMode::Normal);
            self.input_composer.clear();
            self.current_input = self.input_composer.get_input().to_string();
            self.input_mode = InputMode::Normal;
            return Ok(());
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

        // Check for Ctrl+M - show model picker
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('m') {
            self.show_model_picker()?;
            return Ok(());
        }

        // Check for Ctrl+K - kill current session (tmux-style)
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('k') {
            if let Some(session) = self.sessions().current_session() {
                info!("Killing session: {}", session.id);
                let msg = Message::system(format!(
                    "Killed session: {}",
                    session.name.as_deref().unwrap_or("Unnamed")
                ));
                self.messages_mut().add_message(msg.clone());
                self.chat_component
                    .add_message(msg, &mut self.chat_state, &self.card_manager);
                self.chat_component
                    .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                // Close the current session
                self.sessions_mut().clear_current_session();
                // Re-render
                self.current_input.clear();
                self.input_composer.clear();
            }
            return Ok(());
        }

        // Check for Ctrl+D - detach / close session (tmux-style)
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('d') {
            if let Some(session) = self.sessions().current_session() {
                info!("Detaching session: {}", session.id);
                let msg = Message::system(format!(
                    "Detached from session: {}",
                    session.name.as_deref().unwrap_or("Unnamed")
                ));
                self.messages_mut().add_message(msg.clone());
                self.chat_component
                    .add_message(msg, &mut self.chat_state, &self.card_manager);
                self.chat_component
                    .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                // Close the current session
                self.sessions_mut().clear_current_session();
                self.current_input.clear();
                self.input_composer.clear();
            }
            return Ok(());
        }

        // Check for Ctrl+A - attach / switch session (tmux-style: choose-session)
        if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('a') {
            self.list_sessions()?;
            return Ok(());
        }

        // Alt+Arrow pad for pane resize (tmux-style M-Left/Right/Up/Down)
        if modifiers.contains(KeyModifiers::ALT) {
            match code {
                KeyCode::Left | KeyCode::Char('h') => {
                    // Resize left analogue: focus left pane
                    self.focus_prev_pane();
                    return Ok(());
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    // Resize right analogue: focus right pane
                    self.focus_next_pane();
                    return Ok(());
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    // Resize up analogue: focus up
                    self.focus_prev_pane();
                    return Ok(());
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    // Resize down analogue: focus down
                    self.focus_next_pane();
                    return Ok(());
                }
                _ => {}
            }
        }

        // Pass key to input composer for handling (only in Insert/Command mode)
        if self.input_mode == InputMode::Insert || self.input_mode == InputMode::Command {
            // Handle Tab for completion
            if code == KeyCode::Tab {
                if self.completion_popup.is_visible() {
                    self.completion_popup.select_next();
                } else {
                    self.maybe_request_completion()?;
                }
                return Ok(());
            }
            if code == KeyCode::BackTab {
                if self.completion_popup.is_visible() {
                    self.completion_popup.select_prev();
                }
                return Ok(());
            }

            let handled = self.input_composer.handle_key_event(key);
            if handled {
                self.current_input = self.input_composer.get_input().to_string();
                self.input_mode = self.input_composer.input_mode();
                self.maybe_request_completion()?;
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
        if input.starts_with('/') || self.input_mode == InputMode::Command {
            // Strip leading slash if present for execute_slash_command
            let command = if input.starts_with('/') {
                input[1..].to_string()
            } else {
                input
            };
            self.execute_slash_command(command)?;
            self.set_input_mode(InputMode::Normal);
            return Ok(());
        }

        // Send the prompt to the gateway
        let message_content = input.clone();
        let session_id = self
            .sessions()
            .current_session()
            .map(|s| s.id.clone())
            .unwrap_or_default();

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
        self.chat_component
            .add_message(user_message, &mut self.chat_state, &self.card_manager);
        // Auto-scroll to bottom
        self.chat_component
            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);

        Ok(())
    }

    /// Perform a `ChatAction` on the currently-selected message.
    ///
    /// Returns `Ok(())` if the action was a no-op (no selection, out of bounds,
    /// etc.) so that keybindings can be wired safely.
    fn perform_chat_action(&mut self, action: ChatAction) -> Result<()> {
        let idx = match self.chat_component.get_selected_index(&self.chat_state) {
            Some(i) => i,
            None => return Ok(()),
        };
        let cmd = self.chat_component.perform_action(action, idx);
        self.dispatch_chat_command(cmd)
    }

    /// Translate a `ChatCommand` into the corresponding side effects:
    /// clipboard write, composer mutation, or session truncation + resubmit.
    fn dispatch_chat_command(&mut self, cmd: ChatCommand) -> Result<()> {
        match cmd {
            ChatCommand::Noop => Ok(()),
            ChatCommand::Yank(text) => {
                self.input_composer.set_input(&text);
                self.current_input = text;
                self.cursor_position = self.current_input.len();
                self.set_input_mode(InputMode::Insert);
                self.input_composer.set_active(true);
                Ok(())
            }
            ChatCommand::Copy(text) => {
                if let Err(e) = self.clipboard.set_text(&text) {
                    warn!("clipboard write failed: {e}");
                }
                Ok(())
            }
            ChatCommand::Edit {
                message_id,
                new_content,
            } => {
                let id = message_id.clone();
                if let Some(idx) = self.chat_component.index_of_message_id(&message_id) {
                    self.messages_mut().truncate_from(idx);
                } else {
                    warn!("edit: message {id} not found");
                    return Ok(());
                }
                self.sync_chat_with_messages();
                let _ = new_content; // Edit UI is a follow-up; for now we just drop the message.
                Ok(())
            }
            ChatCommand::Regenerate { from_index } => {
                // Find the preceding user message and resubmit its content.
                let prompt = self
                    .messages()
                    .range(0, from_index)
                    .iter()
                    .rev()
                    .find_map(|m| m.is_user().then(|| m.content().to_string()));
                let Some(prompt) = prompt else {
                    warn!("regenerate: no preceding user message at index {from_index}");
                    return Ok(());
                };
                self.messages_mut().truncate_from(from_index);
                self.sync_chat_with_messages();
                // Submit the same prompt again.
                self.resubmit_prompt(prompt)
            }
            ChatCommand::Delete { from_index } => {
                self.messages_mut().truncate_from(from_index);
                self.sync_chat_with_messages();
                Ok(())
            }
            ChatCommand::Branch { .. } => {
                // Branching creates a new session. Until the action menu
                // is wired up, we treat Branch as a no-op and log it.
                info!("branch requested (action menu not yet wired)");
                Ok(())
            }
        }
    }

    /// Re-submit an existing prompt (used by Regenerate).
    fn resubmit_prompt(&mut self, prompt: String) -> Result<()> {
        let session_id = self
            .sessions()
            .current_session()
            .map(|s| s.id.clone())
            .unwrap_or_default();
        let request = PromptSubmitRequest {
            session_id,
            text: prompt,
            images: None,
            truncate_before_user_ordinal: None,
        };
        self.thinking = true;
        self.send_gateway_request(TuiRequest::PromptSubmit(request))?;
        Ok(())
    }

    /// Execute a slash command
    fn execute_slash_command(&mut self, command: String) -> Result<()> {
        info!("Executing slash command: {command}");

        // Handle /model locally with the model picker overlay
        if command == "model" || command.starts_with("model ") {
            self.show_model_picker()?;
            return Ok(());
        }

        // Handle /clear locally
        if command == "clear" {
            self.messages_mut().clear();
            self.card_manager.clear();
            self.sync_chat_with_messages();
            return Ok(());
        }

        let command_clone = command.clone();
        let session_id = self
            .sessions()
            .current_session()
            .map(|s| s.id.clone())
            .unwrap_or_default();

        // Send slash.exec request to gateway
        let request = SlashExecRequest {
            session_id,
            command,
        };

        let tui_request = TuiRequest::SlashExec(request);
        self.send_gateway_request(tui_request)?;

        // Add the command to chat for visibility
        let user_message = Message::user(format!("/{command_clone}"));
        self.messages_mut().add_message(user_message.clone());
        self.chat_component
            .add_message(user_message, &mut self.chat_state, &self.card_manager);
        self.chat_component
            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);

        Ok(())
    }

    /// Create a new session
    fn create_new_session(&mut self) -> Result<()> {
        info!("Creating new session");

        // Send session.create request
        let request = SessionCreateRequest {
            model: None,    // Use default model
            provider: None, // Use default provider
            toolsets: None, // Use default toolsets
            skills: None,   // Use default skills
            worktree: None, // Don't create worktree
        };

        let tui_request = TuiRequest::SessionCreate(request);
        self.send_gateway_request(tui_request)?;

        // Show feedback to user
        let system_message = Message::system("Creating new session...");
        self.messages_mut().add_message(system_message.clone());
        self.chat_component
            .add_message(system_message, &mut self.chat_state, &self.card_manager);
        self.chat_component
            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);

        Ok(())
    }

    /// Resume the most recent session
    fn resume_session(&mut self) -> Result<()> {
        info!("Resuming session");

        // Get the most recent session ID first
        let session_id = self.sessions().most_recent_session().map(|s| s.id.clone());
        let session_name = self
            .sessions()
            .most_recent_session()
            .and_then(|s| s.name.clone());

        if let Some(session_id) = session_id {
            let request = SessionResumeRequest { session_id };

            let tui_request = TuiRequest::SessionResume(request);
            self.send_gateway_request(tui_request)?;

            // Show feedback to user
            let session_name_display = session_name.as_deref().unwrap_or("Unnamed");
            let system_message =
                Message::system(format!("Resuming session: {session_name_display}"));
            self.messages_mut().add_message(system_message.clone());
            self.chat_component.add_message(
                system_message,
                &mut self.chat_state,
                &self.card_manager,
            );
            self.chat_component
                .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
        } else {
            let system_message = Message::system("No previous sessions to resume");
            self.messages_mut().add_message(system_message.clone());
            self.chat_component.add_message(
                system_message,
                &mut self.chat_state,
                &self.card_manager,
            );
            self.chat_component
                .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
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
        self.chat_component
            .add_message(system_message, &mut self.chat_state, &self.card_manager);
        self.chat_component
            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);

        Ok(())
    }

    /// Check if a key event should quit the application
    fn check_quit_key(&mut self, key: KeyEvent) -> bool {
        let KeyEvent {
            code, modifiers, ..
        } = key;

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
            self.chat_component.scroll_up(&mut self.chat_state, 3);
        } else if mouse.kind == MouseEventKind::ScrollDown {
            self.chat_component
                .scroll_down(&mut self.chat_state, 3, &self.card_manager);
        }
        Ok(())
    }

    /// Handle a terminal resize event
    fn handle_resize(&mut self, width: u16, height: u16) -> Result<()> {
        execute!(io::stdout(), crossterm::terminal::SetSize(width, height))?;
        let _ = self.send_gateway_request(TuiRequest::TerminalResize {
            cols: width,
            rows: height,
        });
        Ok(())
    }

    /// Handle a paste event
    fn handle_paste(&mut self, text: &str) -> Result<()> {
        self.input_composer.insert_text(text);
        self.current_input = self.input_composer.get_input().to_string();
        self.maybe_request_completion()?;
        Ok(())
    }

    /// Handle a message from the gateway
    fn handle_gateway_message(&mut self, message: GatewayMessage) -> Result<()> {
        debug!("Received gateway message: {message:?}");

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
                debug!("Status update: {status:?}");
            }
            GatewayMessage::ReasoningAvailable(reasoning) => {
                debug!("Reasoning available: {reasoning:?}");
                self.append_streaming_reasoning(&reasoning);
            }
            GatewayMessage::ReasoningDelta(delta) => {
                debug!("Reasoning delta: {delta:?}");
                self.append_streaming_reasoning(&delta);
            }
            GatewayMessage::MessageStart(start) => {
                debug!("Message start: {start:?}");
                // A new assistant turn is starting. Clear any leftover
                // streaming-reasoning accumulator from the previous turn.
                self.streaming_reasoning.clear();
                self.streaming_message_id = None;
            }
            GatewayMessage::ThinkingDelta(delta) => {
                debug!("Thinking delta: {delta:?}");
                self.append_streaming_reasoning(&delta);
            }
            GatewayMessage::NoticeUpsert(notice) => {
                debug!("Notice upsert: {notice:?}");
            }
            GatewayMessage::NoticeClear(notice) => {
                debug!("Notice clear: {notice:?}");
            }
            GatewayMessage::NotificationShow(notice) => {
                debug!("Notification show: {notice:?}");
            }
            GatewayMessage::NotificationClear(notice) => {
                debug!("Notification clear: {notice:?}");
            }
            GatewayMessage::PreviewRestartProgress(progress) => {
                debug!("Preview restart progress: {progress:?}");
            }
            GatewayMessage::PreviewRestartComplete(complete) => {
                debug!("Preview restart complete: {complete:?}");
            }
            GatewayMessage::VoiceTranscript(transcript) => {
                debug!("Voice transcript: {transcript:?}");
            }
            GatewayMessage::VoiceStatus(status) => {
                debug!("Voice status: {status:?}");
            }
            GatewayMessage::BrowserProgress(progress) => {
                debug!("Browser progress: {progress:?}");
            }
            GatewayMessage::SkinChanged(skin) => {
                debug!("Skin changed: {skin:?}");
            }
            GatewayMessage::BackgroundComplete(complete) => {
                debug!("Background complete: {complete:?}");
            }
            GatewayMessage::ReviewSummary(summary) => {
                debug!("Review summary: {summary:?}");
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
                debug!("Tool generating: {generating:?}");
            }

            // Approvals
            GatewayMessage::ApprovalRequest(request) => {
                self.handle_approval_request(request)?;
            }
            GatewayMessage::ClarifyRequest(request) => {
                self.handle_clarify_request(request)?;
            }
            GatewayMessage::SudoRequest(request) => {
                self.handle_sudo_request(request)?;
            }
            GatewayMessage::SecretRequest(request) => {
                self.handle_secret_request(request)?;
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
            GatewayMessage::ModelOptions(response) => {
                self.handle_model_options(response)?;
            }

            // Unhandled message types
            _ => {
                debug!("Unhandled gateway message type");
            }
        }

        Ok(())
    }

    /// Handle gateway ready message (first message after connection)
    fn handle_gateway_ready(&mut self, ready: GatewayReadyResponse) -> Result<()> {
        info!(
            "Gateway ready: {} sessions, {} models",
            ready.sessions.as_ref().map_or(0, std::vec::Vec::len),
            ready.models.as_ref().map_or(0, std::vec::Vec::len)
        );

        // Populate sessions in manager
        if let Some(sessions) = &ready.sessions {
            self.sessions_mut().set_sessions(sessions.clone());
        }

        // Populate capabilities for the empty-state landing page.
        if let Some(caps) = &ready.capabilities {
            self.capabilities = Capabilities {
                tool_count: caps.tool_count.unwrap_or(0),
                skill_count: caps.skill_count.unwrap_or(0),
                mcp_servers: caps.mcp_servers.clone().unwrap_or_default(),
                ..self.capabilities.clone()
            };
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
        debug!("Gateway activity: {activity}");
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

        // Extract model info from session
        if let Some(ref info) = response.info {
            if let Some(ref model) = info.model {
                self.current_model = Some(model.clone());
            }
        }

        // Extract provider info from the local session object
        if let Some(session) = self.sessions().current_session() {
            if let Some(ref provider) = session.provider {
                self.current_provider = Some(provider.clone());
            }
        }

        // Clear message history for new session
        self.messages_mut().clear();
        self.chat_component.clear_messages(&mut self.chat_state);

        Ok(())
    }

    /// Handle session resume response
    fn handle_session_resume(&mut self, response: SessionResumeResponse) -> Result<()> {
        info!("Session resumed: {}", response.session_id);

        // Ensure session exists in local store
        let session_id = response.session_id.clone();
        if self.sessions().get_session(&session_id).is_none() {
            self.sessions_mut()
                .add_session(Session::new(session_id.clone()));
        }

        // Set the resumed session as current
        self.sessions_mut().set_current_session(session_id);

        // Extract model info from session
        if let Some(ref info) = response.info {
            if let Some(ref model) = info.model {
                self.current_model = Some(model.clone());
            }
        }

        // Extract provider info from the local session object
        if let Some(session) = self.sessions().current_session() {
            if let Some(ref provider_name) = session.provider {
                self.current_provider = Some(provider_name.clone());
            }
        }

        // Clear existing messages
        self.messages_mut().clear();
        self.chat_component.clear_messages(&mut self.chat_state);

        // Add resumed messages to chat
        if let Some(messages) = response.messages {
            for msg in messages {
                let message = Message::new(msg.role, msg.text.unwrap_or_default());
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
        debug!(
            "Tool started: {} ({})",
            tool_start.tool_name, tool_start.call_id
        );

        self.thinking = true;

        // Create a tool card for this tool call with proper call_id
        let data = ToolCardData::running(&tool_start.tool_name)
            .with_call_id(&tool_start.call_id)
            .with_arguments(tool_start.arguments.clone().unwrap_or_default());
        self.card_manager_mut().add_tool_card(data);

        // Show in chat with call_id as message_id for in-place updates later
        let mut message = Message::tool(
            format!("Tool '{}' started...", tool_start.tool_name),
            tool_start.tool_name,
        );
        message.message_id = Some(tool_start.call_id.clone());
        self.messages_mut().add_message(message.clone());
        self.chat_component
            .add_message(message, &mut self.chat_state, &self.card_manager);

        Ok(())
    }

    /// Handle tool progress
    fn handle_tool_progress(&mut self, tool_progress: ToolProgress) -> Result<()> {
        debug!(
            "Tool progress: {} -> {}",
            tool_progress.call_id, tool_progress.output
        );

        // Update card_manager with progress output via call_id
        if let Some(card) = self
            .card_manager_mut()
            .find_by_call_id_mut(&tool_progress.call_id)
        {
            let data = ToolCardData {
                tool_name: card.tool_name().to_string(),
                call_id: tool_progress.call_id.clone(),
                status: ToolStatus::Running,
                duration_ms: None,
                arguments: card.arguments().cloned(), // Preserve arguments
                result: Some(tool_progress.output.clone()),
                error: None,
            };
            card.update_from_data(&data);
        }

        Ok(())
    }

    /// Handle tool complete
    fn handle_tool_complete(&mut self, tool_complete: ToolComplete) -> Result<()> {
        debug!(
            "Tool completed: {} ({}ms)",
            tool_complete.call_id,
            tool_complete.duration_ms.unwrap_or(0)
        );

        self.thinking = false;

        // Convert result to a human-readable string without excessive JSON escaping for normal strings
        let result_str = if tool_complete.result.is_null() {
            "No result".to_string()
        } else if let Some(s) = tool_complete.result.as_str() {
            s.to_string() // Prevents string values from being rendered with literal `\n` instead of actual newlines
        } else {
            serde_json::to_string_pretty(&tool_complete.result)
                .unwrap_or_else(|_| "Error serializing result".to_string())
        };

        // Update card manager with result via call_id
        let status = if tool_complete.error.is_some() {
            ToolStatus::Failed
        } else {
            ToolStatus::Completed
        };

        if let Some(card) = self
            .card_manager_mut()
            .find_by_call_id_mut(&tool_complete.call_id)
        {
            let data = ToolCardData {
                tool_name: card.tool_name().to_string(),
                call_id: tool_complete.call_id.clone(),
                status,
                duration_ms: tool_complete.duration_ms,
                arguments: card.arguments().cloned(), // Preserve arguments
                result: Some(result_str.clone()),
                error: tool_complete.error.clone(),
            };
            card.update_from_data(&data);
        }
        // Update tool message in-place using message_id (set in handle_tool_start)
        let result_text = if let Some(ref error) = tool_complete.error {
            format!("Error: {error}")
        } else {
            result_str
        };
        let duration_str = tool_complete
            .duration_ms
            .map_or(String::new(), |ms| format!(" ({ms}ms)"));
        let content = format!("Tool completed: {result_text}{duration_str}");

        if let Some(updated_msg) = self
            .messages_mut()
            .update_message_by_id(&tool_complete.call_id, content.clone())
        {
            self.chat_component.update_message(updated_msg);
        } else {
            // Fallback: starting message was missed, add a new one
            let message = Message::tool(
                content,
                format!(
                    "{} ({:?}ms)",
                    tool_complete.call_id, tool_complete.duration_ms
                ),
            );
            self.messages_mut().add_message(message.clone());
            self.chat_component
                .add_message(message, &mut self.chat_state, &self.card_manager);
        }
        Ok(())
    }

    fn handle_session_activate(&mut self, response: SessionActivateResponse) -> Result<()> {
        info!("Session activated: {}", response.session_id);

        // Ensure session exists in local store
        let session_id = response.session_id.clone();
        if self.sessions().get_session(&session_id).is_none() {
            self.sessions_mut()
                .add_session(Session::new(session_id.clone()));
        }

        self.sessions_mut().set_current_session(session_id);

        // Add activated messages to chat
        if let Some(messages) = response.messages {
            for msg in messages {
                let message = Message::new(msg.role, msg.text.unwrap_or_default());
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
        info!(
            "Session list received with {} sessions",
            response.sessions.as_ref().map_or(0, std::vec::Vec::len)
        );
        if let Some(sessions) = response.sessions {
            self.sessions_mut().set_sessions(sessions.clone());
            self.session_picker.show(sessions);
        }
        Ok(())
    }
    /// Handle session info
    fn handle_session_info(&mut self, info_val: serde_json::Value) -> Result<()> {
        if let Some(model) = info_val.get("model").and_then(|v| v.as_str()) {
            debug!("Session info: model={model}");
        }
        if let Some(provider) = info_val.get("provider").and_then(|v| v.as_str()) {
            debug!("Session info: provider={provider}");
        }
        Ok(())
    }

    fn handle_message_delta(&mut self, delta: MessageDelta) -> Result<()> {
        debug!("Message delta: text='{}'", delta.text);
        let session_id = delta.session_id.clone().unwrap_or_default();
        let message_id = format!("{session_id}:streaming");
        // Track the streaming message_id for reasoning updates and reset the
        // reasoning accumulator when a new assistant turn begins.
        if self.streaming_message_id.as_deref() != Some(message_id.as_str()) {
            self.streaming_reasoning.clear();
            self.streaming_message_id = Some(message_id.clone());
        }
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
        let mut message =
            Message::streaming_delta(MessageRole::Assistant, delta.text, Some(message_id));
        message.complete = false;
        let message_clone = message.clone();
        self.messages_mut().add_message(message);
        self.chat_component
            .add_message(message_clone, &mut self.chat_state, &self.card_manager);
        Ok(())
    }

    fn handle_message_complete(&mut self, complete: MessageComplete) -> Result<()> {
        debug!("Message complete: text length={}", complete.text.len());
        self.thinking = false;

        let mut message = Message::new(MessageRole::Assistant, complete.text);
        message.usage = complete.usage.clone();
        // Prefer the static reasoning field supplied by the gateway's
        // MessageComplete event. Fall back to whatever the streaming
        // accumulator collected (defensive — this should match in practice).
        if complete.reasoning.is_some() {
            message.reasoning = complete.reasoning;
        } else if !self.streaming_reasoning.is_empty() {
            message.reasoning = Some(self.streaming_reasoning.clone());
        }
        message.warning = complete.warning;

        // Update session usage stats
        if let Some(ref usage) = complete.usage {
            if let Some(session) = self.sessions_mut().current_session_mut() {
                session.add_usage(usage);
            }
        }

        self.messages_mut().add_message(message.clone());
        self.chat_component
            .add_message(message, &mut self.chat_state, &self.card_manager);
        self.chat_component
            .scroll_to_bottom(&mut self.chat_state, &self.card_manager);

        // Turn is finished — clear the streaming-reasoning accumulator and
        // forget the streaming message marker so the next turn starts fresh.
        self.streaming_reasoning.clear();
        self.streaming_message_id = None;
        Ok(())
    }

    /// Extract a reasoning text payload from a `serde_json::Value` produced by
    /// the gateway's ReasoningAvailable / ReasoningDelta / ThinkingDelta
    /// events. Tries common field names (`text`, `delta`, `reasoning`,
    /// `content`) and gracefully handles string-typed values.
    fn extract_reasoning_text(value: &serde_json::Value) -> Option<String> {
        if let Some(s) = value.as_str() {
            return Some(s.to_string());
        }
        if let Some(obj) = value.as_object() {
            for key in ["text", "delta", "reasoning", "content", "value"] {
                if let Some(v) = obj.get(key) {
                    if let Some(s) = v.as_str() {
                        return Some(s.to_string());
                    }
                }
            }
        }
        None
    }

    /// Append a reasoning fragment (from a ReasoningAvailable /
    /// ReasoningDelta / ThinkingDelta gateway event) to the accumulator and
    /// propagate the running total into the currently-streaming assistant
    /// message so the renderer sees it on the next draw.
    fn append_streaming_reasoning(&mut self, value: &serde_json::Value) {
        let Some(text) = Self::extract_reasoning_text(value) else {
            return;
        };
        if text.is_empty() {
            return;
        }
        self.streaming_reasoning.push_str(&text);
        // Reflect the running reasoning total into the live streaming
        // assistant message, both in the canonical `messages` history and
        // in `chat_component.messages` (which the renderer iterates).
        let message_id = self
            .streaming_message_id
            .clone()
            .or_else(|| Some(String::new()));
        let Some(message_id) = message_id else {
            return;
        };
        if message_id.is_empty() {
            return;
        }
        let reasoning_snapshot = self.streaming_reasoning.clone();
        // 1) Update the messages history.
        if let Some(idx) = self
            .messages()
            .to_vec()
            .iter()
            .position(|m| m.message_id.as_deref() == Some(message_id.as_str()))
        {
            if let Some(msg_mut) = self.messages_mut().get_mut(idx) {
                msg_mut.reasoning = Some(reasoning_snapshot.clone());
            }
        }
        // 2) Update the chat component's message list (renderer source).
        //    We clone the existing message and patch `reasoning`, then call
        //    `update_message` which is the public mutator on ChatComponent.
        if let Some(existing) = self
            .chat_component
            .messages()
            .iter()
            .find(|m| m.message_id.as_deref() == Some(message_id.as_str()))
            .cloned()
        {
            let mut updated = existing;
            updated.reasoning = Some(reasoning_snapshot);
            self.chat_component.update_message(updated);
        }
    }
    fn handle_approval_request(&mut self, request: ApprovalRequest) -> Result<()> {
        info!(
            "Approval request: {} - {}",
            request.tool_name, request.message
        );

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

    fn handle_clarify_request(&mut self, request: ClarifyRequest) -> Result<()> {
        info!("Clarify request: {}", request.question);
        self.pending_clarify_id = Some(request.request_id.clone());
        self.prompt_manager
            .show_clarify(request.question, request.choices);
        self.input_composer_mut().set_active(false);
        self.set_input_mode(InputMode::Normal);
        Ok(())
    }

    fn handle_sudo_request(&mut self, request: SudoRequest) -> Result<()> {
        info!("Sudo request: {}", request.request_id);
        self.pending_sudo_id = Some(request.request_id.clone());
        self.prompt_manager
            .show_secret("System (sudo) password required:");
        self.input_composer_mut().set_active(false);
        self.set_input_mode(InputMode::Normal);
        Ok(())
    }

    fn handle_secret_request(&mut self, request: SecretRequest) -> Result<()> {
        info!(
            "Secret request: {} for {}",
            request.request_id, request.env_var
        );
        self.pending_secret_id = Some(request.request_id.clone());
        self.prompt_manager.show_secret(request.prompt);
        self.input_composer_mut().set_active(false);
        self.set_input_mode(InputMode::Normal);
        Ok(())
    }

    /// Handle slash completion
    fn handle_slash_completion(&mut self, completion: CompletionResponse) -> Result<()> {
        debug!("Slash completion: {:?}", completion.items);
        if let Some(items) = completion.items {
            if !items.is_empty() {
                // In Command mode, the input buffer doesn't have the leading slash.
                // If the gateway returned a replace_from, we might need to adjust it.
                let replace_from = if self.input_mode == InputMode::Command {
                    completion
                        .replace_from
                        .map(|rf| rf.saturating_sub(1))
                        .unwrap_or(0)
                } else {
                    completion.replace_from.unwrap_or(1)
                };
                self.completion_popup.show(items, Some(replace_from));
            } else {
                self.completion_popup.hide();
            }
        }
        Ok(())
    }

    /// Handle path completion
    fn handle_path_completion(&mut self, completion: CompletionResponse) -> Result<()> {
        debug!("Path completion: {:?}", completion.items);
        if let Some(items) = completion.items {
            self.completion_popup.show(items, completion.replace_from);
        }
        Ok(())
    }

    /// Trigger completion requests when the input matches slash/path prefixes.
    fn maybe_request_completion(&mut self) -> Result<()> {
        let input = self.input_composer.get_input().to_string();

        // In command mode, the input doesn't have a leading slash, but the completion logic
        // and the gateway expect one.
        let query = if self.input_mode == InputMode::Command {
            format!("/{}", input)
        } else {
            input
        };

        // Deduplicate requests to avoid overwhelming the gateway
        if self.last_completion_query.as_deref() == Some(&query)
            && self.completion_popup.is_visible()
        {
            return Ok(());
        }
        self.last_completion_query = Some(query.clone());

        if let Some(req) = crate::utils::text::completion_request_for_input(&query) {
            let tui_req = match req.method {
                "complete.slash" => TuiRequest::CompleteSlash { text: req.query },
                "complete.path" => TuiRequest::CompletePath { word: req.query },
                _ => return Ok(()),
            };
            self.send_gateway_request(tui_req)?;
        } else if self.completion_popup.is_visible() {
            self.completion_popup.hide();
            self.last_completion_query = None;
        }
        Ok(())
    }

    /// Show the model picker and request provider/model list from gateway
    fn show_model_picker(&mut self) -> Result<()> {
        info!("Showing model picker");
        self.model_picker.show_loading();
        let _session_id = self
            .sessions()
            .current_session()
            .map(|s| s.id.clone())
            .unwrap_or_default();
        self.send_gateway_request(TuiRequest::ModelOptions)?;
        Ok(())
    }

    /// Handle model options response
    fn handle_model_options(&mut self, response: ModelOptionsResponse) -> Result<()> {
        debug!("Received model options: {:?}", response);
        if let Some(providers) = response.providers {
            if !providers.is_empty() {
                info!("Updating model picker with {} providers", providers.len());
                self.model_picker.show(providers);
            } else {
                warn!("Received empty providers list from gateway");
                self.model_picker.hide();
            }
        } else {
            warn!("Received missing providers list from gateway");
            self.model_picker.hide();
        }
        Ok(())
    }

    /// Apply selected model from picker
    fn apply_selected_model(&mut self) -> Result<()> {
        if let Some(model) = self.model_picker.selected_model() {
            if model.is_empty() {
                warn!("Model picker returned an empty model name");
                return Ok(());
            }
            info!("Switching model to: {model}");
            match self.send_gateway_request(TuiRequest::ConfigSet {
                key: "model".to_string(),
                value: model.clone(),
            }) {
                Ok(_) => {
                    let system_message = Message::system(format!("Model switched to: {model}"));
                    self.messages_mut().add_message(system_message.clone());
                    self.chat_component.add_message(
                        system_message,
                        &mut self.chat_state,
                        &self.card_manager,
                    );
                    self.chat_component
                        .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
                    self.model_picker.hide();
                }
                Err(e) => {
                    error!("Failed to send config.set request for model switch: {e}");
                    let error_message = Message::error(format!("Failed to switch model: {e}"));
                    self.messages_mut().add_message(error_message.clone());
                    self.chat_component.add_message(
                        error_message,
                        &mut self.chat_state,
                        &self.card_manager,
                    );
                }
            }
        }
        Ok(())
    }

    /// Handle slash exec response
    fn handle_slash_exec(&mut self, response: SlashExecResponse) -> Result<()> {
        if let Some(output) = response.output {
            info!("Slash command output: {output}");

            let message = Message::system(output);
            self.messages_mut().add_message(message.clone());
            self.chat_component
                .add_message(message, &mut self.chat_state, &self.card_manager);
            self.chat_component
                .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
        }
        if let Some(warning) = response.warning {
            log::warn!("Slash command warning: {warning}");

            let message = Message::system(format!("Warning: {warning}"));
            self.messages_mut().add_message(message.clone());
            self.chat_component
                .add_message(message, &mut self.chat_state, &self.card_manager);
            self.chat_component
                .scroll_to_bottom(&mut self.chat_state, &self.card_manager);
        }
        Ok(())
    }

    /// Refresh system telemetry stats
    fn refresh_system_stats(&mut self) {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        // Calculate CPU usage as global average
        self.cpu_usage = self.sys.global_cpu_usage();
        self.cpu_history.push(self.cpu_usage as u64);
        if self.cpu_history.len() > 1000 {
            self.cpu_history.remove(0);
        }

        // Calculate memory usage percentage
        let used = self.sys.used_memory();
        let total = self.sys.total_memory();
        if total > 0 {
            self.memory_usage = (used as f32 / total as f32) * 100.0;
            self.memory_history.push(self.memory_usage as u64);
            if self.memory_history.len() > 1000 {
                self.memory_history.remove(0);
            }
        }

        // Update token speed history (mock for now)
        let speed = if self.thinking {
            // Random-ish speed when thinking
            (self.animation_frame % 50) + 20
        } else {
            0
        };
        self.token_speed_history.push(speed);
        if self.token_speed_history.len() > 1000 {
            self.token_speed_history.remove(0);
        }
    }

    /// Switch to a new view with transition animation
    fn switch_view(&mut self, new_view: ViewState) {
        if self.current_view != new_view {
            self.previous_view = Some(self.current_view);
            self.current_view = new_view;
            self.transition_progress = 0.0;
            crate::engine::animation_start();
            info!("Switched view to {:?}", new_view);
        }
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
            log::error!("Details: {details}");
        }

        // Hide overlays that might be waiting for a response
        if self.model_picker.is_visible() {
            self.model_picker.hide();
        }

        Ok(())
    }

    /// Sync chat component with message history
    fn sync_chat_with_messages(&mut self) {
        let messages = self.messages().all_messages().clone();
        let state = &mut self.chat_state;
        self.chat_component
            .set_messages(messages, state, &self.card_manager);
    }

    /// Draw the UI
    pub fn draw(&mut self) -> Result<()> {
        // Update toolbar state before drawing
        let connected = self.is_gateway_connected();
        let model = self
            .current_model
            .as_deref()
            .unwrap_or("unknown")
            .to_string();
        let provider = self.current_provider.clone();
        let (session_name, msg_count) = {
            let session_opt = self.sessions().current_session();
            match session_opt {
                Some(s) => {
                    let name = s.name.clone().unwrap_or_else(|| "Unnamed".to_string());
                    let count = s.message_count();
                    (Some(name), count)
                }
                None => (None, 0),
            }
        };

        self.toolbar.update_status(
            connected,
            Some(&model),
            provider.as_deref(),
            session_name.as_deref(),
            msg_count,
        );

        // Pre-compute layout values to avoid borrow conflicts inside the closure
        let hashline_block = self.message_history.last().and_then(|m| {
            if m.is_edit_tool_message() {
                crate::state::hashline::HashlineParser::parse(&m.content)
            } else {
                None
            }
        });
        let has_hashline = hashline_block.is_some();
        let target_height = if has_hashline { 8 } else { 0 };
        // Smooth interpolation: ease-out to target height
        self.actual_activity_height += (target_height as f32 - self.actual_activity_height) * 0.3;
        let activity_height = self.actual_activity_height.round() as u16;
        let n_sessions = self.sessions().len() as u16;
        let session_manager_ptr: *const crate::state::session::SessionManager = self.sessions();
        let current_session_id = self.sessions().current_session_id().cloned();

        // Destructure self to allow disjoint borrows of its fields inside the closure
        let App {
            ref mut terminal,
            ref config,
            ref banner,
            ref card_manager,
            ref hashline_viewer,
            ref input_composer,
            ref toolbar,
            ref session_picker,
            ref mut model_picker,
            ref prompt_manager,
            ref mut completion_popup,
            ref subagent_list,
            ref mut chat_component,
            ref mut chat_state,
            ref mut ide_state,
            current_view,
            previous_view: _,
            ref mut transition_progress,
            cpu_usage,
            memory_usage,
            focus_pane,
            animation_frame,
            ref wave_ticker,
            ..
        } = *self;

        // Advance transition progress
        if *transition_progress < 1.0 {
            *transition_progress += 0.33; // ~3 frames for total transition
            if *transition_progress >= 1.0 {
                *transition_progress = 1.0;
                crate::engine::animation_end();
            }
        }

        let wave_tick = wave_ticker.current_tick();
        let wave_active = wave_ticker.is_active();

        crate::engine::draw_sync(terminal, |frame| {
            use ratatui::layout::Alignment;
            use ratatui::style::{Color, Modifier, Style};
            use ratatui::text::{Line, Span};
            use ratatui::widgets::Paragraph;

            let area = frame.area();
            if area.width < 20 || area.height < 10 {
                return;
            }

            // ── Top tab bar (1 line) ──
            let tab_bar_height = 1u16;
            let tab_content_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(tab_bar_height), Constraint::Min(1)])
                .split(area);
            let tab_area = tab_content_chunks[0];
            let content_area = tab_content_chunks[1];

            let tabs = [
                ("[1] DASHBOARD", ViewState::Dashboard),
                ("[2] IDE", ViewState::Ide),
                ("[3] KANBAN", ViewState::Kanban),
                ("[4] CHAT", ViewState::Chat),
            ];
            let mut tab_spans = Vec::new();
            for (label, view) in &tabs {
                let is_active = *view == current_view;
                let style = if is_active {
                    Style::default()
                        .fg(Color::Rgb(40, 40, 40))
                        .bg(Color::Rgb(250, 189, 47))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Rgb(146, 131, 116))
                        .bg(Color::Rgb(27, 32, 33))
                };
                tab_spans.push(Span::styled(format!(" {label} "), style));
            }
            let tab_para = Paragraph::new(Line::from(tab_spans))
                .alignment(Alignment::Left)
                .style(Style::default().bg(Color::Rgb(27, 32, 33)));
            frame.render_widget(tab_para, tab_area);

            // ── Sidebar for Chat view ──
            let show_sidebar = content_area.width >= 80 && current_view == ViewState::Chat;
            let (main_area, sidebar_area) = if show_sidebar {
                let horiz = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Min(1), Constraint::Length(24)])
                    .split(content_area);
                (horiz[0], horiz[1])
            } else {
                (content_area, Rect::default())
            };

            // ── View-dependent rendering ──
            match current_view {
                ViewState::Dashboard => {
                    crate::ui::dashboard::DashboardView::render(
                        frame,
                        main_area,
                        &config.theme.colors,
                        animation_frame,
                        self.thinking,
                        cpu_usage,
                        memory_usage,
                        &self.cpu_history,
                        &self.memory_history,
                        &self.token_speed_history,
                    );
                }
                ViewState::Ide => {
                    // IDE: content + composer + toolbar
                    let ide_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(1),
                            Constraint::Length(3),
                            Constraint::Length(1),
                        ])
                        .split(main_area);

                    crate::ui::ide::IdeView::render(
                        frame,
                        ide_chunks[0],
                        &config.theme.colors,
                        chat_component,
                        chat_state,
                        ide_state,
                        connected,
                        card_manager,
                        subagent_list,
                        animation_frame,
                        self.thinking,
                    );

                    input_composer.render_clean(frame, ide_chunks[1]);
                    toolbar.render(frame, ide_chunks[2]);
                }
                ViewState::Kanban => {
                    // Kanban: content + composer + toolbar
                    let kanban_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Min(1),
                            Constraint::Length(3),
                            Constraint::Length(1),
                        ])
                        .split(main_area);

                    crate::ui::kanban::KanbanView::render(
                        frame,
                        kanban_chunks[0],
                        &config.theme.colors,
                        animation_frame,
                        self.thinking,
                    );

                    input_composer.render_clean(frame, kanban_chunks[1]);
                    toolbar.render(frame, kanban_chunks[2]);
                }
                ViewState::Chat => {
                    // Chat view with animated focus-pane borders
                    let banner_height = if main_area.height > 40 { 7 } else { 1 };
                    let wave_height: u16 = if wave_active { 2 } else { 0 };

                    let main_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([
                            Constraint::Length(banner_height),
                            Constraint::Min(1),
                            Constraint::Length(activity_height),
                            Constraint::Length(5),
                            Constraint::Length(wave_height),
                            Constraint::Length(3),
                        ])
                        .split(main_area);

                    // Banner
                    if banner_height > 1 {
                        banner.render(frame, main_layout[0]);
                    } else {
                        banner.render_mini(frame, main_layout[0]);
                    }

                    // Chat with animated border
                    let chat_area = main_layout[1];
                    let chat_block = Block::bordered();
                    let chat_inner = chat_block.inner(chat_area);
                    frame.render_widget(chat_block, chat_area);
                    crate::ui::borders::render_gradient_border(
                        frame.buffer_mut(),
                        chat_area,
                        animation_frame,
                        focus_pane == FocusPane::Chat,
                        self.thinking,
                    );
                    chat_state.visible_height = chat_inner.height.saturating_sub(2);
                    chat_component.set_show_logo_on_empty(true);
                    chat_component.set_capabilities(self.capabilities.clone());
                    chat_component.render(
                        frame,
                        chat_inner,
                        chat_state,
                        card_manager,
                        subagent_list,
                        connected,
                        animation_frame,
                    );

                    // Activity (hashline only - cards are drawn inline in chat)
                    if activity_height > 0 {
                        let activity_area = main_layout[2];
                        if let Some(ref block) = hashline_block {
                            hashline_viewer.render(block, activity_area, frame);
                        }
                    }
                    // Composer with animated border
                    let composer_block = Block::bordered();
                    let composer_inner = composer_block.inner(main_layout[3]);
                    frame.render_widget(composer_block, main_layout[3]);
                    crate::ui::borders::render_gradient_border(
                        frame.buffer_mut(),
                        main_layout[3],
                        animation_frame,
                        focus_pane == FocusPane::Composer,
                        self.thinking,
                    );
                    input_composer.render_inner(frame, composer_inner);

                    // Toolbar with animated border
                    let toolbar_block = Block::bordered();
                    let toolbar_inner = toolbar_block.inner(main_layout[5]);
                    frame.render_widget(toolbar_block, main_layout[5]);
                    crate::ui::borders::render_gradient_border(
                        frame.buffer_mut(),
                        main_layout[5],
                        animation_frame,
                        focus_pane == FocusPane::Toolbar,
                        self.thinking,
                    );
                    toolbar.render(frame, toolbar_inner);

                    // Sine-wave loading footer (Phase 4 — Aetheric Shaders)
                    if wave_active {
                        let session_manager = unsafe { &*session_manager_ptr };
                        let usage = if let Some(session) = current_session_id
                            .as_ref()
                            .and_then(|id| session_manager.get_session(id))
                        {
                            let u = &session.total_usage;
                            (
                                u.prompt_category_tokens.unwrap_or(0),
                                u.tool_call_tokens.unwrap_or(0),
                                u.reasoning_tokens.unwrap_or(0),
                                u.output_tokens.unwrap_or(0),
                                u.failed_tool_call_tokens.unwrap_or(0),
                            )
                        } else {
                            (0, 0, 0, 0, 0)
                        };
                        crate::ui::wave::render_wave_footer(
                            frame,
                            main_layout[4],
                            wave_tick,
                            usage,
                        );
                    }

                    // Sidebar with animated border
                    if show_sidebar && sidebar_area.width > 0 {
                        let sidebar_block = Block::bordered();
                        let sidebar_inner = sidebar_block.inner(sidebar_area);
                        frame.render_widget(sidebar_block, sidebar_area);
                        crate::ui::borders::render_gradient_border(
                            frame.buffer_mut(),
                            sidebar_area,
                            animation_frame,
                            focus_pane == FocusPane::Sidebar,
                            self.thinking,
                        );
                        let sidebar_chunks = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Min(1), Constraint::Max(n_sessions * 3 + 3)])
                            .split(sidebar_inner);

                        // Subagents
                        let sub_area = sidebar_chunks[0];
                        if !subagent_list.is_empty() {
                            subagent_list.render(sub_area, frame, animation_frame, self.thinking);
                        }

                        // Session sidebar
                        App::draw_session_sidebar_inner(
                            frame,
                            sidebar_chunks[1],
                            config,
                            session_manager_ptr,
                        );
                    }
                }
            }

            // ── Overlays (render on top of everything) ──
            let overlay_area = frame.area();
            if session_picker.is_visible() {
                session_picker.render(
                    frame,
                    overlay_area,
                    animation_frame,
                    current_session_id.as_deref(),
                    self.thinking,
                );
            }
            if model_picker.is_visible() {
                model_picker.render(frame, overlay_area, animation_frame, self.thinking);
            }
            if prompt_manager.has_active_prompt() {
                prompt_manager.render(frame, overlay_area);
            }
            if completion_popup.is_visible() {
                completion_popup.render(frame, overlay_area, animation_frame);
            }

            // Show help overlay (tmux-style help screen)
            if self.show_help {
                crate::ui::help::HelpView::render(frame, overlay_area);
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
        let current_session = sessions.current_session();

        // 1. Current Session Stats
        if let Some(session) = current_session {
            lines.push(Line::from(vec![Span::styled(
                " SESSION STATS ",
                Style::default().bg(Color::Rgb(40, 40, 50)).bold(),
            )]));

            let usage = &session.total_usage;
            lines.push(Line::from(vec![
                Span::styled("  In:  ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:>8} tokens", usage.prompt_tokens),
                    Style::default().fg(Color::Rgb(166, 226, 46)),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Out: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:>8} tokens", usage.completion_tokens),
                    Style::default().fg(Color::Rgb(253, 151, 31)),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled("  Total:", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:>8} tokens", usage.total_tokens),
                    Style::default().fg(Color::Rgb(102, 217, 239)),
                ),
            ]));

            if let Some(read) = usage.cache_read_tokens {
                if read > 0 {
                    lines.push(Line::from(vec![
                        Span::styled("  Cache Read:", Style::default().fg(Color::Gray)),
                        Span::styled(
                            format!("{:>8} tokens", read),
                            Style::default().fg(Color::Rgb(174, 129, 255)),
                        ),
                    ]));
                }
            }

            lines.push(Line::from(vec![
                Span::styled("  Cost: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("   ${:.4}", session.total_cost),
                    Style::default().fg(Color::Rgb(249, 38, 114)).bold(),
                ),
            ]));

            // Tool call count in current session
            let tool_calls = session
                .messages
                .to_vec()
                .iter()
                .filter(|m| m.is_tool())
                .count();
            lines.push(Line::from(vec![
                Span::styled("  Tools: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:>9} calls", tool_calls),
                    Style::default().fg(Color::Yellow),
                ),
            ]));

            lines.push(Line::from(""));

            lines.push(Line::from(vec![Span::styled(
                " CONTEXT WINDOW ",
                Style::default().bg(Color::Rgb(40, 40, 50)).bold(),
            )]));

            let total = usage.total_tokens;
            let limit = 128000; // Default assumption
            let pct = (total as f64 / limit as f64 * 100.0).min(100.0);

            lines.push(Line::from(vec![
                Span::styled("  Usage: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}%", pct),
                    Style::default().fg(if pct > 80.0 { Color::Red } else { Color::Green }),
                ),
                Span::styled(
                    format!(" ({}k/{}k)", total / 1000, limit / 1000),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));

            lines.push(Line::from(""));
        }

        // 2. Recent Sessions List
        lines.push(Line::from(vec![Span::styled(
            " RECENT SESSIONS ",
            Style::default().bg(Color::Rgb(40, 40, 50)).bold(),
        )]));

        for session in session_list.iter().take(8) {
            let name = session.name.as_deref().unwrap_or("Unnamed");
            let is_current = current_id.is_some_and(|id| id == &session.id);

            let prefix = if is_current { "◈ " } else { "◇ " };
            let style = if is_current {
                Style::new()
                    .fg(Color::Rgb(166, 226, 46))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(Color::Rgb(117, 113, 94))
            };

            let msg_count = session.message_count();
            lines.push(Line::from(Span::styled(format!("{prefix}{name}"), style)));
            lines.push(Line::from(Span::styled(
                format!("  {msg_count} msgs"),
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
        // Leave alternate screen, disable mouse capture, and disable bracketed paste
        let _ = execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            DisableBracketedPaste
        );
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
        debug!(
            "Subagent start: {}",
            event.subagent_id.as_deref().unwrap_or("?")
        );
        let agent_id = event
            .subagent_id
            .clone()
            .unwrap_or_else(|| format!("agent-{}", event.task_index));
        let info =
            crate::ui::subagent::SubagentInfo::new(&agent_id, &event.goal, event.parent_id.clone());
        self.subagent_list_mut().upsert(info);
        // Add system message to chat with special ID for inline rendering
        let mut msg = Message::system(format!("Subagent '{}' started: {}", agent_id, event.goal));
        msg.message_id = Some(format!("subagent:{agent_id}"));
        self.messages_mut().add_message(msg.clone());
        self.chat_component
            .add_message(msg, &mut self.chat_state, &self.card_manager);
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
        debug!(
            "Subagent tool: {} - {}",
            event.subagent_id.as_deref().unwrap_or("?"),
            event.tool_name.as_deref().unwrap_or("?")
        );
        // Update subagent goal to show current tool
        if let Some(agent_id) = &event.subagent_id {
            if let Some(tool_name) = &event.tool_name {
                let info = crate::ui::subagent::SubagentInfo::new(
                    agent_id,
                    format!("Running tool '{tool_name}'..."),
                    event.parent_id.clone(),
                );
                self.subagent_list_mut().upsert(info);
            }
        }
        Ok(())
    }

    fn handle_subagent_complete(&mut self, event: SubagentEvent) -> Result<()> {
        debug!(
            "Subagent complete: {}",
            event.subagent_id.as_deref().unwrap_or("?")
        );
        if let Some(agent_id) = &event.subagent_id {
            let summary = event.summary.clone().unwrap_or_default();
            let mut info = crate::ui::subagent::SubagentInfo::new(
                agent_id,
                &event.goal,
                event.parent_id.clone(),
            );
            if event.status.as_deref() == Some("failed") || event.status.as_deref() == Some("error")
            {
                info.mark_failed();
            } else {
                info.mark_completed(summary.clone());
            }
            self.subagent_list_mut().upsert(info);
            // Update the chat message inline
            let status_str = if event.status.as_deref() == Some("failed")
                || event.status.as_deref() == Some("error")
            {
                "Failed"
            } else {
                "Completed"
            };
            let content = format!("Subagent '{agent_id}' {status_str}: {summary}");
            let msg_id = format!("subagent:{agent_id}");
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
            card_manager: CardManager::new(chat_colors_rgb.clone()),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            gateway_process: None,
            running: true,
            input_mode: InputMode::Insert,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb.clone(), true),
            chat_state: crate::ui::chat::ChatState::default(),
            ide_state: crate::ui::ide::IdeState::default(),
            input_composer: InputComposer::new(chat_colors_rgb.clone()),
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb.clone()),
            banner: crate::ui::banner::Banner::default(),
            prompt_manager: PromptManager::new(chat_colors_rgb.clone()),
            pending_approval_id: None,
            pending_clarify_id: None,
            pending_sudo_id: None,
            pending_secret_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb.clone()),
            model_picker: ModelPicker::new(chat_colors_rgb.clone()),
            session_picker: SessionPicker::new(chat_colors_rgb),
            actual_activity_height: 0.0,
            mouse_context: MouseContext::new(),
            current_model: None,
            current_provider: None,
            current_view: ViewState::Dashboard,
            previous_view: None,
            transition_progress: 1.0,
            sys: sysinfo::System::new_all(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            cpu_history: Vec::with_capacity(500),
            memory_history: Vec::with_capacity(500),
            token_speed_history: Vec::with_capacity(500),
            focus_pane: FocusPane::default(),
            animation_frame: 0,
            wave_ticker: crate::ui::wave::WaveTicker::new(),
            clipboard: Box::new(crate::utils::clipboard::MockClipboard::new()),
            capabilities: Capabilities::default(),
            last_completion_query: None,
            show_help: false,
            prefix_mode: false,
            streaming_reasoning: String::new(),
            streaming_message_id: None,
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
            card_manager: CardManager::new(chat_colors_rgb.clone()),
            subagent_list: SubagentList::new(),
            hashline_viewer: HashlineViewer::new(),
            gateway_process: None,
            running: true,
            input_mode: InputMode::Normal,
            current_input: String::new(),
            cursor_position: 0,
            chat_component: ChatComponent::new(chat_colors_rgb.clone(), true),
            chat_state: crate::ui::chat::ChatState::default(),
            ide_state: crate::ui::ide::IdeState::default(),
            input_composer: InputComposer::new(chat_colors_rgb.clone()),
            toolbar: Toolbar::new(theme_colors_rgb, chat_colors_rgb.clone()),
            banner: crate::ui::banner::Banner::default(),
            prompt_manager: PromptManager::new(chat_colors_rgb.clone()),
            pending_approval_id: None,
            pending_clarify_id: None,
            pending_sudo_id: None,
            pending_secret_id: None,
            completion_popup: CompletionPopup::new(chat_colors_rgb.clone()),
            model_picker: ModelPicker::new(chat_colors_rgb.clone()),
            session_picker: SessionPicker::new(chat_colors_rgb),
            actual_activity_height: 0.0,
            mouse_context: MouseContext::new(),
            current_model: None,
            current_provider: None,
            current_view: ViewState::Dashboard,
            previous_view: None,
            transition_progress: 1.0,
            sys: sysinfo::System::new_all(),
            cpu_usage: 0.0,
            memory_usage: 0.0,
            cpu_history: Vec::with_capacity(500),
            memory_history: Vec::with_capacity(500),
            token_speed_history: Vec::with_capacity(500),
            focus_pane: FocusPane::default(),
            animation_frame: 0,
            wave_ticker: crate::ui::wave::WaveTicker::new(),
            clipboard: Box::new(crate::utils::clipboard::MockClipboard::new()),
            capabilities: Capabilities::default(),
            last_completion_query: None,
            show_help: false,
            prefix_mode: false,
            streaming_reasoning: String::new(),
            streaming_message_id: None,
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
