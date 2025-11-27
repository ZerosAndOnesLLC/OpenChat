using Newtonsoft.Json;

namespace OpenChat.Models
{
    public class MessagesResponse
    {
        [JsonProperty("messages")]
        public List<Message> Messages { get; set; } = new();
    }

    public class Message
    {
        [JsonProperty("id")]
        public Guid Id { get; set; }

        [JsonProperty("channel_id")]
        public Guid? ChannelId { get; set; }

        [JsonProperty("dm_id")]
        public Guid? DmId { get; set; }

        [JsonProperty("user_id")]
        public Guid UserId { get; set; }

        [JsonProperty("content")]
        public string Content { get; set; } = string.Empty;

        [JsonProperty("created_at")]
        public DateTime CreatedAt { get; set; }

        [JsonProperty("updated_at")]
        public DateTime? UpdatedAt { get; set; }

        [JsonProperty("deleted_at")]
        public DateTime? DeletedAt { get; set; }

        [JsonProperty("parent_message_id")]
        public Guid? ParentMessageId { get; set; }

        [JsonProperty("user")]
        public User? User { get; set; }

        [JsonProperty("is_edited")]
        public bool IsEdited { get; set; }
    }
}
