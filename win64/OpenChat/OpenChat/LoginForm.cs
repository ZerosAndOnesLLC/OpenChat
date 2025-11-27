using OpenChat.Services;
using System.Drawing.Drawing2D;
using OpenChat.Models;

namespace OpenChat
{
    public partial class LoginForm : Form
    {
        private readonly ApiClient _apiClient;

        public LoginForm()
        {
            InitializeComponent();
            _apiClient = new ApiClient("https://openchat-api.zerosandones.us:9876");
            SetupUI();
        }

        private void SetupUI()
        {
            // Add rounded corners to the card panel
            pnlCard.Paint += PnlCard_Paint;

            // Add rounded corners and border to the text input
            txtPairingCode.Parent!.Paint += PnlCodeInput_Paint;

            // Add placeholder behavior
            txtPairingCode.GotFocus += TxtPairingCode_GotFocus;
            txtPairingCode.LostFocus += TxtPairingCode_LostFocus;
            SetPlaceholder();
        }

        private void PnlCard_Paint(object? sender, PaintEventArgs e)
        {
            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            var rect = pnlCard.ClientRectangle;
            rect.Width -= 1;
            rect.Height -= 1;
            using var path = GetRoundedRectPath(rect, 12);
            using var brush = new SolidBrush(Color.FromArgb(27, 27, 31));
            e.Graphics.FillPath(brush, path);
        }

        private void PnlCodeInput_Paint(object? sender, PaintEventArgs e)
        {
            // Draw a border around the code input area
            var inputRect = new Rectangle(
                txtPairingCode.Left - 12,
                txtPairingCode.Top - 12,
                txtPairingCode.Width + 24,
                txtPairingCode.Height + 24
            );

            e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
            using var path = GetRoundedRectPath(inputRect, 8);
            using var brush = new SolidBrush(Color.FromArgb(43, 46, 51));
            using var pen = new Pen(Color.FromArgb(62, 65, 71), 1);
            e.Graphics.FillPath(brush, path);
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

        private void SetPlaceholder()
        {
            if (string.IsNullOrEmpty(txtPairingCode.Text))
            {
                txtPairingCode.Text = "ENTER CODE";
                txtPairingCode.ForeColor = Color.FromArgb(97, 96, 97);
            }
        }

        private void TxtPairingCode_GotFocus(object? sender, EventArgs e)
        {
            if (txtPairingCode.Text == "ENTER CODE")
            {
                txtPairingCode.Text = "";
                txtPairingCode.ForeColor = Color.FromArgb(209, 210, 211);
            }
        }

        private void TxtPairingCode_LostFocus(object? sender, EventArgs e)
        {
            SetPlaceholder();
        }

        private void LinkWebApp_LinkClicked(object? sender, LinkLabelLinkClickedEventArgs e)
        {
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
            {
                FileName = "https://openchat.zerosandones.us",
                UseShellExecute = true
            });
        }

        private async void BtnLogin_Click(object? sender, EventArgs e)
        {
            var code = txtPairingCode.Text.Trim();
            if (string.IsNullOrEmpty(code) || code == "ENTER CODE")
            {
                ErrorDialog.Show(this, "Pairing Code Required", "Please enter a pairing code.");
                txtPairingCode.Focus();
                return;
            }

            btnLogin.Enabled = false;
            btnLogin.Text = "Connecting...";
            txtPairingCode.Enabled = false;

            try
            {
                var response = await _apiClient.VerifyCodeAsync(code, "Windows Desktop");

                AppSettings.AccessToken = response.AccessToken;
                AppSettings.CurrentUser = response.User;
                AppSettings.DeviceId = response.DeviceId;

                // Save credentials securely for 365 days
                CredentialManager.SaveCredentials(response.AccessToken, response.DeviceId, response.User);

                this.DialogResult = DialogResult.OK;
                this.Close();
            }
            catch (Exception ex)
            {
                ErrorDialog.Show(this, "Connection Failed", "Unable to connect. Please check your pairing code and try again.", ex.ToString());
                btnLogin.Enabled = true;
                btnLogin.Text = "Connect";
                txtPairingCode.Enabled = true;
                txtPairingCode.Focus();
                txtPairingCode.SelectAll();
            }
        }
    }

    public static class AppSettings
    {
        public static string? AccessToken { get; set; }
        public static OpenChat.Models.UserInfo? CurrentUser { get; set; }
        public static Guid DeviceId { get; set; }
    }
}
