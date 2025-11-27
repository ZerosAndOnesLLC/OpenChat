using OpenChat.Services;

namespace OpenChat
{
    public partial class LoginForm : Form
    {
        private readonly ApiClient _apiClient;

        public LoginForm()
        {
            InitializeComponent();
            _apiClient = new ApiClient("https://openchat-api.zerosandones.us:9876");
            SetupPlaceholder();
        }

        private void SetupPlaceholder()
        {
            txtPairingCode.ForeColor = Color.Gray;
            txtPairingCode.Text = "Enter code";
            txtPairingCode.Enter += TxtPairingCode_Enter;
            txtPairingCode.Leave += TxtPairingCode_Leave;
        }

        private void TxtPairingCode_Enter(object? sender, EventArgs e)
        {
            if (txtPairingCode.Text == "Enter code")
            {
                txtPairingCode.Text = "";
                txtPairingCode.ForeColor = Color.Black;
            }
        }

        private void TxtPairingCode_Leave(object? sender, EventArgs e)
        {
            if (string.IsNullOrWhiteSpace(txtPairingCode.Text))
            {
                txtPairingCode.Text = "Enter code";
                txtPairingCode.ForeColor = Color.Gray;
            }
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

    public static class AppSettings
    {
        public static string? AccessToken { get; set; }
        public static OpenChat.Models.UserInfo? CurrentUser { get; set; }
    }
}
