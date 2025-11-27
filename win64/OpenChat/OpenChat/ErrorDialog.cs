using System.Drawing.Drawing2D;

namespace OpenChat
{
    public class ErrorDialog : Form
    {
        private readonly string _fullError;
        private Label lblTitle;
        private Label lblMessage;
        private TextBox txtDetails;
        private Button btnCopy;
        private Button btnClose;
        private Panel pnlButtons;
        private LinkLabel lnkShowDetails;
        private bool _detailsVisible;

        public ErrorDialog(string title, string message, string? details = null)
        {
            _fullError = string.IsNullOrEmpty(details) ? message : $"{message}\n\nDetails:\n{details}";
            InitializeComponent(title, message, details);
        }

        private void InitializeComponent(string title, string message, string? details)
        {
            SuspendLayout();

            // Form settings
            Text = title;
            Size = new Size(480, 220);
            MinimumSize = new Size(400, 200);
            StartPosition = FormStartPosition.CenterParent;
            FormBorderStyle = FormBorderStyle.FixedDialog;
            MaximizeBox = false;
            MinimizeBox = false;
            BackColor = Theme.Dark.ContentBackground;
            ForeColor = Theme.Dark.TextPrimary;

            // Title label with icon
            lblTitle = new Label
            {
                Text = "Something went wrong",
                Font = new Font("Segoe UI", 14F, FontStyle.Bold),
                ForeColor = Theme.Dark.TextWhite,
                Location = new Point(20, 20),
                Size = new Size(440, 30),
                AutoSize = false
            };

            // Message label
            lblMessage = new Label
            {
                Text = TruncateMessage(message),
                Font = new Font("Segoe UI", 10F),
                ForeColor = Theme.Dark.TextSecondary,
                Location = new Point(20, 55),
                Size = new Size(440, 60),
                AutoSize = false
            };

            // Show details link
            lnkShowDetails = new LinkLabel
            {
                Text = "Show details",
                Font = new Font("Segoe UI", 9F),
                LinkColor = Theme.Dark.AccentGreen,
                ActiveLinkColor = Theme.Dark.ButtonPrimaryHover,
                VisitedLinkColor = Theme.Dark.AccentGreen,
                Location = new Point(20, 120),
                Size = new Size(100, 20),
                Visible = !string.IsNullOrEmpty(details)
            };
            lnkShowDetails.LinkClicked += LnkShowDetails_LinkClicked;

            // Details text box (hidden by default)
            txtDetails = new TextBox
            {
                Text = details ?? message,
                Font = new Font("Consolas", 9F),
                BackColor = Theme.Dark.InputBackground,
                ForeColor = Theme.Dark.TextPrimary,
                BorderStyle = BorderStyle.None,
                Multiline = true,
                ReadOnly = true,
                ScrollBars = ScrollBars.Vertical,
                Location = new Point(20, 145),
                Size = new Size(440, 150),
                Visible = false
            };

            // Button panel
            pnlButtons = new Panel
            {
                Dock = DockStyle.Bottom,
                Height = 60,
                BackColor = Theme.Dark.SidebarBackground,
                Padding = new Padding(20, 10, 20, 10)
            };

            // Copy button
            btnCopy = new Button
            {
                Text = "Copy Error",
                Font = new Font("Segoe UI", 10F),
                Size = new Size(110, 36),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonSecondary,
                ForeColor = Theme.Dark.TextPrimary,
                Cursor = Cursors.Hand,
                Location = new Point(20, 12)
            };
            btnCopy.FlatAppearance.BorderSize = 0;
            btnCopy.Click += BtnCopy_Click;

            // Close button
            btnClose = new Button
            {
                Text = "Close",
                Font = new Font("Segoe UI", 10F, FontStyle.Bold),
                Size = new Size(100, 36),
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Dark.ButtonPrimary,
                ForeColor = Color.White,
                Cursor = Cursors.Hand,
                Location = new Point(350, 12)
            };
            btnClose.FlatAppearance.BorderSize = 0;
            btnClose.FlatAppearance.MouseOverBackColor = Theme.Dark.ButtonPrimaryHover;
            btnClose.Click += (s, e) => Close();

            pnlButtons.Controls.Add(btnCopy);
            pnlButtons.Controls.Add(btnClose);

            Controls.Add(lblTitle);
            Controls.Add(lblMessage);
            Controls.Add(lnkShowDetails);
            Controls.Add(txtDetails);
            Controls.Add(pnlButtons);

            AcceptButton = btnClose;
            CancelButton = btnClose;

            ResumeLayout(false);
        }

        private static string TruncateMessage(string message)
        {
            // Get just the first line or first 200 chars
            var firstLine = message.Split('\n')[0];
            if (firstLine.Length > 200)
            {
                return firstLine[..197] + "...";
            }
            return firstLine;
        }

        private void LnkShowDetails_LinkClicked(object? sender, LinkLabelLinkClickedEventArgs e)
        {
            _detailsVisible = !_detailsVisible;
            txtDetails.Visible = _detailsVisible;
            lnkShowDetails.Text = _detailsVisible ? "Hide details" : "Show details";

            // Resize form
            if (_detailsVisible)
            {
                Height = 420;
            }
            else
            {
                Height = 220;
            }
        }

        private void BtnCopy_Click(object? sender, EventArgs e)
        {
            try
            {
                Clipboard.SetText(_fullError);
                btnCopy.Text = "Copied!";
                btnCopy.BackColor = Theme.Dark.AccentGreen;
                btnCopy.ForeColor = Color.White;

                // Reset after 2 seconds
                var timer = new System.Windows.Forms.Timer { Interval = 2000 };
                timer.Tick += (s, args) =>
                {
                    btnCopy.Text = "Copy Error";
                    btnCopy.BackColor = Theme.Dark.ButtonSecondary;
                    btnCopy.ForeColor = Theme.Dark.TextPrimary;
                    timer.Stop();
                    timer.Dispose();
                };
                timer.Start();
            }
            catch
            {
                // Clipboard might fail in some scenarios
            }
        }

        public static void Show(string title, string message, string? details = null)
        {
            using var dialog = new ErrorDialog(title, message, details);
            dialog.ShowDialog();
        }

        public static void Show(IWin32Window owner, string title, string message, string? details = null)
        {
            using var dialog = new ErrorDialog(title, message, details);
            dialog.ShowDialog(owner);
        }
    }
}
