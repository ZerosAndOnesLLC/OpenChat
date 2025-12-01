using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class Channel
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("org_id")]
        public Guid OrgId { get; set; }

        [JsonProperty("name")]
        public string Name { get; set; } = string.Empty;

        [JsonProperty("description")]
        public string? Description { get; set; }

        [JsonProperty("is_private")]
        public bool IsPrivate { get; set; }

        [JsonProperty("created_by")]
        public Guid CreatedBy { get; set; }

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }

        [JsonProperty("updated_at")]
        public DateTime UpdatedAt { get; set; }

        [JsonProperty("unread_count")]
        public int UnreadCount { get; set; }
    }
}
