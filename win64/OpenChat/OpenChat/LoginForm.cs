using OpenChat.Services;

namespace OpenChat
{
    public partial class LoginForm : Form
    {
        private readonly ApiClient _apiClient;
        private TextBox txtPairingCode;
        private Button btnLogin;
        private Label lblInstructions;
        private Label lblTitle;
        private Label lblSubtitle;
        private Panel pnlHeader;
        private Panel pnlMain;
        private Label lblStep1;
        private Label lblStep2;
        private Label lblStep3;
        private Label lblStep4;
        private Label lblCodeLabel;

        public LoginForm()
        {
            InitializeComponent();
            _apiClient = new ApiClient("https://openchat-api.zerosandones.us:9876");
        }

        private void InitializeComponent()
        {
            this.Text = "OpenChat - Login";
            this.Size = new Size(600, 700);
            this.StartPosition = FormStartPosition.CenterScreen;
            this.FormBorderStyle = FormBorderStyle.FixedDialog;
            this.MaximizeBox = false;
            this.MinimizeBox = false;
            this.BackColor = Color.FromArgb(245, 245, 245);

            // Header Panel
            pnlHeader = new Panel
            {
                Dock = DockStyle.Top,
                Height = 180,
                BackColor = Color.FromArgb(0, 120, 212),
                Padding = new Padding(0)
            };

            // Title
            lblTitle = new Label
            {
                Text = "OpenChat",
                Font = new Font("Segoe UI", 36, FontStyle.Bold),
                Location = new Point(0, 40),
                Size = new Size(600, 60),
                TextAlign = ContentAlignment.MiddleCenter,
                ForeColor = Color.White,
                BackColor = Color.Transparent
            };

            // Subtitle
            lblSubtitle = new Label
            {
                Text = "Desktop Application",
                Font = new Font("Segoe UI", 14, FontStyle.Regular),
                Location = new Point(0, 105),
                Size = new Size(600, 30),
                TextAlign = ContentAlignment.MiddleCenter,
                ForeColor = Color.FromArgb(220, 235, 255),
                BackColor = Color.Transparent
            };

            pnlHeader.Controls.Add(lblTitle);
            pnlHeader.Controls.Add(lblSubtitle);

            // Main Panel
            pnlMain = new Panel
            {
                Location = new Point(40, 200),
                Size = new Size(520, 450),
                BackColor = Color.White,
                Padding = new Padding(30)
            };

            // Add shadow effect
            pnlMain.Paint += (s, e) =>
            {
                var rect = pnlMain.ClientRectangle;
                using (var pen = new Pen(Color.FromArgb(30, 0, 0, 0), 1))
                {
                    e.Graphics.DrawRectangle(pen, 0, 0, rect.Width - 1, rect.Height - 1);
                }
            };

            // Instructions header
            lblInstructions = new Label
            {
                Text = "Connect Your Account",
                Font = new Font("Segoe UI", 18, FontStyle.Bold),
                Location = new Point(30, 30),
                Size = new Size(460, 35),
                TextAlign = ContentAlignment.TopLeft,
                ForeColor = Color.FromArgb(30, 30, 30)
            };

            // Step 1
            lblStep1 = new Label
            {
                Text = "1. Open the OpenChat web app",
                Font = new Font("Segoe UI", 11),
                Location = new Point(30, 90),
                Size = new Size(460, 25),
                ForeColor = Color.FromArgb(70, 70, 70)
            };

            // Web URL as clickable link
            var linkWebApp = new LinkLabel
            {
                Text = "https://openchat.zerosandones.us",
                Font = new Font("Segoe UI", 10),
                Location = new Point(50, 115),
                Size = new Size(440, 20),
                LinkColor = Color.FromArgb(0, 120, 212),
                ActiveLinkColor = Color.FromArgb(0, 100, 200),
                VisitedLinkColor = Color.FromArgb(0, 120, 212)
            };
            linkWebApp.LinkClicked += (s, e) =>
            {
                System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
                {
                    FileName = "https://openchat.zerosandones.us",
                    UseShellExecute = true
                });
            };

            // Step 2
            lblStep2 = new Label
            {
                Text = "2. Log in with your TitaniumVault account",
                Font = new Font("Segoe UI", 11),
                Location = new Point(30, 150),
                Size = new Size(460, 25),
                ForeColor = Color.FromArgb(70, 70, 70)
            };

            // Step 3
            lblStep3 = new Label
            {
                Text = "3. Click your profile → Pair Desktop App",
                Font = new Font("Segoe UI", 11),
                Location = new Point(30, 180),
                Size = new Size(460, 25),
                ForeColor = Color.FromArgb(70, 70, 70)
            };

            // Step 4
            lblStep4 = new Label
            {
                Text = "4. Enter the pairing code below:",
                Font = new Font("Segoe UI", 11),
                Location = new Point(30, 210),
                Size = new Size(460, 25),
                ForeColor = Color.FromArgb(70, 70, 70)
            };

            // Code label
            lblCodeLabel = new Label
            {
                Text = "Pairing Code",
                Font = new Font("Segoe UI", 11, FontStyle.Bold),
                Location = new Point(30, 255),
                Size = new Size(460, 25),
                ForeColor = Color.FromArgb(30, 30, 30)
            };

            // Pairing code input
            txtPairingCode = new TextBox
            {
                Location = new Point(30, 285),
                Size = new Size(460, 45),
                Font = new Font("Segoe UI", 18, FontStyle.Bold),
                TextAlign = HorizontalAlignment.Center,
                CharacterCasing = CharacterCasing.Upper,
                MaxLength = 8,
                BorderStyle = BorderStyle.FixedSingle
            };

            // Add placeholder text behavior
            txtPairingCode.ForeColor = Color.Gray;
            txtPairingCode.Text = "Enter code";
            txtPairingCode.Enter += (s, e) =>
            {
                if (txtPairingCode.Text == "Enter code")
                {
                    txtPairingCode.Text = "";
                    txtPairingCode.ForeColor = Color.Black;
                }
            };
            txtPairingCode.Leave += (s, e) =>
            {
                if (string.IsNullOrWhiteSpace(txtPairingCode.Text))
                {
                    txtPairingCode.Text = "Enter code";
                    txtPairingCode.ForeColor = Color.Gray;
                }
            };

            // Login button
            btnLogin = new Button
            {
                Text = "Connect",
                Location = new Point(30, 355),
                Size = new Size(460, 50),
                Font = new Font("Segoe UI", 14, FontStyle.Bold),
                BackColor = Color.FromArgb(0, 120, 212),
                ForeColor = Color.White,
                FlatStyle = FlatStyle.Flat,
                Cursor = Cursors.Hand
            };
            btnLogin.FlatAppearance.BorderSize = 0;
            btnLogin.FlatAppearance.MouseOverBackColor = Color.FromArgb(0, 100, 200);
            btnLogin.FlatAppearance.MouseDownBackColor = Color.FromArgb(0, 90, 180);
            btnLogin.Click += BtnLogin_Click;

            pnlMain.Controls.Add(lblInstructions);
            pnlMain.Controls.Add(lblStep1);
            pnlMain.Controls.Add(linkWebApp);
            pnlMain.Controls.Add(lblStep2);
            pnlMain.Controls.Add(lblStep3);
            pnlMain.Controls.Add(lblStep4);
            pnlMain.Controls.Add(lblCodeLabel);
            pnlMain.Controls.Add(txtPairingCode);
            pnlMain.Controls.Add(btnLogin);

            this.Controls.Add(pnlMain);
            this.Controls.Add(pnlHeader);
        }

        private async void BtnLogin_Click(object? sender, EventArgs e)
        {
            var code = txtPairingCode.Text.Trim();
            if (string.IsNullOrEmpty(code) || code == "Enter code")
            {
                MessageBox.Show("Please enter a pairing code.", "Pairing Code Required", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                txtPairingCode.Focus();
                return;
            }

            btnLogin.Enabled = false;
            btnLogin.Text = "Connecting...";
            txtPairingCode.Enabled = false;

            try
            {
                var response = await _apiClient.VerifyCodeAsync(code, "Windows Desktop");

                // Save the token securely (for now, just in memory)
                AppSettings.AccessToken = response.AccessToken;
                AppSettings.CurrentUser = response.User;

                this.DialogResult = DialogResult.OK;
                this.Close();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Unable to connect:\n\n{ex.Message}\n\nPlease check your pairing code and try again.",
                    "Connection Failed", MessageBoxButtons.OK, MessageBoxIcon.Error);
                btnLogin.Enabled = true;
                btnLogin.Text = "Connect";
                txtPairingCode.Enabled = true;
                txtPairingCode.Focus();
                txtPairingCode.SelectAll();
            }
        }
    }

    // Simple settings class to store app state
    public static class AppSettings
    {
        public static string? AccessToken { get; set; }
        public static OpenChat.Models.UserInfo? CurrentUser { get; set; }
    }
}
