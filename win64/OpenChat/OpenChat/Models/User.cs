using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class User
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("org_id")]
        public Guid OrgId { get; set; }

        [JsonProperty("tv_user_id")]
        public Guid TvUserId { get; set; }

        [JsonProperty("email")]
        public string Email { get; set; } = string.Empty;

        [JsonProperty("display_name")]
        public string DisplayName { get; set; } = string.Empty;

        [JsonProperty("avatar_url")]
        public string? AvatarUrl { get; set; }

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }

        [JsonProperty("updated_at")]
        public DateTime UpdatedAt { get; set; }
    }

    public class UserStatus
    {
        [JsonProperty("user_id")]
        public Guid UserId { get; set; }

        [JsonProperty("status")]
        public string Status { get; set; } = "offline";

        [JsonProperty("last_seen_at")]
        public DateTime? LastSeenAt { get; set; }
    }
}
