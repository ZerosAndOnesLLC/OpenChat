using OpenChat.Controls;
using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;
using ChatMessage = OpenChat.Models.Message;

namespace OpenChat
{
    public partial class MainForm : Form
    {
        private ApiClient _apiClient;
        private WebSocketClient? _webSocketClient;
        private EmojiCache _emojiCache = null!;
        private MessagePanel _messagePanel = null!;

        private Guid? _currentChannelId;
        private Guid? _currentDmId;
        private List<Channel> _channels = new();
        private List<DirectMessage> _directMessages = new();
        private UserStatus? _currentUserStatus;

        public MainForm()
        {
            InitializeComponent();
            SetupUI();
            SetupMessagePanel();

            _apiClient = new ApiClient("https://openchat-api.zerosandones.us:9876");
            _emojiCache = new EmojiCache(_apiClient);

            if (!string.IsNullOrEmpty(AppSettings.AccessToken))
            {
                _apiClient.SetAccessToken(AppSettings.AccessToken);
                if (AppSettings.CurrentUser != null)
                {
                    _apiClient.SetCurrentUser(AppSettings.CurrentUser);
                }
            }

            // Initialize message panel with services
            _messagePanel.Initialize(_apiClient, _emojiCache, AppSettings.CurrentUser?.Id);

            lblUserName.Text = AppSettings.CurrentUser?.DisplayName ?? "User";
            Load += MainForm_Load;
        }

        private void SetupMessagePanel()
        {
            // Replace RichTextBox with MessagePanel
            _messagePanel = new MessagePanel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.ContentBackground
            };

            _messagePanel.ReactionToggled += OnReactionToggled;
            _messagePanel.AddReactionRequested += OnAddReactionRequested;

            // Remove rtbMessages and add _messagePanel
            pnlContent.Controls.Remove(rtbMessages);
            pnlContent.Controls.Add(_messagePanel);
        }

        private async Task OnReactionToggled(Guid messageId, string emoji)
        {
            if (AppSettings.CurrentUser == null) return;

            try
            {
                await _apiClient.ToggleReactionAsync(messageId, emoji, AppSettings.CurrentUser.Id);

                // Refresh reactions for this message
                var counts = await _apiClient.GetReactionCountsAsync(messageId);
                _messagePanel.UpdateReactions(messageId, counts);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to toggle reaction: {ex.Message}");
            }
        }

        private void OnAddReactionRequested(ChatMessage message, Point screenLocation)
        {
            ShowReactionPicker(message, screenLocation);
        }

        private void ShowReactionPicker(ChatMessage message, Point screenLocation)
        {
            var picker = new ReactionPickerForm(
                async emoji =>
                {
                    await OnReactionToggled(message.Id, emoji);
                },
                () =>
                {
                    // Show full emoji picker for reactions
                    ShowFullEmojiPickerForReaction(message, screenLocation);
                }
            );

            // Position the picker at the click location
            picker.Location = new Point(
                screenLocation.X - picker.Width / 2,
                screenLocation.Y - picker.Height - 5
            );

            // Ensure it stays on screen
            var screen = Screen.FromPoint(screenLocation);
            if (picker.Left < screen.WorkingArea.Left)
                picker.Left = screen.WorkingArea.Left + 10;
            if (picker.Right > screen.WorkingArea.Right)
                picker.Left = screen.WorkingArea.Right - picker.Width - 10;
            if (picker.Top < screen.WorkingArea.Top)
                picker.Top = screenLocation.Y + 10;

            picker.Show();
        }

        private void ShowFullEmojiPickerForReaction(ChatMessage message, Point screenLocation)
        {
            var picker = new EmojiPickerForm(
                _emojiCache,
                async emoji =>
                {
                    await OnReactionToggled(message.Id, emoji);
                }
            );

            // Position near the message
            picker.Location = new Point(
                screenLocation.X - picker.Width / 2,
                screenLocation.Y - picker.Height - 5
            );

            // Ensure it stays on screen
            var screen = Screen.FromPoint(screenLocation);
            if (picker.Left < screen.WorkingArea.Left)
                picker.Left = screen.WorkingArea.Left + 10;
            if (picker.Right > screen.WorkingArea.Right)
                picker.Left = screen.WorkingArea.Right - picker.Width - 10;
            if (picker.Top < screen.WorkingArea.Top)
                picker.Top = screenLocation.Y + 10;

            picker.Show();
        }

        private void SetupUI()
        {
            // Make status indicator circular
            pnlStatusIndicator.Paint += PnlStatusIndicator_Paint;
            pnlStatusIndicator.BackColor = Color.Transparent;

            // Round the user status panel corners
            pnlUserStatus.Paint += PnlUserStatus_Paint;

            // Round the input container corners
            pnlInputContainer.Paint += PnlInputContainer_Paint;

            // Add bottom border to channel header
            pnlChannelHeader.Paint += PnlChannelHeader_Paint;

            // Make user status panel clickable for status picker
            pnlUserStatus.Cursor = Cursors.Hand;
            pnlUserStatus.Click += PnlUserStatus_Click;
            lblUserName.Click += PnlUserStatus_Click;
            pnlStatusIndicator.Click += PnlUserStatus_Click;
        }

        private void PnlStatusIndicator_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var statusColor = GetStatusColor(_currentUserStatus?.Status ?? "online");
            using var brush = new SolidBrush(statusColor);
            e.Graphics.FillEllipse(brush, 0, 4, 10, 10);
        }

        private static Color GetStatusColor(string status)
        {
            return status switch
            {
                "online" => Color.FromArgb(46, 182, 125),
                "away" => Color.FromArgb(250, 168, 26),
                "dnd" => Color.FromArgb(237, 66, 69),
                "offline" or "invisible" => Color.FromArgb(116, 116, 116),
                _ => Color.FromArgb(46, 182, 125)
            };
        }

        private void PnlUserStatus_Click(object? sender, EventArgs e)
        {
            ShowStatusPicker();
        }

        private void ShowStatusPicker()
        {
            var picker = new StatusPickerForm(_apiClient, _currentUserStatus, OnStatusChanged);

            // Position below the user status panel
            var panelLocation = pnlUserStatus.PointToScreen(Point.Empty);
            picker.Location = new Point(
                panelLocation.X,
                panelLocation.Y + pnlUserStatus.Height + 5
            );

            // Ensure it stays on screen
            var screen = Screen.FromControl(this);
            if (picker.Right > screen.WorkingArea.Right)
                picker.Left = screen.WorkingArea.Right - picker.Width - 10;
            if (picker.Bottom > screen.WorkingArea.Bottom)
                picker.Top = panelLocation.Y - picker.Height - 5;

            picker.Show();
        }

        private void OnStatusChanged(UserStatus newStatus)
        {
            _currentUserStatus = newStatus;
            pnlStatusIndicator.Invalidate();
        }

        private void PnlUserStatus_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = pnlUserStatus.ClientRectangle;
            using var path = GetRoundedRectPath(rect, 6);
            using var brush = new SolidBrush(Color.FromArgb(35, 35, 40));
            e.Graphics.FillPath(brush, path);
        }

        private void PnlInputContainer_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = pnlInputContainer.ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            using var path = GetRoundedRectPath(rect, 8);
            using var brush = new SolidBrush(Theme.Dark.InputBackground);
            using var pen = new Pen(Theme.Dark.InputBorder, 1);
            e.Graphics.FillPath(brush, path);
            e.Graphics.DrawPath(pen, path);
        }

        private void PnlChannelHeader_Paint(object? sender, PaintEventArgs e)
        {
            var rect = pnlChannelHeader.ClientRectangle;
            using var pen = new Pen(Theme.Dark.DividerColor, 1);
            e.Graphics.DrawLine(pen, 0, rect.Height - 1, rect.Width, rect.Height - 1);
        }

        private static GraphicsPath GetRoundedRectPath(Rectangle rect, int radius)
        {
            var path = new GraphicsPath();
            var diameter = radius * 2;
            var arc = new Rectangle(rect.Location, new Size(diameter, diameter));

            path.AddArc(arc, 180, 90);
            arc.X = rect.Right - diameter;
            path.AddArc(arc, 270, 90);
            arc.Y = rect.Bottom - diameter;
            path.AddArc(arc, 0, 90);
            arc.X = rect.Left;
            path.AddArc(arc, 90, 90);
            path.CloseFigure();

            return path;
        }

        private void LstChannels_DrawItem(object? sender, DrawItemEventArgs e)
        {
            if (e.Index < 0 || e.Index >= _channels.Count) return;

            var channel = _channels[e.Index];
            var isSelected = (e.State & DrawItemState.Selected) == DrawItemState.Selected;
            var bounds = e.Bounds;

            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            // Background
            var bgColor = isSelected ? Theme.Dark.SelectedBackground : Theme.Dark.SidebarBackground;
            if (!isSelected && bounds.Contains(lstChannels.PointToClient(Cursor.Position)))
            {
                bgColor = Theme.Dark.HoverBackground;
            }

            using (var bgBrush = new SolidBrush(bgColor))
            {
                if (isSelected)
                {
                    var roundRect = new Rectangle(bounds.X + 4, bounds.Y + 2, bounds.Width - 8, bounds.Height - 4);
                    using var path = GetRoundedRectPath(roundRect, 6);
                    e.Graphics.FillPath(bgBrush, path);
                }
                else
                {
                    e.Graphics.FillRectangle(bgBrush, bounds);
                }
            }

            // Channel icon and name
            var textColor = isSelected ? Theme.Dark.TextWhite : Theme.Dark.TextSecondary;
            if (channel.UnreadCount > 0)
            {
                textColor = Theme.Dark.TextWhite;
            }

            using var textBrush = new SolidBrush(textColor);
            var font = channel.UnreadCount > 0 ? Theme.Fonts.SidebarItemBold : Theme.Fonts.SidebarItem;
            var text = $"# {channel.Name}";

            var textRect = new Rectangle(bounds.X + 16, bounds.Y, bounds.Width - 50, bounds.Height);
            var sf = new StringFormat { LineAlignment = StringAlignment.Center, Trimming = StringTrimming.EllipsisCharacter };
            e.Graphics.DrawString(text, font, textBrush, textRect, sf);

            // Unread badge
            if (channel.UnreadCount > 0)
            {
                var badgeText = channel.UnreadCount > 99 ? "99+" : channel.UnreadCount.ToString();
                var badgeFont = new Font("Segoe UI", 8F, FontStyle.Bold);
                var badgeSize = e.Graphics.MeasureString(badgeText, badgeFont);
                var badgeWidth = Math.Max(20, (int)badgeSize.Width + 8);
                var badgeRect = new Rectangle(bounds.Right - badgeWidth - 12, bounds.Y + (bounds.Height - 18) / 2, badgeWidth, 18);

                using var badgePath = GetRoundedRectPath(badgeRect, 9);
                using var badgeBrush = new SolidBrush(Theme.Dark.UnreadBadge);
                e.Graphics.FillPath(badgeBrush, badgePath);

                using var badgeTextBrush = new SolidBrush(Color.White);
                var badgeSf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
                e.Graphics.DrawString(badgeText, badgeFont, badgeTextBrush, badgeRect, badgeSf);
            }
        }

        private void LstDirectMessages_DrawItem(object? sender, DrawItemEventArgs e)
        {
            if (e.Index < 0 || e.Index >= _directMessages.Count) return;

            var dm = _directMessages[e.Index];
            var isSelected = (e.State & DrawItemState.Selected) == DrawItemState.Selected;
            var bounds = e.Bounds;

            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            // Background
            var bgColor = isSelected ? Theme.Dark.SelectedBackground : Theme.Dark.SidebarBackground;
            if (!isSelected && bounds.Contains(lstDirectMessages.PointToClient(Cursor.Position)))
            {
                bgColor = Theme.Dark.HoverBackground;
            }

            using (var bgBrush = new SolidBrush(bgColor))
            {
                if (isSelected)
                {
                    var roundRect = new Rectangle(bounds.X + 4, bounds.Y + 2, bounds.Width - 8, bounds.Height - 4);
                    using var path = GetRoundedRectPath(roundRect, 6);
                    e.Graphics.FillPath(bgBrush, path);
                }
                else
                {
                    e.Graphics.FillRectangle(bgBrush, bounds);
                }
            }

            // Online status indicator
            var statusRect = new Rectangle(bounds.X + 12, bounds.Y + (bounds.Height - 10) / 2, 10, 10);
            using (var statusBrush = new SolidBrush(Theme.Dark.OnlineGreen))
            {
                e.Graphics.FillEllipse(statusBrush, statusRect);
            }

            // User name
            var textColor = isSelected ? Theme.Dark.TextWhite : Theme.Dark.TextSecondary;
            if (dm.UnreadCount > 0)
            {
                textColor = Theme.Dark.TextWhite;
            }

            using var textBrush = new SolidBrush(textColor);
            var font = dm.UnreadCount > 0 ? Theme.Fonts.SidebarItemBold : Theme.Fonts.SidebarItem;
            var displayName = dm.OtherUser?.DisplayName ?? "Unknown User";

            var textRect = new Rectangle(bounds.X + 28, bounds.Y, bounds.Width - 66, bounds.Height);
            var sf = new StringFormat { LineAlignment = StringAlignment.Center, Trimming = StringTrimming.EllipsisCharacter };
            e.Graphics.DrawString(displayName, font, textBrush, textRect, sf);

            // Unread badge
            if (dm.UnreadCount > 0)
            {
                var badgeText = dm.UnreadCount > 99 ? "99+" : dm.UnreadCount.ToString();
                var badgeFont = new Font("Segoe UI", 8F, FontStyle.Bold);
                var badgeSize = e.Graphics.MeasureString(badgeText, badgeFont);
                var badgeWidth = Math.Max(20, (int)badgeSize.Width + 8);
                var badgeRect = new Rectangle(bounds.Right - badgeWidth - 12, bounds.Y + (bounds.Height - 18) / 2, badgeWidth, 18);

                using var badgePath = GetRoundedRectPath(badgeRect, 9);
                using var badgeBrush = new SolidBrush(Theme.Dark.UnreadBadge);
                e.Graphics.FillPath(badgeBrush, badgePath);

                using var badgeTextBrush = new SolidBrush(Color.White);
                var badgeSf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
                e.Graphics.DrawString(badgeText, badgeFont, badgeTextBrush, badgeRect, badgeSf);
            }
        }

        private async void MainForm_Load(object? sender, EventArgs e)
        {
            try
            {
                await ConnectWebSocketAsync();
                await LoadChannelsAsync();
                await LoadDirectMessagesAsync();
                await LoadUserStatusAsync();
                // Pre-load custom emojis in background
                _ = _emojiCache.GetCustomEmojisAsync();
            }
            catch (Exception ex)
            {
                ShowError("Initialization Failed", "Failed to initialize the application.", ex.ToString());
            }
        }

        private async Task LoadUserStatusAsync()
        {
            try
            {
                _currentUserStatus = await _apiClient.GetMyStatusAsync();
                pnlStatusIndicator.Invalidate();
            }
            catch
            {
                // Default to online if we can't fetch status
                _currentUserStatus = new UserStatus { Status = "online" };
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
            this.Invoke(() => ShowError("Connection Error", "WebSocket connection error occurred.", ex.ToString()));
        }

        private void WebSocketClient_MessageReceived(object? sender, Models.Message message)
        {
            this.Invoke(() =>
            {
                if ((message.ChannelId == _currentChannelId && _currentChannelId != null) ||
                    (message.DmId == _currentDmId && _currentDmId != null))
                {
                    _messagePanel.AppendMessage(message);
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
                    lstChannels.Items.Add(channel.Name);
                }
            }
            catch (Exception ex)
            {
                ShowError("Load Failed", "Failed to load channels.", ex.ToString());
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
                    lstDirectMessages.Items.Add(dm.OtherUser?.DisplayName ?? "Unknown");
                }
            }
            catch (Exception ex)
            {
                ShowError("Load Failed", "Failed to load direct messages.", ex.ToString());
            }
        }

        private async void LstChannels_SelectedIndexChanged(object? sender, EventArgs e)
        {
            if (lstChannels.SelectedIndex < 0 || lstChannels.SelectedIndex >= _channels.Count)
                return;

            lstDirectMessages.ClearSelected();

            var channel = _channels[lstChannels.SelectedIndex];
            _currentChannelId = channel.Id;
            _currentDmId = null;
            lblCurrentChannel.Text = $"# {channel.Name}";
            lblChannelDescription.Text = channel.Description ?? "This is the beginning of your conversation";

            await LoadMessagesAsync();
        }

        private async void LstDirectMessages_SelectedIndexChanged(object? sender, EventArgs e)
        {
            if (lstDirectMessages.SelectedIndex < 0 || lstDirectMessages.SelectedIndex >= _directMessages.Count)
                return;

            lstChannels.ClearSelected();

            var dm = _directMessages[lstDirectMessages.SelectedIndex];
            _currentDmId = dm.Id;
            _currentChannelId = null;
            lblCurrentChannel.Text = dm.OtherUser?.DisplayName ?? "Unknown User";
            lblChannelDescription.Text = "Direct message conversation";

            await LoadMessagesAsync();
        }

        private async Task LoadMessagesAsync()
        {
            _messagePanel.ClearMessages();

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
                _messagePanel.SetMessages(messages);
            }
            catch (Exception ex)
            {
                ShowError("Load Failed", "Failed to load messages.", ex.ToString());
            }
        }

        private async void BtnSend_Click(object? sender, EventArgs e)
        {
            await SendMessageAsync();
        }

        private void BtnEmoji_Click(object? sender, EventArgs e)
        {
            ShowEmojiPicker();
        }

        private void ShowEmojiPicker()
        {
            var picker = new WebEmojiPickerForm(
                _emojiCache,
                _apiClient.BaseUrl,
                OnEmojiSelected
            );

            // Position the picker above the emoji button
            var btnLocation = btnEmoji.PointToScreen(Point.Empty);
            picker.Location = new Point(
                btnLocation.X - picker.Width + btnEmoji.Width + 10,
                btnLocation.Y - picker.Height - 10
            );

            // Ensure it stays on screen
            var screen = Screen.FromControl(this);
            if (picker.Left < screen.WorkingArea.Left)
                picker.Left = screen.WorkingArea.Left + 10;
            if (picker.Top < screen.WorkingArea.Top)
                picker.Top = btnLocation.Y + btnEmoji.Height + 10;

            picker.Show();
        }

        private void OnEmojiSelected(string emoji)
        {
            // Insert emoji at cursor position
            var selectionStart = txtMessage.SelectionStart;
            txtMessage.Text = txtMessage.Text.Insert(selectionStart, emoji);
            txtMessage.SelectionStart = selectionStart + emoji.Length;
            txtMessage.Focus();
        }

        private void ShowEmojiUploadDialog()
        {
            using var dialog = new EmojiUploadDialog(_apiClient, _emojiCache, () =>
            {
                // Refresh emoji cache after successful upload
                _ = _emojiCache.GetCustomEmojisAsync(forceRefresh: true);
            });
            dialog.ShowDialog(this);
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
                ShowError("No Conversation Selected", "Please select a channel or direct message first.");
                return;
            }

            try
            {
                var sentMessage = await _apiClient.SendMessageAsync(_currentChannelId, _currentDmId, content);
                txtMessage.Clear();

                // Optimistically display the message immediately (don't wait for WebSocket)
                if (sentMessage != null)
                {
                    // Add user info if not present
                    if (sentMessage.User == null && AppSettings.CurrentUser != null)
                    {
                        sentMessage.User = new Models.User
                        {
                            Id = AppSettings.CurrentUser.Id,
                            DisplayName = AppSettings.CurrentUser.DisplayName
                        };
                    }
                    _messagePanel.AppendMessage(sentMessage);
                }
            }
            catch (Exception ex)
            {
                ShowError("Send Failed", "Failed to send message.", ex.ToString());
            }
        }

        protected override void OnFormClosing(FormClosingEventArgs e)
        {
            _webSocketClient?.DisconnectAsync().Wait();
            base.OnFormClosing(e);
        }

        private void ShowError(string title, string message, string? details = null)
        {
            ErrorDialog.Show(this, title, message, details);
        }

        private void BtnBrowseChannels_Click(object? sender, EventArgs e)
        {
            using var form = new BrowseChannelsForm(_apiClient, OnChannelJoined);
            form.ShowDialog(this);
        }

        private async void OnChannelJoined(Channel channel)
        {
            // Reload channels to include the newly joined one
            await LoadChannelsAsync();
        }

        private void BtnNewDm_Click(object? sender, EventArgs e)
        {
            if (AppSettings.CurrentUser == null) return;

            using var form = new NewDmForm(_apiClient, AppSettings.CurrentUser.Id, OnDmCreated);
            form.ShowDialog(this);
        }

        private async void OnDmCreated(DirectMessage dm)
        {
            // Reload DMs and select the new one
            await LoadDirectMessagesAsync();

            // Find and select the new DM
            var index = _directMessages.FindIndex(d => d.Id == dm.Id);
            if (index >= 0)
            {
                lstDirectMessages.SelectedIndex = index;
            }
        }
    }
}
