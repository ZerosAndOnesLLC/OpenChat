using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;

namespace OpenChat
{
    public class NewDmForm : Form
    {
        private readonly ApiClient _apiClient;
        private readonly Guid _currentUserId;
        private readonly Action<DirectMessage> _onDmCreated;
        private List<User> _users = new();
        private List<User> _filteredUsers = new();

        private TextBox txtSearch = null!;
        private ListBox lstUsers = null!;
        private Button btnMessage = null!;
        private Label lblStatus = null!;

        public NewDmForm(ApiClient apiClient, Guid currentUserId, Action<DirectMessage> onDmCreated)
        {
            _apiClient = apiClient;
            _currentUserId = currentUserId;
            _onDmCreated = onDmCreated;

            InitializeComponent();
            _ = LoadUsersAsync();
        }

        private void InitializeComponent()
        {
            SuspendLayout();

            Text = "New Direct Message";
            Size = new Size(400, 450);
            StartPosition = FormStartPosition.CenterParent;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            BackColor = Theme.Dark.ContentBackground;
            ForeColor = Theme.Dark.TextPrimary;

            // Header
            var lblHeader = new Label
            {
                Text = "Start a Conversation",
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
                PlaceholderText = "Search users..."
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

            // Users list
            lstUsers = new ListBox
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.ContentBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = new Font("Segoe UI", 10F),
                BorderStyle = BorderStyle.None,
                DrawMode = DrawMode.OwnerDrawFixed,
                ItemHeight = 48,
                IntegralHeight = false
            };
            lstUsers.DrawItem += LstUsers_DrawItem;
            lstUsers.SelectedIndexChanged += LstUsers_SelectedIndexChanged;
            lstUsers.DoubleClick += LstUsers_DoubleClick;

            var pnlList = new Panel
            {
                Dock = DockStyle.Fill,
                Padding = new Padding(10, 0, 10, 0)
            };
            pnlList.Controls.Add(lstUsers);

            // Buttons panel
            var pnlButtons = new Panel
            {
                Dock = DockStyle.Bottom,
                Height = 60,
                Padding = new Padding(20, 10, 20, 15)
            };

            btnMessage = new Button
            {
                Text = "Start Conversation",
                Dock = DockStyle.Right,
                Width = 140,
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonPrimary,
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 10F, FontStyle.Bold),
                Cursor = Cursors.Hand,
                Enabled = false
            };
            btnMessage.FlatAppearance.BorderSize = 0;
            btnMessage.Click += BtnMessage_Click;

            var btnClose = new Button
            {
                Text = "Cancel",
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

            pnlButtons.Controls.Add(btnMessage);
            pnlButtons.Controls.Add(btnClose);

            // Add controls
            Controls.Add(pnlList);
            Controls.Add(lblStatus);
            Controls.Add(pnlSearch);
            Controls.Add(lblHeader);
            Controls.Add(pnlButtons);

            ResumeLayout(false);
        }

        private async Task LoadUsersAsync()
        {
            try
            {
                var allUsers = await _apiClient.GetUsersAsync();
                // Exclude current user
                _users = allUsers.Where(u => u.Id != _currentUserId).ToList();
                ApplyFilter();
                UpdateStatus();
            }
            catch (Exception ex)
            {
                lblStatus.Text = $"Failed to load users: {ex.Message}";
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
                _filteredUsers = _users.ToList();
            }
            else
            {
                _filteredUsers = _users
                    .Where(u => u.DisplayName.ToLowerInvariant().Contains(query) ||
                               u.Email.ToLowerInvariant().Contains(query))
                    .ToList();
            }

            lstUsers.Items.Clear();
            foreach (var user in _filteredUsers)
            {
                lstUsers.Items.Add(user.DisplayName);
            }

            UpdateStatus();
        }

        private void UpdateStatus()
        {
            if (_users.Count == 0)
            {
                lblStatus.Text = "No users found in your organization";
            }
            else if (_filteredUsers.Count == 0)
            {
                lblStatus.Text = "No users match your search";
            }
            else
            {
                lblStatus.Text = $"{_filteredUsers.Count} user{(_filteredUsers.Count != 1 ? "s" : "")} found";
            }
        }

        private void LstUsers_DrawItem(object? sender, DrawItemEventArgs e)
        {
            if (e.Index < 0 || e.Index >= _filteredUsers.Count) return;

            var user = _filteredUsers[e.Index];
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

            // Avatar circle
            var avatarRect = new Rectangle(bounds.X + 16, bounds.Y + 10, 28, 28);
            using (var avatarBrush = new SolidBrush(Theme.Dark.ButtonPrimary))
            {
                e.Graphics.FillEllipse(avatarBrush, avatarRect);
            }

            // Avatar initial
            var initial = user.DisplayName.Length > 0 ? user.DisplayName[0].ToString().ToUpper() : "?";
            using (var initialBrush = new SolidBrush(Color.White))
            {
                var sf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };
                e.Graphics.DrawString(initial, new Font("Segoe UI", 11F, FontStyle.Bold), initialBrush, avatarRect, sf);
            }

            // User name
            using (var nameBrush = new SolidBrush(Theme.Dark.TextWhite))
            {
                e.Graphics.DrawString(user.DisplayName, new Font("Segoe UI", 10F, FontStyle.Bold), nameBrush, bounds.X + 56, bounds.Y + 8);
            }

            // Email
            using (var emailBrush = new SolidBrush(Theme.Dark.TextMuted))
            {
                e.Graphics.DrawString(user.Email, new Font("Segoe UI", 9F), emailBrush, bounds.X + 56, bounds.Y + 27);
            }
        }

        private void LstUsers_SelectedIndexChanged(object? sender, EventArgs e)
        {
            btnMessage.Enabled = lstUsers.SelectedIndex >= 0;
        }

        private async void LstUsers_DoubleClick(object? sender, EventArgs e)
        {
            await CreateDmAsync();
        }

        private async void BtnMessage_Click(object? sender, EventArgs e)
        {
            await CreateDmAsync();
        }

        private async Task CreateDmAsync()
        {
            if (lstUsers.SelectedIndex < 0 || lstUsers.SelectedIndex >= _filteredUsers.Count)
                return;

            var user = _filteredUsers[lstUsers.SelectedIndex];
            btnMessage.Enabled = false;
            btnMessage.Text = "Creating...";

            try
            {
                var dm = await _apiClient.CreateDmAsync(new List<Guid> { user.Id });
                if (dm != null)
                {
                    dm.OtherUser = user;
                    _onDmCreated(dm);
                    Close();
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to create conversation: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                btnMessage.Text = "Start Conversation";
                btnMessage.Enabled = lstUsers.SelectedIndex >= 0;
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
