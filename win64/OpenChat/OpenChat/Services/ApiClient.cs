using Newtonsoft.Json;
using OpenChat.Models;
using System.Text;

namespace OpenChat.Services
{
    public class ApiClient
    {
        private readonly HttpClient _httpClient;
        private string? _accessToken;
        private UserInfo? _currentUser;

        public string? AccessToken => _accessToken;
        public UserInfo? CurrentUser => _currentUser;

        public ApiClient(string baseUrl)
        {
            _httpClient = new HttpClient
            {
                BaseAddress = new Uri(baseUrl)
            };
        }

        public void SetAccessToken(string token)
        {
            _accessToken = token;
            _httpClient.DefaultRequestHeaders.Remove("Authorization");
            _httpClient.DefaultRequestHeaders.Add("Authorization", $"Bearer {token}");
        }

        public void SetCurrentUser(UserInfo user)
        {
            _currentUser = user;
        }

        // Authentication
        public async Task<VerifyCodeResponse> VerifyCodeAsync(string code, string deviceName)
        {
            var request = new
            {
                code = code,
                device_name = deviceName,
                device_fingerprint = Environment.MachineName
            };

            var response = await PostAsync<VerifyCodeResponse>("/api/auth/device/verify-code", request);
            if (response != null)
            {
                SetAccessToken(response.AccessToken);
                SetCurrentUser(response.User);
            }
            return response!;
        }

        // Channels
        public async Task<List<Channel>> GetChannelsAsync()
        {
            return await GetAsync<List<Channel>>("/api/channels") ?? new List<Channel>();
        }

        public async Task<Channel?> GetChannelAsync(Guid channelId)
        {
            return await GetAsync<Channel>($"/api/channels/{channelId}");
        }

        public async Task<List<Models.Message>> GetChannelMessagesAsync(Guid channelId, int limit = 50, Guid? before = null)
        {
            var url = $"/api/channels/{channelId}/messages?limit={limit}";
            if (before.HasValue)
            {
                url += $"&before={before.Value}";
            }
            return await GetAsync<List<Models.Message>>(url) ?? new List<Models.Message>();
        }

        // Direct Messages
        public async Task<List<DirectMessage>> GetDirectMessagesAsync()
        {
            return await GetAsync<List<DirectMessage>>("/api/dms") ?? new List<DirectMessage>();
        }

        public async Task<List<Models.Message>> GetDmMessagesAsync(Guid dmId, int limit = 50, Guid? before = null)
        {
            var url = $"/api/dms/{dmId}/messages?limit={limit}";
            if (before.HasValue)
            {
                url += $"&before={before.Value}";
            }
            return await GetAsync<List<Models.Message>>(url) ?? new List<Models.Message>();
        }

        // Messages
        public async Task<Models.Message?> SendMessageAsync(Guid? channelId, Guid? dmId, string content)
        {
            var request = new
            {
                channel_id = channelId,
                dm_id = dmId,
                content = content
            };

            return await PostAsync<Models.Message>("/api/messages", request);
        }

        // Users
        public async Task<List<User>> GetUsersAsync()
        {
            return await GetAsync<List<User>>("/api/users") ?? new List<User>();
        }

        // Generic HTTP methods
        private async Task<T?> GetAsync<T>(string endpoint)
        {
            try
            {
                var response = await _httpClient.GetAsync(endpoint);
                response.EnsureSuccessStatusCode();
                var content = await response.Content.ReadAsStringAsync();
                return JsonConvert.DeserializeObject<T>(content);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"GET {endpoint} failed: {ex.Message}");
                throw;
            }
        }

        private async Task<T?> PostAsync<T>(string endpoint, object data)
        {
            try
            {
                var json = JsonConvert.SerializeObject(data);
                var content = new StringContent(json, Encoding.UTF8, "application/json");
                var response = await _httpClient.PostAsync(endpoint, content);
                response.EnsureSuccessStatusCode();
                var responseContent = await response.Content.ReadAsStringAsync();
                return JsonConvert.DeserializeObject<T>(responseContent);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"POST {endpoint} failed: {ex.Message}");
                throw;
            }
        }
    }
}
