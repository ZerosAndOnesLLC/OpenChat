using OpenChat.Services;

namespace OpenChat
{
    internal static class Program
    {
        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            // To customize application configuration such as set high DPI settings or default font,
            // see https://aka.ms/applicationconfiguration.
            ApplicationConfiguration.Initialize();

            // Try to load stored credentials first
            if (CredentialManager.TryLoadCredentials(out var accessToken, out var deviceId, out var user))
            {
                // Valid stored credentials found - skip login
                AppSettings.AccessToken = accessToken;
                AppSettings.DeviceId = deviceId;
                AppSettings.CurrentUser = user;
                Application.Run(new MainForm());
            }
            else
            {
                // No valid credentials - show login form
                using (var loginForm = new LoginForm())
                {
                    if (loginForm.ShowDialog() == DialogResult.OK)
                    {
                        // If login successful, show main chat form
                        Application.Run(new MainForm());
                    }
                }
            }
        }
    }
}