using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class DirectMessage
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("org_id")]
        public Guid OrgId { get; set; }

        [JsonProperty("user1_id")]
        public Guid User1Id { get; set; }

        [JsonProperty("user2_id")]
        public Guid User2Id { get; set; }

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }

        [JsonProperty("updated_at")]
        public DateTime UpdatedAt { get; set; }

        [JsonProperty("other_user")]
        public User? OtherUser { get; set; }

        [JsonProperty("unread_count")]
        public int UnreadCount { get; set; }
    }
}
