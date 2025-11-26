using OpenChat.Models;
using OpenChat.Services;

namespace OpenChat
{
    public partial class MainForm : Form
    {
        private ApiClient _apiClient;
        private WebSocketClient? _webSocketClient;

        // UI Controls
        private ListBox lstChannels;
        private ListBox lstDirectMessages;
        private RichTextBox rtbMessages;
        private TextBox txtMessage;
        private Button btnSend;
        private Label lblCurrentChannel;
        private Panel pnlChannelList;
        private Panel pnlMessages;
        private Panel pnlMessageInput;
        private TabControl tabConversations;

        private Guid? _currentChannelId;
        private Guid? _currentDmId;
        private List<Channel> _channels = new();
        private List<DirectMessage> _directMessages = new();

        public MainForm()
        {
            InitializeComponent();
            _apiClient = new ApiClient("https://openchat-api.zerosandones.us:9876");

            if (!string.IsNullOrEmpty(AppSettings.AccessToken))
            {
                _apiClient.SetAccessToken(AppSettings.AccessToken);
                if (AppSettings.CurrentUser != null)
                {
                    _apiClient.SetCurrentUser(AppSettings.CurrentUser);
                }
            }

            Load += MainForm_Load;
        }

        private void InitializeComponent()
        {
            this.Text = "OpenChat";
            this.Size = new Size(1200, 800);
            this.StartPosition = FormStartPosition.CenterScreen;

            // Left panel for channel/DM list
            pnlChannelList = new Panel
            {
                Dock = DockStyle.Left,
                Width = 250,
                BackColor = Color.FromArgb(45, 45, 48)
            };

            // Tab control for channels and DMs
            tabConversations = new TabControl
            {
                Dock = DockStyle.Fill,
                Location = new Point(0, 50)
            };

            var tabChannels = new TabPage("Channels");
            var tabDMs = new TabPage("Direct Messages");

            lstChannels = new ListBox
            {
                Dock = DockStyle.Fill,
                BackColor = Color.FromArgb(45, 45, 48),
                ForeColor = Color.White,
                BorderStyle = BorderStyle.None,
                Font = new Font("Segoe UI", 10)
            };
            lstChannels.SelectedIndexChanged += LstChannels_SelectedIndexChanged;

            lstDirectMessages = new ListBox
            {
                Dock = DockStyle.Fill,
                BackColor = Color.FromArgb(45, 45, 48),
                ForeColor = Color.White,
                BorderStyle = BorderStyle.None,
                Font = new Font("Segoe UI", 10)
            };
            lstDirectMessages.SelectedIndexChanged += LstDirectMessages_SelectedIndexChanged;

            tabChannels.Controls.Add(lstChannels);
            tabDMs.Controls.Add(lstDirectMessages);
            tabConversations.TabPages.Add(tabChannels);
            tabConversations.TabPages.Add(tabDMs);

            pnlChannelList.Controls.Add(tabConversations);

            // User info label at top of channel list
            var lblUser = new Label
            {
                Text = AppSettings.CurrentUser?.DisplayName ?? "User",
                Dock = DockStyle.Top,
                Height = 50,
                BackColor = Color.FromArgb(30, 30, 30),
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 12, FontStyle.Bold),
                TextAlign = ContentAlignment.MiddleCenter
            };
            pnlChannelList.Controls.Add(lblUser);
            lblUser.BringToFront();

            // Messages panel
            pnlMessages = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Color.White
            };

            // Current channel label
            lblCurrentChannel = new Label
            {
                Dock = DockStyle.Top,
                Height = 50,
                BackColor = Color.FromArgb(250, 250, 250),
                ForeColor = Color.Black,
                Font = new Font("Segoe UI", 14, FontStyle.Bold),
                TextAlign = ContentAlignment.MiddleLeft,
                Padding = new Padding(20, 0, 0, 0),
                Text = "Select a channel"
            };

            // Messages display
            rtbMessages = new RichTextBox
            {
                Dock = DockStyle.Fill,
                ReadOnly = true,
                BackColor = Color.White,
                BorderStyle = BorderStyle.None,
                Font = new Font("Segoe UI", 10),
                Padding = new Padding(10)
            };

            pnlMessages.Controls.Add(rtbMessages);
            pnlMessages.Controls.Add(lblCurrentChannel);

            // Message input panel
            pnlMessageInput = new Panel
            {
                Dock = DockStyle.Bottom,
                Height = 80,
                BackColor = Color.FromArgb(240, 240, 240),
                Padding = new Padding(10)
            };

            txtMessage = new TextBox
            {
                Dock = DockStyle.Fill,
                Multiline = true,
                Font = new Font("Segoe UI", 10),
                BorderStyle = BorderStyle.FixedSingle
            };
            txtMessage.KeyDown += TxtMessage_KeyDown;

            btnSend = new Button
            {
                Dock = DockStyle.Right,
                Width = 100,
                Text = "Send",
                BackColor = Color.FromArgb(0, 120, 212),
                ForeColor = Color.White,
                FlatStyle = FlatStyle.Flat,
                Font = new Font("Segoe UI", 10, FontStyle.Bold)
            };
            btnSend.FlatAppearance.BorderSize = 0;
            btnSend.Click += BtnSend_Click;

            pnlMessageInput.Controls.Add(txtMessage);
            pnlMessageInput.Controls.Add(btnSend);

            this.Controls.Add(pnlMessages);
            this.Controls.Add(pnlMessageInput);
            this.Controls.Add(pnlChannelList);
        }

        private async void MainForm_Load(object? sender, EventArgs e)
        {
            try
            {
                // Connect to WebSocket
                await ConnectWebSocketAsync();

                // Load channels and DMs
                await LoadChannelsAsync();
                await LoadDirectMessagesAsync();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to initialize: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private async Task ConnectWebSocketAsync()
        {
            if (string.IsNullOrEmpty(AppSettings.AccessToken))
                return;

            _webSocketClient = new WebSocketClient("wss://openchat-api.zerosandones.us:9876/api/ws", AppSettings.AccessToken);
            _webSocketClient.MessageReceived += WebSocketClient_MessageReceived;
            _webSocketClient.Error += (s, ex) =>
            {
                this.Invoke(() => MessageBox.Show($"WebSocket error: {ex.Message}", "Error"));
            };

            await _webSocketClient.ConnectAsync();
        }

        private void WebSocketClient_MessageReceived(object? sender, Models.Message message)
        {
            // Update UI on the UI thread
            this.Invoke(() =>
            {
                // Only add message if it's for the current channel/DM
                if ((message.ChannelId == _currentChannelId && _currentChannelId != null) ||
                    (message.DmId == _currentDmId && _currentDmId != null))
                {
                    AppendMessage(message);
                }
            });
        }

        private async Task LoadChannelsAsync()
        {
            try
            {
                _channels = await _apiClient.GetChannelsAsync();
                lstChannels.Items.Clear();
                foreach (var channel in _channels)
                {
                    var display = $"# {channel.Name}";
                    if (channel.UnreadCount > 0)
                    {
                        display += $" ({channel.UnreadCount})";
                    }
                    lstChannels.Items.Add(display);
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to load channels: {ex.Message}", "Error");
            }
        }

        private async Task LoadDirectMessagesAsync()
        {
            try
            {
                _directMessages = await _apiClient.GetDirectMessagesAsync();
                lstDirectMessages.Items.Clear();
                foreach (var dm in _directMessages)
                {
                    var display = dm.OtherUser?.DisplayName ?? "Unknown User";
                    if (dm.UnreadCount > 0)
                    {
                        display += $" ({dm.UnreadCount})";
                    }
                    lstDirectMessages.Items.Add(display);
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to load direct messages: {ex.Message}", "Error");
            }
        }

        private async void LstChannels_SelectedIndexChanged(object? sender, EventArgs e)
        {
            if (lstChannels.SelectedIndex < 0 || lstChannels.SelectedIndex >= _channels.Count)
                return;

            var channel = _channels[lstChannels.SelectedIndex];
            _currentChannelId = channel.Id;
            _currentDmId = null;
            lblCurrentChannel.Text = $"# {channel.Name}";

            await LoadMessagesAsync();
        }

        private async void LstDirectMessages_SelectedIndexChanged(object? sender, EventArgs e)
        {
            if (lstDirectMessages.SelectedIndex < 0 || lstDirectMessages.SelectedIndex >= _directMessages.Count)
                return;

            var dm = _directMessages[lstDirectMessages.SelectedIndex];
            _currentDmId = dm.Id;
            _currentChannelId = null;
            lblCurrentChannel.Text = $"@ {dm.OtherUser?.DisplayName}";

            await LoadMessagesAsync();
        }

        private async Task LoadMessagesAsync()
        {
            rtbMessages.Clear();

            try
            {
                List<Models.Message> messages;
                if (_currentChannelId.HasValue)
                {
                    messages = await _apiClient.GetChannelMessagesAsync(_currentChannelId.Value);
                }
                else if (_currentDmId.HasValue)
                {
                    messages = await _apiClient.GetDmMessagesAsync(_currentDmId.Value);
                }
                else
                {
                    return;
                }

                // Display messages in reverse order (oldest first)
                messages.Reverse();
                foreach (var message in messages)
                {
                    AppendMessage(message);
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to load messages: {ex.Message}", "Error");
            }
        }

        private void AppendMessage(Models.Message message)
        {
            var userName = message.User?.DisplayName ?? "Unknown User";
            var timestamp = message.CreatedAt.ToLocalTime().ToString("HH:mm");
            var content = message.Content;

            rtbMessages.SelectionStart = rtbMessages.TextLength;
            rtbMessages.SelectionLength = 0;

            // User name in bold
            rtbMessages.SelectionFont = new Font(rtbMessages.Font, FontStyle.Bold);
            rtbMessages.SelectionColor = Color.FromArgb(0, 120, 212);
            rtbMessages.AppendText($"{userName} ");

            // Timestamp in gray
            rtbMessages.SelectionFont = new Font(rtbMessages.Font, FontStyle.Regular);
            rtbMessages.SelectionColor = Color.Gray;
            rtbMessages.AppendText($"{timestamp}\n");

            // Message content
            rtbMessages.SelectionFont = new Font(rtbMessages.Font, FontStyle.Regular);
            rtbMessages.SelectionColor = Color.Black;
            rtbMessages.AppendText($"{content}\n\n");

            // Scroll to bottom
            rtbMessages.SelectionStart = rtbMessages.TextLength;
            rtbMessages.ScrollToCaret();
        }

        private async void BtnSend_Click(object? sender, EventArgs e)
        {
            await SendMessageAsync();
        }

        private async void TxtMessage_KeyDown(object? sender, KeyEventArgs e)
        {
            if (e.KeyCode == Keys.Enter && !e.Shift)
            {
                e.Handled = true;
                e.SuppressKeyPress = true;
                await SendMessageAsync();
            }
        }

        private async Task SendMessageAsync()
        {
            var content = txtMessage.Text.Trim();
            if (string.IsNullOrEmpty(content))
                return;

            if (!_currentChannelId.HasValue && !_currentDmId.HasValue)
            {
                MessageBox.Show("Please select a channel or direct message first.", "Info");
                return;
            }

            try
            {
                await _apiClient.SendMessageAsync(_currentChannelId, _currentDmId, content);
                txtMessage.Clear();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to send message: {ex.Message}", "Error");
            }
        }

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            _webSocketClient?.DisconnectAsync().Wait();
            base.OnFormClosing(e);
        }
    }
}
