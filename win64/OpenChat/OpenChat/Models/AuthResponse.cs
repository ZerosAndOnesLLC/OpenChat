using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class VerifyCodeResponse
    {
        [JsonProperty("access_token")]
        public string AccessToken { get; set; } = string.Empty;

        [JsonProperty("user")]
        public UserInfo User { get; set; } = new UserInfo();

        [JsonProperty("device_id")]
        public Guid DeviceId { get; set; }
    }

    public class UserInfo
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("org_id")]
        public Guid OrgId { get; set; }

        [JsonProperty("email")]
        public string Email { get; set; } = string.Empty;

        [JsonProperty("display_name")]
        public string DisplayName { get; set; } = string.Empty;

        [JsonProperty("avatar_url")]
        public string? AvatarUrl { get; set; }
    }
}
