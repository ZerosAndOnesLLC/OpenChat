using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;

namespace OpenChat
{
    public class BrowseChannelsForm : Form
    {
        private readonly ApiClient _apiClient;
        private readonly Action<Channel> _onChannelJoined;
        private List<Channel> _publicChannels = new();
        private List<Channel> _filteredChannels = new();

        private TextBox txtSearch = null!;
        private ListBox lstChannels = null!;
        private Button btnJoin = null!;
        private Label lblStatus = null!;

        public BrowseChannelsForm(ApiClient apiClient, Action<Channel> onChannelJoined)
        {
            _apiClient = apiClient;
            _onChannelJoined = onChannelJoined;

            InitializeComponent();
            _ = LoadPublicChannelsAsync();
        }

        private void InitializeComponent()
        {
            SuspendLayout();

            Text = "Browse Channels";
            Size = new Size(450, 500);
            StartPosition = FormStartPosition.CenterParent;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            BackColor = Theme.Dark.ContentBackground;
            ForeColor = Theme.Dark.TextPrimary;

            // Header
            var lblHeader = new Label
            {
                Text = "Browse Public Channels",
                Font = new Font("Segoe UI", 14F, FontStyle.Bold),
                ForeColor = Theme.Dark.TextWhite,
                Dock = DockStyle.Top,
                Height = 50,
                Padding = new Padding(20, 16, 20, 8)
            };

            // Search box container
            var pnlSearch = new Panel
            {
                Dock = DockStyle.Top,
                Height = 50,
                Padding = new Padding(20, 5, 20, 10)
            };

            txtSearch = new TextBox
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.InputBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = new Font("Segoe UI", 11F),
                BorderStyle = BorderStyle.None,
                PlaceholderText = "Search channels..."
            };
            txtSearch.TextChanged += TxtSearch_TextChanged;

            var pnlSearchWrapper = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.InputBackground,
                Padding = new Padding(12, 8, 12, 8)
            };
            pnlSearchWrapper.Paint += PnlSearchWrapper_Paint;
            pnlSearchWrapper.Controls.Add(txtSearch);
            pnlSearch.Controls.Add(pnlSearchWrapper);

            // Status label
            lblStatus = new Label
            {
                Text = "Loading...",
                Font = new Font("Segoe UI", 9F),
                ForeColor = Theme.Dark.TextMuted,
                Dock = DockStyle.Top,
                Height = 25,
                Padding = new Padding(20, 5, 20, 5)
            };

            // Channel list
            lstChannels = new ListBox
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.ContentBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = new Font("Segoe UI", 10F),
                BorderStyle = BorderStyle.None,
                DrawMode = DrawMode.OwnerDrawFixed,
                ItemHeight = 60,
                IntegralHeight = false
            };
            lstChannels.DrawItem += LstChannels_DrawItem;
            lstChannels.SelectedIndexChanged += LstChannels_SelectedIndexChanged;
            lstChannels.DoubleClick += LstChannels_DoubleClick;

            var pnlList = new Panel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(10, 0, 10, 0)
            };
            pnlList.Controls.Add(lstChannels);

            // Buttons panel
            var pnlButtons = new Panel
            {
                Dock = DockStyle.Bottom,
                Height = 60,
                Padding = new Padding(20, 10, 20, 15)
            };

            btnJoin = new Button
            {
                Text = "Join Channel",
                Dock = DockStyle.Right,
                Width = 120,
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonPrimary,
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 10F, FontStyle.Bold),
                Cursor = Cursors.Hand,
                Enabled = false
            };
            btnJoin.FlatAppearance.BorderSize = 0;
            btnJoin.Click += BtnJoin_Click;

            var btnClose = new Button
            {
                Text = "Close",
                Dock = DockStyle.Right,
                Width = 80,
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.Transparent,
                ForeColor = Theme.Dark.TextSecondary,
                Font = new Font("Segoe UI", 10F),
                Cursor = Cursors.Hand,
                Margin = new Padding(0, 0, 10, 0)
            };
            btnClose.FlatAppearance.BorderSize = 0;
            btnClose.FlatAppearance.MouseOverBackColor = Theme.Dark.HoverBackground;
            btnClose.Click += (s, e) => Close();

            pnlButtons.Controls.Add(btnJoin);
            pnlButtons.Controls.Add(btnClose);

            // Add controls
            Controls.Add(pnlList);
            Controls.Add(lblStatus);
            Controls.Add(pnlSearch);
            Controls.Add(lblHeader);
            Controls.Add(pnlButtons);

            ResumeLayout(false);
        }

        private async Task LoadPublicChannelsAsync()
        {
            try
            {
                _publicChannels = await _apiClient.GetPublicChannelsAsync();
                ApplyFilter();
                UpdateStatus();
            }
            catch (Exception ex)
            {
                lblStatus.Text = $"Failed to load channels: {ex.Message}";
            }
        }

        private void TxtSearch_TextChanged(object? sender, EventArgs e)
        {
            ApplyFilter();
        }

        private void ApplyFilter()
        {
            var query = txtSearch.Text.Trim().ToLowerInvariant();

            if (string.IsNullOrEmpty(query))
            {
                _filteredChannels = _publicChannels.ToList();
            }
            else
            {
                _filteredChannels = _publicChannels
                    .Where(c => c.Name.ToLowerInvariant().Contains(query) ||
                               (c.Description?.ToLowerInvariant().Contains(query) ?? false))
                    .ToList();
            }

            lstChannels.Items.Clear();
            foreach (var channel in _filteredChannels)
            {
                lstChannels.Items.Add(channel.Name);
            }

            UpdateStatus();
        }

        private void UpdateStatus()
        {
            if (_publicChannels.Count == 0)
            {
                lblStatus.Text = "No public channels available to join";
            }
            else if (_filteredChannels.Count == 0)
            {
                lblStatus.Text = "No channels match your search";
            }
            else
            {
                lblStatus.Text = $"{_filteredChannels.Count} channel{(_filteredChannels.Count != 1 ? "s" : "")} available";
            }
        }

        private void LstChannels_DrawItem(object? sender, DrawItemEventArgs e)
        {
            if (e.Index < 0 || e.Index >= _filteredChannels.Count) return;

            var channel = _filteredChannels[e.Index];
            var isSelected = (e.State & DrawItemState.Selected) == DrawItemState.Selected;
            var bounds = e.Bounds;

            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            // Background
            var bgColor = isSelected ? Theme.Dark.SelectedBackground : Theme.Dark.ContentBackground;
            using (var bgBrush = new SolidBrush(bgColor))
            {
                var roundRect = new Rectangle(bounds.X + 2, bounds.Y + 2, bounds.Width - 4, bounds.Height - 4);
                using var path = GetRoundedRectPath(roundRect, 6);
                e.Graphics.FillPath(bgBrush, path);
            }

            // Channel icon
            using (var iconBrush = new SolidBrush(Theme.Dark.TextMuted))
            {
                e.Graphics.DrawString("#", new Font("Segoe UI", 16F, FontStyle.Bold), iconBrush, bounds.X + 16, bounds.Y + 14);
            }

            // Channel name
            using (var nameBrush = new SolidBrush(Theme.Dark.TextWhite))
            {
                e.Graphics.DrawString(channel.Name, new Font("Segoe UI", 11F, FontStyle.Bold), nameBrush, bounds.X + 48, bounds.Y + 10);
            }

            // Description
            var description = channel.Description ?? "No description";
            if (description.Length > 50)
                description = description.Substring(0, 47) + "...";

            using (var descBrush = new SolidBrush(Theme.Dark.TextMuted))
            {
                e.Graphics.DrawString(description, new Font("Segoe UI", 9F), descBrush, bounds.X + 48, bounds.Y + 34);
            }
        }

        private void LstChannels_SelectedIndexChanged(object? sender, EventArgs e)
        {
            btnJoin.Enabled = lstChannels.SelectedIndex >= 0;
        }

        private async void LstChannels_DoubleClick(object? sender, EventArgs e)
        {
            await JoinSelectedChannelAsync();
        }

        private async void BtnJoin_Click(object? sender, EventArgs e)
        {
            await JoinSelectedChannelAsync();
        }

        private async Task JoinSelectedChannelAsync()
        {
            if (lstChannels.SelectedIndex < 0 || lstChannels.SelectedIndex >= _filteredChannels.Count)
                return;

            var channel = _filteredChannels[lstChannels.SelectedIndex];
            btnJoin.Enabled = false;
            btnJoin.Text = "Joining...";

            try
            {
                var joinedChannel = await _apiClient.JoinChannelAsync(channel.Id);
                if (joinedChannel != null)
                {
                    _onChannelJoined(joinedChannel);
                    MessageBox.Show($"Successfully joined #{channel.Name}!", "Success", MessageBoxButtons.OK, MessageBoxIcon.Information);

                    // Remove from list since we've joined
                    _publicChannels.Remove(channel);
                    ApplyFilter();
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to join channel: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            finally
            {
                btnJoin.Text = "Join Channel";
                btnJoin.Enabled = lstChannels.SelectedIndex >= 0;
            }
        }

        private void PnlSearchWrapper_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = ((Panel)sender!).ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            using var path = GetRoundedRectPath(rect, 6);
            using var pen = new Pen(Theme.Dark.InputBorder, 1);
            e.Graphics.DrawPath(pen, path);
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
    }
}
