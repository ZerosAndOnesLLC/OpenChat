using OpenChat.Models;
using OpenChat.Services;
using System.Drawing.Drawing2D;

namespace OpenChat
{
    public class StatusPickerForm : Form
    {
        private readonly ApiClient _apiClient;
        private readonly Action<UserStatus> _onStatusChanged;
        private UserStatus? _currentStatus;

        private Panel pnlMain = null!;
        private TextBox txtCustomMessage = null!;
        private ComboBox cmbClearAfter = null!;
        private string _selectedStatus = "online";

        private static readonly (string Status, string Label, string Description, Color Color)[] StatusOptions = new[]
        {
            ("online", "Online", "Available to chat", Color.FromArgb(46, 182, 125)),
            ("away", "Away", "Stepped away", Color.FromArgb(250, 168, 26)),
            ("dnd", "Do Not Disturb", "No notifications", Color.FromArgb(237, 66, 69)),
            ("offline", "Invisible", "Appear offline", Color.FromArgb(116, 116, 116))
        };

        private static readonly (int Minutes, string Label)[] ClearOptions = new[]
        {
            (0, "Don't clear"),
            (30, "30 minutes"),
            (60, "1 hour"),
            (240, "4 hours"),
            (480, "8 hours"),
            (1440, "24 hours")
        };

        public StatusPickerForm(ApiClient apiClient, UserStatus? currentStatus, Action<UserStatus> onStatusChanged)
        {
            _apiClient = apiClient;
            _currentStatus = currentStatus;
            _onStatusChanged = onStatusChanged;
            _selectedStatus = currentStatus?.Status ?? "online";

            InitializeComponent();
            LoadCurrentStatus();
        }

        private void InitializeComponent()
        {
            SuspendLayout();

            FormBorderStyle = FormBorderStyle.None;
            StartPosition = FormStartPosition.Manual;
            Size = new Size(280, 340);
            BackColor = Theme.Dark.EmojiPickerBackground;
            ShowInTaskbar = false;
            TopMost = true;

            pnlMain = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.EmojiPickerBackground,
                Padding = new Padding(1)
            };
            pnlMain.Paint += PnlMain_Paint;

            // Header
            var lblHeader = new Label
            {
                Text = "Set your status",
                Font = new Font("Segoe UI", 11F, FontStyle.Bold),
                ForeColor = Theme.Dark.TextWhite,
                Dock = DockStyle.Top,
                Height = 40,
                Padding = new Padding(16, 12, 16, 8)
            };

            // Status options panel
            var pnlStatusOptions = new Panel
            {
                Dock = DockStyle.Top,
                Height = 140,
                Padding = new Padding(8, 0, 8, 8)
            };

            int yOffset = 0;
            foreach (var option in StatusOptions)
            {
                var btn = CreateStatusButton(option.Status, option.Label, option.Description, option.Color);
                btn.Location = new Point(8, yOffset);
                btn.Size = new Size(pnlStatusOptions.Width - 16, 32);
                btn.Anchor = AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right;
                pnlStatusOptions.Controls.Add(btn);
                yOffset += 34;
            }

            // Divider
            var divider = new Panel
            {
                Dock = DockStyle.Top,
                Height = 1,
                BackColor = Theme.Dark.DividerColor,
                Margin = new Padding(16, 8, 16, 8)
            };

            // Custom message section
            var lblCustomMessage = new Label
            {
                Text = "Custom status message",
                Font = new Font("Segoe UI", 9F),
                ForeColor = Theme.Dark.TextSecondary,
                Dock = DockStyle.Top,
                Height = 24,
                Padding = new Padding(16, 8, 16, 0)
            };

            var pnlCustomInput = new Panel
            {
                Dock = DockStyle.Top,
                Height = 40,
                Padding = new Padding(16, 4, 16, 4)
            };

            txtCustomMessage = new TextBox
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.InputBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = new Font("Segoe UI", 10F),
                BorderStyle = BorderStyle.None,
                PlaceholderText = "What's your status?"
            };

            var pnlInputWrapper = new Panel
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.InputBackground,
                Padding = new Padding(8, 6, 8, 6)
            };
            pnlInputWrapper.Paint += PnlInputWrapper_Paint;
            pnlInputWrapper.Controls.Add(txtCustomMessage);
            pnlCustomInput.Controls.Add(pnlInputWrapper);

            // Clear after section
            var lblClearAfter = new Label
            {
                Text = "Clear after",
                Font = new Font("Segoe UI", 9F),
                ForeColor = Theme.Dark.TextSecondary,
                Dock = DockStyle.Top,
                Height = 24,
                Padding = new Padding(16, 8, 16, 0)
            };

            var pnlClearAfter = new Panel
            {
                Dock = DockStyle.Top,
                Height = 36,
                Padding = new Padding(16, 4, 16, 4)
            };

            cmbClearAfter = new ComboBox
            {
                Dock = DockStyle.Fill,
                BackColor = Theme.Dark.InputBackground,
                ForeColor = Theme.Dark.TextPrimary,
                Font = new Font("Segoe UI", 10F),
                FlatStyle = FlatStyle.Flat,
                DropDownStyle = ComboBoxStyle.DropDownList
            };
            foreach (var option in ClearOptions)
            {
                cmbClearAfter.Items.Add(option.Label);
            }
            cmbClearAfter.SelectedIndex = 0;
            pnlClearAfter.Controls.Add(cmbClearAfter);

            // Buttons panel
            var pnlButtons = new Panel
            {
                Dock = DockStyle.Bottom,
                Height = 50,
                Padding = new Padding(16, 8, 16, 12)
            };

            var btnSave = new Button
            {
                Text = "Save",
                Dock = DockStyle.Right,
                Width = 80,
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonPrimary,
                ForeColor = Color.White,
                Font = new Font("Segoe UI", 10F, FontStyle.Bold),
                Cursor = Cursors.Hand
            };
            btnSave.FlatAppearance.BorderSize = 0;
            btnSave.Click += BtnSave_Click;

            var btnCancel = new Button
            {
                Text = "Cancel",
                Dock = DockStyle.Right,
                Width = 80,
                FlatStyle = FlatStyle.Flat,
                BackColor = Color.Transparent,
                ForeColor = Theme.Dark.TextSecondary,
                Font = new Font("Segoe UI", 10F),
                Cursor = Cursors.Hand,
                Margin = new Padding(0, 0, 8, 0)
            };
            btnCancel.FlatAppearance.BorderSize = 0;
            btnCancel.Click += (s, e) => Close();

            pnlButtons.Controls.Add(btnSave);
            pnlButtons.Controls.Add(btnCancel);

            // Add controls in reverse order (bottom to top for dock)
            pnlMain.Controls.Add(pnlButtons);
            pnlMain.Controls.Add(pnlClearAfter);
            pnlMain.Controls.Add(lblClearAfter);
            pnlMain.Controls.Add(pnlCustomInput);
            pnlMain.Controls.Add(lblCustomMessage);
            pnlMain.Controls.Add(divider);
            pnlMain.Controls.Add(pnlStatusOptions);
            pnlMain.Controls.Add(lblHeader);

            Controls.Add(pnlMain);
            ResumeLayout(false);
        }

        private Panel CreateStatusButton(string status, string label, string description, Color statusColor)
        {
            var panel = new Panel
            {
                Height = 32,
                Cursor = Cursors.Hand,
                Tag = status
            };

            panel.Paint += (s, e) =>
            {
                e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

                var isSelected = _selectedStatus == status;
                var isHovered = panel.ClientRectangle.Contains(panel.PointToClient(Cursor.Position));

                if (isSelected || isHovered)
                {
                    using var bgBrush = new SolidBrush(isSelected ? Theme.Dark.SelectedBackground : Theme.Dark.HoverBackground);
                    using var path = GetRoundedRectPath(panel.ClientRectangle, 6);
                    e.Graphics.FillPath(bgBrush, path);
                }

                // Status indicator
                using var statusBrush = new SolidBrush(statusColor);
                e.Graphics.FillEllipse(statusBrush, 12, 10, 12, 12);

                // Label
                using var labelBrush = new SolidBrush(Theme.Dark.TextWhite);
                e.Graphics.DrawString(label, new Font("Segoe UI", 10F), labelBrush, 32, 6);

                // Checkmark if selected
                if (isSelected)
                {
                    using var checkBrush = new SolidBrush(Theme.Dark.ButtonPrimary);
                    e.Graphics.DrawString("✓", new Font("Segoe UI", 10F, FontStyle.Bold), checkBrush, panel.Width - 28, 6);
                }
            };

            panel.MouseEnter += (s, e) => panel.Invalidate();
            panel.MouseLeave += (s, e) => panel.Invalidate();
            panel.Click += (s, e) =>
            {
                _selectedStatus = status;
                pnlMain.Invalidate(true);
                foreach (Control c in panel.Parent!.Controls)
                {
                    c.Invalidate();
                }
            };

            return panel;
        }

        private void LoadCurrentStatus()
        {
            if (_currentStatus != null)
            {
                _selectedStatus = _currentStatus.Status;
                txtCustomMessage.Text = _currentStatus.CustomMessage ?? "";
            }
        }

        private async void BtnSave_Click(object? sender, EventArgs e)
        {
            try
            {
                var clearMinutes = ClearOptions[cmbClearAfter.SelectedIndex].Minutes;
                var request = new UpdateStatusRequest
                {
                    Status = _selectedStatus,
                    CustomMessage = string.IsNullOrWhiteSpace(txtCustomMessage.Text) ? null : txtCustomMessage.Text.Trim(),
                    ClearAfterMinutes = clearMinutes > 0 ? clearMinutes : null
                };

                var result = await _apiClient.UpdateMyStatusAsync(request);
                if (result != null)
                {
                    _onStatusChanged(result);
                }
                Close();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Failed to update status: {ex.Message}", "Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private void PnlMain_Paint(object? sender, PaintEventArgs e)
        {
            using var pen = new Pen(Theme.Dark.EmojiPickerBorder, 1);
            var rect = pnlMain.ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            using var path = GetRoundedRectPath(rect, 8);
            e.Graphics.DrawPath(pen, path);
        }

        private void PnlInputWrapper_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = ((Panel)sender!).ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            using var path = GetRoundedRectPath(rect, 4);
            using var pen = new Pen(Theme.Dark.InputBorder, 1);
            e.Graphics.DrawPath(pen, path);
        }

        protected override void OnDeactivate(EventArgs e)
        {
            base.OnDeactivate(e);
            Close();
        }

        protected override CreateParams CreateParams
        {
            get
            {
                var cp = base.CreateParams;
                cp.ExStyle |= 0x00000080; // WS_EX_TOOLWINDOW
                return cp;
            }
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
