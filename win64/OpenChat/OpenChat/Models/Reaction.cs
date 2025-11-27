using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class Reaction
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("message_id")]
        public Guid MessageId { get; set; }

        [JsonProperty("user_id")]
        public Guid UserId { get; set; }

        [JsonProperty("emoji")]
        public string Emoji { get; set; } = string.Empty;

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }
    }

    public class ReactionCount
    {
        [JsonProperty("emoji")]
        public string Emoji { get; set; } = string.Empty;

        [JsonProperty("count")]
        public long Count { get; set; }

        [JsonProperty("user_ids")]
        public List<Guid> UserIds { get; set; } = new();

        /// <summary>
        /// Check if the current user has reacted with this emoji
        /// </summary>
        public bool HasCurrentUserReacted(Guid? currentUserId)
        {
            return currentUserId.HasValue && UserIds.Contains(currentUserId.Value);
        }
    }

    public class AddReactionRequest
    {
        [JsonProperty("emoji")]
        public string Emoji { get; set; } = string.Empty;
    }
}
