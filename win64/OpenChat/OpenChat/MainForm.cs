using OpenChat.Models;
using OpenChat.Services;

namespace OpenChat
{
    public partial class MainForm : Form
    {
        private ApiClient _apiClient;
        private WebSocketClient? _webSocketClient;

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

            lblUser.Text = AppSettings.CurrentUser?.DisplayName ?? "User";
            Load += MainForm_Load;
        }

        private async void MainForm_Load(object? sender, EventArgs e)
        {
            try
            {
                await ConnectWebSocketAsync();
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
            _webSocketClient.Error += WebSocketClient_Error;

            await _webSocketClient.ConnectAsync();
        }

        private void WebSocketClient_Error(object? sender, Exception ex)
        {
            this.Invoke(() => MessageBox.Show($"WebSocket error: {ex.Message}", "Error"));
        }

        private void WebSocketClient_MessageReceived(object? sender, Models.Message message)
        {
            this.Invoke(() =>
            {
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

            rtbMessages.SelectionFont = new Font(rtbMessages.Font, FontStyle.Bold);
            rtbMessages.SelectionColor = Color.FromArgb(0, 120, 212);
            rtbMessages.AppendText($"{userName} ");

            rtbMessages.SelectionFont = new Font(rtbMessages.Font, FontStyle.Regular);
            rtbMessages.SelectionColor = Color.Gray;
            rtbMessages.AppendText($"{timestamp}\n");

            rtbMessages.SelectionFont = new Font(rtbMessages.Font, FontStyle.Regular);
            rtbMessages.SelectionColor = Color.Black;
            rtbMessages.AppendText($"{content}\n\n");

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
