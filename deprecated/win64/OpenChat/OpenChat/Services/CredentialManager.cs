using Newtonsoft.Json;
using OpenChat.Models;
using System.Security.Cryptography;
using System.Text;

namespace OpenChat.Services
{
    /// <summary>
    /// Manages secure storage and retrieval of authentication credentials using Windows DPAPI.
    /// Credentials are stored encrypted in LocalAppData and valid for 365 days.
    /// </summary>
    public static class CredentialManager
    {
        private static readonly string CredentialDirectory = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "OpenChat"
        );

        private static readonly string CredentialFilePath = Path.Combine(CredentialDirectory, "credentials.dat");
        private const int TokenValidityDays = 365;

        /// <summary>
        /// Stored credential data structure
        /// </summary>
        private class StoredCredentials
        {
            public string AccessToken { get; set; } = string.Empty;
            public Guid DeviceId { get; set; }
            public UserInfo User { get; set; } = new UserInfo();
            public DateTime ExpiresAt { get; set; }
        }

        /// <summary>
        /// Saves credentials securely using Windows DPAPI encryption.
        /// </summary>
        public static void SaveCredentials(string accessToken, Guid deviceId, UserInfo user)
        {
            try
            {
                Directory.CreateDirectory(CredentialDirectory);

                var credentials = new StoredCredentials
                {
                    AccessToken = accessToken,
                    DeviceId = deviceId,
                    User = user,
                    ExpiresAt = DateTime.UtcNow.AddDays(TokenValidityDays)
                };

                var json = JsonConvert.SerializeObject(credentials);
                var plainBytes = Encoding.UTF8.GetBytes(json);

                // Encrypt using DPAPI (CurrentUser scope - only this Windows user can decrypt)
                var encryptedBytes = ProtectedData.Protect(
                    plainBytes,
                    null,
                    DataProtectionScope.CurrentUser
                );

                File.WriteAllBytes(CredentialFilePath, encryptedBytes);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to save credentials: {ex.Message}");
            }
        }

        /// <summary>
        /// Loads and decrypts stored credentials if they exist and are valid.
        /// Returns true if valid credentials were loaded, false otherwise.
        /// </summary>
        public static bool TryLoadCredentials(out string? accessToken, out Guid deviceId, out UserInfo? user)
        {
            accessToken = null;
            deviceId = Guid.Empty;
            user = null;

            try
            {
                if (!File.Exists(CredentialFilePath))
                {
                    return false;
                }

                var encryptedBytes = File.ReadAllBytes(CredentialFilePath);

                // Decrypt using DPAPI
                var plainBytes = ProtectedData.Unprotect(
                    encryptedBytes,
                    null,
                    DataProtectionScope.CurrentUser
                );

                var json = Encoding.UTF8.GetString(plainBytes);
                var credentials = JsonConvert.DeserializeObject<StoredCredentials>(json);

                if (credentials == null)
                {
                    return false;
                }

                // Check if credentials have expired
                if (DateTime.UtcNow >= credentials.ExpiresAt)
                {
                    ClearCredentials();
                    return false;
                }

                accessToken = credentials.AccessToken;
                deviceId = credentials.DeviceId;
                user = credentials.User;

                return !string.IsNullOrEmpty(accessToken) && user != null;
            }
            catch (CryptographicException)
            {
                // Decryption failed - credentials may be corrupted or from different user
                ClearCredentials();
                return false;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to load credentials: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Clears stored credentials (logout).
        /// </summary>
        public static void ClearCredentials()
        {
            try
            {
                if (File.Exists(CredentialFilePath))
                {
                    File.Delete(CredentialFilePath);
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to clear credentials: {ex.Message}");
            }
        }

        /// <summary>
        /// Checks if valid stored credentials exist without loading them.
        /// </summary>
        public static bool HasValidCredentials()
        {
            return TryLoadCredentials(out _, out _, out _);
        }
    }
}
